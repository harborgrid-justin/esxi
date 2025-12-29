/**
 * Accessible color picker component
 */

import React, { useState, useRef, useCallback } from 'react';
import { RGB } from '../../types';
import { hexToRgb, rgbToHex, isValidHex } from '../../utils/colorMath';

export interface ColorPickerProps {
  /** Current color (hex or RGB) */
  color: string | RGB;
  /** On change callback */
  onChange: (color: string | RGB) => void;
  /** Label for accessibility */
  label?: string;
  /** Show RGB inputs */
  showRGB?: boolean;
  /** Custom class name */
  className?: string;
}

/**
 * Accessible color picker with hex and RGB inputs
 */
export const ColorPicker: React.FC<ColorPickerProps> = ({
  color,
  onChange,
  label = 'Color',
  showRGB = true,
  className = '',
}) => {
  const hexValue = typeof color === 'string' ? color : rgbToHex(color);
  const rgbValue = typeof color === 'string' ? hexToRgb(color) : color;

  const [inputValue, setInputValue] = useState(hexValue);
  const inputRef = useRef<HTMLInputElement>(null);

  const handleHexChange = useCallback(
    (value: string) => {
      setInputValue(value);
      if (isValidHex(value)) {
        onChange(value);
      }
    },
    [onChange]
  );

  const handleRGBChange = useCallback(
    (channel: 'r' | 'g' | 'b', value: number) => {
      const newRgb = { ...rgbValue, [channel]: Math.max(0, Math.min(255, value)) };
      onChange(newRgb);
    },
    [rgbValue, onChange]
  );

  const handleColorPickerChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const hex = e.target.value;
      setInputValue(hex);
      onChange(hex);
    },
    [onChange]
  );

  return (
    <div className={`color-picker ${className}`}>
      <div className="color-picker__main">
        <input
          ref={inputRef}
          type="color"
          value={hexValue}
          onChange={handleColorPickerChange}
          className="color-picker__input"
          aria-label={label}
        />

        <div className="color-picker__preview" style={{ backgroundColor: hexValue }}>
          <span className="color-picker__preview-text">{hexValue}</span>
        </div>
      </div>

      <div className="color-picker__hex">
        <label htmlFor={`hex-${label}`} className="color-picker__label">
          Hex
        </label>
        <input
          id={`hex-${label}`}
          type="text"
          value={inputValue}
          onChange={(e) => handleHexChange(e.target.value)}
          className="color-picker__hex-input"
          placeholder="#000000"
          maxLength={7}
        />
      </div>

      {showRGB && (
        <div className="color-picker__rgb">
          <div className="color-picker__rgb-channel">
            <label htmlFor={`r-${label}`} className="color-picker__label">
              R
            </label>
            <input
              id={`r-${label}`}
              type="number"
              value={rgbValue.r}
              onChange={(e) => handleRGBChange('r', parseInt(e.target.value) || 0)}
              min={0}
              max={255}
              className="color-picker__rgb-input"
            />
          </div>

          <div className="color-picker__rgb-channel">
            <label htmlFor={`g-${label}`} className="color-picker__label">
              G
            </label>
            <input
              id={`g-${label}`}
              type="number"
              value={rgbValue.g}
              onChange={(e) => handleRGBChange('g', parseInt(e.target.value) || 0)}
              min={0}
              max={255}
              className="color-picker__rgb-input"
            />
          </div>

          <div className="color-picker__rgb-channel">
            <label htmlFor={`b-${label}`} className="color-picker__label">
              B
            </label>
            <input
              id={`b-${label}`}
              type="number"
              value={rgbValue.b}
              onChange={(e) => handleRGBChange('b', parseInt(e.target.value) || 0)}
              min={0}
              max={255}
              className="color-picker__rgb-input"
            />
          </div>
        </div>
      )}

      <style>{`
        .color-picker {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .color-picker__main {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .color-picker__input {
          width: 60px;
          height: 60px;
          border: 2px solid #e5e7eb;
          border-radius: 8px;
          cursor: pointer;
        }

        .color-picker__preview {
          flex: 1;
          height: 60px;
          border: 2px solid #e5e7eb;
          border-radius: 8px;
          display: flex;
          align-items: center;
          justify-content: center;
          font-family: monospace;
          font-weight: 600;
          font-size: 14px;
        }

        .color-picker__preview-text {
          padding: 4px 8px;
          background: rgba(255, 255, 255, 0.9);
          border-radius: 4px;
          color: #000;
        }

        .color-picker__hex,
        .color-picker__rgb {
          display: flex;
          gap: 8px;
          align-items: center;
        }

        .color-picker__label {
          font-size: 12px;
          font-weight: 500;
          color: #6b7280;
          min-width: 20px;
        }

        .color-picker__hex-input {
          flex: 1;
          padding: 8px 12px;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          font-family: monospace;
          font-size: 14px;
        }

        .color-picker__hex-input:focus {
          outline: none;
          border-color: #3b82f6;
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        }

        .color-picker__rgb {
          display: grid;
          grid-template-columns: repeat(3, 1fr);
          gap: 8px;
        }

        .color-picker__rgb-channel {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .color-picker__rgb-input {
          padding: 8px;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          font-size: 14px;
          width: 100%;
        }

        .color-picker__rgb-input:focus {
          outline: none;
          border-color: #3b82f6;
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        }

        /* Remove number input spinners */
        .color-picker__rgb-input::-webkit-inner-spin-button,
        .color-picker__rgb-input::-webkit-outer-spin-button {
          -webkit-appearance: none;
          margin: 0;
        }

        .color-picker__rgb-input[type=number] {
          -moz-appearance: textfield;
        }
      `}</style>
    </div>
  );
};
