/**
 * Inaccessible Component Fixtures
 * Examples of INCORRECT implementations with common accessibility issues
 * Use these for testing linting rules and learning what NOT to do
 *
 * ⚠️ WARNING: These components intentionally violate accessibility standards
 * DO NOT use these patterns in production code
 */

import React from 'react';

/**
 * BAD: Div used as button without proper semantics
 * Issues:
 * - Not a semantic button element
 * - Missing role
 * - Missing keyboard handlers
 * - Not focusable by default
 */
export const DivAsButton: React.FC<{
  onClick: () => void;
  children: React.ReactNode;
}> = ({ onClick, children }) => {
  return (
    <div onClick={onClick} className="button-like">
      {children}
    </div>
  );
};

/**
 * BAD: Button without accessible text
 * Issues:
 * - Icon-only button without aria-label
 * - Screen readers can't describe the button's purpose
 */
export const ButtonWithoutLabel: React.FC<{
  onClick: () => void;
}> = ({ onClick }) => {
  return (
    <button onClick={onClick}>
      <span className="icon">×</span>
    </button>
  );
};

/**
 * BAD: Form input without label
 * Issues:
 * - No label element
 * - No aria-label
 * - Placeholder used as label (incorrect)
 */
export const InputWithoutLabel: React.FC<{
  value: string;
  onChange: (value: string) => void;
}> = ({ value, onChange }) => {
  return (
    <input
      type="text"
      placeholder="Enter your name"
      value={value}
      onChange={(e) => onChange(e.target.value)}
    />
  );
};

/**
 * BAD: Image without alt text
 * Issues:
 * - Missing alt attribute
 * - Screen readers can't describe the image
 */
export const ImageWithoutAlt: React.FC<{
  src: string;
}> = ({ src }) => {
  return <img src={src} />;
};

/**
 * BAD: Link that looks like a button
 * Issues:
 * - Link without href (should be button)
 * - onClick on link instead of navigation
 */
export const LinkAsButton: React.FC<{
  onClick: () => void;
  children: React.ReactNode;
}> = ({ onClick, children }) => {
  return (
    <a onClick={onClick} className="button-style">
      {children}
    </a>
  );
};

/**
 * BAD: Modal without proper ARIA
 * Issues:
 * - Missing role="dialog"
 * - Missing aria-modal
 * - No aria-labelledby
 * - No focus management
 * - No keyboard handling
 */
export const InaccessibleModal: React.FC<{
  isOpen: boolean;
  onClose: () => void;
  children: React.ReactNode;
}> = ({ isOpen, onClose, children }) => {
  if (!isOpen) return null;

  return (
    <div className="modal-overlay">
      <div className="modal-content">
        <div onClick={onClose} className="close">×</div>
        {children}
      </div>
    </div>
  );
};

/**
 * BAD: Color contrast violation
 * Issues:
 * - Insufficient color contrast (light gray on white)
 * - Violates WCAG AA standards
 */
export const LowContrastText: React.FC<{
  children: React.ReactNode;
}> = ({ children }) => {
  return (
    <p style={{ color: '#d3d3d3', backgroundColor: '#ffffff' }}>
      {children}
    </p>
  );
};

/**
 * BAD: Click handler on non-interactive element
 * Issues:
 * - onClick on non-interactive element (span)
 * - Missing keyboard handler
 * - Missing role
 * - Not focusable
 */
export const ClickableSpan: React.FC<{
  onClick: () => void;
  children: React.ReactNode;
}> = ({ onClick, children }) => {
  return (
    <span onClick={onClick} className="clickable">
      {children}
    </span>
  );
};

/**
 * BAD: Invalid ARIA usage
 * Issues:
 * - Invalid aria-role attribute (should be just "role")
 * - Invalid ARIA role
 * - Misspelled aria attributes
 */
export const InvalidARIA: React.FC = () => {
  return (
    <div
      aria-role="buttton" // typo in role value
      aria-labelled-by="some-id" // should be aria-labelledby
      aria-description="test" // invalid attribute
    >
      Invalid ARIA
    </div>
  );
};

/**
 * BAD: Redundant ARIA role
 * Issues:
 * - Redundant role on button element
 * - Button already has implicit role
 */
export const RedundantRole: React.FC<{
  onClick: () => void;
  children: React.ReactNode;
}> = ({ onClick, children }) => {
  return (
    <button role="button" onClick={onClick}>
      {children}
    </button>
  );
};

