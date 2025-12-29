/**
 * Contrast suggestions component
 * Provides auto-fix suggestions for accessibility
 */

import React from 'react';
import { RGB, ColorSuggestion, WCAGConformance } from '../../types';
import { generateColorSuggestions } from '../../algorithms/ColorOptimizer';
import { rgbToHex } from '../../utils/colorMath';

export interface ContrastSuggestionsProps {
  /** Foreground color */
  foreground: RGB;
  /** Background color */
  background: RGB;
  /** Target conformance level */
  target?: WCAGConformance;
  /** Maximum suggestions to show */
  maxSuggestions?: number;
  /** On suggestion selected */
  onSelect?: (color: RGB) => void;
  /** Custom class name */
  className?: string;
}

/**
 * Display accessible color suggestions
 */
export const ContrastSuggestions: React.FC<ContrastSuggestionsProps> = ({
  foreground,
  background,
  target = WCAGConformance.NORMAL_TEXT_AA,
  maxSuggestions = 10,
  onSelect,
  className = '',
}) => {
  const [suggestions, setSuggestions] = React.useState<ColorSuggestion[]>([]);
  const [loading, setLoading] = React.useState(true);

  React.useEffect(() => {
    setLoading(true);
    try {
      const results = generateColorSuggestions(foreground, background, {
        target,
        preserveHue: true,
        suggestionCount: maxSuggestions,
        maxDistance: 100,
      });
      setSuggestions(results);
    } catch (error) {
      console.error('Failed to generate suggestions:', error);
      setSuggestions([]);
    } finally {
      setLoading(false);
    }
  }, [foreground, background, target, maxSuggestions]);

  if (loading) {
    return (
      <div className={`contrast-suggestions ${className}`}>
        <div className="contrast-suggestions__loading">Generating suggestions...</div>
      </div>
    );
  }

  if (suggestions.length === 0) {
    return (
      <div className={`contrast-suggestions ${className}`}>
        <div className="contrast-suggestions__empty">
          <p>No accessible alternatives found within reasonable color distance.</p>
          <p className="contrast-suggestions__hint">
            Try choosing a significantly lighter or darker color.
          </p>
        </div>
        <style>{styles}</style>
      </div>
    );
  }

  return (
    <div className={`contrast-suggestions ${className}`}>
      <div className="contrast-suggestions__header">
        <h3>Accessible Alternatives</h3>
        <p>Click a suggestion to apply it. Ordered by similarity to original color.</p>
      </div>

      <div className="contrast-suggestions__grid">
        {suggestions.map((suggestion, index) => (
          <button
            key={index}
            className="contrast-suggestions__item"
            onClick={() => onSelect?.(suggestion.color)}
            style={{ backgroundColor: suggestion.hex }}
          >
            <div className="contrast-suggestions__overlay">
              <div className="contrast-suggestions__rank">#{index + 1}</div>
              <div className="contrast-suggestions__hex">{suggestion.hex}</div>
              <div className="contrast-suggestions__ratio">
                {suggestion.contrast.ratio.toFixed(2)}:1
              </div>
              <div className="contrast-suggestions__badge">
                {suggestion.contrast.wcag.normalTextAAA
                  ? 'AAA'
                  : suggestion.contrast.wcag.normalTextAA
                  ? 'AA'
                  : 'Large Text'}
              </div>
              <div className="contrast-suggestions__modification">
                {suggestion.modification}
              </div>
              <div className="contrast-suggestions__distance">
                ΔE: {suggestion.distance.toFixed(1)}
              </div>
            </div>
          </button>
        ))}
      </div>

      <style>{styles}</style>
    </div>
  );
};

const styles = `
  .contrast-suggestions {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }

  .contrast-suggestions__loading,
  .contrast-suggestions__empty {
    padding: 48px 24px;
    text-align: center;
    background: #f9fafb;
    border: 2px dashed #d1d5db;
    border-radius: 8px;
    color: #6b7280;
  }

  .contrast-suggestions__empty p {
    margin: 0 0 8px 0;
  }

  .contrast-suggestions__hint {
    font-size: 14px;
    opacity: 0.8;
  }

  .contrast-suggestions__header {
    margin-bottom: 20px;
  }

  .contrast-suggestions__header h3 {
    margin: 0 0 8px 0;
    font-size: 18px;
    font-weight: 600;
  }

  .contrast-suggestions__header p {
    margin: 0;
    font-size: 14px;
    color: #6b7280;
  }

  .contrast-suggestions__grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 16px;
  }

  .contrast-suggestions__item {
    position: relative;
    height: 200px;
    border: 2px solid #e5e7eb;
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.2s;
    padding: 0;
    background: none;
  }

  .contrast-suggestions__item:hover {
    border-color: #3b82f6;
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  }

  .contrast-suggestions__overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 16px;
    background: linear-gradient(to bottom, rgba(0, 0, 0, 0.4), rgba(0, 0, 0, 0.7));
    color: white;
  }

  .contrast-suggestions__rank {
    position: absolute;
    top: 8px;
    left: 8px;
    width: 28px;
    height: 28px;
    background: rgba(255, 255, 255, 0.9);
    color: #111827;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-weight: 700;
  }

  .contrast-suggestions__hex {
    font-family: monospace;
    font-size: 16px;
    font-weight: 700;
  }

  .contrast-suggestions__ratio {
    font-size: 20px;
    font-weight: 700;
  }

  .contrast-suggestions__badge {
    padding: 4px 12px;
    background: rgba(34, 197, 94, 0.9);
    border-radius: 12px;
    font-size: 11px;
    font-weight: 600;
  }

  .contrast-suggestions__modification {
    font-size: 12px;
    opacity: 0.9;
    text-transform: capitalize;
  }

  .contrast-suggestions__distance {
    font-size: 11px;
    opacity: 0.8;
    font-family: monospace;
  }

  @media (max-width: 640px) {
    .contrast-suggestions__grid {
      grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    }

    .contrast-suggestions__item {
      height: 160px;
    }
  }
`;
