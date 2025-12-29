/**
 * Accessible Component Fixtures
 * Examples of properly implemented accessible React components
 * Use these as reference implementations and for testing
 */

import React from 'react';

/**
 * Accessible Button Component
 * Demonstrates proper button semantics and keyboard handling
 */
export const AccessibleButton: React.FC<{
  onClick: () => void;
  children: React.ReactNode;
  disabled?: boolean;
  'aria-label'?: string;
}> = ({ onClick, children, disabled = false, 'aria-label': ariaLabel }) => {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      aria-label={ariaLabel}
      className="accessible-button"
    >
      {children}
    </button>
  );
};

/**
 * Accessible Form Input
 * Demonstrates proper label association and ARIA attributes
 */
export const AccessibleInput: React.FC<{
  id: string;
  label: string;
  type?: string;
  value: string;
  onChange: (value: string) => void;
  required?: boolean;
  error?: string;
}> = ({ id, label, type = 'text', value, onChange, required = false, error }) => {
  const errorId = error ? `${id}-error` : undefined;

  return (
    <div className="form-group">
      <label htmlFor={id}>
        {label}
        {required && <span aria-label="required"> *</span>}
      </label>
      <input
        id={id}
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        required={required}
        aria-invalid={!!error}
        aria-describedby={errorId}
        aria-required={required}
      />
      {error && (
        <div id={errorId} role="alert" className="error-message">
          {error}
        </div>
      )}
    </div>
  );
};

/**
 * Accessible Modal Dialog
 * Demonstrates proper modal semantics, focus management, and keyboard handling
 */
export const AccessibleModal: React.FC<{
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
}> = ({ isOpen, onClose, title, children }) => {
  const modalRef = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    if (isOpen && modalRef.current) {
      modalRef.current.focus();
    }
  }, [isOpen]);

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Escape') {
      onClose();
    }
  };

  if (!isOpen) {
    return null;
  }

  return (
    <div
      className="modal-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
      ref={modalRef}
      tabIndex={-1}
      onKeyDown={handleKeyDown}
    >
      <div className="modal-content">
        <header>
          <h2 id="modal-title">{title}</h2>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close modal"
            className="close-button"
          >
            ×
          </button>
        </header>
        <div className="modal-body">{children}</div>
      </div>
    </div>
  );
};

/**
 * Accessible Navigation Menu
 * Demonstrates proper navigation semantics and keyboard navigation
 */
export const AccessibleNav: React.FC<{
  items: Array<{ label: string; href: string; current?: boolean }>;
}> = ({ items }) => {
  return (
    <nav aria-label="Main navigation">
      <ul>
        {items.map((item, index) => (
          <li key={index}>
            <a
              href={item.href}
              aria-current={item.current ? 'page' : undefined}
            >
              {item.label}
            </a>
          </li>
        ))}
      </ul>
    </nav>
  );
};

/**
 * Accessible Image with Alternative Text
 */
export const AccessibleImage: React.FC<{
  src: string;
  alt: string;
  decorative?: boolean;
}> = ({ src, alt, decorative = false }) => {
  return (
    <img
      src={src}
      alt={decorative ? '' : alt}
      role={decorative ? 'presentation' : undefined}
    />
  );
};

/**
 * Accessible Accordion Component
 * Demonstrates proper disclosure widget implementation
 */
export const AccessibleAccordion: React.FC<{
  items: Array<{ title: string; content: React.ReactNode }>;
}> = ({ items }) => {
  const [expandedIndex, setExpandedIndex] = React.useState<number | null>(null);

  const toggleItem = (index: number) => {
    setExpandedIndex(expandedIndex === index ? null : index);
  };

  const handleKeyDown = (event: React.KeyboardEvent, index: number) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      toggleItem(index);
    }
  };

  return (
    <div className="accordion">
      {items.map((item, index) => {
        const isExpanded = expandedIndex === index;
        const buttonId = `accordion-button-${index}`;
        const panelId = `accordion-panel-${index}`;

        return (
          <div key={index} className="accordion-item">
            <h3>
              <button
                id={buttonId}
                type="button"
                aria-expanded={isExpanded}
                aria-controls={panelId}
                onClick={() => toggleItem(index)}
                onKeyDown={(e) => handleKeyDown(e, index)}
              >
                {item.title}
              </button>
            </h3>
            <div
              id={panelId}
              role="region"
              aria-labelledby={buttonId}
              hidden={!isExpanded}
            >
              {item.content}
            </div>
          </div>
        );
      })}
    </div>
  );
};

