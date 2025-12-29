/**
 * Focus Tracking Hook
 * Tracks focus changes for analysis and debugging
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { FocusEvent } from '../types';

export interface UseFocusTrackingOptions {
  maxHistorySize?: number;
  trackMouseFocus?: boolean;
  trackScriptFocus?: boolean;
}

export interface UseFocusTrackingReturn {
  focusHistory: FocusEvent[];
  currentFocus: FocusEvent | null;
  isTracking: boolean;
  startTracking: () => void;
  stopTracking: () => void;
  clearHistory: () => void;
  getStats: () => FocusStats;
}

export interface FocusStats {
  totalEvents: number;
  focusEvents: number;
  blurEvents: number;
  keyboardTriggered: number;
  mouseTriggered: number;
  scriptTriggered: number;
  uniqueElements: number;
}

export function useFocusTracking(
  maxHistorySize: number = 100
): UseFocusTrackingReturn {
  const [focusHistory, setFocusHistory] = useState<FocusEvent[]>([]);
  const [currentFocus, setCurrentFocus] = useState<FocusEvent | null>(null);
  const [isTracking, setIsTracking] = useState(false);

  const lastMouseDownRef = useRef<number>(0);
  const previousElementRef = useRef<HTMLElement | null>(null);

  /**
   * Determines what triggered the focus event
   */
  const getTriggeredBy = useCallback((): 'keyboard' | 'mouse' | 'script' => {
    const now = Date.now();
    const timeSinceMouseDown = now - lastMouseDownRef.current;

    // If mouse down happened within 100ms, likely mouse-triggered
    if (timeSinceMouseDown < 100) {
      return 'mouse';
    }

    // Check if Tab key was recently pressed
    // This is a simplification - in practice we'd track actual keyboard events
    return 'keyboard';
  }, []);

  /**
   * Handles focus events
   */
  const handleFocus = useCallback(
    (event: Event) => {
      if (!isTracking) return;

      const target = event.target as HTMLElement;
      if (!target) return;

      const focusEvent: FocusEvent = {
        type: 'focus',
        element: target,
        timestamp: Date.now(),
        triggeredBy: getTriggeredBy(),
        previousElement: previousElementRef.current,
      };

      setCurrentFocus(focusEvent);
      setFocusHistory((prev) => {
        const newHistory = [...prev, focusEvent];
        return newHistory.slice(-maxHistorySize);
      });

      previousElementRef.current = target;
    },
    [isTracking, getTriggeredBy, maxHistorySize]
  );

  /**
   * Handles blur events
   */
  const handleBlur = useCallback(
    (event: Event) => {
      if (!isTracking) return;

      const target = event.target as HTMLElement;
      if (!target) return;

      const blurEvent: FocusEvent = {
        type: 'blur',
        element: target,
        timestamp: Date.now(),
        triggeredBy: getTriggeredBy(),
        previousElement: previousElementRef.current,
      };

      setFocusHistory((prev) => {
        const newHistory = [...prev, blurEvent];
        return newHistory.slice(-maxHistorySize);
      });
    },
    [isTracking, getTriggeredBy, maxHistorySize]
  );

  /**
   * Handles mouse down events (to detect mouse-triggered focus)
   */
  const handleMouseDown = useCallback(() => {
    lastMouseDownRef.current = Date.now();
  }, []);

  /**
   * Starts tracking focus events
   */
  const startTracking = useCallback(() => {
    setIsTracking(true);
  }, []);

  /**
   * Stops tracking focus events
   */
  const stopTracking = useCallback(() => {
    setIsTracking(false);
  }, []);

  /**
   * Clears focus history
   */
  const clearHistory = useCallback(() => {
    setFocusHistory([]);
    setCurrentFocus(null);
    previousElementRef.current = null;
  }, []);

  /**
   * Gets focus statistics
   */
  const getStats = useCallback((): FocusStats => {
    const focusEvents = focusHistory.filter((e) => e.type === 'focus');
    const blurEvents = focusHistory.filter((e) => e.type === 'blur');

    return {
      totalEvents: focusHistory.length,
      focusEvents: focusEvents.length,
      blurEvents: blurEvents.length,
      keyboardTriggered: focusHistory.filter((e) => e.triggeredBy === 'keyboard')
        .length,
      mouseTriggered: focusHistory.filter((e) => e.triggeredBy === 'mouse').length,
      scriptTriggered: focusHistory.filter((e) => e.triggeredBy === 'script').length,
      uniqueElements: new Set(focusHistory.map((e) => e.element)).size,
    };
  }, [focusHistory]);

  /**
   * Setup event listeners
   */
  useEffect(() => {
    if (!isTracking) return;

    document.addEventListener('focus', handleFocus, true);
    document.addEventListener('blur', handleBlur, true);
    document.addEventListener('mousedown', handleMouseDown);

    return () => {
      document.removeEventListener('focus', handleFocus, true);
      document.removeEventListener('blur', handleBlur, true);
      document.removeEventListener('mousedown', handleMouseDown);
    };
  }, [isTracking, handleFocus, handleBlur, handleMouseDown]);

  return {
    focusHistory,
    currentFocus,
    isTracking,
    startTracking,
    stopTracking,
    clearHistory,
    getStats,
  };
}
