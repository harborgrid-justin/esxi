/**
 * Alternative colors component
 * Shows alternative accessible color options
 */

import React from 'react';
import { RGB } from '../../types';
import { lighten, darken, saturate, desaturate, rotateHue } from '../../algorithms/ColorConverter';
import { calculateWCAGContrast } from '../../algorithms/ContrastCalculator';
import { rgbToHex } from '../../utils/colorMath';

export interface AlternativeColorsProps {
  /** Base color */
  color: RGB;
  /** Background to test against */
  background: RGB;
  /** Minimum contrast ratio */
  minRatio?: number;
  /** On color selected */
  onSelect?: (color: RGB) => void;
  /** Custom class name */
  className?: string;
}

/**
 * Display alternative color variations
 */
export const AlternativeColors: React.FC<AlternativeColorsProps> = ({
  color,
  background,
  minRatio = 4.5,
  onSelect,
  className = '',
}) => {
  const alternatives = React.useMemo(() => {
    const results: Array<{
      color: RGB;
      name: string;
      type: string;
      ratio: number;
      passes: boolean;
    }> = [];

    // Lightness variations
    for (let i = 10; i <= 90; i += 10) {
      const lightened = lighten(color, i);
      const ratio = calculateWCAGContrast(lightened, background);
      results.push({
        color: lightened,
        name: `+${i}% Lighter`,
        type: 'lightness',
        ratio,
        passes: ratio >= minRatio,
      });

      const darkened = darken(color, i);
      const darkenedRatio = calculateWCAGContrast(darkened, background);
      results.push({
        color: darkened,
        name: `-${i}% Darker`,
        type: 'lightness',
        ratio: darkenedRatio,
        passes: darkenedRatio >= minRatio,
      });
    }

    // Saturation variations
    for (let i = 20; i <= 80; i += 20) {
      const saturated = saturate(color, i);
      const ratio = calculateWCAGContrast(saturated, background);
      results.push({
        color: saturated,
        name: `+${i}% Saturated`,
        type: 'saturation',
        ratio,
        passes: ratio >= minRatio,
      });

      const desaturated = desaturate(color, i);
      const desaturatedRatio = calculateWCAGContrast(desaturated, background);
      results.push({
        color: desaturated,
        name: `-${i}% Saturated`,
        type: 'saturation',
        ratio: desaturatedRatio,
        passes: desaturatedRatio >= minRatio,
      });
    }

    // Hue rotations
    for (let i = 30; i <= 180; i += 30) {
      const rotated = rotateHue(color, i);
      const ratio = calculateWCAGContrast(rotated, background);
      results.push({
        color: rotated,
        name: `+${i}° Hue`,
        type: 'hue',
        ratio,
        passes: ratio >= minRatio,
      });

      const rotatedNeg = rotateHue(color, -i);
      const ratioNeg = calculateWCAGContrast(rotatedNeg, background);
      results.push({
        color: rotatedNeg,
        name: `-${i}° Hue`,
        type: 'hue',
        ratio: ratioNeg,
        passes: ratioNeg >= minRatio,
      });
    }

    // Sort: passing first, then by ratio
    return results.sort((a, b) => {
      if (a.passes && !b.passes) return -1;
      if (!a.passes && b.passes) return 1;
      return b.ratio - a.ratio;
    });
  }, [color, background, minRatio]);

  const passingAlternatives = alternatives.filter((a) => a.passes);
  const failingAlternatives = alternatives.filter((a) => !a.passes);

  return (
    <div className={`alternative-colors ${className}`}>
      {passingAlternatives.length > 0 && (
        <div className="alternative-colors__section">
          <h3>Passing Alternatives ({passingAlternatives.length})</h3>
          <div className="alternative-colors__grid">
            {passingAlternatives.slice(0, 12).map((alt, index) => (
              <button
                key={index}
                className="alternative-colors__item passing"
                onClick={() => onSelect?.(alt.color)}
                style={{ backgroundColor: rgbToHex(alt.color) }}
              >
                <div className="alternative-colors__overlay">
                  <div className="alternative-colors__name">{alt.name}</div>
                  <div className="alternative-colors__ratio">{alt.ratio.toFixed(2)}:1</div>
                  <div className="alternative-colors__type">{alt.type}</div>
                </div>
              </button>
            ))}
          </div>
        </div>
      )}

      {failingAlternatives.length > 0 && (
        <div className="alternative-colors__section">
          <h3>Other Variations ({failingAlternatives.length})</h3>
          <p className="alternative-colors__note">
            These variations do not meet the contrast requirement but may be useful for reference.
          </p>
          <div className="alternative-colors__grid">
            {failingAlternatives.slice(0, 12).map((alt, index) => (
              <button
                key={index}
                className="alternative-colors__item failing"
                onClick={() => onSelect?.(alt.color)}
                style={{ backgroundColor: rgbToHex(alt.color) }}
              >
                <div className="alternative-colors__overlay">
                  <div className="alternative-colors__name">{alt.name}</div>
                  <div className="alternative-colors__ratio">{alt.ratio.toFixed(2)}:1</div>
                  <div className="alternative-colors__type">{alt.type}</div>
                </div>
              </button>
            ))}
          </div>
        </div>
      )}

      <style>{`
        .alternative-colors {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }

        .alternative-colors__section {
          margin-bottom: 32px;
        }

        .alternative-colors__section h3 {
          margin: 0 0 8px 0;
          font-size: 18px;
          font-weight: 600;
        }

        .alternative-colors__note {
          margin: 0 0 16px 0;
          font-size: 14px;
          color: #6b7280;
        }

        .alternative-colors__grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
          gap: 12px;
        }

        .alternative-colors__item {
          position: relative;
          height: 140px;
          border: 2px solid #e5e7eb;
          border-radius: 8px;
          overflow: hidden;
          cursor: pointer;
          transition: all 0.2s;
          padding: 0;
          background: none;
        }

        .alternative-colors__item.passing {
          border-color: #22c55e;
        }

        .alternative-colors__item.failing {
          border-color: #f87171;
          opacity: 0.7;
        }

        .alternative-colors__item:hover {
          transform: translateY(-2px);
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        }

        .alternative-colors__item.passing:hover {
          border-color: #16a34a;
        }

        .alternative-colors__item.failing:hover {
          opacity: 1;
          border-color: #dc2626;
        }

        .alternative-colors__overlay {
          position: absolute;
          inset: 0;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          gap: 6px;
          padding: 12px;
          background: linear-gradient(to bottom, rgba(0, 0, 0, 0.3), rgba(0, 0, 0, 0.6));
          color: white;
        }

        .alternative-colors__name {
          font-size: 13px;
          font-weight: 600;
          text-align: center;
        }

        .alternative-colors__ratio {
          font-size: 18px;
          font-weight: 700;
          font-family: monospace;
        }

        .alternative-colors__type {
          font-size: 11px;
          opacity: 0.8;
          text-transform: capitalize;
          background: rgba(255, 255, 255, 0.2);
          padding: 2px 8px;
          border-radius: 8px;
        }

        @media (max-width: 640px) {
          .alternative-colors__grid {
            grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
          }

          .alternative-colors__item {
            height: 120px;
          }
        }
      `}</style>
    </div>
  );
};
