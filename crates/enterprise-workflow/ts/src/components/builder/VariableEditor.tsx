/**
 * Variable Editor - Manage workflow variables
 */

import React, { useState } from 'react';
import { Variable, VariableType } from '../../types';

export interface VariableEditorProps {
  variables: Variable[];
  onChange?: (variables: Variable[]) => void;
}

export const VariableEditor: React.FC<VariableEditorProps> = ({
  variables,
  onChange
}) => {
  const [editingVar, setEditingVar] = useState<Variable | null>(null);

  const handleAdd = () => {
    const newVar: Variable = {
      id: `var_${Date.now()}`,
      name: 'newVariable',
      type: 'string',
      value: '',
      required: false
    };
    onChange?.([...variables, newVar]);
  };

  const handleUpdate = (id: string, updates: Partial<Variable>) => {
    onChange?.(
      variables.map(v => v.id === id ? { ...v, ...updates } : v)
    );
  };

  const handleDelete = (id: string) => {
    onChange?.(variables.filter(v => v.id !== id));
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Variables</h3>
        <button onClick={handleAdd} style={styles.addButton}>
          + Add Variable
        </button>
      </div>

      <div style={styles.list}>
        {variables.map(variable => (
          <div key={variable.id} style={styles.variableCard}>
            <div style={styles.variableHeader}>
              <input
                type="text"
                value={variable.name}
                onChange={(e) => handleUpdate(variable.id, { name: e.target.value })}
                style={styles.nameInput}
                placeholder="Variable name"
              />
              <button
                onClick={() => handleDelete(variable.id)}
                style={styles.deleteButton}
              >
                ×
              </button>
            </div>

            <div style={styles.variableBody}>
              <div style={styles.row}>
                <select
                  value={variable.type}
                  onChange={(e) => handleUpdate(variable.id, { type: e.target.value as VariableType })}
                  style={styles.select}
                >
                  <option value="string">String</option>
                  <option value="number">Number</option>
                  <option value="boolean">Boolean</option>
                  <option value="object">Object</option>
                  <option value="array">Array</option>
                  <option value="date">Date</option>
                </select>

                <label style={styles.checkbox}>
                  <input
                    type="checkbox"
                    checked={variable.required || false}
                    onChange={(e) => handleUpdate(variable.id, { required: e.target.checked })}
                  />
                  Required
                </label>
              </div>

              <input
                type="text"
                value={variable.value || ''}
                onChange={(e) => handleUpdate(variable.id, { value: e.target.value })}
                style={styles.input}
                placeholder="Default value"
              />

              {variable.description !== undefined && (
                <textarea
                  value={variable.description}
                  onChange={(e) => handleUpdate(variable.id, { description: e.target.value })}
                  style={styles.textarea}
                  placeholder="Description"
                  rows={2}
                />
              )}
            </div>
          </div>
        ))}

        {variables.length === 0 && (
          <div style={styles.emptyState}>
            No variables defined. Click "+ Add Variable" to create one.
          </div>
        )}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%'
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
  addButton: {
    padding: '8px 16px',
    backgroundColor: '#2563eb',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '14px'
  },
  list: {
    flex: 1,
    overflowY: 'auto',
    padding: '16px'
  },
  variableCard: {
    marginBottom: '12px',
    padding: '12px',
    backgroundColor: '#f9f9f9',
    border: '1px solid #ddd',
    borderRadius: '6px'
  },
  variableHeader: {
    display: 'flex',
    gap: '8px',
    marginBottom: '12px'
  },
  nameInput: {
    flex: 1,
    padding: '8px 12px',
    border: '1px solid #ddd',
    borderRadius: '4px',
    fontSize: '14px',
    fontWeight: 500
  },
  deleteButton: {
    padding: '4px 12px',
    backgroundColor: '#ef4444',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '18px'
  },
  variableBody: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px'
  },
  row: {
    display: 'flex',
    gap: '8px',
    alignItems: 'center'
  },
  select: {
    flex: 1,
    padding: '8px 12px',
    border: '1px solid #ddd',
    borderRadius: '4px',
    fontSize: '14px'
  },
  checkbox: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    fontSize: '14px',
    whiteSpace: 'nowrap'
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
    boxSizing: 'border-box',
    resize: 'vertical'
  },
  emptyState: {
    padding: '40px 16px',
    textAlign: 'center',
    color: '#999'
  }
};
