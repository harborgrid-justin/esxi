/**
 * Incident Response - Security Incident Management
 * Incident tracking, response, and resolution
 */

import { nanoid } from 'nanoid';
import { Incident, IncidentSeverity, IncidentStatus, IncidentTimelineEntry } from '../types';

export class IncidentResponse {
  private incidents: Map<string, Incident> = new Map();

  /**
   * Create incident
   */
  async createIncident(data: {
    title: string;
    description: string;
    severity: IncidentSeverity;
    category: string;
    reportedBy: string;
    affectedSystems?: string[];
    affectedUsers?: string[];
  }): Promise<Incident> {
    const incident: Incident = {
      id: nanoid(),
      title: data.title,
      description: data.description,
      severity: data.severity,
      status: IncidentStatus.NEW,
      category: data.category,
      detectedAt: new Date(),
      reportedBy: data.reportedBy,
      affectedSystems: data.affectedSystems || [],
      affectedUsers: data.affectedUsers || [],
      timeline: [
        {
          id: nanoid(),
          timestamp: new Date(),
          event: 'Incident Created',
          details: data.description,
          actor: data.reportedBy,
          automated: false,
        },
      ],
      metadata: {},
    };

    this.incidents.set(incident.id, incident);
    return incident;
  }

  /**
   * Update incident status
   */
  async updateStatus(
    incidentId: string,
    status: IncidentStatus,
    actor: string,
    details?: string
  ): Promise<void> {
    const incident = this.incidents.get(incidentId);
    if (!incident) {
      throw new Error('Incident not found');
    }

    incident.status = status;
    this.addTimelineEntry(incident, `Status changed to ${status}`, actor, details || '');
  }

  /**
   * Add timeline entry
   */
  private addTimelineEntry(
    incident: Incident,
    event: string,
    actor: string,
    details: string
  ): void {
    const entry: IncidentTimelineEntry = {
      id: nanoid(),
      timestamp: new Date(),
      event,
      details,
      actor,
      automated: false,
    };
    incident.timeline.push(entry);
  }

  /**
   * Assign incident
   */
  async assignIncident(incidentId: string, assignee: string, assignedBy: string): Promise<void> {
    const incident = this.incidents.get(incidentId);
    if (!incident) {
      throw new Error('Incident not found');
    }

    incident.assignedTo = assignee;
    this.addTimelineEntry(incident, 'Incident Assigned', assignedBy, `Assigned to ${assignee}`);
  }

  /**
   * Resolve incident
   */
  async resolveIncident(
    incidentId: string,
    resolution: string,
    rootCause: string,
    resolvedBy: string
  ): Promise<void> {
    const incident = this.incidents.get(incidentId);
    if (!incident) {
      throw new Error('Incident not found');
    }

    incident.status = IncidentStatus.RECOVERED;
    incident.resolution = resolution;
    incident.rootCause = rootCause;
    this.addTimelineEntry(incident, 'Incident Resolved', resolvedBy, resolution);
  }

  /**
   * Get all incidents
   */
  getAllIncidents(): Incident[] {
    return Array.from(this.incidents.values());
  }

  /**
   * Get open incidents
   */
  getOpenIncidents(): Incident[] {
    return Array.from(this.incidents.values()).filter(
      i => i.status !== IncidentStatus.CLOSED && i.status !== IncidentStatus.RECOVERED
    );
  }

  /**
   * Get incident by ID
   */
  getIncident(id: string): Incident | undefined {
    return this.incidents.get(id);
  }
}

export const incidentResponse = new IncidentResponse();
