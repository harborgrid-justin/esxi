import React from 'react';
import { format, startOfMonth, endOfMonth, eachDayOfInterval, isSameDay, parseISO } from 'date-fns';
import type { ScanSchedule } from '../../types';

export interface ScheduleCalendarProps {
  schedules: ScanSchedule[];
  onDayClick?: (date: Date) => void;
}

/**
 * Calendar view of scheduled scans
 */
export const ScheduleCalendar: React.FC<ScheduleCalendarProps> = ({
  schedules,
  onDayClick,
}) => {
  const [currentMonth, setCurrentMonth] = React.useState(new Date());

  const monthStart = startOfMonth(currentMonth);
  const monthEnd = endOfMonth(currentMonth);
  const days = eachDayOfInterval({ start: monthStart, end: monthEnd });

  const getSchedulesForDay = (date: Date) => {
    return schedules.filter((schedule) => {
      if (!schedule.next_run) return false;
      return isSameDay(parseISO(schedule.next_run), date);
    });
  };

  const nextMonth = () => {
    setCurrentMonth(new Date(currentMonth.getFullYear(), currentMonth.getMonth() + 1, 1));
  };

  const prevMonth = () => {
    setCurrentMonth(new Date(currentMonth.getFullYear(), currentMonth.getMonth() - 1, 1));
  };

  return (
    <div className="schedule-calendar">
      <div className="calendar-header">
        <button className="nav-btn" onClick={prevMonth}>
          ‹
        </button>
        <h3>{format(currentMonth, 'MMMM yyyy')}</h3>
        <button className="nav-btn" onClick={nextMonth}>
          ›
        </button>
      </div>

      <div className="calendar-grid">
        <div className="weekday-header">Sun</div>
        <div className="weekday-header">Mon</div>
        <div className="weekday-header">Tue</div>
        <div className="weekday-header">Wed</div>
        <div className="weekday-header">Thu</div>
        <div className="weekday-header">Fri</div>
        <div className="weekday-header">Sat</div>

        {days.map((day) => {
          const daySchedules = getSchedulesForDay(day);
          const isToday = isSameDay(day, new Date());

          return (
            <div
              key={day.toISOString()}
              className={`calendar-day ${isToday ? 'today' : ''} ${
                daySchedules.length > 0 ? 'has-schedules' : ''
              }`}
              onClick={() => onDayClick?.(day)}
            >
              <div className="day-number">{format(day, 'd')}</div>
              {daySchedules.length > 0 && (
                <div className="schedules-indicator">
                  <span className="schedule-count">{daySchedules.length}</span>
                  <span className="schedule-label">
                    {daySchedules.length === 1 ? 'scan' : 'scans'}
                  </span>
                </div>
              )}
            </div>
          );
        })}
      </div>

      <div className="calendar-legend">
        <div className="legend-item">
          <div className="legend-color today"></div>
          <span>Today</span>
        </div>
        <div className="legend-item">
          <div className="legend-color has-schedules"></div>
          <span>Scheduled Scans</span>
        </div>
      </div>

      <style>{`
        .schedule-calendar {
          padding: 24px;
          background: white;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .calendar-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 24px;
        }

        .calendar-header h3 {
          margin: 0;
          font-size: 20px;
          font-weight: 600;
          color: #111827;
        }

        .nav-btn {
          padding: 8px 16px;
          background: #f3f4f6;
          border: none;
          border-radius: 4px;
          font-size: 20px;
          cursor: pointer;
          transition: background-color 0.2s;
        }

        .nav-btn:hover {
          background: #e5e7eb;
        }

        .calendar-grid {
          display: grid;
          grid-template-columns: repeat(7, 1fr);
          gap: 8px;
        }

        .weekday-header {
          padding: 12px;
          text-align: center;
          font-size: 13px;
          font-weight: 600;
          color: #6b7280;
          text-transform: uppercase;
        }

        .calendar-day {
          min-height: 100px;
          padding: 8px;
          background: #f9fafb;
          border: 1px solid #e5e7eb;
          border-radius: 6px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .calendar-day:hover {
          background: #f3f4f6;
          border-color: #d1d5db;
        }

        .calendar-day.today {
          background: #eff6ff;
          border-color: #3b82f6;
        }

        .calendar-day.has-schedules {
          background: #fef3c7;
          border-color: #f59e0b;
        }

        .calendar-day.today.has-schedules {
          background: #dbeafe;
          border-color: #3b82f6;
        }

        .day-number {
          font-size: 14px;
          font-weight: 600;
          color: #374151;
          margin-bottom: 8px;
        }

        .schedules-indicator {
          display: flex;
          flex-direction: column;
          align-items: center;
          padding: 8px;
          background: white;
          border-radius: 4px;
          box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
        }

        .schedule-count {
          font-size: 20px;
          font-weight: 700;
          color: #f59e0b;
        }

        .schedule-label {
          font-size: 11px;
          color: #6b7280;
          text-transform: uppercase;
        }

        .calendar-legend {
          display: flex;
          gap: 24px;
          margin-top: 24px;
          padding-top: 16px;
          border-top: 1px solid #e5e7eb;
        }

        .legend-item {
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 13px;
          color: #6b7280;
        }

        .legend-color {
          width: 16px;
          height: 16px;
          border-radius: 4px;
          border: 1px solid #e5e7eb;
        }

        .legend-color.today {
          background: #eff6ff;
          border-color: #3b82f6;
        }

        .legend-color.has-schedules {
          background: #fef3c7;
          border-color: #f59e0b;
        }
      `}</style>
    </div>
  );
};
