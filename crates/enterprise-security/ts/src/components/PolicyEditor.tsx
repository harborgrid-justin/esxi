/**
 * Policy Editor - Security Policy Management UI
 * Create and edit security policies with rule builder
 */

import React, { useState } from 'react';
import { SecurityPolicy, SecurityPolicyType, PolicyEnforcement, PolicyAction } from '../types';

export interface PolicyEditorProps {
  policy?: SecurityPolicy;
  onSave: (policy: Partial<SecurityPolicy>) => void;
  onCancel: () => void;
}

export const PolicyEditor: React.FC<PolicyEditorProps> = ({ policy, onSave, onCancel }) => {
  const [name, setName] = useState(policy?.name || '');
  const [type, setType] = useState<SecurityPolicyType>(policy?.type || SecurityPolicyType.ACCESS_CONTROL);
  const [enforcement, setEnforcement] = useState<PolicyEnforcement>(policy?.enforcement || PolicyEnforcement.ENFORCE);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave({
      name,
      type,
      enforcement,
      rules: policy?.rules || [],
      metadata: policy?.metadata || {},
    });
  };

  return (
    <div className="policy-editor">
      <h2>{policy ? 'Edit Policy' : 'Create Policy'}</h2>
      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label>Policy Name</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Enter policy name"
            required
          />
        </div>

        <div className="form-group">
          <label>Policy Type</label>
          <select value={type} onChange={(e) => setType(e.target.value as SecurityPolicyType)}>
            <option value={SecurityPolicyType.PASSWORD}>Password</option>
            <option value={SecurityPolicyType.SESSION}>Session</option>
            <option value={SecurityPolicyType.ACCESS_CONTROL}>Access Control</option>
            <option value={SecurityPolicyType.DATA_PROTECTION}>Data Protection</option>
            <option value={SecurityPolicyType.NETWORK}>Network</option>
            <option value={SecurityPolicyType.API}>API</option>
            <option value={SecurityPolicyType.ENCRYPTION}>Encryption</option>
          </select>
        </div>

        <div className="form-group">
          <label>Enforcement</label>
          <select value={enforcement} onChange={(e) => setEnforcement(e.target.value as PolicyEnforcement)}>
            <option value={PolicyEnforcement.ENFORCE}>Enforce</option>
            <option value={PolicyEnforcement.AUDIT}>Audit Only</option>
            <option value={PolicyEnforcement.DISABLED}>Disabled</option>
          </select>
        </div>

        <div className="form-actions">
          <button type="button" onClick={onCancel}>Cancel</button>
          <button type="submit">Save Policy</button>
        </div>
      </form>

      <style>{`
        .policy-editor {
          background: #fff;
          padding: 24px;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0,0,0,0.1);
          max-width: 600px;
        }
        .form-group {
          margin-bottom: 20px;
        }
        .form-group label {
          display: block;
          margin-bottom: 8px;
          font-weight: 600;
        }
        .form-group input,
        .form-group select {
          width: 100%;
          padding: 10px;
          border: 1px solid #ddd;
          border-radius: 4px;
        }
        .form-actions {
          display: flex;
          gap: 12px;
          justify-content: flex-end;
        }
        button {
          padding: 10px 20px;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }
        button[type="submit"] {
          background: #2196f3;
          color: white;
        }
      `}</style>
    </div>
  );
};
