/**
 * Palette contrast matrix component
 * Shows contrast ratios between all color pairs
 */

import React from 'react';
import { ColorPalette } from '../../types';

export interface PaletteMatrixProps {
  /** Color palette */
  palette: ColorPalette;
  /** Minimum contrast ratio for pass */
  minRatio?: number;
  /** Custom class name */
  className?: string;
}

/**
 * Display contrast matrix for palette
 */
export const PaletteMatrix: React.FC<PaletteMatrixProps> = ({
  palette,
  minRatio = 4.5,
  className = '',
}) => {
  const getCell = (fgId: string, bgId: string) => {
    return palette.contrastMatrix.find(
      (pair) => pair.foreground.id === fgId && pair.background.id === bgId
    );
  };

  const getCellColor = (ratio: number) => {
    if (ratio >= 7) return '#22c55e';
    if (ratio >= 4.5) return '#3b82f6';
    if (ratio >= 3) return '#f59e0b';
    return '#ef4444';
  };

  // Include background in the matrix
  const allColors = [
    ...palette.colors,
    {
      id: 'background',
      name: 'Background',
      color: { r: 255, g: 255, b: 255 },
      hex: '#FFFFFF',
      role: 'background' as const,
    },
  ];

  return (
    <div className={`palette-matrix ${className}`}>
      <h3>Contrast Matrix</h3>
      <p className="palette-matrix__description">
        Contrast ratios between all color combinations. Green indicates AAA compliance (7:1+),
        blue indicates AA compliance (4.5:1+), orange indicates large text only (3:1+), and red
        indicates failure.
      </p>

      <div className="palette-matrix__table-wrapper">
        <table className="palette-matrix__table">
          <thead>
            <tr>
              <th className="palette-matrix__corner">FG \ BG</th>
              {allColors.map((color) => (
                <th key={color.id} className="palette-matrix__header">
                  <div
                    className="palette-matrix__header-color"
                    style={{ backgroundColor: color.hex }}
                  />
                  <div className="palette-matrix__header-name">{color.name}</div>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {palette.colors.map((fgColor) => (
              <tr key={fgColor.id}>
                <th className="palette-matrix__row-header">
                  <div
                    className="palette-matrix__header-color"
                    style={{ backgroundColor: fgColor.hex }}
                  />
                  <div className="palette-matrix__header-name">{fgColor.name}</div>
                </th>
                {allColors.map((bgColor) => {
                  if (fgColor.id === bgColor.id) {
                    return (
                      <td key={bgColor.id} className="palette-matrix__cell same">
                        —
                      </td>
                    );
                  }

                  const cell = getCell(fgColor.id, bgColor.id);
                  if (!cell) {
                    return (
                      <td key={bgColor.id} className="palette-matrix__cell">
                        N/A
                      </td>
                    );
                  }

                  const ratio = cell.contrast.ratio;
                  const isPassing = ratio >= minRatio;

                  return (
                    <td
                      key={bgColor.id}
                      className={`palette-matrix__cell ${isPassing ? 'pass' : 'fail'}`}
                      style={{ backgroundColor: getCellColor(ratio) }}
                    >
                      <div className="palette-matrix__ratio">{ratio.toFixed(1)}</div>
                      <div className="palette-matrix__level">
                        {cell.contrast.wcag.normalTextAAA
                          ? 'AAA'
                          : cell.contrast.wcag.normalTextAA
                          ? 'AA'
                          : cell.contrast.wcag.largeTextAA
                          ? 'Large'
                          : 'Fail'}
                      </div>
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="palette-matrix__legend">
        <div className="palette-matrix__legend-item">
          <div className="palette-matrix__legend-color" style={{ backgroundColor: '#22c55e' }} />
          <span>AAA (7:1+)</span>
        </div>
        <div className="palette-matrix__legend-item">
          <div className="palette-matrix__legend-color" style={{ backgroundColor: '#3b82f6' }} />
          <span>AA (4.5:1+)</span>
        </div>
        <div className="palette-matrix__legend-item">
          <div className="palette-matrix__legend-color" style={{ backgroundColor: '#f59e0b' }} />
          <span>Large Text (3:1+)</span>
        </div>
        <div className="palette-matrix__legend-item">
          <div className="palette-matrix__legend-color" style={{ backgroundColor: '#ef4444' }} />
          <span>Fail (&lt;3:1)</span>
        </div>
      </div>

      <style>{`
        .palette-matrix {
          margin-top: 32px;
        }

        .palette-matrix h3 {
          margin: 0 0 8px 0;
          font-size: 20px;
          font-weight: 600;
        }

        .palette-matrix__description {
          margin: 0 0 20px 0;
          font-size: 14px;
          color: #6b7280;
          line-height: 1.5;
        }

        .palette-matrix__table-wrapper {
          overflow-x: auto;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          margin-bottom: 16px;
        }

        .palette-matrix__table {
          width: 100%;
          border-collapse: collapse;
          background: white;
        }

        .palette-matrix__corner {
          background: #f9fafb;
          padding: 12px;
          font-size: 12px;
          font-weight: 600;
          text-align: left;
          border-right: 1px solid #e5e7eb;
          border-bottom: 1px solid #e5e7eb;
        }

        .palette-matrix__header,
        .palette-matrix__row-header {
          background: #f9fafb;
          padding: 12px;
          text-align: center;
          border: 1px solid #e5e7eb;
        }

        .palette-matrix__header-color {
          width: 32px;
          height: 32px;
          border-radius: 4px;
          border: 1px solid #e5e7eb;
          margin: 0 auto 8px auto;
        }

        .palette-matrix__header-name {
          font-size: 12px;
          font-weight: 600;
          max-width: 100px;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .palette-matrix__row-header {
          text-align: left;
          min-width: 120px;
        }

        .palette-matrix__row-header .palette-matrix__header-color {
          display: inline-block;
          vertical-align: middle;
          margin: 0 8px 0 0;
          width: 24px;
          height: 24px;
        }

        .palette-matrix__row-header .palette-matrix__header-name {
          display: inline-block;
          vertical-align: middle;
          max-width: none;
        }

        .palette-matrix__cell {
          padding: 12px;
          text-align: center;
          border: 1px solid #e5e7eb;
          color: white;
          font-weight: 600;
          min-width: 80px;
        }

        .palette-matrix__cell.same {
          background: #f3f4f6;
          color: #9ca3af;
        }

        .palette-matrix__ratio {
          font-size: 16px;
          margin-bottom: 4px;
        }

        .palette-matrix__level {
          font-size: 11px;
          opacity: 0.9;
        }

        .palette-matrix__legend {
          display: flex;
          gap: 16px;
          flex-wrap: wrap;
        }

        .palette-matrix__legend-item {
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 14px;
        }

        .palette-matrix__legend-color {
          width: 20px;
          height: 20px;
          border-radius: 4px;
        }

        @media (max-width: 768px) {
          .palette-matrix__header-name,
          .palette-matrix__row-header .palette-matrix__header-name {
            font-size: 10px;
          }

          .palette-matrix__cell {
            min-width: 60px;
            padding: 8px;
          }

          .palette-matrix__ratio {
            font-size: 14px;
          }

          .palette-matrix__level {
            font-size: 10px;
          }
        }
      `}</style>
    </div>
  );
};
