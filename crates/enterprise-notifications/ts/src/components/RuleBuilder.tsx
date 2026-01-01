/**
 * RuleBuilder - Alert rule builder UI
 */

import React, { useState } from 'react';
import { AlertRule, RuleCondition, Threshold, AlertSeverity, RuleConditionOperator } from '../types';

export interface RuleBuilderProps {
  rule?: AlertRule;
  onSave: (rule: Partial<AlertRule>) => void;
  onCancel?: () => void;
  className?: string;
}

export const RuleBuilder: React.FC<RuleBuilderProps> = ({
  rule,
  onSave,
  onCancel,
  className = '',
}) => {
  const [name, setName] = useState(rule?.name ?? '');
  const [description, setDescription] = useState(rule?.description ?? '');
  const [severity, setSeverity] = useState(rule?.severity ?? AlertSeverity.WARNING);
  const [conditions, setConditions] = useState<RuleCondition[]>(rule?.conditions ?? []);
  const [thresholds, setThresholds] = useState<Threshold[]>(rule?.thresholds ?? []);
  const [enabled, setEnabled] = useState(rule?.enabled ?? true);

  const handleAddCondition = () => {
    setConditions([
      ...conditions,
      {
        id: `cond_${Date.now()}`,
        field: '',
        operator: RuleConditionOperator.EQUALS,
        value: '',
        valueType: 'static',
      },
    ]);
  };

  const handleAddThreshold = () => {
    setThresholds([
      ...thresholds,
      {
        id: `thresh_${Date.now()}`,
        name: '',
        type: 'static',
        metric: '',
        operator: RuleConditionOperator.GREATER_THAN,
        value: 0,
      },
    ]);
  };

  const handleSave = () => {
    onSave({
      ...rule,
      name,
      description,
      severity,
      conditions,
      thresholds,
      enabled,
    });
  };

  return (
    <div className={`rule-builder ${className}`}>
      <h2>{rule ? 'Edit Rule' : 'Create New Rule'}</h2>

      <div className="form-group">
        <label>Rule Name</label>
        <input
          type="text"
          value={name}
          onChange={e => setName(e.target.value)}
          placeholder="Enter rule name"
        />
      </div>

      <div className="form-group">
        <label>Description</label>
        <textarea
          value={description}
          onChange={e => setDescription(e.target.value)}
          placeholder="Enter description"
        />
      </div>

      <div className="form-group">
        <label>Severity</label>
        <select value={severity} onChange={e => setSeverity(e.target.value as AlertSeverity)}>
          <option value={AlertSeverity.INFO}>Info</option>
          <option value={AlertSeverity.WARNING}>Warning</option>
          <option value={AlertSeverity.ERROR}>Error</option>
          <option value={AlertSeverity.CRITICAL}>Critical</option>
          <option value={AlertSeverity.FATAL}>Fatal</option>
        </select>
      </div>

      {/* Conditions */}
      <div className="conditions-section">
        <h3>Conditions</h3>
        {conditions.map((condition, index) => (
          <div key={condition.id} className="condition-row">
            <input
              type="text"
              placeholder="Field"
              value={condition.field}
              onChange={e => {
                const updated = [...conditions];
                updated[index] = { ...condition, field: e.target.value };
                setConditions(updated);
              }}
            />
            <select
              value={condition.operator}
              onChange={e => {
                const updated = [...conditions];
                updated[index] = { ...condition, operator: e.target.value as RuleConditionOperator };
                setConditions(updated);
              }}
            >
              <option value={RuleConditionOperator.EQUALS}>Equals</option>
              <option value={RuleConditionOperator.NOT_EQUALS}>Not Equals</option>
              <option value={RuleConditionOperator.GREATER_THAN}>Greater Than</option>
              <option value={RuleConditionOperator.LESS_THAN}>Less Than</option>
            </select>
            <input
              type="text"
              placeholder="Value"
              value={String(condition.value)}
              onChange={e => {
                const updated = [...conditions];
                updated[index] = { ...condition, value: e.target.value };
                setConditions(updated);
              }}
            />
            <button
              onClick={() => setConditions(conditions.filter((_, i) => i !== index))}
            >
              Remove
            </button>
          </div>
        ))}
        <button onClick={handleAddCondition}>Add Condition</button>
      </div>

      {/* Thresholds */}
      <div className="thresholds-section">
        <h3>Thresholds</h3>
        {thresholds.map((threshold, index) => (
          <div key={threshold.id} className="threshold-row">
            <input
              type="text"
              placeholder="Metric"
              value={threshold.metric}
              onChange={e => {
                const updated = [...thresholds];
                updated[index] = { ...threshold, metric: e.target.value };
                setThresholds(updated);
              }}
            />
            <select
              value={threshold.operator}
              onChange={e => {
                const updated = [...thresholds];
                updated[index] = { ...threshold, operator: e.target.value as RuleConditionOperator };
                setThresholds(updated);
              }}
            >
              <option value={RuleConditionOperator.GREATER_THAN}>{'>'}</option>
              <option value={RuleConditionOperator.LESS_THAN}>{'<'}</option>
              <option value={RuleConditionOperator.EQUALS}>{'='}</option>
            </select>
            <input
              type="number"
              placeholder="Value"
              value={threshold.value}
              onChange={e => {
                const updated = [...thresholds];
                updated[index] = { ...threshold, value: parseFloat(e.target.value) };
                setThresholds(updated);
              }}
            />
            <button
              onClick={() => setThresholds(thresholds.filter((_, i) => i !== index))}
            >
              Remove
            </button>
          </div>
        ))}
        <button onClick={handleAddThreshold}>Add Threshold</button>
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
        <button onClick={handleSave} className="primary">
          Save Rule
        </button>
        {onCancel && (
          <button onClick={onCancel} className="secondary">
            Cancel
          </button>
        )}
      </div>
    </div>
  );
};

export default RuleBuilder;
