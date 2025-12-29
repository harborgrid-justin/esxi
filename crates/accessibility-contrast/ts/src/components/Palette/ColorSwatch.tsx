/**
 * Color swatch component for palette display
 */

import React, { useState } from 'react';
import { PaletteColor } from '../../types';
import { rgbToHex, hexToRgb } from '../../utils/colorMath';

export interface ColorSwatchProps {
  /** Color data */
  color: PaletteColor;
  /** On remove callback */
  onRemove?: () => void;
  /** On update callback */
  onUpdate?: (updates: Partial<PaletteColor>) => void;
  /** Show controls */
  showControls?: boolean;
  /** Custom class name */
  className?: string;
}

/**
 * Display and edit a single color swatch
 */
export const ColorSwatch: React.FC<ColorSwatchProps> = ({
  color,
  onRemove,
  onUpdate,
  showControls = true,
  className = '',
}) => {
  const [isEditing, setIsEditing] = useState(false);
  const [editName, setEditName] = useState(color.name);
  const [editHex, setEditHex] = useState(color.hex);

  const handleSave = () => {
    if (onUpdate) {
      onUpdate({
        name: editName,
        hex: editHex,
        color: hexToRgb(editHex),
      });
    }
    setIsEditing(false);
  };

  const handleCancel = () => {
    setEditName(color.name);
    setEditHex(color.hex);
    setIsEditing(false);
  };

  const textColor = color.color.r + color.color.g + color.color.b > 382 ? '#000' : '#FFF';

  return (
    <div className={`color-swatch ${className}`}>
      <div
        className="color-swatch__preview"
        style={{ backgroundColor: color.hex, color: textColor }}
      >
        <div className="color-swatch__hex">{color.hex}</div>
        <div className="color-swatch__rgb">
          RGB({color.color.r}, {color.color.g}, {color.color.b})
        </div>
      </div>

      <div className="color-swatch__info">
        {isEditing ? (
          <>
            <input
              type="text"
              value={editName}
              onChange={(e) => setEditName(e.target.value)}
              className="color-swatch__input"
              placeholder="Color name"
            />
            <input
              type="text"
              value={editHex}
              onChange={(e) => setEditHex(e.target.value)}
              className="color-swatch__input"
              placeholder="#000000"
            />
            <div className="color-swatch__edit-actions">
              <button onClick={handleSave} className="color-swatch__btn save">
                Save
              </button>
              <button onClick={handleCancel} className="color-swatch__btn cancel">
                Cancel
              </button>
            </div>
          </>
        ) : (
          <>
            <div className="color-swatch__name">{color.name}</div>
            {color.role && (
              <div className="color-swatch__role">{color.role}</div>
            )}
            {showControls && (
              <div className="color-swatch__actions">
                <button
                  onClick={() => setIsEditing(true)}
                  className="color-swatch__btn edit"
                >
                  Edit
                </button>
                {onRemove && (
                  <button onClick={onRemove} className="color-swatch__btn remove">
                    Remove
                  </button>
                )}
              </div>
            )}
          </>
        )}
      </div>

      <style>{`
        .color-swatch {
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          overflow: hidden;
          background: white;
        }

        .color-swatch__preview {
          height: 120px;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          gap: 8px;
          padding: 16px;
        }

        .color-swatch__hex {
          font-family: monospace;
          font-size: 16px;
          font-weight: 700;
        }

        .color-swatch__rgb {
          font-family: monospace;
          font-size: 12px;
          opacity: 0.8;
        }

        .color-swatch__info {
          padding: 16px;
        }

        .color-swatch__name {
          font-weight: 600;
          font-size: 14px;
          margin-bottom: 4px;
        }

        .color-swatch__role {
          font-size: 12px;
          color: #6b7280;
          text-transform: capitalize;
          margin-bottom: 12px;
        }

        .color-swatch__actions,
        .color-swatch__edit-actions {
          display: flex;
          gap: 8px;
        }

        .color-swatch__btn {
          padding: 6px 12px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          background: white;
          cursor: pointer;
          font-size: 12px;
          font-weight: 500;
          transition: all 0.2s;
        }

        .color-swatch__btn:hover {
          background: #f9fafb;
        }

        .color-swatch__btn.edit {
          flex: 1;
        }

        .color-swatch__btn.remove {
          color: #dc2626;
          border-color: #fca5a5;
        }

        .color-swatch__btn.remove:hover {
          background: #fef2f2;
        }

        .color-swatch__btn.save {
          flex: 1;
          background: #3b82f6;
          color: white;
          border-color: #3b82f6;
        }

        .color-swatch__btn.save:hover {
          background: #2563eb;
        }

        .color-swatch__btn.cancel {
          flex: 1;
        }

        .color-swatch__input {
          width: 100%;
          padding: 8px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          font-size: 14px;
          margin-bottom: 8px;
        }

        .color-swatch__input:focus {
          outline: none;
          border-color: #3b82f6;
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        }
      `}</style>
    </div>
  );
};
