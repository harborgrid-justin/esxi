/**
 * HeatmapChart - Heatmap visualization component
 */

import React from 'react';
import { ChartOptions } from '../../types';

export interface HeatmapChartProps {
  data: any[];
  xKey: string;
  yKey: string;
  options?: ChartOptions;
}

export const HeatmapChart: React.FC<HeatmapChartProps> = ({
  data,
  xKey,
  yKey,
  options = {
    legend: true,
    grid: true,
    tooltip: true,
    animation: true,
    stacked: false,
  },
}) => {
  // Extract unique X and Y values
  const xValues = [...new Set(data.map((d) => d[xKey]))];
  const yValues = [...new Set(data.map((d) => d[yKey]))];

  // Find min/max for color scaling
  const values = data.map((d) => d.value || 0);
  const minValue = Math.min(...values);
  const maxValue = Math.max(...values);

  const getColor = (value: number): string => {
    const intensity = (value - minValue) / (maxValue - minValue);
    const colors = options.colors || ['#f3f3f3', '#1976d2'];

    if (colors.length === 2) {
      // Simple gradient between two colors
      const r1 = parseInt(colors[0].slice(1, 3), 16);
      const g1 = parseInt(colors[0].slice(3, 5), 16);
      const b1 = parseInt(colors[0].slice(5, 7), 16);
      const r2 = parseInt(colors[1].slice(1, 3), 16);
      const g2 = parseInt(colors[1].slice(3, 5), 16);
      const b2 = parseInt(colors[1].slice(5, 7), 16);

      const r = Math.round(r1 + (r2 - r1) * intensity);
      const g = Math.round(g1 + (g2 - g1) * intensity);
      const b = Math.round(b1 + (b2 - b1) * intensity);

      return `rgb(${r}, ${g}, ${b})`;
    }

    return colors[0];
  };

  const getValue = (x: any, y: any): number => {
    const item = data.find((d) => d[xKey] === x && d[yKey] === y);
    return item?.value || 0;
  };

  return (
    <div className="heatmap-chart">
      <div className="heatmap-grid">
        {/* Y-axis labels */}
        <div className="y-axis">
          <div className="y-label-spacer"></div>
          {yValues.map((y, index) => (
            <div key={index} className="y-label">
              {String(y)}
            </div>
          ))}
        </div>

        {/* Heatmap cells */}
        <div className="heatmap-content">
          {/* X-axis labels */}
          <div className="x-axis">
            {xValues.map((x, index) => (
              <div key={index} className="x-label">
                {String(x)}
              </div>
            ))}
          </div>

          {/* Grid */}
          <div className="grid">
            {yValues.map((y, yIndex) => (
              <div key={yIndex} className="row">
                {xValues.map((x, xIndex) => {
                  const value = getValue(x, y);
                  return (
                    <div
                      key={xIndex}
                      className="cell"
                      style={{
                        backgroundColor: getColor(value),
                      }}
                      title={`${x}, ${y}: ${value}`}
                    >
                      {options.legend && <span className="cell-value">{value}</span>}
                    </div>
                  );
                })}
              </div>
            ))}
          </div>
        </div>
      </div>

      {options.legend && (
        <div className="legend">
          <span>Low</span>
          <div className="gradient"></div>
          <span>High</span>
        </div>
      )}

      <style jsx>{`
        .heatmap-chart {
          width: 100%;
          height: 100%;
          display: flex;
          flex-direction: column;
          padding: 8px;
        }

        .heatmap-grid {
          display: flex;
          flex: 1;
          overflow: auto;
        }

        .y-axis {
          display: flex;
          flex-direction: column;
          margin-right: 8px;
        }

        .y-label-spacer {
          height: 30px;
        }

        .y-label {
          display: flex;
          align-items: center;
          justify-content: flex-end;
          padding-right: 8px;
          font-size: 12px;
          height: 40px;
        }

        .heatmap-content {
          flex: 1;
          display: flex;
          flex-direction: column;
        }

        .x-axis {
          display: flex;
          height: 30px;
          margin-bottom: 4px;
        }

        .x-label {
          flex: 1;
          min-width: 60px;
          text-align: center;
          font-size: 12px;
          display: flex;
          align-items: center;
          justify-content: center;
        }

        .grid {
          display: flex;
          flex-direction: column;
        }

        .row {
          display: flex;
          margin-bottom: 2px;
        }

        .cell {
          flex: 1;
          min-width: 60px;
          height: 40px;
          margin-right: 2px;
          border-radius: 2px;
          display: flex;
          align-items: center;
          justify-content: center;
          cursor: pointer;
          transition: transform 0.2s;
        }

        .cell:hover {
          transform: scale(1.05);
          z-index: 10;
        }

        .cell-value {
          font-size: 11px;
          font-weight: 600;
        }

        .legend {
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 8px;
          margin-top: 12px;
          font-size: 12px;
        }

        .gradient {
          width: 100px;
          height: 12px;
          background: linear-gradient(to right, #f3f3f3, #1976d2);
          border-radius: 2px;
        }
      `}</style>
    </div>
  );
};
