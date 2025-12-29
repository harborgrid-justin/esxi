/**
 * Preview screen reader announcements
 */

import React from 'react';
import type { Announcement, AccessibilityNode, ScreenReaderType } from '../../types';

export interface AnnouncementPreviewProps {
  announcement: Announcement | null;
  screenReader: ScreenReaderType;
  currentNode: AccessibilityNode | null;
  className?: string;
}

export const AnnouncementPreview: React.FC<AnnouncementPreviewProps> = ({
  announcement,
  screenReader,
  currentNode,
  className = '',
}) => {
  if (!announcement || !currentNode) {
    return (
      <div className={`announcement-preview ${className}`}>
        <div className="no-announcement">
          <p>Navigate to an element to see how {screenReader} would announce it.</p>
          <p className="hint">Use the accessibility tree or virtual screen reader to navigate.</p>
        </div>

        <style>{`
          .announcement-preview {
            border: 1px solid #e0e0e0;
            border-radius: 4px;
            background: white;
            padding: 20px;
          }

          .no-announcement {
            text-align: center;
            padding: 40px 20px;
            color: #666;
          }

          .no-announcement .hint {
            font-size: 14px;
            color: #999;
          }
        `}</style>
      </div>
    );
  }

  return (
    <div className={`announcement-preview ${className}`}>
      <div className="preview-header">
        <h3>{screenReader} Announcement</h3>
        <div className="verbosity-badge">{announcement.verbosity}</div>
      </div>

      <div className="announcement-display">
        <div className="announcement-text">
          <div className="text-label">What {screenReader} says:</div>
          <div className="text-content">{announcement.text}</div>
        </div>

        <div className="announcement-breakdown">
          <h4>Breakdown</h4>

          <div className="breakdown-section">
            <label>Role:</label>
            <span>{announcement.role}</span>
          </div>

          {announcement.name && (
            <div className="breakdown-section">
              <label>Name:</label>
              <span>{announcement.name}</span>
            </div>
          )}

          {announcement.state.length > 0 && (
            <div className="breakdown-section">
              <label>State:</label>
              <span>{announcement.state.join(', ')}</span>
            </div>
          )}

          {announcement.properties.length > 0 && (
            <div className="breakdown-section">
              <label>Properties:</label>
              <span>{announcement.properties.join(', ')}</span>
            </div>
          )}

          {announcement.context.length > 0 && (
            <div className="breakdown-section">
              <label>Context:</label>
              <span>{announcement.context.join(', ')}</span>
            </div>
          )}
        </div>
      </div>

      <div className="element-details">
        <h4>Element Details</h4>

        <div className="detail-grid">
          <div className="detail-item">
            <label>Role:</label>
            <span className="role-value">{currentNode.role}</span>
          </div>

          {currentNode.name && (
            <div className="detail-item">
              <label>Accessible Name:</label>
              <span>{currentNode.name}</span>
            </div>
          )}

          {currentNode.description && (
            <div className="detail-item">
              <label>Description:</label>
              <span>{currentNode.description}</span>
            </div>
          )}

          {currentNode.value && (
            <div className="detail-item">
              <label>Value:</label>
              <span>{currentNode.value}</span>
            </div>
          )}

          <div className="detail-item">
            <label>Focusable:</label>
            <span>{currentNode.focusable ? 'Yes' : 'No'}</span>
          </div>

          {currentNode.level && (
            <div className="detail-item">
              <label>Level:</label>
              <span>{currentNode.level}</span>
            </div>
          )}

          {currentNode.tabIndex >= 0 && (
            <div className="detail-item">
              <label>Tab Index:</label>
              <span>{currentNode.tabIndex}</span>
            </div>
          )}
        </div>

        <div className="state-badges">
          {currentNode.disabled && <span className="state-badge disabled">Disabled</span>}
          {currentNode.required && <span className="state-badge required">Required</span>}
          {currentNode.invalid && <span className="state-badge invalid">Invalid</span>}
          {currentNode.readonly && <span className="state-badge readonly">Readonly</span>}
          {currentNode.hidden && <span className="state-badge hidden">Hidden</span>}
          {currentNode.expanded === true && <span className="state-badge">Expanded</span>}
          {currentNode.expanded === false && <span className="state-badge">Collapsed</span>}
          {currentNode.checked === true && <span className="state-badge">Checked</span>}
          {currentNode.checked === false && <span className="state-badge">Unchecked</span>}
          {currentNode.selected === true && <span className="state-badge">Selected</span>}
        </div>
      </div>

      <style>{`
        .announcement-preview {
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          background: white;
        }

        .preview-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 15px 20px;
          border-bottom: 1px solid #e0e0e0;
          background: #f8f9fa;
        }

        .preview-header h3 {
          margin: 0;
          font-size: 18px;
        }

        .verbosity-badge {
          padding: 4px 12px;
          border-radius: 4px;
          background: #e3f2fd;
          color: #1976d2;
          font-size: 12px;
          font-weight: 500;
          text-transform: capitalize;
        }

        .announcement-display {
          padding: 20px;
        }

        .announcement-text {
          padding: 20px;
          background: #f0f7ff;
          border-left: 4px solid #2196f3;
          border-radius: 4px;
          margin-bottom: 20px;
        }

        .text-label {
          font-size: 12px;
          color: #666;
          margin-bottom: 10px;
          text-transform: uppercase;
          font-weight: 600;
        }

        .text-content {
          font-size: 18px;
          line-height: 1.6;
          color: #333;
          font-weight: 500;
        }

        .announcement-breakdown {
          margin-top: 20px;
        }

        .announcement-breakdown h4 {
          margin: 0 0 15px 0;
          font-size: 14px;
          color: #666;
          text-transform: uppercase;
        }

        .breakdown-section {
          display: flex;
          gap: 10px;
          margin-bottom: 10px;
          padding: 8px;
          background: #f8f9fa;
          border-radius: 4px;
        }

        .breakdown-section label {
          font-weight: 600;
          color: #666;
          min-width: 100px;
        }

        .breakdown-section span {
          flex: 1;
        }

        .element-details {
          padding: 20px;
          border-top: 1px solid #e0e0e0;
          background: #fafafa;
        }

        .element-details h4 {
          margin: 0 0 15px 0;
          font-size: 14px;
          color: #666;
          text-transform: uppercase;
        }

        .detail-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
          gap: 15px;
          margin-bottom: 15px;
        }

        .detail-item {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .detail-item label {
          font-size: 12px;
          color: #666;
          font-weight: 600;
        }

        .detail-item span {
          font-size: 14px;
          color: #333;
        }

        .role-value {
          color: #9c27b0;
          font-weight: 600;
        }

        .state-badges {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
        }

        .state-badge {
          padding: 4px 10px;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 500;
          background: #e0e0e0;
          color: #333;
        }

        .state-badge.disabled {
          background: #ffebee;
          color: #c62828;
        }

        .state-badge.required {
          background: #fff3e0;
          color: #e65100;
        }

        .state-badge.invalid {
          background: #ffebee;
          color: #d32f2f;
        }

        .state-badge.readonly {
          background: #f5f5f5;
          color: #757575;
        }

        .state-badge.hidden {
          background: #f5f5f5;
          color: #757575;
        }
      `}</style>
    </div>
  );
};