/**
 * Accessible Tab Panel
 * Demonstrates proper tab widget implementation with keyboard navigation
 */
export const AccessibleTabs: React.FC<{
  tabs: Array<{ label: string; content: React.ReactNode }>;
}> = ({ tabs }) => {
  const [selectedIndex, setSelectedIndex] = React.useState(0);

  const handleKeyDown = (event: React.KeyboardEvent, index: number) => {
    let newIndex = index;

    if (event.key === 'ArrowRight') {
      newIndex = (index + 1) % tabs.length;
    } else if (event.key === 'ArrowLeft') {
      newIndex = (index - 1 + tabs.length) % tabs.length;
    } else if (event.key === 'Home') {
      newIndex = 0;
    } else if (event.key === 'End') {
      newIndex = tabs.length - 1;
    } else {
      return;
    }

    event.preventDefault();
    setSelectedIndex(newIndex);
  };

  return (
    <div className="tabs">
      <div role="tablist" aria-label="Content tabs">
        {tabs.map((tab, index) => (
          <button
            key={index}
            role="tab"
            id={`tab-${index}`}
            aria-controls={`panel-${index}`}
            aria-selected={selectedIndex === index}
            tabIndex={selectedIndex === index ? 0 : -1}
            onClick={() => setSelectedIndex(index)}
            onKeyDown={(e) => handleKeyDown(e, index)}
          >
            {tab.label}
          </button>
        ))}
      </div>
      {tabs.map((tab, index) => (
        <div
          key={index}
          role="tabpanel"
          id={`panel-${index}`}
          aria-labelledby={`tab-${index}`}
          hidden={selectedIndex !== index}
          tabIndex={0}
        >
          {tab.content}
        </div>
      ))}
    </div>
  );
};

/**
 * Accessible Alert/Notification
 * Demonstrates proper live region usage
 */
export const AccessibleAlert: React.FC<{
  message: string;
  type?: 'info' | 'success' | 'warning' | 'error';
  onClose?: () => void;
}> = ({ message, type = 'info', onClose }) => {
  const roleMap = {
    info: 'status',
    success: 'status',
    warning: 'alert',
    error: 'alert',
  };

  const ariaLiveMap = {
    info: 'polite' as const,
    success: 'polite' as const,
    warning: 'assertive' as const,
    error: 'assertive' as const,
  };

  return (
    <div
      role={roleMap[type]}
      aria-live={ariaLiveMap[type]}
      aria-atomic="true"
      className={`alert alert-${type}`}
    >
      {message}
      {onClose && (
        <button
          type="button"
          onClick={onClose}
          aria-label="Dismiss alert"
          className="close-button"
        >
          ×
        </button>
      )}
    </div>
  );
};

/**
 * Accessible Skip Link
 * Allows keyboard users to skip navigation
 */
export const AccessibleSkipLink: React.FC<{
  href: string;
  children: React.ReactNode;
}> = ({ href, children }) => {
  return (
    <a href={href} className="skip-link">
      {children}
    </a>
  );
};

/**
 * Accessible Breadcrumb Navigation
 */
export const AccessibleBreadcrumb: React.FC<{
  items: Array<{ label: string; href?: string }>;
}> = ({ items }) => {
  return (
    <nav aria-label="Breadcrumb">
      <ol>
        {items.map((item, index) => {
          const isLast = index === items.length - 1;
          return (
            <li key={index}>
              {item.href && !isLast ? (
                <a href={item.href}>{item.label}</a>
              ) : (
                <span aria-current={isLast ? 'page' : undefined}>
                  {item.label}
                </span>
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
};
