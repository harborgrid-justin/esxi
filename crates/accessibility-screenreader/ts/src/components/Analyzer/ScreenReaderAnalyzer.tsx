/**
 * Main screen reader analyzer component
 */

import React, { useState } from 'react';
import type { ScreenReaderType, BrowserType } from '../../types';
import { useScreenReader } from '../../hooks/useScreenReader';
import { AccessibilityTree } from './AccessibilityTree';
import { ReadingOrderViewer } from './ReadingOrderViewer';
import { AnnouncementPreview } from './AnnouncementPreview';

export interface ScreenReaderAnalyzerProps {
  root?: Element;
  defaultScreenReader?: ScreenReaderType;
  defaultBrowser?: BrowserType;
  className?: string;
}

export const ScreenReaderAnalyzer: React.FC<ScreenReaderAnalyzerProps> = ({
  root,
  defaultScreenReader = 'NVDA',
  defaultBrowser = 'Chrome',
  className = '',
}) => {
  const [activeTab, setActiveTab] = useState<'tree' | 'reading-order' | 'announcements'>('tree');
  const [selectedScreenReader, setSelectedScreenReader] = useState<ScreenReaderType>(defaultScreenReader);
  const [selectedBrowser, setSelectedBrowser] = useState<BrowserType>(defaultBrowser);

  const {
    report,
    tree,
    currentNode,
    announcement,
    isAnalyzing,
    analyze,
    navigateTo,
    setScreenReader,
  } = useScreenReader({
    root,
    autoAnalyze: true,
    screenReader: selectedScreenReader,
    browser: selectedBrowser,
  });

  const handleScreenReaderChange = (sr: ScreenReaderType) => {
    setSelectedScreenReader(sr);
    setScreenReader(sr);
  };

  return (
    <div className={`screen-reader-analyzer ${className}`}>
      {/* Header */}
      <div className="analyzer-header">
        <h1>Screen Reader Compatibility Analyzer</h1>

        <div className="controls">
          <div className="control-group">
            <label>Screen Reader:</label>
            <select
              value={selectedScreenReader}
              onChange={(e) => handleScreenReaderChange(e.target.value as ScreenReaderType)}
            >
              <option value="NVDA">NVDA</option>
              <option value="JAWS">JAWS</option>
              <option value="VoiceOver">VoiceOver</option>
              <option value="TalkBack">TalkBack</option>
              <option value="Narrator">Narrator</option>
            </select>
          </div>

          <div className="control-group">
            <label>Browser:</label>
            <select
              value={selectedBrowser}
              onChange={(e) => setSelectedBrowser(e.target.value as BrowserType)}
            >
              <option value="Chrome">Chrome</option>
              <option value="Firefox">Firefox</option>
              <option value="Safari">Safari</option>
              <option value="Edge">Edge</option>
            </select>
          </div>

          <button onClick={analyze} disabled={isAnalyzing}>
            {isAnalyzing ? 'Analyzing...' : 'Re-analyze'}
          </button>
        </div>
      </div>

      {/* Score Summary */}
      {report && (
        <div className="score-summary">
          <div className="score-badge">
            <div className="score-value" data-score={report.score >= 80 ? 'good' : report.score >= 60 ? 'fair' : 'poor'}>
              {report.score}
            </div>
            <div className="score-label">Accessibility Score</div>
          </div>

          <div className="summary-stats">
            <div className="stat">
              <span className="stat-value">{report.summary.totalNodes}</span>
              <span className="stat-label">Total Nodes</span>
            </div>
            <div className="stat">
              <span className="stat-value">{report.summary.focusableNodes}</span>
              <span className="stat-label">Focusable</span>
            </div>
            <div className="stat critical">
              <span className="stat-value">{report.summary.criticalIssues}</span>
              <span className="stat-label">Critical</span>
            </div>
            <div className="stat serious">
              <span className="stat-value">{report.summary.seriousIssues}</span>
              <span className="stat-label">Serious</span>
            </div>
            <div className="stat moderate">
              <span className="stat-value">{report.summary.moderateIssues}</span>
              <span className="stat-label">Moderate</span>
            </div>
            <div className="stat minor">
              <span className="stat-value">{report.summary.minorIssues}</span>
              <span className="stat-label">Minor</span>
            </div>
          </div>
        </div>
      )}

      {/* Tabs */}
      <div className="analyzer-tabs">
        <button
          className={activeTab === 'tree' ? 'active' : ''}
          onClick={() => setActiveTab('tree')}
        >
          Accessibility Tree
        </button>
        <button
          className={activeTab === 'reading-order' ? 'active' : ''}
          onClick={() => setActiveTab('reading-order')}
        >
          Reading Order
        </button>
        <button
          className={activeTab === 'announcements' ? 'active' : ''}
          onClick={() => setActiveTab('announcements')}
        >
          Announcements
        </button>
      </div>

      {/* Tab Content */}
      <div className="analyzer-content">
        {activeTab === 'tree' && tree && (
          <AccessibilityTree
            tree={tree}
            currentNode={currentNode}
            onNodeSelect={navigateTo}
          />
        )}

        {activeTab === 'reading-order' && report && (
          <ReadingOrderViewer
            readingOrder={report.readingOrder}
            onNodeSelect={navigateTo}
          />
        )}

        {activeTab === 'announcements' && (
          <AnnouncementPreview
            announcement={announcement}
            screenReader={selectedScreenReader}
            currentNode={currentNode}
          />
        )}
      </div>

      <style>{`
        .screen-reader-analyzer {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
          max-width: 1200px;
          margin: 0 auto;
          padding: 20px;
        }

        .analyzer-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 20px;
          padding-bottom: 20px;
          border-bottom: 2px solid #e0e0e0;
        }

        .analyzer-header h1 {
          margin: 0;
          font-size: 24px;
          color: #333;
        }

        .controls {
          display: flex;
          gap: 15px;
        }

        .control-group {
          display: flex;
          flex-direction: column;
          gap: 5px;
        }

        .control-group label {
          font-size: 12px;
          color: #666;
          font-weight: 500;
        }

        .control-group select {
          padding: 6px 12px;
          border: 1px solid #ccc;
          border-radius: 4px;
          font-size: 14px;
        }

        button {
          padding: 8px 16px;
          background: #007bff;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-size: 14px;
          font-weight: 500;
        }

        button:hover {
          background: #0056b3;
        }

        button:disabled {
          background: #ccc;
          cursor: not-allowed;
        }

        .score-summary {
          display: flex;
          gap: 30px;
          margin-bottom: 20px;
          padding: 20px;
          background: #f8f9fa;
          border-radius: 8px;
        }

        .score-badge {
          text-align: center;
        }

        .score-value {
          font-size: 48px;
          font-weight: bold;
          line-height: 1;
        }

        .score-value[data-score="good"] {
          color: #28a745;
        }

        .score-value[data-score="fair"] {
          color: #ffc107;
        }

        .score-value[data-score="poor"] {
          color: #dc3545;
        }

        .score-label {
          font-size: 14px;
          color: #666;
          margin-top: 5px;
        }

        .summary-stats {
          display: flex;
          gap: 20px;
          flex: 1;
        }

        .stat {
          display: flex;
          flex-direction: column;
          align-items: center;
        }

        .stat-value {
          font-size: 24px;
          font-weight: bold;
          color: #333;
        }

        .stat.critical .stat-value {
          color: #dc3545;
        }

        .stat.serious .stat-value {
          color: #fd7e14;
        }

        .stat.moderate .stat-value {
          color: #ffc107;
        }

        .stat.minor .stat-value {
          color: #17a2b8;
        }

        .stat-label {
          font-size: 12px;
          color: #666;
          margin-top: 4px;
        }

        .analyzer-tabs {
          display: flex;
          gap: 10px;
          margin-bottom: 20px;
          border-bottom: 2px solid #e0e0e0;
        }

        .analyzer-tabs button {
          background: none;
          color: #666;
          padding: 10px 20px;
          border: none;
          border-bottom: 3px solid transparent;
          cursor: pointer;
        }

        .analyzer-tabs button:hover {
          color: #333;
          background: #f8f9fa;
        }

        .analyzer-tabs button.active {
          color: #007bff;
          border-bottom-color: #007bff;
        }

        .analyzer-content {
          min-height: 400px;
        }
      `}</style>
    </div>
  );
};
