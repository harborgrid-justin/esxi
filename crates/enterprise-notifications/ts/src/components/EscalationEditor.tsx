/**
 * EscalationEditor - Escalation policy editor
 */

import React, { useState } from 'react';
import { EscalationPolicy, EscalationLevel } from '../types';

export interface EscalationEditorProps {
  policy?: EscalationPolicy;
  onSave: (policy: Partial<EscalationPolicy>) => void;
  onCancel?: () => void;
  className?: string;
}

export const EscalationEditor: React.FC<EscalationEditorProps> = ({
  policy,
  onSave,
  onCancel,
  className = '',
}) => {
  const [name, setName] = useState(policy?.name ?? '');
  const [description, setDescription] = useState(policy?.description ?? '');
  const [levels, setLevels] = useState<EscalationLevel[]>(policy?.levels ?? []);
  const [enabled, setEnabled] = useState(policy?.enabled ?? true);

  const handleAddLevel = () => {
    setLevels([
      ...levels,
      {
        level: levels.length,
        delayMinutes: 15,
        actions: [],
        recipients: [],
        channels: [],
      },
    ]);
  };

  const handleSave = () => {
    onSave({
      ...policy,
      name,
      description,
      levels,
      enabled,
    });
  };

  return (
    <div className={`escalation-editor ${className}`}>
      <h2>{policy ? 'Edit Escalation Policy' : 'Create Escalation Policy'}</h2>

      <div className="form-group">
        <label>Policy Name</label>
        <input
          type="text"
          value={name}
          onChange={e => setName(e.target.value)}
        />
      </div>

      <div className="form-group">
        <label>Description</label>
        <textarea
          value={description}
          onChange={e => setDescription(e.target.value)}
        />
      </div>

      <div className="levels-section">
        <h3>Escalation Levels</h3>
        {levels.map((level, index) => (
          <div key={index} className="level-row">
            <h4>Level {level.level + 1}</h4>
            <label>
              Delay (minutes):
              <input
                type="number"
                value={level.delayMinutes}
                onChange={e => {
                  const updated = [...levels];
                  updated[index] = { ...level, delayMinutes: parseInt(e.target.value) };
                  setLevels(updated);
                }}
              />
            </label>
            <button onClick={() => setLevels(levels.filter((_, i) => i !== index))}>
              Remove Level
            </button>
          </div>
        ))}
        <button onClick={handleAddLevel}>Add Level</button>
      </div>

      <div className="form-group">
        <label>
          <input
            type="checkbox"
            checked={enabled}
            onChange={e => setEnabled(e.target.checked)}
          />
          Enabled
        </label>
      </div>

      <div className="actions">
        <button onClick={handleSave} className="primary">Save</button>
        {onCancel && <button onClick={onCancel} className="secondary">Cancel</button>}
      </div>
    </div>
  );
};

export default EscalationEditor;