/**
 * BAD: Non-semantic heading
 * Issues:
 * - Div styled to look like heading instead of using h1-h6
 * - Screen readers can't identify as heading
 * - Breaks document outline
 */
export const NonSemanticHeading: React.FC<{
  children: React.ReactNode;
}> = ({ children }) => {
  return (
    <div className="heading-large" style={{ fontSize: '24px', fontWeight: 'bold' }}>
      {children}
    </div>
  );
};

/**
 * BAD: Table used for layout
 * Issues:
 * - Table element used for layout instead of data
 * - Confuses screen reader users
 * - Should use CSS Grid or Flexbox
 */
export const TableLayout: React.FC = () => {
  return (
    <table border={0}>
      <tr>
        <td>Sidebar</td>
        <td>Main Content</td>
      </tr>
    </table>
  );
};

/**
 * BAD: Disabled button without explanation
 * Issues:
 * - Disabled state with no explanation why
 * - Users don't know how to enable it
 */
export const DisabledButtonNoExplanation: React.FC = () => {
  return (
    <button disabled>
      Submit
    </button>
  );
};

/**
 * BAD: Form with no fieldset grouping
 * Issues:
 * - Related inputs not grouped
 * - No legend for radio group
 */
export const UngroupedRadios: React.FC = () => {
  return (
    <div>
      <input type="radio" name="option" id="opt1" />
      <label htmlFor="opt1">Option 1</label>
      <input type="radio" name="option" id="opt2" />
      <label htmlFor="opt2">Option 2</label>
    </div>
  );
};

/**
 * BAD: Auto-playing media without controls
 * Issues:
 * - Video autoplays
 * - No way to pause
 * - Distracting for users
 * - WCAG violation
 */
export const AutoPlayVideo: React.FC<{
  src: string;
}> = ({ src }) => {
  return <video autoPlay loop muted src={src} />;
};

/**
 * BAD: Keyboard trap
 * Issues:
 * - Users can't tab out of this element
 * - Traps keyboard focus
 */
export const KeyboardTrap: React.FC = () => {
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Tab') {
      e.preventDefault(); // Prevents tabbing away
    }
  };

  return (
    <div onKeyDown={handleKeyDown} tabIndex={0}>
      You can't tab out of here!
    </div>
  );
};

/**
 * BAD: Missing language attribute
 * Issues:
 * - No lang attribute on HTML element
 * - Screen readers can't determine language
 */
export const MissingLangAttribute: React.FC = () => {
  return (
    <div>
      <p>This content has no language specified</p>
    </div>
  );
};

/**
 * BAD: Error message not associated with input
 * Issues:
 * - Error message not linked with aria-describedby
 * - No aria-invalid
 * - Screen reader users won't hear the error
 */
export const DisconnectedError: React.FC = () => {
  return (
    <div>
      <label htmlFor="email">Email</label>
      <input type="email" id="email" />
      <div className="error">Please enter a valid email</div>
    </div>
  );
};

/**
 * BAD: Custom select without proper ARIA
 * Issues:
 * - No role="combobox" or role="listbox"
 * - No aria-expanded
 * - No keyboard navigation
 * - Can't be operated by screen readers
 */
export const CustomSelectNoAria: React.FC<{
  options: string[];
  onSelect: (option: string) => void;
}> = ({ options, onSelect }) => {
  const [isOpen, setIsOpen] = React.useState(false);

  return (
    <div className="custom-select">
      <div onClick={() => setIsOpen(!isOpen)}>
        Select an option
      </div>
      {isOpen && (
        <div className="options">
          {options.map((option, index) => (
            <div
              key={index}
              onClick={() => {
                onSelect(option);
                setIsOpen(false);
              }}
            >
              {option}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

/**
 * BAD: Abstract ARIA role used directly
 * Issues:
 * - "command" is an abstract role
 * - Should use concrete role like "button"
 */
export const AbstractRole: React.FC = () => {
  return (
    <div role="command">
      Abstract Role
    </div>
  );
};

/**
 * BAD: Time-limited content without control
 * Issues:
 * - Content disappears after timeout
 * - No way to extend or pause
 * - WCAG 2.2.1 violation
 */
export const TimedContent: React.FC<{
  children: React.ReactNode;
}> = ({ children }) => {
  const [visible, setVisible] = React.useState(true);

  React.useEffect(() => {
    const timer = setTimeout(() => setVisible(false), 3000);
    return () => clearTimeout(timer);
  }, []);

  return visible ? <div>{children}</div> : null;
};
