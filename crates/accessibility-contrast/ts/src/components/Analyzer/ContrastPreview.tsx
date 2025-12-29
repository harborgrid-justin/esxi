/**
 * Live contrast preview component
 */

import React from 'react';
import { RGB, ContrastResult } from '../../types';
import { rgbToHex } from '../../utils/colorMath';

export interface ContrastPreviewProps {
  /** Foreground color */
  foreground: RGB;
  /** Background color */
  background: RGB;
  /** Contrast result */
  contrast: ContrastResult;
  /** Custom class name */
  className?: string;
}

/**
 * Preview component showing actual text on background
 */
export const ContrastPreview: React.FC<ContrastPreviewProps> = ({
  foreground,
  background,
  contrast,
  className = '',
}) => {
  const fgHex = rgbToHex(foreground);
  const bgHex = rgbToHex(background);

  return (
    <div className={`contrast-preview ${className}`}>
      <div
        className="contrast-preview__large"
        style={{
          backgroundColor: bgHex,
          color: fgHex,
        }}
      >
        <div className="contrast-preview__text">
          <h1 className="contrast-preview__heading">The quick brown fox</h1>
          <p className="contrast-preview__body">
            The quick brown fox jumps over the lazy dog. This preview demonstrates how your
            text will appear with the selected color combination.
          </p>
          <p className="contrast-preview__small">
            Small text (14px) is the most demanding for contrast requirements.
          </p>
        </div>
      </div>

      <div className="contrast-preview__samples">
        <div
          className="contrast-preview__sample"
          style={{ backgroundColor: bgHex, color: fgHex }}
        >
          <strong>Normal Text</strong>
          <span className={contrast.wcag.normalTextAA ? 'pass' : 'fail'}>
            {contrast.wcag.normalTextAA ? '✓ AA' : '✗ AA'}
          </span>
          <span className={contrast.wcag.normalTextAAA ? 'pass' : 'fail'}>
            {contrast.wcag.normalTextAAA ? '✓ AAA' : '✗ AAA'}
          </span>
        </div>

        <div
          className="contrast-preview__sample large"
          style={{ backgroundColor: bgHex, color: fgHex }}
        >
          <strong>Large Text (18pt+)</strong>
          <span className={contrast.wcag.largeTextAA ? 'pass' : 'fail'}>
            {contrast.wcag.largeTextAA ? '✓ AA' : '✗ AA'}
          </span>
          <span className={contrast.wcag.largeTextAAA ? 'pass' : 'fail'}>
            {contrast.wcag.largeTextAAA ? '✓ AAA' : '✗ AAA'}
          </span>
        </div>

        <div
          className="contrast-preview__sample"
          style={{ backgroundColor: bgHex, color: fgHex }}
        >
          <strong>UI Components</strong>
          <span className={contrast.wcag.uiComponents ? 'pass' : 'fail'}>
            {contrast.wcag.uiComponents ? '✓ Pass' : '✗ Fail'}
          </span>
        </div>
      </div>

      <style>{`
        .contrast-preview {
          border: 2px solid #e5e7eb;
          border-radius: 12px;
          overflow: hidden;
          margin: 24px 0;
        }

        .contrast-preview__large {
          padding: 48px 32px;
          min-height: 200px;
        }

        .contrast-preview__text {
          max-width: 600px;
        }

        .contrast-preview__heading {
          font-size: 32px;
          font-weight: 700;
          margin: 0 0 16px 0;
          line-height: 1.2;
        }

        .contrast-preview__body {
          font-size: 16px;
          line-height: 1.6;
          margin: 0 0 16px 0;
        }

        .contrast-preview__small {
          font-size: 14px;
          line-height: 1.5;
          margin: 0;
        }

        .contrast-preview__samples {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 1px;
          background: #e5e7eb;
        }

        .contrast-preview__sample {
          padding: 20px;
          display: flex;
          flex-direction: column;
          gap: 8px;
          font-size: 14px;
        }

        .contrast-preview__sample.large {
          font-size: 18px;
        }

        .contrast-preview__sample strong {
          font-weight: 600;
          margin-bottom: 4px;
        }

        .contrast-preview__sample span {
          display: inline-block;
          padding: 4px 8px;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 600;
          margin-right: 8px;
        }

        .contrast-preview__sample span.pass {
          background: rgba(34, 197, 94, 0.2);
          color: #15803d;
        }

        .contrast-preview__sample span.fail {
          background: rgba(239, 68, 68, 0.2);
          color: #991b1b;
        }

        @media (max-width: 640px) {
          .contrast-preview__large {
            padding: 32px 20px;
          }

          .contrast-preview__heading {
            font-size: 24px;
          }

          .contrast-preview__samples {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
};
