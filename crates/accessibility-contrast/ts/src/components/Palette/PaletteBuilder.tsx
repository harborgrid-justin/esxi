/**
 * Accessible palette builder component
 */

import React, { useState } from 'react';
import { PaletteColor } from '../../types';
import { usePalette } from '../../hooks/usePalette';
import { ColorSwatch } from './ColorSwatch';
import { PaletteMatrix } from './PaletteMatrix';
import { hexToRgb, rgbToHex } from '../../utils/colorMath';

export interface PaletteBuilderProps {
  /** Initial palette name */
  initialName?: string;
  /** Initial colors */
  initialColors?: PaletteColor[];
  /** Background color */
  background?: string;
  /** On palette change */
  onChange?: (colors: PaletteColor[]) => void;
  /** Custom class name */
  className?: string;
}

/**
 * Build and analyze accessible color palettes
 */
export const PaletteBuilder: React.FC<PaletteBuilderProps> = ({
  initialName = 'My Palette',
  initialColors = [],
  background = '#FFFFFF',
  onChange,
  className = '',
}) => {
  const {
    palette,
    isCompliant,
    addColor,
    removeColor,
    updateColor,
    clearColors,
    optimize,
    exportJSON,
  } = usePalette({
    name: initialName,
    initialColors,
    background,
  });

  const [newColorHex, setNewColorHex] = useState('#3B82F6');
  const [newColorName, setNewColorName] = useState('');

  React.useEffect(() => {
    if (onChange) {
      onChange(palette.colors);
    }
  }, [palette.colors, onChange]);

  const handleAddColor = () => {
    const color: PaletteColor = {
      id: `color-${Date.now()}`,
      name: newColorName || `Color ${palette.colors.length + 1}`,
      color: hexToRgb(newColorHex),
      hex: newColorHex,
      role: 'custom',
    };

    addColor(color);
    setNewColorName('');
    setNewColorHex('#3B82F6');
  };

  const handleExport = () => {
    const json = exportJSON();
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${palette.name.toLowerCase().replace(/\s+/g, '-')}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className={`palette-builder ${className}`}>
      <div className="palette-builder__header">
        <div>
          <h2>{palette.name}</h2>
          <div className="palette-builder__status">
            {isCompliant ? (
              <span className="palette-builder__badge success">✓ WCAG Compliant</span>
            ) : (
              <span className="palette-builder__badge warning">⚠ Not Compliant</span>
            )}
            <span className="palette-builder__count">{palette.colors.length} colors</span>
          </div>
        </div>

        <div className="palette-builder__actions">
          <button onClick={optimize} className="palette-builder__button">
            Optimize
          </button>
          <button onClick={handleExport} className="palette-builder__button">
            Export
          </button>
          <button onClick={clearColors} className="palette-builder__button danger">
            Clear All
          </button>
        </div>
      </div>

      <div className="palette-builder__add">
        <input
          type="color"
          value={newColorHex}
          onChange={(e) => setNewColorHex(e.target.value)}
          className="palette-builder__color-input"
        />
        <input
          type="text"
          value={newColorName}
          onChange={(e) => setNewColorName(e.target.value)}
          placeholder="Color name (optional)"
          className="palette-builder__text-input"
        />
        <button onClick={handleAddColor} className="palette-builder__button primary">
          Add Color
        </button>
      </div>

      {palette.colors.length > 0 ? (
        <>
          <div className="palette-builder__colors">
            {palette.colors.map((color) => (
              <ColorSwatch
                key={color.id}
                color={color}
                onRemove={() => removeColor(color.id)}
                onUpdate={(updates) => updateColor(color.id, updates)}
              />
            ))}
          </div>

          <PaletteMatrix palette={palette} />
        </>
      ) : (
        <div className="palette-builder__empty">
          <p>No colors in palette. Add a color to get started.</p>
        </div>
      )}

      <style>{`
        .palette-builder {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
          max-width: 1200px;
          margin: 0 auto;
          padding: 24px;
        }

        .palette-builder__header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 24px;
          flex-wrap: wrap;
          gap: 16px;
        }

        .palette-builder__header h2 {
          margin: 0 0 8px 0;
          font-size: 24px;
          font-weight: 600;
        }

        .palette-builder__status {
          display: flex;
          gap: 12px;
          align-items: center;
        }

        .palette-builder__badge {
          padding: 4px 12px;
          border-radius: 12px;
          font-size: 12px;
          font-weight: 600;
        }

        .palette-builder__badge.success {
          background: #dcfce7;
          color: #15803d;
        }

        .palette-builder__badge.warning {
          background: #fef3c7;
          color: #92400e;
        }

        .palette-builder__count {
          font-size: 14px;
          color: #6b7280;
        }

        .palette-builder__actions {
          display: flex;
          gap: 8px;
        }

        .palette-builder__button {
          padding: 8px 16px;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          background: white;
          cursor: pointer;
          font-size: 14px;
          font-weight: 500;
          transition: all 0.2s;
        }

        .palette-builder__button:hover {
          background: #f9fafb;
        }

        .palette-builder__button.primary {
          background: #3b82f6;
          color: white;
          border-color: #3b82f6;
        }

        .palette-builder__button.primary:hover {
          background: #2563eb;
        }

        .palette-builder__button.danger {
          color: #dc2626;
          border-color: #fca5a5;
        }

        .palette-builder__button.danger:hover {
          background: #fef2f2;
        }

        .palette-builder__add {
          display: flex;
          gap: 12px;
          align-items: center;
          padding: 20px;
          background: #f9fafb;
          border: 2px dashed #d1d5db;
          border-radius: 8px;
          margin-bottom: 24px;
        }

        .palette-builder__color-input {
          width: 60px;
          height: 40px;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          cursor: pointer;
        }

        .palette-builder__text-input {
          flex: 1;
          padding: 8px 12px;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          font-size: 14px;
        }

        .palette-builder__text-input:focus {
          outline: none;
          border-color: #3b82f6;
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        }

        .palette-builder__colors {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
          gap: 16px;
          margin-bottom: 32px;
        }

        .palette-builder__empty {
          padding: 48px;
          text-align: center;
          color: #6b7280;
          background: #f9fafb;
          border: 2px dashed #d1d5db;
          border-radius: 8px;
        }

        @media (max-width: 640px) {
          .palette-builder__header {
            flex-direction: column;
          }

          .palette-builder__actions {
            width: 100%;
          }

          .palette-builder__button {
            flex: 1;
          }

          .palette-builder__add {
            flex-direction: column;
          }

          .palette-builder__color-input,
          .palette-builder__text-input {
            width: 100%;
          }
        }
      `}</style>
    </div>
  );
};
