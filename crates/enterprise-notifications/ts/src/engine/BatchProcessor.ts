/**
 * BatchProcessor - Batch notification processing
 * Efficiently handles bulk notification operations
 */

import { EventEmitter } from 'events';
import { Notification, NotificationPriority, BatchNotification } from '../types';

export interface BatchProcessorConfig {
  maxBatchSize: number;
  processingInterval: number; // milliseconds
  maxConcurrent: number;
  autoFlushInterval: number; // milliseconds
  enableRateLimit: boolean;
  rateLimit?: number; // notifications per second
}

export interface BatchJob {
  id: string;
  notifications: Notification[];
  priority: NotificationPriority;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  progress: {
    total: number;
    processed: number;
    successful: number;
    failed: number;
  };
  createdAt: Date;
  startedAt?: Date;
  completedAt?: Date;
  errors: Array<{ notificationId: string; error: string }>;
}

type ProcessorCallback = (notification: Notification) => Promise<void>;

export class BatchProcessor extends EventEmitter {
  private config: BatchProcessorConfig;
  private batches: Map<string, Notification[]>;
  private jobs: Map<string, BatchJob>;
  private processor?: ProcessorCallback;
  private processingInterval?: NodeJS.Timeout;
  private flushInterval?: NodeJS.Timeout;
  private isRunning: boolean;
  private rateLimitTokens: number;
  private lastTokenRefill: number;

  constructor(config: Partial<BatchProcessorConfig> = {}) {
    super();
    this.config = {
      maxBatchSize: config.maxBatchSize ?? 1000,
      processingInterval: config.processingInterval ?? 1000,
      maxConcurrent: config.maxConcurrent ?? 5,
      autoFlushInterval: config.autoFlushInterval ?? 30000,
      enableRateLimit: config.enableRateLimit ?? true,
      rateLimit: config.rateLimit ?? 100,
    };

    this.batches = new Map();
    this.jobs = new Map();
    this.isRunning = false;
    this.rateLimitTokens = this.config.rateLimit ?? 100;
    this.lastTokenRefill = Date.now();
  }

  /**
   * Set the notification processor
   */
  setProcessor(processor: ProcessorCallback): void {
    this.processor = processor;
  }

  /**
   * Start batch processing
   */
  start(): void {
    if (this.isRunning) {
      throw new Error('BatchProcessor is already running');
    }

    if (!this.processor) {
      throw new Error('No processor configured');
    }

    this.isRunning = true;

    // Process batches periodically
    this.processingInterval = setInterval(() => {
      this.processPendingJobs().catch(error => {
        this.emit('error', error);
      });
    }, this.config.processingInterval);

    // Auto-flush batches periodically
    this.flushInterval = setInterval(() => {
      this.flushAllBatches();
    }, this.config.autoFlushInterval);

    // Refill rate limit tokens
    if (this.config.enableRateLimit) {
      setInterval(() => {
        this.refillRateLimitTokens();
      }, 1000);
    }

    this.emit('started');
  }

