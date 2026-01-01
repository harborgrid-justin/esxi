/**
 * Audit Service
 * Handles audit logging and event tracking
 */

import { AuditEvent, AuditEventType } from '../types';

export interface AuditServiceConfig {
  maxEvents?: number;
  persistEvents?: boolean;
  includeMetadata?: boolean;
}

export interface AuditQuery {
  participantId?: string;
  documentId?: string;
  eventTypes?: AuditEventType[];
  startDate?: Date;
  endDate?: Date;
  limit?: number;
  offset?: number;
}

export interface AuditStats {
  totalEvents: number;
  eventsByType: Map<AuditEventType, number>;
  eventsByParticipant: Map<string, number>;
  eventsByDocument: Map<string, number>;
  timeRange: {
    earliest: Date | null;
    latest: Date | null;
  };
}

export class AuditService {
  private events: AuditEvent[] = [];
  private config: Required<AuditServiceConfig>;
  private eventListeners: Set<(event: AuditEvent) => void> = new Set();

  constructor(config: AuditServiceConfig = {}) {
    this.config = {
      maxEvents: config.maxEvents ?? 10000,
      persistEvents: config.persistEvents ?? false,
      includeMetadata: config.includeMetadata ?? true,
    };
  }

  /**
   * Log an audit event
   */
  log(
    type: AuditEventType,
    documentId: string,
    participantId: string,
    details: Record<string, unknown>,
    metadata?: {
      ipAddress?: string;
      userAgent?: string;
    }
  ): AuditEvent {
    const event: AuditEvent = {
      id: this.generateEventId(),
      type,
      documentId,
      participantId,
      timestamp: new Date(),
      details,
      ...(this.config.includeMetadata && metadata),
    };

    this.addEvent(event);
    this.notifyListeners(event);

    return event;
  }

  /**
   * Add an event to the log
   */
  private addEvent(event: AuditEvent): void {
    this.events.push(event);

    // Trim events if exceeds max
    if (this.events.length > this.config.maxEvents) {
      this.events = this.events.slice(-this.config.maxEvents);
    }
  }

  /**
   * Query audit events
   */
  query(query: AuditQuery = {}): AuditEvent[] {
    let results = [...this.events];

    // Filter by participant
    if (query.participantId) {
      results = results.filter((e) => e.participantId === query.participantId);
    }

    // Filter by document
    if (query.documentId) {
      results = results.filter((e) => e.documentId === query.documentId);
    }

    // Filter by event types
    if (query.eventTypes && query.eventTypes.length > 0) {
      results = results.filter((e) => query.eventTypes!.includes(e.type));
    }

    // Filter by date range
    if (query.startDate) {
      results = results.filter((e) => e.timestamp >= query.startDate!);
    }
    if (query.endDate) {
      results = results.filter((e) => e.timestamp <= query.endDate!);
    }

    // Sort by timestamp (newest first)
    results.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());

    // Apply pagination
    if (query.offset) {
      results = results.slice(query.offset);
    }
    if (query.limit) {
      results = results.slice(0, query.limit);
    }

