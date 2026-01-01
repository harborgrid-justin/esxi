/**
 * Cursor Overlay Component
 * Displays cursors of other participants in the document
 */

import React from 'react';
import { Cursor, Participant } from '../types';

export interface CursorOverlayProps {
  cursors: Map<string, Cursor>;
  participants: Map<string, Participant>;
  containerRef: React.RefObject<HTMLElement>;
  className?: string;
}

export const CursorOverlay: React.FC<CursorOverlayProps> = ({
  cursors,
  participants,
  containerRef,
  className = '',
}) => {
  const getCursorPosition = (cursor: Cursor): { top: number; left: number } | null => {
    if (!containerRef.current) return null;

    // Calculate pixel position from line/column
    // This is a simplified implementation - actual implementation
    // would need to measure text layout
    const lineHeight = 20; // Default line height in pixels
    const charWidth = 8; // Average character width

    return {
      top: cursor.position.line * lineHeight,
      left: cursor.position.column * charWidth,
    };
  };

  return (
    <div className={`pointer-events-none absolute inset-0 ${className}`}>
      {Array.from(cursors.entries()).map(([participantId, cursor]) => {
        const participant = participants.get(participantId);
        if (!participant) return null;

        const position = getCursorPosition(cursor);
        if (!position) return null;

        return (
          <div
            key={participantId}
            className="absolute"
            style={{
              top: position.top,
              left: position.left,
              transform: 'translateX(-1px)',
            }}
          >
            {/* Cursor Line */}
            <div
              className="w-0.5 h-5 animate-pulse"
              style={{ backgroundColor: participant.color }}
            />

            {/* Cursor Label */}
            <div
              className="absolute top-0 left-1 px-2 py-0.5 text-xs text-white rounded whitespace-nowrap"
              style={{ backgroundColor: participant.color }}
            >
              {participant.displayName}
            </div>
          </div>
        );
      })}
    </div>
  );
};
