/**
 * Virtual screen reader simulator component
 */

import React, { useState } from 'react';
import type { ScreenReaderType, AccessibilityNode, Announcement } from '../../types';
import { useScreenReader } from '../../hooks/useScreenReader';

export interface VirtualScreenReaderProps {
  root?: Element;
  defaultScreenReader?: ScreenReaderType;
  className?: string;
}

export const VirtualScreenReader: React.FC<VirtualScreenReaderProps> = ({
  root,
  defaultScreenReader = 'NVDA',
  className = '',
}) => {
  const [screenReader, setScreenReaderType] = useState<ScreenReaderType>(defaultScreenReader);
  const [announcements, setAnnouncements] = useState<Announcement[]>([]);

  const {
    tree,
    currentNode,
    announcement,
    navigateNext,
    navigatePrevious,
    navigateNextHeading,
    navigateNextLandmark,
    navigateNextLink,
    navigateNextFormField,
    setScreenReader,
  } = useScreenReader({
    root,
    screenReader,
    autoAnalyze: true,
  });

  const handleNavigation = (fn: () => void) => {
    fn();
    if (announcement) {
      setAnnouncements(prev => [...prev, announcement]);
    }
  };

  const handleScreenReaderChange = (sr: ScreenReaderType) => {
    setScreenReaderType(sr);
    setScreenReader(sr);
    setAnnouncements([]);
  };

  const clearHistory = () => {
    setAnnouncements([]);
  };

  const getKeyboardShortcuts = () => {
    switch (screenReader) {
      case 'NVDA':
        return {
          next: 'Down Arrow',
          previous: 'Up Arrow',
          heading: 'H',
          landmark: 'D',
          link: 'K',
          formField: 'F',
        };
      case 'JAWS':
        return {
          next: 'Down Arrow',
          previous: 'Up Arrow',
          heading: 'H',
          landmark: 'R',
          link: 'Tab / K',
          formField: 'F',
        };
      case 'VoiceOver':
        return {
          next: 'VO + Right Arrow',
          previous: 'VO + Left Arrow',
          heading: 'VO + Cmd + H',
          landmark: 'VO + Cmd + L',
          link: 'VO + Cmd + L',
          formField: 'VO + Cmd + J',
        };
      default:
        return {
          next: 'Down Arrow',
          previous: 'Up Arrow',
          heading: 'H',
          landmark: 'D',
          link: 'K',
          formField: 'F',
        };
    }
  };

  const shortcuts = getKeyboardShortcuts();

  return (
    <div className={`virtual-screen-reader ${className}`}>
      <div className="vsr-header">
        <h3>Virtual Screen Reader</h3>
        <div className="vsr-controls">
          <select
            value={screenReader}
            onChange={(e) => handleScreenReaderChange(e.target.value as ScreenReaderType)}
          >
            <option value="NVDA">NVDA</option>
            <option value="JAWS">JAWS</option>
            <option value="VoiceOver">VoiceOver</option>
          </select>
          <button onClick={clearHistory}>Clear History</button>
        </div>
      </div>

      <div className="vsr-content">
        <div className="navigation-panel">
          <h4>Navigation</h4>
          <div className="nav-buttons">
            <button onClick={() => handleNavigation(navigatePrevious)} title={shortcuts.previous}>
              ← Previous
            </button>
            <button onClick={() => handleNavigation(navigateNext)} title={shortcuts.next}>
              Next →
            </button>
          </div>

          <div className="nav-buttons">
            <button onClick={() => handleNavigation(navigateNextHeading)} title={shortcuts.heading}>
              Next Heading
            </button>
            <button onClick={() => handleNavigation(navigateNextLandmark)} title={shortcuts.landmark}>
              Next Landmark
            </button>
          </div>

          <div className="nav-buttons">
            <button onClick={() => handleNavigation(navigateNextLink)} title={shortcuts.link}>
              Next Link
            </button>
            <button onClick={() => handleNavigation(navigateNextFormField)} title={shortcuts.formField}>
              Next Form Field
            </button>
          </div>

          {currentNode && (
            <div className="current-element">
              <h5>Current Element</h5>
              <div className="element-info">
                <div><strong>Role:</strong> {currentNode.role}</div>
                {currentNode.name && <div><strong>Name:</strong> {currentNode.name}</div>}
                <div><strong>Focusable:</strong> {currentNode.focusable ? 'Yes' : 'No'}</div>
              </div>
            </div>
          )}
        </div>

        <div className="announcement-panel">
          <h4>Announcement History</h4>
          {announcements.length === 0 ? (
            <div className="empty-history">
              <p>No announcements yet.</p>
              <p className="hint">Use the navigation buttons to start.</p>
            </div>
          ) : (
            <div className="announcement-list">
              {announcements.map((ann, index) => (
                <div key={index} className="announcement-item">
                  <div className="announcement-number">#{announcements.length - index}</div>
                  <div className="announcement-content">
                    <div className="announcement-text">{ann.text}</div>
                    <div className="announcement-meta">
                      {ann.role} • {ann.verbosity}
                    </div>
                  </div>
                </div>
              )).reverse()}
            </div>
          )}
        </div>
      </div>

      <div className="keyboard-shortcuts">
        <h5>Keyboard Shortcuts ({screenReader})</h5>
        <div className="shortcuts-grid">
          <div><kbd>{shortcuts.next}</kbd> Next element</div>
          <div><kbd>{shortcuts.previous}</kbd> Previous element</div>
          <div><kbd>{shortcuts.heading}</kbd> Next heading</div>
          <div><kbd>{shortcuts.landmark}</kbd> Next landmark</div>
          <div><kbd>{shortcuts.link}</kbd> Next link</div>
          <div><kbd>{shortcuts.formField}</kbd> Next form field</div>
        </div>
      </div>

      <style>{`
        .virtual-screen-reader {
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          background: white;
        }

        .vsr-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 15px 20px;
          border-bottom: 1px solid #e0e0e0;
          background: #f8f9fa;
        }

        .vsr-header h3 {
          margin: 0;
          font-size: 18px;
        }

        .vsr-controls {
          display: flex;
          gap: 10px;
        }

        .vsr-controls select {
          padding: 6px 12px;
          border: 1px solid #ccc;
          border-radius: 4px;
        }

        .vsr-content {
          display: grid;
          grid-template-columns: 300px 1fr;
          gap: 20px;
          padding: 20px;
          min-height: 400px;
        }

        .navigation-panel,
        .announcement-panel {
          display: flex;
          flex-direction: column;
        }

        .navigation-panel h4,
        .announcement-panel h4 {
          margin: 0 0 15px 0;
          font-size: 14px;
          color: #666;
          text-transform: uppercase;
        }

        .nav-buttons {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 10px;
          margin-bottom: 10px;
        }

        .nav-buttons button {
          padding: 10px;
          font-size: 14px;
        }

        .current-element {
          margin-top: 20px;
          padding: 15px;
          background: #f0f7ff;
          border-radius: 4px;
          border: 1px solid #2196f3;
        }

        .current-element h5 {
          margin: 0 0 10px 0;
          font-size: 12px;
          color: #1976d2;
          text-transform: uppercase;
        }

        .element-info {
          font-size: 13px;
        }

        .element-info div {
          margin: 5px 0;
        }

        .empty-history {
          text-align: center;
          padding: 40px 20px;
          color: #999;
        }

        .empty-history .hint {
          font-size: 14px;
          margin-top: 10px;
        }

        .announcement-list {
          display: flex;
          flex-direction: column;
          gap: 10px;
          max-height: 500px;
          overflow-y: auto;
        }

        .announcement-item {
          display: flex;
          gap: 10px;
          padding: 12px;
          background: #f8f9fa;
          border-radius: 4px;
          border-left: 3px solid #2196f3;
        }

        .announcement-number {
          font-size: 12px;
          color: #666;
          min-width: 30px;
        }

        .announcement-content {
          flex: 1;
        }

        .announcement-text {
          font-size: 14px;
          line-height: 1.5;
          margin-bottom: 5px;
        }

        .announcement-meta {
          font-size: 11px;
          color: #999;
        }

        .keyboard-shortcuts {
          padding: 15px 20px;
          border-top: 1px solid #e0e0e0;
          background: #fafafa;
        }

        .keyboard-shortcuts h5 {
          margin: 0 0 10px 0;
          font-size: 12px;
          color: #666;
          text-transform: uppercase;
        }

        .shortcuts-grid {
          display: grid;
          grid-template-columns: repeat(3, 1fr);
          gap: 10px;
          font-size: 12px;
        }

        .shortcuts-grid div {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        kbd {
          padding: 2px 6px;
          background: #fff;
          border: 1px solid #ccc;
          border-radius: 3px;
          font-size: 11px;
          font-family: monospace;
        }
      `}</style>
    </div>
  );
};
