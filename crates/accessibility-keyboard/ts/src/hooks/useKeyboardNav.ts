/**
 * Keyboard Navigation Hook
 * Custom React hook for keyboard navigation management
 */

import { useEffect, useState, useCallback, useRef } from 'react';

export interface UseKeyboardNavOptions {
  enabled?: boolean;
  trapFocus?: boolean;
  returnFocusOnCleanup?: boolean;
  initialFocusRef?: React.RefObject<HTMLElement>;
}

export interface UseKeyboardNavReturn {
  containerRef: React.RefObject<HTMLDivElement>;
  focusableElements: HTMLElement[];
  currentIndex: number;
  focusFirst: () => void;
  focusLast: () => void;
  focusNext: () => void;
  focusPrevious: () => void;
  focusElement: (index: number) => void;
}

export function useKeyboardNav(options: UseKeyboardNavOptions = {}): UseKeyboardNavReturn {
  const {
    enabled = true,
    trapFocus = false,
    returnFocusOnCleanup = true,
    initialFocusRef,
  } = options;

  const containerRef = useRef<HTMLDivElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);
  const [focusableElements, setFocusableElements] = useState<HTMLElement[]>([]);
  const [currentIndex, setCurrentIndex] = useState<number>(-1);

  /**
   * Gets all focusable elements in container
   */
  const getFocusableElements = useCallback((): HTMLElement[] => {
    if (!containerRef.current) return [];

    const selector = [
      'a[href]:not([disabled])',
      'button:not([disabled])',
      'textarea:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      '[tabindex]:not([tabindex="-1"])',
    ].join(',');

    return Array.from(
      containerRef.current.querySelectorAll(selector)
    ) as HTMLElement[];
  }, []);

  /**
   * Updates focusable elements list
   */
  const updateFocusableElements = useCallback(() => {
    const elements = getFocusableElements();
    setFocusableElements(elements);

    // Update current index if active element is in list
    const activeElement = document.activeElement as HTMLElement;
    const index = elements.indexOf(activeElement);
    setCurrentIndex(index);
  }, [getFocusableElements]);

  /**
   * Focuses first element
   */
  const focusFirst = useCallback(() => {
    if (focusableElements.length > 0) {
      focusableElements[0].focus();
      setCurrentIndex(0);
    }
  }, [focusableElements]);

  /**
   * Focuses last element
   */
  const focusLast = useCallback(() => {
    if (focusableElements.length > 0) {
      const lastIndex = focusableElements.length - 1;
      focusableElements[lastIndex].focus();
      setCurrentIndex(lastIndex);
    }
  }, [focusableElements]);

  /**
   * Focuses next element
   */
  const focusNext = useCallback(() => {
    if (focusableElements.length === 0) return;

    const nextIndex = (currentIndex + 1) % focusableElements.length;
    focusableElements[nextIndex].focus();
    setCurrentIndex(nextIndex);
  }, [focusableElements, currentIndex]);

  /**
   * Focuses previous element
   */
  const focusPrevious = useCallback(() => {
    if (focusableElements.length === 0) return;

    const prevIndex =
      currentIndex <= 0 ? focusableElements.length - 1 : currentIndex - 1;
    focusableElements[prevIndex].focus();
    setCurrentIndex(prevIndex);
  }, [focusableElements, currentIndex]);

  /**
   * Focuses element at specific index
   */
  const focusElement = useCallback(
    (index: number) => {
      if (index >= 0 && index < focusableElements.length) {
        focusableElements[index].focus();
        setCurrentIndex(index);
      }
    },
    [focusableElements]
  );

  /**
   * Handles keyboard events
   */
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!enabled || !trapFocus) return;

      if (event.key === 'Tab') {
        event.preventDefault();

        if (event.shiftKey) {
          focusPrevious();
        } else {
          focusNext();
        }
      }
    },
    [enabled, trapFocus, focusNext, focusPrevious]
  );

  /**
   * Setup effect
   */
  useEffect(() => {
    if (!enabled) return;

    // Store previous focus
    previousFocusRef.current = document.activeElement as HTMLElement;

    // Update focusable elements
    updateFocusableElements();

    // Focus initial element
    if (initialFocusRef?.current) {
      initialFocusRef.current.focus();
    } else if (focusableElements.length > 0) {
      focusFirst();
    }

    // Add keyboard listener
    if (trapFocus) {
      document.addEventListener('keydown', handleKeyDown);
    }

    // Observe DOM changes
    const observer = new MutationObserver(updateFocusableElements);
    if (containerRef.current) {
      observer.observe(containerRef.current, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ['disabled', 'tabindex'],
      });
    }

    return () => {
      // Remove listener
      if (trapFocus) {
        document.removeEventListener('keydown', handleKeyDown);
      }

      // Disconnect observer
      observer.disconnect();

      // Return focus
      if (returnFocusOnCleanup && previousFocusRef.current) {
        previousFocusRef.current.focus();
      }
    };
  }, [
    enabled,
    trapFocus,
    returnFocusOnCleanup,
    initialFocusRef,
    handleKeyDown,
    updateFocusableElements,
    focusFirst,
  ]);

  /**
   * Update focusable elements when container changes
   */
  useEffect(() => {
    updateFocusableElements();
  }, [updateFocusableElements]);

  return {
    containerRef,
    focusableElements,
    currentIndex,
    focusFirst,
    focusLast,
    focusNext,
    focusPrevious,
    focusElement,
  };
}
