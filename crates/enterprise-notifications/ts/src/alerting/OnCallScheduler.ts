/**
 * OnCallScheduler - On-call rotation management
 * Manages on-call schedules, rotations, and overrides
 */

import { EventEmitter } from 'events';
import { OnCallSchedule, OnCallRotation, OnCallOverride, OnCallAssignment } from '../types';

export class OnCallScheduler extends EventEmitter {
  private schedules: Map<string, OnCallSchedule>;

  constructor() {
    super();
    this.schedules = new Map();
  }

  /**
   * Register on-call schedule
   */
  registerSchedule(schedule: OnCallSchedule): void {
    this.schedules.set(schedule.id, schedule);
    this.emit('schedule:registered', schedule);
  }

  /**
   * Get current on-call user for schedule
   */
  getCurrentOnCall(scheduleId: string): OnCallAssignment[] {
    const schedule = this.schedules.get(scheduleId);
    if (!schedule || !schedule.enabled) {
      return [];
    }

    const now = new Date();

    // Check for overrides first
    const override = this.getActiveOverride(schedule, now);
    if (override) {
      return [
        {
          userId: override.userId,
          userName: `Override: ${override.userId}`,
          startTime: override.startDate,
          endTime: override.endDate,
          rotationId: 'override',
        },
      ];
    }

    // Calculate from rotation
    const assignments: OnCallAssignment[] = [];

    for (const rotation of schedule.rotations) {
      const assignment = this.calculateRotationAssignment(schedule, rotation, now);
      if (assignment) {
        assignments.push(assignment);
      }
    }

    return assignments;
  }

  /**
   * Get active override
   */
  private getActiveOverride(schedule: OnCallSchedule, time: Date): OnCallOverride | undefined {
    return schedule.overrides.find(
      override => time >= override.startDate && time <= override.endDate
    );
  }

  /**
   * Calculate rotation assignment
   */
  private calculateRotationAssignment(
    schedule: OnCallSchedule,
    rotation: OnCallRotation,
    time: Date
  ): OnCallAssignment | undefined {
    const { users, handoffTime } = rotation;

    if (users.length === 0) {
      return undefined;
    }

    // Calculate which user is on call
    const startDate = new Date(schedule.rotationStartDate);
    let elapsed: number;

    switch (schedule.rotationType) {
      case 'daily':
        elapsed = Math.floor((time.getTime() - startDate.getTime()) / (24 * 60 * 60 * 1000));
        break;
      case 'weekly':
        elapsed = Math.floor((time.getTime() - startDate.getTime()) / (7 * 24 * 60 * 60 * 1000));
        break;
      default:
        elapsed = 0;
    }

    const userIndex = elapsed % users.length;
    const userId = users[userIndex];

    // Calculate shift times
    const shiftStart = this.calculateShiftStart(time, handoffTime);
    const shiftEnd = this.calculateShiftEnd(
      shiftStart,
      schedule.rotationType === 'daily' ? 1 : 7
    );

    return {
      userId: userId!,
      userName: userId!,
      startTime: shiftStart,
      endTime: shiftEnd,
      rotationId: rotation.id,
    };
  }

  /**
   * Calculate shift start time
   */
  private calculateShiftStart(date: Date, handoffTime: string): Date {
    const [hours, minutes] = handoffTime.split(':').map(Number);
    const start = new Date(date);
    start.setHours(hours!, minutes!, 0, 0);

    if (start > date) {
      start.setDate(start.getDate() - 1);
    }

    return start;
  }

  /**
   * Calculate shift end time
   */
  private calculateShiftEnd(start: Date, days: number): Date {
    const end = new Date(start);
    end.setDate(end.getDate() + days);
    return end;
  }

  /**
   * Add override
   */
  addOverride(scheduleId: string, override: OnCallOverride): boolean {
    const schedule = this.schedules.get(scheduleId);
    if (!schedule) {
      return false;
    }

    schedule.overrides.push(override);
    this.emit('override:added', scheduleId, override);

    return true;
  }

  /**
   * Remove override
   */
  removeOverride(scheduleId: string, overrideId: string): boolean {
    const schedule = this.schedules.get(scheduleId);
    if (!schedule) {
      return false;
    }

    const index = schedule.overrides.findIndex(o => o.id === overrideId);
    if (index === -1) {
      return false;
    }

    schedule.overrides.splice(index, 1);
    this.emit('override:removed', scheduleId, overrideId);

    return true;
  }

  /**
   * Get upcoming schedule
   */
  getUpcomingSchedule(scheduleId: string, days: number = 7): OnCallAssignment[] {
    const schedule = this.schedules.get(scheduleId);
    if (!schedule || !schedule.enabled) {
      return [];
    }

    const assignments: OnCallAssignment[] = [];
    const now = new Date();
    const endDate = new Date(now.getTime() + days * 24 * 60 * 60 * 1000);

    let currentDate = new Date(now);

    while (currentDate < endDate) {
      const dayAssignments = this.getCurrentOnCall(scheduleId);
      assignments.push(...dayAssignments);

      // Move to next rotation period
      currentDate = new Date(currentDate.getTime() + 24 * 60 * 60 * 1000);
    }

    return assignments;
  }
}

export default OnCallScheduler;
