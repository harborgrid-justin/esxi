/**
 * Main contrast analyzer component
 */

import React, { useState } from 'react';
import { RGB } from '../../types';
import { ColorPicker } from './ColorPicker';
import { ContrastPreview } from './ContrastPreview';
import { ContrastRatio } from './ContrastRatio';
import { useContrast } from '../../hooks/useContrast';

export interface ContrastAnalyzerProps {
  /** Initial foreground color */
  initialForeground?: string | RGB;
  /** Initial background color */
  initialBackground?: string | RGB;
  /** Show suggestions */
  showSuggestions?: boolean;
  /** Custom class name */
  className?: string;
  /** On color change callback */
  onChange?: (foreground: RGB, background: RGB) => void;
}

/**
 * Complete contrast analyzer with color pickers and preview
 */
export const ContrastAnalyzer: React.FC<ContrastAnalyzerProps> = ({
  initialForeground = '#000000',
  initialBackground = '#FFFFFF',
  showSuggestions = true,
  className = '',
  onChange,
}) => {
  const {
    contrast,
    foregroundRGB,
    backgroundRGB,
    foregroundHex,
    backgroundHex,
    isAccessible,
    isExcellent,
    grade,
    suggestions,
    error,
    setForeground,
    setBackground,
  } = useContrast({
    foreground: initialForeground,
    background: initialBackground,
    autoSuggest: showSuggestions,
  });

  // Notify parent of changes
  React.useEffect(() => {
    if (foregroundRGB && backgroundRGB && onChange) {
      onChange(foregroundRGB, backgroundRGB);
    }
  }, [foregroundRGB, backgroundRGB, onChange]);

  return (
    <div className={`contrast-analyzer ${className}`}>
      <div className="contrast-analyzer__header">
        <h2>Color Contrast Analyzer</h2>
        {error && <div className="contrast-analyzer__error">{error}</div>}
      </div>

      <div className="contrast-analyzer__pickers">
        <div className="contrast-analyzer__picker-group">
          <label>Foreground (Text)</label>
          <ColorPicker
            color={foregroundHex || '#000000'}
            onChange={setForeground}
            label="Foreground"
          />
        </div>

        <div className="contrast-analyzer__picker-group">
          <label>Background</label>
          <ColorPicker
            color={backgroundHex || '#FFFFFF'}
            onChange={setBackground}
            label="Background"
          />
        </div>
      </div>

      {contrast && foregroundRGB && backgroundRGB && (
        <>
          <ContrastRatio
            ratio={contrast.ratio}
            wcag={contrast.wcag}
            apca={contrast.apca}
            grade={grade}
          />

          <ContrastPreview
            foreground={foregroundRGB}
            background={backgroundRGB}
            contrast={contrast}
          />
        </>
      )}

      {showSuggestions && suggestions.length > 0 && (
        <div className="contrast-analyzer__suggestions">
          <h3>Accessible Alternatives</h3>
          <div className="contrast-analyzer__suggestions-list">
            {suggestions.slice(0, 5).map((suggestion, index) => (
              <button
                key={index}
                className="contrast-analyzer__suggestion"
                onClick={() => setForeground(suggestion.color)}
                style={{
                  backgroundColor: suggestion.hex,
                  color: backgroundHex || '#FFFFFF',
                }}
              >
                <span className="contrast-analyzer__suggestion-hex">
                  {suggestion.hex}
                </span>
                <span className="contrast-analyzer__suggestion-ratio">
                  {suggestion.contrast.ratio.toFixed(2)}:1
                </span>
              </button>
            ))}
          </div>
        </div>
      )}

      <style>{`
        .contrast-analyzer {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
          max-width: 800px;
          margin: 0 auto;
          padding: 24px;
        }

        .contrast-analyzer__header {
          margin-bottom: 24px;
        }

        .contrast-analyzer__header h2 {
          margin: 0 0 8px 0;
          font-size: 24px;
          font-weight: 600;
        }

        .contrast-analyzer__error {
          color: #dc2626;
          background: #fee2e2;
          padding: 12px;
          border-radius: 6px;
          margin-top: 12px;
        }

        .contrast-analyzer__pickers {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 24px;
          margin-bottom: 24px;
        }

        .contrast-analyzer__picker-group label {
          display: block;
          font-weight: 500;
          margin-bottom: 8px;
          font-size: 14px;
        }

        .contrast-analyzer__suggestions {
          margin-top: 24px;
        }

        .contrast-analyzer__suggestions h3 {
          margin: 0 0 12px 0;
          font-size: 18px;
          font-weight: 600;
        }

        .contrast-analyzer__suggestions-list {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
          gap: 12px;
        }

        .contrast-analyzer__suggestion {
          border: 2px solid #e5e7eb;
          border-radius: 8px;
          padding: 16px;
          cursor: pointer;
          transition: all 0.2s;
          text-align: center;
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .contrast-analyzer__suggestion:hover {
          border-color: #3b82f6;
          transform: translateY(-2px);
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
        }

        .contrast-analyzer__suggestion-hex {
          font-family: monospace;
          font-size: 12px;
          font-weight: 600;
        }

        .contrast-analyzer__suggestion-ratio {
          font-size: 11px;
          opacity: 0.8;
        }

        @media (max-width: 640px) {
          .contrast-analyzer__pickers {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
};
