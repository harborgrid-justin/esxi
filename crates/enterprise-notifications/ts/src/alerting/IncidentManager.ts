/**
 * IncidentManager - Incident lifecycle management
 * Tracks incidents, responders, and timeline events
 */

import { EventEmitter } from 'events';
import { Incident, IncidentResponder, IncidentTimelineEvent, Alert, AlertSeverity } from '../types';

export class IncidentManager extends EventEmitter {
  private incidents: Map<string, Incident>;
  private alertToIncident: Map<string, string>;

  constructor() {
    super();
    this.incidents = new Map();
    this.alertToIncident = new Map();
  }

  /**
   * Create incident from alert
   */
  createIncident(alert: Alert, title?: string): Incident {
    const incident: Incident = {
      id: this.generateIncidentId(),
      tenantId: alert.tenantId,
      title: title ?? alert.name,
      description: alert.description,
      severity: alert.severity,
      status: 'open',
      assignedTo: alert.assignedTo,
      alertIds: [alert.id],
      primaryAlertId: alert.id,
      detectedAt: alert.createdAt,
      responders: [],
      timeline: [
        {
          id: this.generateEventId(),
          type: 'created',
          description: `Incident created from alert: ${alert.name}`,
          timestamp: new Date(),
          metadata: { alertId: alert.id },
        },
      ],
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    this.incidents.set(incident.id, incident);
    this.alertToIncident.set(alert.id, incident.id);

    this.emit('incident:created', incident);
    return incident;
  }

  /**
   * Add alert to incident
   */
  addAlert(incidentId: string, alert: Alert): boolean {
    const incident = this.incidents.get(incidentId);
    if (!incident) {
      return false;
    }

    if (!incident.alertIds.includes(alert.id)) {
      incident.alertIds.push(alert.id);
      this.alertToIncident.set(alert.id, incidentId);

      this.addTimelineEvent(incident, {
        type: 'note',
        description: `Alert added: ${alert.name}`,
        metadata: { alertId: alert.id },
      });
    }

    return true;
  }

  /**
   * Add responder to incident
   */
  addResponder(incidentId: string, responder: Omit<IncidentResponder, 'joinedAt' | 'status'>): boolean {
    const incident = this.incidents.get(incidentId);
    if (!incident) {
      return false;
    }

    const fullResponder: IncidentResponder = {
      ...responder,
      joinedAt: new Date(),
      status: 'active',
    };

    incident.responders.push(fullResponder);

    this.addTimelineEvent(incident, {
      type: 'note',
      description: `${responder.name} joined as ${responder.role}`,
      userId: responder.userId,
      userName: responder.name,
    });

    return true;
  }

  /**
   * Update incident status
   */
  updateStatus(
    incidentId: string,
    status: Incident['status'],
    userId?: string,
    userName?: string
  ): boolean {
    const incident = this.incidents.get(incidentId);
    if (!incident) {
      return false;
    }

    const oldStatus = incident.status;
    incident.status = status;
    incident.updatedAt = new Date();

    if (status === 'resolved') {
      incident.resolvedAt = new Date();
    } else if (status === 'closed') {
      incident.closedAt = new Date();
    }

    this.addTimelineEvent(incident, {
      type: 'status_change',
      description: `Status changed from ${oldStatus} to ${status}`,
      userId,
      userName,
    });

    this.emit('incident:updated', incident);
    return true;
  }

  /**
   * Add timeline event
   */
  addTimelineEvent(
    incident: Incident,
    event: Omit<IncidentTimelineEvent, 'id' | 'timestamp'>
  ): void {
    const fullEvent: IncidentTimelineEvent = {
      id: this.generateEventId(),
      timestamp: new Date(),
      ...event,
    };

    incident.timeline.push(fullEvent);
    incident.updatedAt = new Date();

    this.emit('incident:timeline', incident, fullEvent);
  }

  /**
   * Get incident by ID
   */
  getIncident(incidentId: string): Incident | undefined {
    return this.incidents.get(incidentId);
  }

  /**
   * Get incident by alert
   */
  getIncidentByAlert(alertId: string): Incident | undefined {
    const incidentId = this.alertToIncident.get(alertId);
    return incidentId ? this.incidents.get(incidentId) : undefined;
  }

  /**
   * Get all incidents
   */
  getIncidents(filter?: {
    status?: Incident['status'][];
    severity?: AlertSeverity[];
    assignedTo?: string;
  }): Incident[] {
    let incidents = Array.from(this.incidents.values());

    if (filter) {
      incidents = incidents.filter(incident => {
        if (filter.status && !filter.status.includes(incident.status)) {
          return false;
        }
        if (filter.severity && !filter.severity.includes(incident.severity)) {
          return false;
        }
        if (filter.assignedTo && incident.assignedTo !== filter.assignedTo) {
          return false;
        }
        return true;
      });
    }

    return incidents.sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());
  }

  /**
   * Generate incident ID
   */
  private generateIncidentId(): string {
    const num = this.incidents.size + 1;
    return `INC-${num.toString().padStart(6, '0')}`;
  }

  /**
   * Generate event ID
   */
  private generateEventId(): string {
    return `evt_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

export default IncidentManager;
