/**
 * PDF Reading Order Viewer Component
 * Displays and validates PDF reading order
 */

import React from 'react';
import type { ReadingOrderItem } from '../../types/index.js';

export interface ReadingOrderViewerProps {
  readingOrder: ReadingOrderItem[];
  className?: string;
}

export function ReadingOrderViewer({ readingOrder, className = '' }: ReadingOrderViewerProps): JSX.Element {
  return (
    <div className={`reading-order-viewer ${className}`}>
      <h3>Reading Order</h3>
      <p>Content reading order as interpreted by screen readers</p>

      <div className="reading-order-viewer__items">
        {readingOrder.map((item, index) => (
          <div key={item.id} className="reading-order-item">
            <div className="reading-order-item__number">{index + 1}</div>
            <div className="reading-order-item__content">
              <div className="reading-order-item__type">{item.type}</div>
              <div className="reading-order-item__text">{item.content}</div>
              <div className="reading-order-item__meta">
                Page {item.page}
                {item.role && ` • ${item.role}`}
              </div>
            </div>
          </div>
        ))}
      </div>

      {readingOrder.length === 0 && (
        <p className="reading-order-viewer__empty">No reading order information available</p>
      )}
    </div>
  );
}
