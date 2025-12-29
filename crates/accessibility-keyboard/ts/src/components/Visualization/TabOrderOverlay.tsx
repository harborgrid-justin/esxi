/**
 * Tab Order Overlay Component
 * Visual overlay showing tab order on the page
 */

import React, { useEffect, useState } from 'react';
import { FocusableElement } from '../../types';
import { TabOrderAnalyzer } from '../../analyzers/TabOrderAnalyzer';

export interface TabOrderOverlayProps {
  show: boolean;
  highlightIssues?: boolean;
  onClose?: () => void;
}

export const TabOrderOverlay: React.FC<TabOrderOverlayProps> = ({
  show,
  highlightIssues = true,
  onClose,
}) => {
  const [elements, setElements] = useState<FocusableElement[]>([]);
  const [overlayElements, setOverlayElements] = useState<JSX.Element[]>([]);

  useEffect(() => {
    if (show) {
      analyzeAndRender();
    } else {
      setOverlayElements([]);
    }
  }, [show, highlightIssues]);

  const analyzeAndRender = async () => {
    const analyzer = new TabOrderAnalyzer();
    const result = await analyzer.analyze(document.body);
    setElements(result.elements);

    const overlays = result.elements.map((element, index) => {
      const rect = element.element.getBoundingClientRect();
      const hasIssue = result.issues.some((issue) => issue.element === element);

      return (
        <div
          key={index}
          style={{
            position: 'fixed',
            top: `${rect.top + window.scrollY}px`,
            left: `${rect.left + window.scrollX}px`,
            width: `${rect.width}px`,
            height: `${rect.height}px`,
            border: `3px solid ${hasIssue && highlightIssues ? '#dc3545' : '#007bff'}`,
            backgroundColor:
              hasIssue && highlightIssues
                ? 'rgba(220, 53, 69, 0.1)'
                : 'rgba(0, 123, 255, 0.1)',
            pointerEvents: 'none',
            zIndex: 999998,
            borderRadius: '4px',
          }}
        >
          <div
            style={{
              position: 'absolute',
              top: '-28px',
              left: '0',
              backgroundColor: hasIssue && highlightIssues ? '#dc3545' : '#007bff',
              color: 'white',
              padding: '4px 10px',
              borderRadius: '4px',
              fontSize: '14px',
              fontWeight: 700,
              fontFamily: 'system-ui, -apple-system, sans-serif',
              boxShadow: '0 2px 4px rgba(0,0,0,0.2)',
            }}
          >
            {index + 1}
            {element.tabIndex > 0 && (
              <span style={{ fontSize: '11px', marginLeft: '6px', opacity: 0.9 }}>
                (tabindex: {element.tabIndex})
              </span>
            )}
          </div>
        </div>
      );
    });

    setOverlayElements(overlays);
  };

  if (!show) return null;

  return (
    <>
      {/* Background overlay */}
      <div
        style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          backgroundColor: 'rgba(0, 0, 0, 0.3)',
          zIndex: 999997,
          backdropFilter: 'blur(2px)',
        }}
        onClick={onClose}
      />

      {/* Tab order overlays */}
      {overlayElements}

      {/* Control panel */}
      <div
        style={{
          position: 'fixed',
          top: '20px',
          right: '20px',
          backgroundColor: 'white',
          padding: '16px',
          borderRadius: '8px',
          boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
          zIndex: 999999,
          minWidth: '250px',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '12px' }}>
          <h4 style={{ margin: 0, fontSize: '16px' }}>Tab Order Overlay</h4>
          <button
            onClick={onClose}
            style={{
              border: 'none',
              background: 'none',
              fontSize: '20px',
              cursor: 'pointer',
              padding: 0,
              lineHeight: 1,
            }}
          >
            ×
          </button>
        </div>

        <div style={{ fontSize: '13px', color: '#6c757d', marginBottom: '12px' }}>
          Showing {elements.length} focusable elements
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div
              style={{
                width: '20px',
                height: '20px',
                border: '3px solid #007bff',
                backgroundColor: 'rgba(0, 123, 255, 0.1)',
              }}
            />
            <span style={{ fontSize: '12px' }}>Normal tab order</span>
          </div>
          {highlightIssues && (
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <div
                style={{
                  width: '20px',
                  height: '20px',
                  border: '3px solid #dc3545',
                  backgroundColor: 'rgba(220, 53, 69, 0.1)',
                }}
              />
              <span style={{ fontSize: '12px' }}>Tab order issue</span>
            </div>
          )}
        </div>

        <div style={{ marginTop: '16px', paddingTop: '12px', borderTop: '1px solid #dee2e6' }}>
          <button
            onClick={analyzeAndRender}
            style={{
              width: '100%',
              padding: '8px',
              backgroundColor: '#007bff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '13px',
            }}
          >
            Refresh
          </button>
        </div>
      </div>
    </>
  );
};
