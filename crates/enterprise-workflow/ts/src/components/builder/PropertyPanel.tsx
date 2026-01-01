/**
 * Property Panel - Node configuration panel
 */

import React, { useState } from 'react';
import { WorkflowStep, Action, ActionType } from '../../types';

export interface PropertyPanelProps {
  step?: WorkflowStep;
  onStepChange?: (step: WorkflowStep) => void;
  onClose?: () => void;
}

export const PropertyPanel: React.FC<PropertyPanelProps> = ({
  step,
  onStepChange,
  onClose
}) => {
  const [formData, setFormData] = useState(step || {} as WorkflowStep);

  const handleChange = (field: string, value: any) => {
    const updated = { ...formData, [field]: value };
    setFormData(updated);
    onStepChange?.(updated);
  };

  if (!step) {
    return (
      <div style={styles.container}>
        <div style={styles.emptyState}>
          Select a node to edit its properties
        </div>
      </div>
    );
  }

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Node Properties</h3>
        {onClose && (
          <button onClick={onClose} style={styles.closeButton}>×</button>
        )}
      </div>

      <div style={styles.content}>
        <div style={styles.field}>
          <label style={styles.label}>Name</label>
          <input
            type="text"
            value={formData.name || ''}
            onChange={(e) => handleChange('name', e.target.value)}
            style={styles.input}
            placeholder="Step name"
          />
        </div>

        <div style={styles.field}>
          <label style={styles.label}>Description</label>
          <textarea
            value={formData.description || ''}
            onChange={(e) => handleChange('description', e.target.value)}
            style={styles.textarea}
            placeholder="Step description"
            rows={3}
          />
        </div>

        {step.type === 'action' && step.action && (
          <ActionProperties
            action={step.action}
            onChange={(action) => handleChange('action', action)}
          />
        )}

        {step.type === 'wait' && step.waitConfig && (
          <div style={styles.field}>
            <label style={styles.label}>Wait Type</label>
            <select
              value={step.waitConfig.type}
              onChange={(e) => handleChange('waitConfig', {
                ...step.waitConfig,
                type: e.target.value
              })}
              style={styles.select}
            >
              <option value="duration">Duration</option>
              <option value="until">Until Time</option>
              <option value="event">Wait for Event</option>
            </select>
          </div>
        )}
      </div>
    </div>
  );
};

const ActionProperties: React.FC<{
  action: Action;
  onChange: (action: Action) => void;
}> = ({ action, onChange }) => {
  return (
    <div style={styles.section}>
      <h4 style={styles.sectionTitle}>Action Configuration</h4>

      <div style={styles.field}>
        <label style={styles.label}>Action Type</label>
        <select
          value={action.type}
          onChange={(e) => onChange({ ...action, type: e.target.value as ActionType })}
          style={styles.select}
        >
          <option value={ActionType.HTTP}>HTTP Request</option>
          <option value={ActionType.EMAIL}>Email</option>
          <option value={ActionType.NOTIFICATION}>Notification</option>
          <option value={ActionType.DATABASE}>Database</option>
          <option value={ActionType.TRANSFORM}>Transform</option>
          <option value={ActionType.APPROVAL}>Approval</option>
        </select>
      </div>

      <div style={styles.field}>
        <label style={styles.label}>Configuration (JSON)</label>
        <textarea
          value={JSON.stringify(action.config, null, 2)}
          onChange={(e) => {
            try {
              const config = JSON.parse(e.target.value);
              onChange({ ...action, config });
            } catch {}
          }}
          style={styles.textarea}
          rows={10}
          placeholder="{}"
        />
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    width: '350px',
    height: '100%',
    backgroundColor: '#fff',
    borderLeft: '1px solid #ddd',
    display: 'flex',
    flexDirection: 'column'
  },
  header: {
    padding: '16px',
    borderBottom: '1px solid #ddd',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center'
  },
  title: {
    margin: 0,
    fontSize: '16px',
    fontWeight: 600
  },
  closeButton: {
    border: 'none',
    background: 'none',
    fontSize: '24px',
    cursor: 'pointer',
    padding: '0 8px'
  },
  content: {
    flex: 1,
    overflowY: 'auto',
    padding: '16px'
  },
  emptyState: {
    padding: '40px 16px',
    textAlign: 'center',
    color: '#999'
  },
  field: {
    marginBottom: '16px'
  },
  label: {
    display: 'block',
    marginBottom: '6px',
    fontSize: '14px',
    fontWeight: 500,
    color: '#333'
  },
  input: {
    width: '100%',
    padding: '8px 12px',
    border: '1px solid #ddd',
    borderRadius: '4px',
    fontSize: '14px',
    boxSizing: 'border-box'
  },
  textarea: {
    width: '100%',
    padding: '8px 12px',
    border: '1px solid #ddd',
    borderRadius: '4px',
    fontSize: '14px',
    fontFamily: 'monospace',
    boxSizing: 'border-box',
    resize: 'vertical'
  },
  select: {
    width: '100%',
    padding: '8px 12px',
    border: '1px solid #ddd',
    borderRadius: '4px',
    fontSize: '14px',
    boxSizing: 'border-box'
  },
  section: {
    marginTop: '24px',
    paddingTop: '16px',
    borderTop: '1px solid #eee'
  },
  sectionTitle: {
    margin: '0 0 16px 0',
    fontSize: '14px',
    fontWeight: 600,
    color: '#666'
  }
};
