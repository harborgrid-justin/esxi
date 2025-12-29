/**
 * Simulation preview component
 * Shows original vs simulated colors side by side
 */

import React from 'react';
import { RGB, ColorBlindnessType } from '../../types';
import { rgbToHex } from '../../utils/colorMath';

export interface SimulationPreviewProps {
  /** Original color */
  original: RGB;
  /** Simulated color */
  simulated: RGB;
  /** Type of color blindness */
  type: ColorBlindnessType;
  /** Human-readable type name */
  typeName: string;
  /** Custom class name */
  className?: string;
}

/**
 * Preview showing original and simulated colors
 */
export const SimulationPreview: React.FC<SimulationPreviewProps> = ({
  original,
  simulated,
  type,
  typeName,
  className = '',
}) => {
  const originalHex = rgbToHex(original);
  const simulatedHex = rgbToHex(simulated);

  // Calculate if colors are different
  const isDifferent =
    original.r !== simulated.r || original.g !== simulated.g || original.b !== simulated.b;

  const difference = Math.sqrt(
    Math.pow(original.r - simulated.r, 2) +
      Math.pow(original.g - simulated.g, 2) +
      Math.pow(original.b - simulated.b, 2)
  );

  return (
    <div className={`simulation-preview ${className}`}>
      <div className="simulation-preview__comparison">
        <div className="simulation-preview__side">
          <div
            className="simulation-preview__color"
            style={{ backgroundColor: originalHex }}
          >
            <div className="simulation-preview__overlay">
              <div className="simulation-preview__label">Normal Vision</div>
              <div className="simulation-preview__hex">{originalHex}</div>
              <div className="simulation-preview__rgb">
                RGB({original.r}, {original.g}, {original.b})
              </div>
            </div>
          </div>
        </div>

        <div className="simulation-preview__arrow">
          <svg width="40" height="40" viewBox="0 0 40 40" fill="none">
            <path
              d="M15 10L25 20L15 30"
              stroke="currentColor"
              strokeWidth="3"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </div>

        <div className="simulation-preview__side">
          <div
            className="simulation-preview__color"
            style={{ backgroundColor: simulatedHex }}
          >
            <div className="simulation-preview__overlay">
              <div className="simulation-preview__label">{typeName}</div>
              <div className="simulation-preview__hex">{simulatedHex}</div>
              <div className="simulation-preview__rgb">
                RGB({simulated.r}, {simulated.g}, {simulated.b})
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="simulation-preview__info">
        <div className="simulation-preview__stat">
          <div className="simulation-preview__stat-label">Color Difference</div>
          <div className="simulation-preview__stat-value">
            {difference.toFixed(1)} <span className="simulation-preview__stat-unit">Δ</span>
          </div>
        </div>

        <div className="simulation-preview__stat">
          <div className="simulation-preview__stat-label">Status</div>
          <div
            className={`simulation-preview__stat-badge ${
              isDifferent ? 'different' : 'same'
            }`}
          >
            {isDifferent ? 'Colors Differ' : 'No Change'}
          </div>
        </div>
      </div>

      <div className="simulation-preview__samples">
        <div
          className="simulation-preview__sample"
          style={{ backgroundColor: originalHex }}
        >
          <h4>Normal Vision</h4>
          <p>
            This is how text and UI elements appear to people with normal color vision. The
            color should be distinct and clearly visible.
          </p>
        </div>

        <div
          className="simulation-preview__sample"
          style={{ backgroundColor: simulatedHex }}
        >
          <h4>{typeName}</h4>
          <p>
            This is how the same color appears to people with {typeName.toLowerCase()}. Notice
            any differences in hue, saturation, or brightness.
          </p>
        </div>
      </div>

      <style>{`
        .simulation-preview {
          border: 1px solid #e5e7eb;
          border-radius: 12px;
          overflow: hidden;
          background: white;
        }

        .simulation-preview__comparison {
          display: grid;
          grid-template-columns: 1fr auto 1fr;
          align-items: center;
          background: #f9fafb;
        }

        .simulation-preview__side {
          position: relative;
        }

        .simulation-preview__color {
          height: 250px;
          position: relative;
        }

        .simulation-preview__overlay {
          position: absolute;
          inset: 0;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          gap: 8px;
          padding: 20px;
          background: linear-gradient(to bottom, rgba(0, 0, 0, 0.3), rgba(0, 0, 0, 0.5));
          color: white;
        }

        .simulation-preview__label {
          font-size: 16px;
          font-weight: 600;
          text-align: center;
        }

        .simulation-preview__hex {
          font-family: monospace;
          font-size: 20px;
          font-weight: 700;
        }

        .simulation-preview__rgb {
          font-family: monospace;
          font-size: 14px;
          opacity: 0.9;
        }

        .simulation-preview__arrow {
          padding: 0 20px;
          color: #6b7280;
        }

        .simulation-preview__info {
          display: grid;
          grid-template-columns: 1fr 1fr;
          border-top: 1px solid #e5e7eb;
          border-bottom: 1px solid #e5e7eb;
        }

        .simulation-preview__stat {
          padding: 20px;
          text-align: center;
        }

        .simulation-preview__stat:first-child {
          border-right: 1px solid #e5e7eb;
        }

        .simulation-preview__stat-label {
          font-size: 12px;
          color: #6b7280;
          margin-bottom: 8px;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }

        .simulation-preview__stat-value {
          font-size: 24px;
          font-weight: 700;
          color: #111827;
        }

        .simulation-preview__stat-unit {
          font-size: 18px;
          color: #6b7280;
        }

        .simulation-preview__stat-badge {
          display: inline-block;
          padding: 6px 16px;
          border-radius: 12px;
          font-size: 14px;
          font-weight: 600;
        }

        .simulation-preview__stat-badge.same {
          background: #dcfce7;
          color: #15803d;
        }

        .simulation-preview__stat-badge.different {
          background: #fef3c7;
          color: #92400e;
        }

        .simulation-preview__samples {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 1px;
          background: #e5e7eb;
        }

        .simulation-preview__sample {
          padding: 24px;
          background: white;
        }

        .simulation-preview__sample h4 {
          margin: 0 0 12px 0;
          font-size: 16px;
          font-weight: 600;
        }

        .simulation-preview__sample p {
          margin: 0;
          font-size: 14px;
          line-height: 1.6;
          color: #6b7280;
        }

        @media (max-width: 768px) {
          .simulation-preview__comparison {
            grid-template-columns: 1fr;
            grid-template-rows: auto auto auto;
          }

          .simulation-preview__arrow {
            transform: rotate(90deg);
            padding: 10px 0;
          }

          .simulation-preview__color {
            height: 200px;
          }

          .simulation-preview__samples {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
};
