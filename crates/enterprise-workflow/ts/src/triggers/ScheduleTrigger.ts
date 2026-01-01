/**
 * Schedule Trigger - Cron-based workflow scheduling
 */

import { EventEmitter } from 'eventemitter3';
import cron from 'node-cron';
import { parseExpression } from 'cron-parser';
import { Trigger, ScheduleTriggerConfig, Context } from '../types';

export interface ScheduledJob {
  triggerId: string;
  task: cron.ScheduledTask;
  nextRun: Date;
  runCount: number;
}

export class ScheduleTrigger extends EventEmitter {
  private jobs: Map<string, ScheduledJob>;

  constructor() {
    super();
    this.jobs = new Map();
  }

  /**
   * Register a scheduled trigger
   */
  register(trigger: Trigger): void {
    const config = trigger.config as ScheduleTriggerConfig;

    if (!config.cronExpression) {
      throw new Error('Schedule trigger must have a cron expression');
    }

    // Validate cron expression
    if (!cron.validate(config.cronExpression)) {
      throw new Error(`Invalid cron expression: ${config.cronExpression}`);
    }

    // Check if already registered
    if (this.jobs.has(trigger.id)) {
      this.unregister(trigger.id);
    }

    // Create scheduled task
    const task = cron.schedule(
      config.cronExpression,
      () => this.executeSchedule(trigger),
      {
        scheduled: trigger.enabled,
        timezone: config.timezone || 'UTC'
      }
    );

    // Calculate next run time
    const nextRun = this.getNextRun(config.cronExpression, config.timezone);

    this.jobs.set(trigger.id, {
      triggerId: trigger.id,
      task,
      nextRun,
      runCount: 0
    });

    this.emit('schedule:registered', {
      triggerId: trigger.id,
      cronExpression: config.cronExpression,
      nextRun
    });
  }

  /**
   * Unregister a scheduled trigger
   */
  unregister(triggerId: string): void {
    const job = this.jobs.get(triggerId);
    if (job) {
      job.task.stop();
      this.jobs.delete(triggerId);
      this.emit('schedule:unregistered', { triggerId });
    }
  }

  /**
   * Execute scheduled trigger
   */
  private async executeSchedule(trigger: Trigger): Promise<void> {
    const config = trigger.config as ScheduleTriggerConfig;
    const job = this.jobs.get(trigger.id);

    if (!job) return;

    // Check if schedule is still active
    if (config.endDate && new Date() > config.endDate) {
      this.unregister(trigger.id);
      this.emit('schedule:expired', { triggerId: trigger.id });
      return;
    }

    // Check max runs
    if (config.maxRuns && job.runCount >= config.maxRuns) {
      this.unregister(trigger.id);
      this.emit('schedule:max_runs_reached', {
        triggerId: trigger.id,
        runCount: job.runCount
      });
      return;
    }

    // Create execution context
    const context: Partial<Context> = {
      variables: new Map([
        ['schedule_time', new Date()],
        ['schedule_cron', config.cronExpression],
        ['schedule_run_count', job.runCount + 1]
      ]),
      metadata: {
        source: 'schedule',
        cronExpression: config.cronExpression,
        runCount: job.runCount + 1
      }
    };

    // Emit trigger event
    this.emit('schedule:triggered', {
      triggerId: trigger.id,
      context,
      runCount: job.runCount + 1
    });

    // Update job statistics
    job.runCount++;
    job.nextRun = this.getNextRun(config.cronExpression, config.timezone);
  }

  /**
   * Start a scheduled trigger
   */
  start(triggerId: string): void {
    const job = this.jobs.get(triggerId);
    if (job) {
      job.task.start();
      this.emit('schedule:started', { triggerId });
    }
  }

  /**
   * Stop a scheduled trigger
   */
  stop(triggerId: string): void {
    const job = this.jobs.get(triggerId);
    if (job) {
      job.task.stop();
      this.emit('schedule:stopped', { triggerId });
    }
  }

  /**
   * Get next run time
   */
  private getNextRun(cronExpression: string, timezone?: string): Date {
    const interval = parseExpression(cronExpression, {
      currentDate: new Date(),
      tz: timezone || 'UTC'
    });
    return interval.next().toDate();
  }

  /**
   * Get all scheduled jobs
   */
  getScheduled(): ScheduledJob[] {
    return Array.from(this.jobs.values());
  }

  /**
   * Get job by trigger ID
   */
  getJob(triggerId: string): ScheduledJob | undefined {
    return this.jobs.get(triggerId);
  }

  /**
   * Get upcoming runs
   */
  getUpcomingRuns(triggerId: string, count: number = 5): Date[] {
    const job = this.jobs.get(triggerId);
    if (!job) return [];

    const trigger = Array.from(this.jobs.values())
      .find(j => j.triggerId === triggerId);

    if (!trigger) return [];

    // This would need the actual cron expression from the trigger
    // For now, return a simplified version
    return [job.nextRun];
  }

  /**
   * Validate schedule configuration
   */
  validate(config: ScheduleTriggerConfig): string[] {
    const errors: string[] = [];

    if (!config.cronExpression) {
      errors.push('Cron expression is required');
    } else if (!cron.validate(config.cronExpression)) {
      errors.push(`Invalid cron expression: ${config.cronExpression}`);
    }

    if (config.startDate && config.endDate) {
      if (config.startDate >= config.endDate) {
        errors.push('End date must be after start date');
      }
    }

    if (config.maxRuns !== undefined && config.maxRuns <= 0) {
      errors.push('Max runs must be greater than 0');
    }

    return errors;
  }

  /**
   * Stop all scheduled jobs
   */
  stopAll(): void {
    this.jobs.forEach((job, triggerId) => {
      job.task.stop();
    });
    this.emit('schedule:all_stopped');
  }

  /**
   * Clear all scheduled jobs
   */
  clear(): void {
    this.jobs.forEach((job) => {
      job.task.stop();
    });
    this.jobs.clear();
    this.emit('schedule:cleared');
  }
}