  /**
   * Stop batch processing
   */
  async stop(): Promise<void> {
    if (!this.isRunning) {
      return;
    }

    this.isRunning = false;

    if (this.processingInterval) {
      clearInterval(this.processingInterval);
    }

    if (this.flushInterval) {
      clearInterval(this.flushInterval);
    }

    // Wait for current jobs to complete
    while (this.getActiveJobCount() > 0) {
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    this.emit('stopped');
  }

  /**
   * Add notification to batch
   */
  add(notification: Notification, batchKey: string = 'default'): void {
    let batch = this.batches.get(batchKey);
    if (!batch) {
      batch = [];
      this.batches.set(batchKey, batch);
    }

    batch.push(notification);

    // Auto-flush if batch size exceeded
    if (batch.length >= this.config.maxBatchSize) {
      this.flush(batchKey);
    }

    this.emit('notification:added', notification, batchKey);
  }

  /**
   * Add multiple notifications to batch
   */
  addBatch(notifications: Notification[], batchKey: string = 'default'): void {
    for (const notification of notifications) {
      this.add(notification, batchKey);
    }
  }

  /**
   * Flush a specific batch
   */
  flush(batchKey: string = 'default'): BatchJob | undefined {
    const batch = this.batches.get(batchKey);
    if (!batch || batch.length === 0) {
      return undefined;
    }

    const job = this.createJob(batch);
    this.batches.set(batchKey, []); // Clear batch
    this.emit('batch:flushed', job);

    return job;
  }

  /**
   * Flush all batches
   */
  flushAllBatches(): BatchJob[] {
    const jobs: BatchJob[] = [];
    for (const batchKey of this.batches.keys()) {
      const job = this.flush(batchKey);
      if (job) {
        jobs.push(job);
      }
    }
    return jobs;
  }

  /**
   * Create a batch job
   */
  createJob(notifications: Notification[], priority?: NotificationPriority): BatchJob {
    const jobId = this.generateJobId();
    const job: BatchJob = {
      id: jobId,
      notifications,
      priority: priority ?? NotificationPriority.NORMAL,
      status: 'pending',
      progress: {
        total: notifications.length,
        processed: 0,
        successful: 0,
        failed: 0,
      },
      createdAt: new Date(),
      errors: [],
    };

    this.jobs.set(jobId, job);
    this.emit('job:created', job);

    return job;
  }

  /**
   * Get job status
   */
  getJob(jobId: string): BatchJob | undefined {
    return this.jobs.get(jobId);
  }

  /**
   * Cancel a job
   */
  cancelJob(jobId: string): boolean {
    const job = this.jobs.get(jobId);
    if (!job || job.status !== 'pending') {
      return false;
    }

    job.status = 'failed';
    job.completedAt = new Date();
    this.emit('job:cancelled', job);

    return true;
  }

  /**
   * Process pending jobs
   */
  private async processPendingJobs(): Promise<void> {
    if (!this.processor) {
      return;
    }

    const activeJobs = Array.from(this.jobs.values()).filter(
      job => job.status === 'processing'
    ).length;

    if (activeJobs >= this.config.maxConcurrent) {
      return;
    }

    // Get pending jobs sorted by priority
    const pendingJobs = Array.from(this.jobs.values())
      .filter(job => job.status === 'pending')
      .sort((a, b) => this.comparePriority(a.priority, b.priority));

    for (const job of pendingJobs) {
      if (activeJobs >= this.config.maxConcurrent) {
        break;
      }

      this.processJob(job).catch(error => {
        this.emit('error', error);
      });
    }
  }

  /**
   * Process a single job
   */
  private async processJob(job: BatchJob): Promise<void> {
    if (!this.processor) {
      return;
    }

    job.status = 'processing';
    job.startedAt = new Date();
    this.emit('job:started', job);

    for (const notification of job.notifications) {
      if (!this.isRunning) {
        break;
      }

      // Rate limiting
      if (this.config.enableRateLimit) {
        await this.acquireRateLimitToken();
      }

      try {
        await this.processor(notification);
        job.progress.successful++;
        this.emit('job:notification:success', job, notification);
      } catch (error) {
        job.progress.failed++;
        job.errors.push({
          notificationId: notification.id,
          error: error instanceof Error ? error.message : String(error),
        });
        this.emit('job:notification:failed', job, notification, error);
      }

      job.progress.processed++;
      this.emit('job:progress', job);
    }

    job.status = job.progress.failed === 0 ? 'completed' : 'failed';
    job.completedAt = new Date();
    this.emit('job:completed', job);
  }

  /**
   * Acquire rate limit token
   */
  private async acquireRateLimitToken(): Promise<void> {
    while (this.rateLimitTokens <= 0) {
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    this.rateLimitTokens--;
  }

  /**
   * Refill rate limit tokens
   */
  private refillRateLimitTokens(): void {
    const now = Date.now();
    const elapsed = now - this.lastTokenRefill;
    const tokensToAdd = Math.floor((elapsed / 1000) * (this.config.rateLimit ?? 100));

    if (tokensToAdd > 0) {
      this.rateLimitTokens = Math.min(
        this.config.rateLimit ?? 100,
        this.rateLimitTokens + tokensToAdd
      );
      this.lastTokenRefill = now;
    }
  }

  /**
   * Compare priorities for sorting
   */
  private comparePriority(a: NotificationPriority, b: NotificationPriority): number {
    const priorities = {
      [NotificationPriority.CRITICAL]: 5,
      [NotificationPriority.URGENT]: 4,
      [NotificationPriority.HIGH]: 3,
      [NotificationPriority.NORMAL]: 2,
      [NotificationPriority.LOW]: 1,
    };
    return priorities[b] - priorities[a];
  }

  /**
   * Get active job count
   */
  private getActiveJobCount(): number {
    return Array.from(this.jobs.values()).filter(job => job.status === 'processing').length;
  }

  /**
   * Get statistics
   */
  getStats(): {
    batches: number;
    pendingNotifications: number;
    jobs: {
      total: number;
      pending: number;
      processing: number;
      completed: number;
      failed: number;
    };
  } {
    let pendingNotifications = 0;
    for (const batch of this.batches.values()) {
      pendingNotifications += batch.length;
    }

    const jobs = {
      total: this.jobs.size,
      pending: 0,
      processing: 0,
      completed: 0,
      failed: 0,
    };

    for (const job of this.jobs.values()) {
      switch (job.status) {
        case 'pending':
          jobs.pending++;
          break;
        case 'processing':
          jobs.processing++;
          break;
        case 'completed':
          jobs.completed++;
          break;
        case 'failed':
          jobs.failed++;
          break;
      }
    }

    return {
      batches: this.batches.size,
      pendingNotifications,
      jobs,
    };
  }

  /**
   * Clear completed jobs
   */
  clearCompletedJobs(olderThan?: Date): number {
    let cleared = 0;
    for (const [id, job] of this.jobs.entries()) {
      if (job.status === 'completed' || job.status === 'failed') {
        if (!olderThan || (job.completedAt && job.completedAt < olderThan)) {
          this.jobs.delete(id);
          cleared++;
        }
      }
    }
    return cleared;
  }

  /**
   * Generate unique job ID
   */
  private generateJobId(): string {
    return `job_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

export default BatchProcessor;
