/**
 * Color blindness simulator component
 */

import React, { useState } from 'react';
import { ColorBlindnessType } from '../../types';
import { useColorBlindness } from '../../hooks/useColorBlindness';
import { SimulationPreview } from './SimulationPreview';

export interface ColorBlindSimulatorProps {
  /** Initial color */
  initialColor?: string;
  /** Show all simulations */
  showAll?: boolean;
  /** Custom class name */
  className?: string;
}

/**
 * Simulate color blindness for a single color or palette
 */
export const ColorBlindSimulator: React.FC<ColorBlindSimulatorProps> = ({
  initialColor = '#3B82F6',
  showAll = true,
  className = '',
}) => {
  const [selectedType, setSelectedType] = useState<ColorBlindnessType>(
    ColorBlindnessType.DEUTERANOPIA
  );

  const {
    original,
    simulated,
    originalHex,
    simulatedHex,
    type,
    severity,
    allSimulations,
    typeName,
    typeDescription,
    setType,
    setSeverity,
    setColor,
  } = useColorBlindness({
    color: initialColor,
    type: selectedType,
    severity: 1.0,
    simulateAll: showAll,
  });

  const handleTypeChange = (newType: ColorBlindnessType) => {
    setSelectedType(newType);
    setType(newType);
  };

  const isAnomalousType = [
    ColorBlindnessType.PROTANOMALY,
    ColorBlindnessType.DEUTERANOMALY,
    ColorBlindnessType.TRITANOMALY,
    ColorBlindnessType.ACHROMATOMALY,
  ].includes(type);

  return (
    <div className={`colorblind-simulator ${className}`}>
      <div className="colorblind-simulator__header">
        <h2>Color Blindness Simulator</h2>
        <p>Test your colors against various types of color vision deficiency</p>
      </div>

      <div className="colorblind-simulator__controls">
        <div className="colorblind-simulator__color-input">
          <label htmlFor="color-input">Color to Test</label>
          <div className="colorblind-simulator__color-row">
            <input
              id="color-input"
              type="color"
              value={originalHex || '#3B82F6'}
              onChange={(e) => setColor(e.target.value)}
              className="colorblind-simulator__picker"
            />
            <input
              type="text"
              value={originalHex || '#3B82F6'}
              onChange={(e) => setColor(e.target.value)}
              className="colorblind-simulator__hex-input"
            />
          </div>
        </div>

        <div className="colorblind-simulator__type-select">
          <label htmlFor="type-select">Deficiency Type</label>
          <select
            id="type-select"
            value={type}
            onChange={(e) => handleTypeChange(e.target.value as ColorBlindnessType)}
            className="colorblind-simulator__select"
          >
            <optgroup label="Complete (Dichromacy)">
              <option value={ColorBlindnessType.PROTANOPIA}>Protanopia (Red-Blind)</option>
              <option value={ColorBlindnessType.DEUTERANOPIA}>Deuteranopia (Green-Blind)</option>
              <option value={ColorBlindnessType.TRITANOPIA}>Tritanopia (Blue-Blind)</option>
              <option value={ColorBlindnessType.ACHROMATOPSIA}>Achromatopsia (Total)</option>
            </optgroup>
            <optgroup label="Partial (Anomalous Trichromacy)">
              <option value={ColorBlindnessType.PROTANOMALY}>Protanomaly (Red-Weak)</option>
              <option value={ColorBlindnessType.DEUTERANOMALY}>Deuteranomaly (Green-Weak)</option>
              <option value={ColorBlindnessType.TRITANOMALY}>Tritanomaly (Blue-Weak)</option>
              <option value={ColorBlindnessType.ACHROMATOMALY}>Achromatomaly (Blue Cone)</option>
            </optgroup>
          </select>
        </div>

        {isAnomalousType && (
          <div className="colorblind-simulator__severity">
            <label htmlFor="severity-slider">
              Severity: {Math.round(severity * 100)}%
            </label>
            <input
              id="severity-slider"
              type="range"
              min="0"
              max="1"
              step="0.1"
              value={severity}
              onChange={(e) => setSeverity(parseFloat(e.target.value))}
              className="colorblind-simulator__slider"
            />
          </div>
        )}
      </div>

      <div className="colorblind-simulator__info">
        <h3>{typeName}</h3>
        <p>{typeDescription}</p>
      </div>

      {original && simulated && (
        <SimulationPreview
          original={original}
          simulated={simulated}
          type={type}
          typeName={typeName}
        />
      )}

      {showAll && allSimulations.length > 0 && (
        <div className="colorblind-simulator__all">
          <h3>All Simulations</h3>
          <div className="colorblind-simulator__grid">
            {allSimulations.map((sim) => (
              <div
                key={sim.type}
                className="colorblind-simulator__item"
                onClick={() => handleTypeChange(sim.type)}
                style={{ cursor: 'pointer' }}
              >
                <div className="colorblind-simulator__item-colors">
                  <div
                    className="colorblind-simulator__item-color"
                    style={{
                      backgroundColor: `rgb(${sim.original.r}, ${sim.original.g}, ${sim.original.b})`,
                    }}
                  />
                  <div className="colorblind-simulator__item-arrow">→</div>
                  <div
                    className="colorblind-simulator__item-color"
                    style={{
                      backgroundColor: `rgb(${sim.simulated.r}, ${sim.simulated.g}, ${sim.simulated.b})`,
                    }}
                  />
                </div>
                <div className="colorblind-simulator__item-name">
                  {sim.type.replace(/([A-Z])/g, ' $1').trim()}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      <style>{`
        .colorblind-simulator {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
          max-width: 1000px;
          margin: 0 auto;
          padding: 24px;
        }

        .colorblind-simulator__header {
          margin-bottom: 24px;
        }

        .colorblind-simulator__header h2 {
          margin: 0 0 8px 0;
          font-size: 24px;
          font-weight: 600;
        }

        .colorblind-simulator__header p {
          margin: 0;
          font-size: 14px;
          color: #6b7280;
        }

        .colorblind-simulator__controls {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 16px;
          margin-bottom: 24px;
        }

        .colorblind-simulator__color-input,
        .colorblind-simulator__type-select,
        .colorblind-simulator__severity {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .colorblind-simulator__severity {
          grid-column: 1 / -1;
        }

        .colorblind-simulator__controls label {
          font-size: 14px;
          font-weight: 500;
        }

        .colorblind-simulator__color-row {
          display: flex;
          gap: 12px;
        }

        .colorblind-simulator__picker {
          width: 60px;
          height: 40px;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          cursor: pointer;
        }

        .colorblind-simulator__hex-input {
          flex: 1;
          padding: 8px 12px;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          font-family: monospace;
          font-size: 14px;
        }

        .colorblind-simulator__hex-input:focus {
          outline: none;
          border-color: #3b82f6;
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        }

        .colorblind-simulator__select {
          padding: 8px 12px;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          font-size: 14px;
          background: white;
        }

        .colorblind-simulator__select:focus {
          outline: none;
          border-color: #3b82f6;
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        }

        .colorblind-simulator__slider {
          width: 100%;
        }

        .colorblind-simulator__info {
          background: #f9fafb;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          padding: 20px;
          margin-bottom: 24px;
        }

        .colorblind-simulator__info h3 {
          margin: 0 0 8px 0;
          font-size: 16px;
          font-weight: 600;
        }

        .colorblind-simulator__info p {
          margin: 0;
          font-size: 14px;
          color: #6b7280;
          line-height: 1.5;
        }

        .colorblind-simulator__all {
          margin-top: 32px;
        }

        .colorblind-simulator__all h3 {
          margin: 0 0 16px 0;
          font-size: 18px;
          font-weight: 600;
        }

        .colorblind-simulator__grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
          gap: 16px;
        }

        .colorblind-simulator__item {
          border: 2px solid #e5e7eb;
          border-radius: 8px;
          padding: 16px;
          transition: all 0.2s;
        }

        .colorblind-simulator__item:hover {
          border-color: #3b82f6;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
        }

        .colorblind-simulator__item-colors {
          display: flex;
          align-items: center;
          gap: 12px;
          margin-bottom: 12px;
        }

        .colorblind-simulator__item-color {
          flex: 1;
          height: 60px;
          border-radius: 6px;
          border: 1px solid #e5e7eb;
        }

        .colorblind-simulator__item-arrow {
          font-size: 20px;
          color: #6b7280;
        }

        .colorblind-simulator__item-name {
          font-size: 13px;
          font-weight: 500;
          text-align: center;
          text-transform: capitalize;
        }

        @media (max-width: 640px) {
          .colorblind-simulator__controls {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
};
