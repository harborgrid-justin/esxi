import React, { useState } from 'react';
import { ScheduledExport, ReportConfig, ExportFormat } from '../../types';

interface ScheduledExportsProps {
  schedules: ScheduledExport[];
  onAdd?: (schedule: ScheduledExport) => void;
  onEdit?: (schedule: ScheduledExport) => void;
  onDelete?: (scheduleId: string) => void;
  onToggle?: (scheduleId: string, enabled: boolean) => void;
  className?: string;
}

export const ScheduledExports: React.FC<ScheduledExportsProps> = ({
  schedules,
  onAdd,
  onEdit,
  onDelete,
  onToggle,
  className = '',
}) => {
  const [isCreating, setIsCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    frequency: 'weekly' as ScheduledExport['schedule']['frequency'],
    dayOfWeek: 1,
    time: '09:00',
    recipients: '',
    formats: ['pdf'] as ExportFormat[],
  });

  const handleCreate = () => {
    // This would typically create a new schedule
    setIsCreating(false);
    // onAdd would be called here
  };

  const handleToggleFormat = (format: ExportFormat) => {
    setFormData((prev) => ({
      ...prev,
      formats: prev.formats.includes(format)
        ? prev.formats.filter((f) => f !== format)
        : [...prev.formats, format],
    }));
  };

  const getDayName = (dayOfWeek: number): string => {
    const days = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
    return days[dayOfWeek];
  };

  const getNextRunDisplay = (nextRun: Date): string => {
    const now = new Date();
    const diff = nextRun.getTime() - now.getTime();
    const hours = Math.floor(diff / (1000 * 60 * 60));

    if (hours < 24) {
      return `in ${hours} hours`;
    } else {
      const days = Math.floor(hours / 24);
      return `in ${days} days`;
    }
  };

  return (
    <div
      className={`scheduled-exports ${className}`}
      style={styles.container}
      role="region"
      aria-label="Scheduled exports"
    >
      <header style={styles.header}>
        <h2 style={styles.title}>Scheduled Report Exports</h2>
        <button
          onClick={() => setIsCreating(true)}
          style={styles.addButton}
          aria-label="Create new scheduled export"
        >
          + New Schedule
        </button>
      </header>

      {/* Create/Edit Form */}
      {(isCreating || editingId) && (
        <div style={styles.formCard} role="form" aria-label="Schedule configuration">
          <h3 style={styles.formTitle}>
            {isCreating ? 'Create New Schedule' : 'Edit Schedule'}
          </h3>

          <div style={styles.formGrid}>
            {/* Frequency */}
            <div style={styles.formGroup}>
              <label htmlFor="frequency" style={styles.label}>
                Frequency
              </label>
              <select
                id="frequency"
                value={formData.frequency}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    frequency: e.target.value as ScheduledExport['schedule']['frequency'],
                  })
                }
                style={styles.select}
              >
                <option value="daily">Daily</option>
                <option value="weekly">Weekly</option>
                <option value="monthly">Monthly</option>
                <option value="quarterly">Quarterly</option>
              </select>
            </div>

            {/* Day of Week (for weekly) */}
            {formData.frequency === 'weekly' && (
              <div style={styles.formGroup}>
                <label htmlFor="day-of-week" style={styles.label}>
                  Day of Week
                </label>
                <select
                  id="day-of-week"
                  value={formData.dayOfWeek}
                  onChange={(e) =>
                    setFormData({ ...formData, dayOfWeek: parseInt(e.target.value) })
                  }
                  style={styles.select}
                >
                  {[0, 1, 2, 3, 4, 5, 6].map((day) => (
                    <option key={day} value={day}>
                      {getDayName(day)}
                    </option>
                  ))}
                </select>
              </div>
            )}

            {/* Time */}
            <div style={styles.formGroup}>
              <label htmlFor="time" style={styles.label}>
                Time (UTC)
              </label>
              <input
                id="time"
                type="time"
                value={formData.time}
                onChange={(e) => setFormData({ ...formData, time: e.target.value })}
                style={styles.input}
              />
            </div>

            {/* Recipients */}
            <div style={{ ...styles.formGroup, gridColumn: '1 / -1' }}>
              <label htmlFor="recipients" style={styles.label}>
                Recipients (comma-separated emails)
              </label>
              <input
                id="recipients"
                type="text"
                value={formData.recipients}
                onChange={(e) => setFormData({ ...formData, recipients: e.target.value })}
                placeholder="user1@example.com, user2@example.com"
                style={styles.input}
              />
            </div>

            {/* Export Formats */}
            <div style={{ ...styles.formGroup, gridColumn: '1 / -1' }}>
              <label style={styles.label}>Export Formats</label>
              <div style={styles.formatCheckboxes}>
                {(['pdf', 'excel', 'html', 'json'] as ExportFormat[]).map((format) => (
                  <label key={format} style={styles.checkboxLabel}>
                    <input
                      type="checkbox"
                      checked={formData.formats.includes(format)}
                      onChange={() => handleToggleFormat(format)}
                      style={styles.checkbox}
                    />
                    <span>{format.toUpperCase()}</span>
                  </label>
                ))}
              </div>
            </div>
          </div>

          <div style={styles.formActions}>
            <button
              onClick={() => {
                setIsCreating(false);
                setEditingId(null);
              }}
              style={styles.cancelButton}
            >
              Cancel
            </button>
            <button onClick={handleCreate} style={styles.saveButton}>
              {isCreating ? 'Create Schedule' : 'Save Changes'}
            </button>
          </div>
        </div>
      )}

      {/* Schedules List */}
      <div style={styles.schedulesList}>
        {schedules.length === 0 ? (
          <div style={styles.emptyState}>
            <p style={styles.emptyText}>No scheduled exports configured.</p>
            <p style={styles.emptySubtext}>
              Create a schedule to automatically generate and send reports.
            </p>
          </div>
        ) : (
          <ul style={styles.list} role="list">
            {schedules.map((schedule) => (
              <li key={schedule.id} style={styles.scheduleCard}>
                <div style={styles.scheduleHeader}>
                  <div style={styles.scheduleInfo}>
                    <h4 style={styles.scheduleName}>{schedule.reportConfig.title}</h4>
                    <div style={styles.scheduleDetails}>
                      <span style={styles.frequency}>
                        {schedule.schedule.frequency.charAt(0).toUpperCase() +
                          schedule.schedule.frequency.slice(1)}
                      </span>
                      {schedule.schedule.dayOfWeek !== undefined && (
                        <span>on {getDayName(schedule.schedule.dayOfWeek)}</span>
                      )}
                      <span>at {schedule.schedule.time}</span>
                    </div>
                  </div>

                  <div style={styles.scheduleStatus}>
                    <label style={styles.toggleSwitch}>
                      <input
                        type="checkbox"
                        checked={schedule.enabled}
                        onChange={(e) => onToggle?.(schedule.id, e.target.checked)}
                        style={styles.toggleInput}
                        aria-label={`${schedule.enabled ? 'Disable' : 'Enable'} schedule`}
                      />
                      <span
                        style={{
                          ...styles.toggleSlider,
                          ...(schedule.enabled ? styles.toggleSliderActive : {}),
                        }}
                      />
                    </label>
                    <span
                      style={{
                        ...styles.statusBadge,
                        ...(schedule.enabled ? styles.statusBadgeActive : styles.statusBadgeInactive),
                      }}
                    >
                      {schedule.enabled ? 'Active' : 'Inactive'}
                    </span>
                  </div>
                </div>

                <div style={styles.scheduleBody}>
                  <div style={styles.scheduleMetadata}>
                    <div>
                      <strong>Recipients:</strong> {schedule.recipients.length}
                    </div>
                    <div>
                      <strong>Formats:</strong> {schedule.formats.join(', ').toUpperCase()}
                    </div>
                    {schedule.lastRun && (
                      <div>
                        <strong>Last run:</strong> {schedule.lastRun.toLocaleDateString()}
                      </div>
                    )}
                    <div>
                      <strong>Next run:</strong> {getNextRunDisplay(schedule.nextRun)}
                    </div>
                  </div>

                  <div style={styles.recipientsList}>
                    <strong>Email to:</strong>
                    <ul style={styles.emailList}>
                      {schedule.recipients.slice(0, 3).map((email, idx) => (
                        <li key={idx} style={styles.emailItem}>
                          {email}
                        </li>
                      ))}
                      {schedule.recipients.length > 3 && (
                        <li style={styles.emailMore}>
                          +{schedule.recipients.length - 3} more
                        </li>
                      )}
                    </ul>
                  </div>
                </div>

                <div style={styles.scheduleActions}>
                  <button
                    onClick={() => setEditingId(schedule.id)}
                    style={styles.actionButton}
                    aria-label="Edit schedule"
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => onDelete?.(schedule.id)}
                    style={{ ...styles.actionButton, ...styles.deleteButton }}
                    aria-label="Delete schedule"
                  >
                    Delete
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
};

const styles = {
  container: {
    padding: '2rem',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '2rem',
  },
  title: {
    margin: 0,
    fontSize: '1.75rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  addButton: {
    padding: '0.75rem 1.5rem',
    fontSize: '1rem',
    fontWeight: 'bold' as const,
    border: 'none',
    borderRadius: '4px',
    backgroundColor: '#0066cc',
    color: '#fff',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  },
  formCard: {
    backgroundColor: '#f9f9f9',
    padding: '2rem',
    borderRadius: '8px',
    border: '2px solid #0066cc',
    marginBottom: '2rem',
  },
  formTitle: {
    margin: '0 0 1.5rem 0',
    fontSize: '1.25rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  formGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(2, 1fr)',
    gap: '1.5rem',
    marginBottom: '1.5rem',
  },
  formGroup: {
    display: 'flex',
    flexDirection: 'column' as const,
  },
  label: {
    marginBottom: '0.5rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  select: {
    padding: '0.75rem',
    fontSize: '1rem',
    border: '1px solid #ccc',
    borderRadius: '4px',
  },
  input: {
    padding: '0.75rem',
    fontSize: '1rem',
    border: '1px solid #ccc',
    borderRadius: '4px',
  },
  formatCheckboxes: {
    display: 'flex',
    gap: '1.5rem',
    flexWrap: 'wrap' as const,
  },
  checkboxLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    cursor: 'pointer',
  },
  checkbox: {
    width: '18px',
    height: '18px',
    cursor: 'pointer',
  },
  formActions: {
    display: 'flex',
    justifyContent: 'flex-end',
    gap: '1rem',
  },
  cancelButton: {
    padding: '0.75rem 1.5rem',
    fontSize: '1rem',
    border: '1px solid #ccc',
    borderRadius: '4px',
    backgroundColor: '#fff',
    cursor: 'pointer',
  },
  saveButton: {
    padding: '0.75rem 1.5rem',
    fontSize: '1rem',
    fontWeight: 'bold' as const,
    border: 'none',
    borderRadius: '4px',
    backgroundColor: '#28a745',
    color: '#fff',
    cursor: 'pointer',
  },
  schedulesList: {
    marginTop: '2rem',
  },
  list: {
    listStyle: 'none',
    padding: 0,
    margin: 0,
  },
  scheduleCard: {
    backgroundColor: '#fff',
    padding: '1.5rem',
    borderRadius: '8px',
    border: '1px solid #e0e0e0',
    marginBottom: '1rem',
    boxShadow: '0 2px 4px rgba(0,0,0,0.05)',
  },
  scheduleHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: '1rem',
    paddingBottom: '1rem',
    borderBottom: '1px solid #e0e0e0',
  },
  scheduleInfo: {
    flex: 1,
  },
  scheduleName: {
    margin: '0 0 0.5rem 0',
    fontSize: '1.25rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  scheduleDetails: {
    display: 'flex',
    gap: '1rem',
    fontSize: '0.875rem',
    color: '#666',
  },
  frequency: {
    fontWeight: 'bold' as const,
    color: '#0066cc',
  },
  scheduleStatus: {
    display: 'flex',
    alignItems: 'center',
    gap: '1rem',
  },
  toggleSwitch: {
    position: 'relative' as const,
    display: 'inline-block',
    width: '50px',
    height: '24px',
  },
  toggleInput: {
    opacity: 0,
    width: 0,
    height: 0,
  },
  toggleSlider: {
    position: 'absolute' as const,
    cursor: 'pointer',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: '#ccc',
    borderRadius: '24px',
    transition: '0.3s',
  },
  toggleSliderActive: {
    backgroundColor: '#28a745',
  },
  statusBadge: {
    padding: '0.25rem 0.75rem',
    borderRadius: '12px',
    fontSize: '0.875rem',
    fontWeight: 'bold' as const,
  },
  statusBadgeActive: {
    backgroundColor: '#d4edda',
    color: '#28a745',
  },
  statusBadgeInactive: {
    backgroundColor: '#f8d7da',
    color: '#dc3545',
  },
  scheduleBody: {
    marginBottom: '1rem',
  },
  scheduleMetadata: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '1rem',
    marginBottom: '1rem',
    fontSize: '0.875rem',
    color: '#666',
  },
  recipientsList: {
    fontSize: '0.875rem',
  },
  emailList: {
    listStyle: 'none',
    padding: '0.5rem 0',
    margin: 0,
  },
  emailItem: {
    padding: '0.25rem 0',
    color: '#666',
  },
  emailMore: {
    padding: '0.25rem 0',
    color: '#999',
    fontStyle: 'italic' as const,
  },
  scheduleActions: {
    display: 'flex',
    gap: '0.75rem',
    justifyContent: 'flex-end',
  },
  actionButton: {
    padding: '0.5rem 1rem',
    fontSize: '0.875rem',
    border: '1px solid #ccc',
    borderRadius: '4px',
    backgroundColor: '#fff',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  },
  deleteButton: {
    borderColor: '#dc3545',
    color: '#dc3545',
  },
  emptyState: {
    padding: '3rem',
    textAlign: 'center' as const,
    backgroundColor: '#f9f9f9',
    borderRadius: '8px',
    border: '2px dashed #e0e0e0',
  },
  emptyText: {
    margin: '0 0 0.5rem 0',
    fontSize: '1.125rem',
    fontWeight: 'bold' as const,
    color: '#666',
  },
  emptySubtext: {
    margin: 0,
    fontSize: '0.875rem',
    color: '#999',
  },
};
