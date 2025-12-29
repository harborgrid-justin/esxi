/**
 * ARIA Validator Component
 * Main ARIA validation UI component
 */

import React, { useState } from 'react';
import { ValidationResult } from '../../types';
import { useARIAValidation } from '../../hooks/useARIAValidation';
import { RoleChecker } from './RoleChecker';
import { AttributeChecker } from './AttributeChecker';
import { StateChecker } from './StateChecker';

export interface ARIAValidatorProps {
  target?: HTMLElement | Document;
  autoValidate?: boolean;
  onValidation?: (result: ValidationResult) => void;
}

export const ARIAValidator: React.FC<ARIAValidatorProps> = ({
  target,
  autoValidate = false,
  onValidation,
}) => {
  const { validate, isValidating, lastResult } = useARIAValidation();
  const [activeTab, setActiveTab] = useState<'roles' | 'attributes' | 'states'>('roles');

  React.useEffect(() => {
    if (autoValidate && target) {
      const result = validate(target);
      onValidation?.(result);
    }
  }, [target, autoValidate, validate, onValidation]);

  const handleValidate = () => {
    if (target) {
      const result = validate(target);
      onValidation?.(result);
    }
  };

  return (
    <div className="aria-validator">
      <div className="aria-validator__header">
        <h2>ARIA Validator</h2>
        <button
          onClick={handleValidate}
          disabled={!target || isValidating}
          className="aria-validator__validate-btn"
        >
          {isValidating ? 'Validating...' : 'Validate'}
        </button>
      </div>

      <div className="aria-validator__tabs">
        <button
          onClick={() => setActiveTab('roles')}
          className={`tab ${activeTab === 'roles' ? 'active' : ''}`}
        >
          Roles
        </button>
        <button
          onClick={() => setActiveTab('attributes')}
          className={`tab ${activeTab === 'attributes' ? 'active' : ''}`}
        >
          Attributes
        </button>
        <button
          onClick={() => setActiveTab('states')}
          className={`tab ${activeTab === 'states' ? 'active' : ''}`}
        >
          States
        </button>
      </div>

      <div className="aria-validator__content">
        {activeTab === 'roles' && <RoleChecker target={target} result={lastResult} />}
        {activeTab === 'attributes' && <AttributeChecker target={target} result={lastResult} />}
        {activeTab === 'states' && <StateChecker target={target} result={lastResult} />}
      </div>

      {lastResult && (
        <div className="aria-validator__summary">
          <div className={`summary-item ${lastResult.valid ? 'valid' : 'invalid'}`}>
            <strong>Status:</strong> {lastResult.valid ? 'Valid' : 'Invalid'}
          </div>
          <div className="summary-item error">
            <strong>Errors:</strong> {lastResult.errors.length}
          </div>
          <div className="summary-item warning">
            <strong>Warnings:</strong> {lastResult.warnings.length}
          </div>
          <div className="summary-item info">
            <strong>Info:</strong> {lastResult.info.length}
          </div>
        </div>
      )}
    </div>
  );
};
