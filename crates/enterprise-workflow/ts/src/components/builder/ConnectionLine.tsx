/**
 * Connection Line - Custom connection line component
 */

import React from 'react';
import { getBezierPath, ConnectionLineComponentProps } from 'reactflow';

export const ConnectionLine: React.FC<ConnectionLineComponentProps> = ({
  fromX,
  fromY,
  toX,
  toY,
  connectionLineStyle
}) => {
  const [edgePath] = getBezierPath({
    sourceX: fromX,
    sourceY: fromY,
    targetX: toX,
    targetY: toY
  });

  return (
    <g>
      <path
        fill="none"
        stroke="#2563eb"
        strokeWidth={2}
        className="animated"
        d={edgePath}
        style={connectionLineStyle}
      />
      <circle
        cx={toX}
        cy={toY}
        fill="#fff"
        r={3}
        stroke="#2563eb"
        strokeWidth={1.5}
      />
    </g>
  );
};
