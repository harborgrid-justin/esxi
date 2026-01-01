/**
 * QuietHoursService - Do Not Disturb management
 */

import { EventEmitter } from 'events';
import { QuietHours, NotificationPriority } from '../types';

export class QuietHoursService extends EventEmitter {
  /**
   * Check if notification should be suppressed
   */
  shouldSuppress(quietHours: QuietHours, priority: NotificationPriority): boolean {
    if (!quietHours.enabled) {
      return false;
    }

    // Allow urgent/critical notifications
    if (quietHours.allowUrgent && priority === 'urgent') {
      return false;
    }
    if (quietHours.allowCritical && priority === 'critical') {
      return false;
    }

    const now = new Date();

    // Check day of week
    if (quietHours.days && quietHours.days.length > 0) {
      const currentDay = now.getDay();
      if (!quietHours.days.includes(currentDay)) {
        return false;
      }
    }

    // Check time range
    const currentTime = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;

    return currentTime >= quietHours.startTime && currentTime <= quietHours.endTime;
  }
}

export default QuietHoursService;
