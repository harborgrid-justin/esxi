import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatCoordinate(value: number, precision: number = 6): string {
  return value.toFixed(precision);
}

export function formatDistance(meters: number, units: 'meters' | 'kilometers' | 'miles' | 'feet' = 'meters'): string {
  switch (units) {
    case 'kilometers':
      return `${(meters / 1000).toFixed(2)} km`;
    case 'miles':
      return `${(meters / 1609.34).toFixed(2)} mi`;
    case 'feet':
      return `${(meters * 3.28084).toFixed(2)} ft`;
    default:
      return `${meters.toFixed(2)} m`;
  }
}

export function formatArea(squareMeters: number, units: 'meters' | 'kilometers' | 'miles' | 'feet' = 'meters'): string {
  switch (units) {
    case 'kilometers':
      return `${(squareMeters / 1000000).toFixed(2)} km²`;
    case 'miles':
      return `${(squareMeters / 2589988.11).toFixed(2)} mi²`;
    case 'feet':
      return `${(squareMeters * 10.7639).toFixed(2)} ft²`;
    default:
      return `${squareMeters.toFixed(2)} m²`;
  }
}

export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout | null = null;

  return function executedFunction(...args: Parameters<T>) {
    const later = () => {
      timeout = null;
      func(...args);
    };

    if (timeout) {
      clearTimeout(timeout);
    }
    timeout = setTimeout(later, wait);
  };
}

export function throttle<T extends (...args: any[]) => any>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean;

  return function executedFunction(...args: Parameters<T>) {
    if (!inThrottle) {
      func(...args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}
