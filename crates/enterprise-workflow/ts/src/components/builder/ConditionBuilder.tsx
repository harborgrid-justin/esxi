/**
 * Condition Builder - Visual condition editor
 */

import React from 'react';
import { Condition, ConditionOperator, LogicalOperator } from '../../types';

export interface ConditionBuilderProps {
  condition?: Condition;
  onChange?: (condition: Condition) => void;
}

export const ConditionBuilder: React.FC<ConditionBuilderProps> = ({
  condition,
  onChange
}) => {
  const handleUpdate = (updates: Partial<Condition>) => {
    if (condition) {
      onChange?.({ ...condition, ...updates });
    }
  };

  if (!condition) {
    const newCondition: Condition = {
      id: `cond_${Date.now()}`,
      type: 'simple',
      operator: ConditionOperator.EQUALS
    };
    onChange?.(newCondition);
    return null;
  }

  return (
    <div style={styles.container}>
      <div style={styles.typeSelector}>
        <label style={styles.radio}>
          <input
            type="radio"
            checked={condition.type === 'simple'}
            onChange={() => handleUpdate({ type: 'simple', conditions: undefined })}
          />
          Simple Condition
        </label>
        <label style={styles.radio}>
          <input
            type="radio"
            checked={condition.type === 'composite'}
            onChange={() => handleUpdate({
              type: 'composite',
              operator: undefined,
              conditions: []
            })}
          />
          Composite Condition
        </label>
      </div>

      {condition.type === 'simple' && (
        <SimpleConditionEditor condition={condition} onChange={onChange} />
      )}

      {condition.type === 'composite' && (
        <CompositeConditionEditor condition={condition} onChange={onChange} />
      )}
    </div>
  );
};

const SimpleConditionEditor: React.FC<{
  condition: Condition;
  onChange?: (condition: Condition) => void;
}> = ({ condition, onChange }) => {
  return (
    <div style={styles.editor}>
      <div style={styles.field}>
        <label style={styles.label}>Left Operand</label>
        <input
          type="text"
          value={typeof condition.left === 'string' ? condition.left : ''}
          onChange={(e) => onChange?.({ ...condition, left: e.target.value })}
          style={styles.input}
          placeholder="Variable or value"
        />
      </div>

      <div style={styles.field}>
        <label style={styles.label}>Operator</label>
        <select
          value={condition.operator}
          onChange={(e) => onChange?.({
            ...condition,
            operator: e.target.value as ConditionOperator
          })}
          style={styles.select}
        >
          <option value={ConditionOperator.EQUALS}>Equals (==)</option>
          <option value={ConditionOperator.NOT_EQUALS}>Not Equals (!=)</option>
          <option value={ConditionOperator.GREATER_THAN}>Greater Than (&gt;)</option>
          <option value={ConditionOperator.LESS_THAN}>Less Than (&lt;)</option>
          <option value={ConditionOperator.GREATER_THAN_OR_EQUAL}>Greater or Equal (&gt;=)</option>
          <option value={ConditionOperator.LESS_THAN_OR_EQUAL}>Less or Equal (&lt;=)</option>
          <option value={ConditionOperator.CONTAINS}>Contains</option>
          <option value={ConditionOperator.STARTS_WITH}>Starts With</option>
          <option value={ConditionOperator.ENDS_WITH}>Ends With</option>
          <option value={ConditionOperator.MATCHES_REGEX}>Matches Regex</option>
          <option value={ConditionOperator.IN}>In</option>
          <option value={ConditionOperator.IS_NULL}>Is Null</option>
          <option value={ConditionOperator.IS_NOT_NULL}>Is Not Null</option>
        </select>
      </div>

      {condition.operator !== ConditionOperator.IS_NULL &&
       condition.operator !== ConditionOperator.IS_NOT_NULL && (
        <div style={styles.field}>
          <label style={styles.label}>Right Operand</label>
          <input
            type="text"
            value={typeof condition.right === 'string' ? condition.right : ''}
            onChange={(e) => onChange?.({ ...condition, right: e.target.value })}
            style={styles.input}
            placeholder="Variable or value"
          />
        </div>
      )}
    </div>
  );
};

const CompositeConditionEditor: React.FC<{
  condition: Condition;
  onChange?: (condition: Condition) => void;
}> = ({ condition, onChange }) => {
  const handleAddCondition = () => {
    const newCondition: Condition = {
      id: `cond_${Date.now()}`,
      type: 'simple',
      operator: ConditionOperator.EQUALS
    };

    onChange?.({
      ...condition,
      conditions: [...(condition.conditions || []), newCondition]
    });
  };

  return (
    <div style={styles.editor}>
      <div style={styles.field}>
        <label style={styles.label}>Logical Operator</label>
        <select
          value={condition.logicalOperator}
          onChange={(e) => onChange?.({
            ...condition,
            logicalOperator: e.target.value as LogicalOperator
          })}
          style={styles.select}
        >
          <option value={LogicalOperator.AND}>AND</option>
          <option value={LogicalOperator.OR}>OR</option>
          <option value={LogicalOperator.NOT}>NOT</option>
        </select>
      </div>

      <div style={styles.conditions}>
        {(condition.conditions || []).map((subCondition, index) => (
          <div key={subCondition.id} style={styles.subCondition}>
            <div style={styles.subConditionHeader}>
              Condition {index + 1}
            </div>
            <SimpleConditionEditor
              condition={subCondition}
              onChange={(updated) => {
                const conditions = [...(condition.conditions || [])];
                conditions[index] = updated;
                onChange?.({ ...condition, conditions });
              }}
            />
          </div>
        ))}
      </div>

      <button onClick={handleAddCondition} style={styles.addButton}>
        + Add Condition
      </button>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    padding: '16px'
  },
  typeSelector: {
    display: 'flex',
    gap: '16px',
    marginBottom: '16px',
    paddingBottom: '16px',
    borderBottom: '1px solid #ddd'
  },
  radio: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '14px',
    cursor: 'pointer'
  },
  editor: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px'
  },
  field: {
    display: 'flex',
    flexDirection: 'column',
    gap: '6px'
  },
  label: {
    fontSize: '14px',
    fontWeight: 500,
    color: '#333'
  },
  input: {
    padding: '8px 12px',
    border: '1px solid #ddd',
    borderRadius: '4px',
    fontSize: '14px'
  },
  select: {
    padding: '8px 12px',
    border: '1px solid #ddd',
    borderRadius: '4px',
    fontSize: '14px'
  },
  conditions: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
    marginTop: '12px'
  },
  subCondition: {
    padding: '12px',
    backgroundColor: '#f9f9f9',
    border: '1px solid #ddd',
    borderRadius: '6px'
  },
  subConditionHeader: {
    fontSize: '12px',
    fontWeight: 600,
    color: '#666',
    marginBottom: '8px'
  },
  addButton: {
    padding: '8px 16px',
    backgroundColor: '#2563eb',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '14px',
    marginTop: '8px'
  }
};
