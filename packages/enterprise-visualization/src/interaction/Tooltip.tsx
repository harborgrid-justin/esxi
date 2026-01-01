/**
 * Rich Tooltip Component for Data Visualizations
 * Provides context-aware, interactive tooltips with custom content
 */

import React, { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { TooltipConfig, TooltipData, Position } from '../types';

interface TooltipProps {
  config?: TooltipConfig;
  containerRef?: React.RefObject<HTMLElement>;
}

export const Tooltip: React.FC<TooltipProps> = ({ config = {}, containerRef }) => {
  const [tooltipData, setTooltipData] = useState<TooltipData>({
    content: '',
    position: { x: 0, y: 0 },
    visible: false,
  });

  const tooltipRef = useRef<HTMLDivElement>(null);
  const showTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const hideTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const {
    position = 'mouse',
    offset = { x: 10, y: 10 },
    showDelay = 0,
    hideDelay = 0,
    maxWidth = 300,
    className = '',
  } = config;

  useEffect(() => {
    return () => {
      if (showTimeoutRef.current) clearTimeout(showTimeoutRef.current);
      if (hideTimeoutRef.current) clearTimeout(hideTimeoutRef.current);
    };
  }, []);

  const show = (content: React.ReactNode | string, pos: Position, title?: string) => {
    if (hideTimeoutRef.current) {
      clearTimeout(hideTimeoutRef.current);
      hideTimeoutRef.current = null;
    }

    const showTooltip = () => {
      setTooltipData({
        content,
        position: pos,
        visible: true,
        title,
      });
    };

    if (showDelay > 0) {
      showTimeoutRef.current = setTimeout(showTooltip, showDelay);
    } else {
      showTooltip();
    }
  };

  const hide = () => {
    if (showTimeoutRef.current) {
      clearTimeout(showTimeoutRef.current);
      showTimeoutRef.current = null;
    }

    const hideTooltip = () => {
      setTooltipData((prev) => ({ ...prev, visible: false }));
    };

    if (hideDelay > 0) {
      hideTimeoutRef.current = setTimeout(hideTooltip, hideDelay);
    } else {
      hideTooltip();
    }
  };

  const update = (content: React.ReactNode | string, pos: Position, title?: string) => {
    setTooltipData({
      content,
      position: pos,
      visible: true,
      title,
    });
  };

  // Expose methods via ref
  useEffect(() => {
    if (containerRef?.current) {
      (containerRef.current as any).showTooltip = show;
      (containerRef.current as any).hideTooltip = hide;
      (containerRef.current as any).updateTooltip = update;
    }
  }, [containerRef]);

  // Calculate tooltip position
  const getTooltipStyle = (): React.CSSProperties => {
    if (!tooltipRef.current || !tooltipData.visible) {
      return { display: 'none' };
    }

    const tooltipRect = tooltipRef.current.getBoundingClientRect();
    let x = tooltipData.position.x + offset.x;
    let y = tooltipData.position.y + offset.y;

    // Adjust position to keep tooltip in viewport
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    if (x + tooltipRect.width > viewportWidth) {
      x = tooltipData.position.x - tooltipRect.width - offset.x;
    }

    if (y + tooltipRect.height > viewportHeight) {
      y = tooltipData.position.y - tooltipRect.height - offset.y;
    }

    // Ensure tooltip doesn't go off-screen on left/top
    x = Math.max(5, x);
    y = Math.max(5, y);

    return {
      position: 'fixed',
      left: `${x}px`,
      top: `${y}px`,
      maxWidth: `${maxWidth}px`,
      pointerEvents: 'none',
      zIndex: 10000,
    };
  };

  const tooltipContent = (
    <div
      ref={tooltipRef}
      className={`visualization-tooltip ${className}`}
      style={{
        ...getTooltipStyle(),
        backgroundColor: 'rgba(0, 0, 0, 0.9)',
        color: 'white',
        padding: '12px',
        borderRadius: '6px',
        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
        fontSize: '13px',
        lineHeight: '1.5',
        transition: 'opacity 0.2s ease-in-out',
        opacity: tooltipData.visible ? 1 : 0,
      }}
    >
      {tooltipData.title && (
        <div
          style={{
            fontWeight: 'bold',
            marginBottom: '6px',
            paddingBottom: '6px',
            borderBottom: '1px solid rgba(255, 255, 255, 0.2)',
            fontSize: '14px',
          }}
        >
          {tooltipData.title}
        </div>
      )}
      <div>{tooltipData.content}</div>
    </div>
  );

  return createPortal(tooltipContent, document.body);
};

/**
 * Hook for using tooltip functionality
 */
export const useTooltip = (config?: TooltipConfig) => {
  const [tooltipData, setTooltipData] = useState<TooltipData>({
    content: '',
    position: { x: 0, y: 0 },
    visible: false,
  });

  const show = (content: React.ReactNode | string, position: Position, title?: string) => {
    setTooltipData({
      content,
      position,
      visible: true,
      title,
    });
  };

  const hide = () => {
    setTooltipData((prev) => ({ ...prev, visible: false }));
  };

  const update = (content: React.ReactNode | string, position: Position, title?: string) => {
    setTooltipData({
      content,
      position,
      visible: true,
      title,
    });
  };

  const TooltipComponent = () => (
    <Tooltip config={config} containerRef={{ current: null }} />
  );

  return {
    show,
    hide,
    update,
    tooltipData,
    TooltipComponent,
  };
};

/**
 * Higher-order component for adding tooltip functionality
 */
export const withTooltip = <P extends object>(
  Component: React.ComponentType<P>,
  config?: TooltipConfig
) => {
  return React.forwardRef<HTMLElement, P>((props, ref) => {
    const containerRef = useRef<HTMLElement>(null);

    React.useImperativeHandle(ref, () => containerRef.current as HTMLElement);

    return (
      <>
        <Component {...props} ref={containerRef} />
        <Tooltip config={config} containerRef={containerRef} />
      </>
    );
  });
};

/**
 * Create formatted tooltip content for data points
 */
export const createDataTooltip = (data: {
  label?: string;
  value: number | string;
  category?: string;
  metadata?: Record<string, any>;
}): React.ReactNode => {
  return (
    <div style={{ minWidth: '150px' }}>
      {data.label && (
        <div style={{ marginBottom: '4px' }}>
          <strong>{data.label}</strong>
        </div>
      )}
      <div style={{ marginBottom: '4px' }}>
        Value: <strong>{typeof data.value === 'number' ? data.value.toLocaleString() : data.value}</strong>
      </div>
      {data.category && (
        <div style={{ marginBottom: '4px' }}>
          Category: <span style={{ color: '#aaa' }}>{data.category}</span>
        </div>
      )}
      {data.metadata && Object.keys(data.metadata).length > 0 && (
        <div style={{ marginTop: '8px', paddingTop: '8px', borderTop: '1px solid rgba(255,255,255,0.2)' }}>
          {Object.entries(data.metadata).map(([key, value]) => (
            <div key={key} style={{ fontSize: '11px', color: '#ccc' }}>
              {key}: {String(value)}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

/**
 * Create formatted tooltip for time series data
 */
export const createTimeSeriesTooltip = (data: {
  timestamp: Date | string;
  value: number;
  series?: string;
}): React.ReactNode => {
  const date = typeof data.timestamp === 'string' ? new Date(data.timestamp) : data.timestamp;

  return (
    <div style={{ minWidth: '150px' }}>
      <div style={{ marginBottom: '4px', color: '#aaa', fontSize: '11px' }}>
        {date.toLocaleDateString()} {date.toLocaleTimeString()}
      </div>
      {data.series && (
        <div style={{ marginBottom: '4px' }}>
          <strong>{data.series}</strong>
        </div>
      )}
      <div>
        Value: <strong>{data.value.toLocaleString()}</strong>
      </div>
    </div>
  );
};

/**
 * Create formatted tooltip for network nodes
 */
export const createNetworkTooltip = (data: {
  id: string | number;
  label?: string;
  connections?: number;
  value?: number;
}): React.ReactNode => {
  return (
    <div style={{ minWidth: '150px' }}>
      <div style={{ marginBottom: '4px' }}>
        <strong>{data.label || data.id}</strong>
      </div>
      {data.value !== undefined && (
        <div style={{ marginBottom: '4px' }}>
          Value: <strong>{data.value.toLocaleString()}</strong>
        </div>
      )}
      {data.connections !== undefined && (
        <div style={{ color: '#aaa', fontSize: '11px' }}>
          {data.connections} connection{data.connections !== 1 ? 's' : ''}
        </div>
      )}
    </div>
  );
};

export default Tooltip;
