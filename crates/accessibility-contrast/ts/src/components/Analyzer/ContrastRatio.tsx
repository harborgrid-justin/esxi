/**
 * Contrast ratio display component with pass/fail indicators
 */

import React from 'react';
import { ContrastResult } from '../../types';

export interface ContrastRatioProps {
  /** Contrast ratio */
  ratio: number;
  /** WCAG compliance */
  wcag: ContrastResult['wcag'];
  /** APCA compliance */
  apca: ContrastResult['apca'];
  /** Grade (A-F) */
  grade?: string;
  /** Custom class name */
  className?: string;
}

/**
 * Display contrast ratio with compliance indicators
 */
export const ContrastRatio: React.FC<ContrastRatioProps> = ({
  ratio,
  wcag,
  apca,
  grade,
  className = '',
}) => {
  const getGradeColor = (g: string): string => {
    if (g === 'A+' || g === 'A') return '#22c55e';
    if (g === 'B') return '#3b82f6';
    if (g === 'C') return '#f59e0b';
    if (g === 'D') return '#f97316';
    return '#ef4444';
  };

  return (
    <div className={`contrast-ratio ${className}`}>
      <div className="contrast-ratio__main">
        <div className="contrast-ratio__score">
          <div className="contrast-ratio__number">{ratio.toFixed(2)}</div>
          <div className="contrast-ratio__label">Contrast Ratio</div>
          {grade && (
            <div
              className="contrast-ratio__grade"
              style={{ backgroundColor: getGradeColor(grade) }}
            >
              {grade}
            </div>
          )}
        </div>

        <div className="contrast-ratio__wcag">
          <h4>WCAG 2.1 Compliance</h4>
          <div className="contrast-ratio__checks">
            <div className={`contrast-ratio__check ${wcag.normalTextAA ? 'pass' : 'fail'}`}>
              <span className="contrast-ratio__check-icon">
                {wcag.normalTextAA ? '✓' : '✗'}
              </span>
              <span className="contrast-ratio__check-label">Normal Text AA</span>
              <span className="contrast-ratio__check-value">4.5:1</span>
            </div>

            <div className={`contrast-ratio__check ${wcag.normalTextAAA ? 'pass' : 'fail'}`}>
              <span className="contrast-ratio__check-icon">
                {wcag.normalTextAAA ? '✓' : '✗'}
              </span>
              <span className="contrast-ratio__check-label">Normal Text AAA</span>
              <span className="contrast-ratio__check-value">7:1</span>
            </div>

            <div className={`contrast-ratio__check ${wcag.largeTextAA ? 'pass' : 'fail'}`}>
              <span className="contrast-ratio__check-icon">
                {wcag.largeTextAA ? '✓' : '✗'}
              </span>
              <span className="contrast-ratio__check-label">Large Text AA</span>
              <span className="contrast-ratio__check-value">3:1</span>
            </div>

            <div className={`contrast-ratio__check ${wcag.largeTextAAA ? 'pass' : 'fail'}`}>
              <span className="contrast-ratio__check-icon">
                {wcag.largeTextAAA ? '✓' : '✗'}
              </span>
              <span className="contrast-ratio__check-label">Large Text AAA</span>
              <span className="contrast-ratio__check-value">4.5:1</span>
            </div>

            <div className={`contrast-ratio__check ${wcag.uiComponents ? 'pass' : 'fail'}`}>
              <span className="contrast-ratio__check-icon">
                {wcag.uiComponents ? '✓' : '✗'}
              </span>
              <span className="contrast-ratio__check-label">UI Components</span>
              <span className="contrast-ratio__check-value">3:1</span>
            </div>
          </div>
        </div>

        <div className="contrast-ratio__apca">
          <h4>APCA (Advanced Perceptual Contrast)</h4>
          <div className="contrast-ratio__apca-score">
            <span className="contrast-ratio__apca-value">
              Lc {Math.abs(apca.score).toFixed(1)}
            </span>
            <span className={`contrast-ratio__apca-status ${apca.compliant ? 'pass' : 'fail'}`}>
              {apca.compliant ? 'Compliant' : 'Non-compliant'}
            </span>
          </div>
          {apca.minFontSize && (
            <div className="contrast-ratio__apca-detail">
              Minimum font size: {apca.minFontSize}px
            </div>
          )}
        </div>
      </div>

      <style>{`
        .contrast-ratio {
          background: #f9fafb;
          border: 1px solid #e5e7eb;
          border-radius: 12px;
          padding: 24px;
          margin: 24px 0;
        }

        .contrast-ratio__main {
          display: grid;
          gap: 24px;
        }

        .contrast-ratio__score {
          text-align: center;
          padding: 24px;
          background: white;
          border-radius: 8px;
          position: relative;
        }

        .contrast-ratio__number {
          font-size: 48px;
          font-weight: 700;
          color: #111827;
          line-height: 1;
        }

        .contrast-ratio__label {
          font-size: 14px;
          color: #6b7280;
          margin-top: 8px;
          font-weight: 500;
        }

        .contrast-ratio__grade {
          position: absolute;
          top: 16px;
          right: 16px;
          width: 40px;
          height: 40px;
          border-radius: 50%;
          display: flex;
          align-items: center;
          justify-content: center;
          color: white;
          font-weight: 700;
          font-size: 16px;
        }

        .contrast-ratio__wcag,
        .contrast-ratio__apca {
          background: white;
          border-radius: 8px;
          padding: 20px;
        }

        .contrast-ratio__wcag h4,
        .contrast-ratio__apca h4 {
          margin: 0 0 16px 0;
          font-size: 16px;
          font-weight: 600;
          color: #111827;
        }

        .contrast-ratio__checks {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .contrast-ratio__check {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 12px;
          border-radius: 6px;
          font-size: 14px;
        }

        .contrast-ratio__check.pass {
          background: #f0fdf4;
          border: 1px solid #bbf7d0;
        }

        .contrast-ratio__check.fail {
          background: #fef2f2;
          border: 1px solid #fecaca;
        }

        .contrast-ratio__check-icon {
          font-weight: 700;
          font-size: 16px;
        }

        .contrast-ratio__check.pass .contrast-ratio__check-icon {
          color: #22c55e;
        }

        .contrast-ratio__check.fail .contrast-ratio__check-icon {
          color: #ef4444;
        }

        .contrast-ratio__check-label {
          flex: 1;
          font-weight: 500;
        }

        .contrast-ratio__check-value {
          color: #6b7280;
          font-size: 12px;
          font-family: monospace;
        }

        .contrast-ratio__apca-score {
          display: flex;
          align-items: center;
          gap: 12px;
          margin-bottom: 8px;
        }

        .contrast-ratio__apca-value {
          font-size: 24px;
          font-weight: 700;
          font-family: monospace;
        }

        .contrast-ratio__apca-status {
          padding: 4px 12px;
          border-radius: 12px;
          font-size: 12px;
          font-weight: 600;
        }

        .contrast-ratio__apca-status.pass {
          background: #dcfce7;
          color: #15803d;
        }

        .contrast-ratio__apca-status.fail {
          background: #fee2e2;
          color: #991b1b;
        }

        .contrast-ratio__apca-detail {
          font-size: 13px;
          color: #6b7280;
        }

        @media (min-width: 768px) {
          .contrast-ratio__main {
            grid-template-columns: 200px 1fr;
            grid-template-rows: auto auto;
          }

          .contrast-ratio__score {
            grid-row: 1 / 3;
          }
        }
      `}</style>
    </div>
  );
};