    return results;
  }

  /**
   * Get event by ID
   */
  getEvent(eventId: string): AuditEvent | undefined {
    return this.events.find((e) => e.id === eventId);
  }

  /**
   * Get all events
   */
  getAllEvents(): AuditEvent[] {
    return [...this.events];
  }

  /**
   * Get events for a participant
   */
  getParticipantEvents(participantId: string): AuditEvent[] {
    return this.query({ participantId });
  }

  /**
   * Get events for a document
   */
  getDocumentEvents(documentId: string): AuditEvent[] {
    return this.query({ documentId });
  }

  /**
   * Get events by type
   */
  getEventsByType(type: AuditEventType): AuditEvent[] {
    return this.query({ eventTypes: [type] });
  }

  /**
   * Get events in time range
   */
  getEventsInRange(startDate: Date, endDate: Date): AuditEvent[] {
    return this.query({ startDate, endDate });
  }

  /**
   * Get recent events
   */
  getRecentEvents(limit: number = 50): AuditEvent[] {
    return this.query({ limit });
  }

  /**
   * Calculate audit statistics
   */
  getStats(): AuditStats {
    const eventsByType = new Map<AuditEventType, number>();
    const eventsByParticipant = new Map<string, number>();
    const eventsByDocument = new Map<string, number>();

    let earliest: Date | null = null;
    let latest: Date | null = null;

    for (const event of this.events) {
      // Count by type
      eventsByType.set(event.type, (eventsByType.get(event.type) || 0) + 1);

      // Count by participant
      eventsByParticipant.set(
        event.participantId,
        (eventsByParticipant.get(event.participantId) || 0) + 1
      );

      // Count by document
      eventsByDocument.set(
        event.documentId,
        (eventsByDocument.get(event.documentId) || 0) + 1
      );

      // Track time range
      if (!earliest || event.timestamp < earliest) {
        earliest = event.timestamp;
      }
      if (!latest || event.timestamp > latest) {
        latest = event.timestamp;
      }
    }

    return {
      totalEvents: this.events.length,
      eventsByType,
      eventsByParticipant,
      eventsByDocument,
      timeRange: { earliest, latest },
    };
  }

  /**
   * Export events as JSON
   */
  exportJSON(): string {
    return JSON.stringify(this.events, null, 2);
  }

  /**
   * Import events from JSON
   */
  importJSON(json: string): void {
    try {
      const events = JSON.parse(json) as AuditEvent[];
      this.events = events;
    } catch (error) {
      console.error('Failed to import audit events:', error);
    }
  }

  /**
   * Export events as CSV
   */
  exportCSV(): string {
    const headers = [
      'ID',
      'Type',
      'Document ID',
      'Participant ID',
      'Timestamp',
      'Details',
    ];
    const rows = this.events.map((event) => [
      event.id,
      event.type,
      event.documentId,
      event.participantId,
      event.timestamp.toISOString(),
      JSON.stringify(event.details),
    ]);

    return [headers, ...rows].map((row) => row.join(',')).join('\n');
  }

  /**
   * Clear old events
   */
  clearOldEvents(beforeDate: Date): number {
    const initialCount = this.events.length;
    this.events = this.events.filter((e) => e.timestamp >= beforeDate);
    return initialCount - this.events.length;
  }

  /**
   * Clear all events
   */
  clearAll(): void {
    this.events = [];
  }

  /**
   * Add event listener
   */
  addEventListener(listener: (event: AuditEvent) => void): () => void {
    this.eventListeners.add(listener);
    return () => this.eventListeners.delete(listener);
  }

  /**
   * Notify listeners of new event
   */
  private notifyListeners(event: AuditEvent): void {
    this.eventListeners.forEach((listener) => {
      try {
        listener(event);
      } catch (error) {
        console.error('Audit event listener error:', error);
      }
    });
  }

  /**
   * Generate unique event ID
   */
  private generateEventId(): string {
    return `audit_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Log document created event
   */
  logDocumentCreated(
    documentId: string,
    participantId: string,
    details: Record<string, unknown>
  ): AuditEvent {
    return this.log(
      AuditEventType.DOCUMENT_CREATED,
      documentId,
      participantId,
      details
    );
  }

  /**
   * Log document updated event
   */
  logDocumentUpdated(
    documentId: string,
    participantId: string,
    details: Record<string, unknown>
  ): AuditEvent {
    return this.log(
      AuditEventType.DOCUMENT_UPDATED,
      documentId,
      participantId,
      details
    );
  }

  /**
   * Log participant joined event
   */
  logParticipantJoined(
    documentId: string,
    participantId: string,
    details: Record<string, unknown>
  ): AuditEvent {
    return this.log(
      AuditEventType.PARTICIPANT_JOINED,
      documentId,
      participantId,
      details
    );
  }

  /**
   * Log participant left event
   */
  logParticipantLeft(
    documentId: string,
    participantId: string,
    details: Record<string, unknown>
  ): AuditEvent {
    return this.log(
      AuditEventType.PARTICIPANT_LEFT,
      documentId,
      participantId,
      details
    );
  }

  /**
   * Log conflict detected event
   */
  logConflictDetected(
    documentId: string,
    participantId: string,
    details: Record<string, unknown>
  ): AuditEvent {
    return this.log(
      AuditEventType.CONFLICT_DETECTED,
      documentId,
      participantId,
      details
    );
  }

  /**
   * Log conflict resolved event
   */
  logConflictResolved(
    documentId: string,
    participantId: string,
    details: Record<string, unknown>
  ): AuditEvent {
    return this.log(
      AuditEventType.CONFLICT_RESOLVED,
      documentId,
      participantId,
      details
    );
  }
}
