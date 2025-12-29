/**
 * Dashboard Context - Centralized State Management
 * Uses Zustand for performant, accessible state updates
 */

import React, { createContext, useContext, ReactNode } from 'react';
import { create } from 'zustand';
import type {
  DashboardState,
  DashboardFilters,
  WCAGLevel,
  IssueSeverity,
  IssueCategory,
  IssueStatus,
} from '../types';

interface DashboardStore extends DashboardState {
  // Filter actions
  setWCAGLevels: (levels: WCAGLevel[]) => void;
  setSeverities: (severities: IssueSeverity[]) => void;
  setCategories: (categories: IssueCategory[]) => void;
  setStatuses: (statuses: IssueStatus[]) => void;
  setDateRange: (start: Date | null, end: Date | null) => void;
  setSearchQuery: (query: string) => void;
  resetFilters: () => void;

  // Selection actions
  setSelectedPage: (pageId: string | null) => void;
  setSelectedIssue: (issueId: string | null) => void;

  // View actions
  setViewMode: (mode: DashboardState['viewMode']) => void;
  setDateRangePreset: (preset: DashboardState['dateRangePreset']) => void;
}

const initialFilters: DashboardFilters = {
  wcagLevels: ['A', 'AA', 'AAA'],
  severities: ['critical', 'serious', 'moderate', 'minor'],
  categories: ['perceivable', 'operable', 'understandable', 'robust'],
  statuses: ['open', 'in-progress'],
  dateRange: {
    start: null,
    end: null,
  },
  searchQuery: '',
};

const useDashboardStore = create<DashboardStore>((set) => ({
  // Initial state
  filters: initialFilters,
  selectedPage: null,
  selectedIssue: null,
  viewMode: 'overview',
  dateRangePreset: 'month',

  // Filter actions
  setWCAGLevels: (levels) =>
    set((state) => ({
      filters: { ...state.filters, wcagLevels: levels },
    })),

  setSeverities: (severities) =>
    set((state) => ({
      filters: { ...state.filters, severities },
    })),

  setCategories: (categories) =>
    set((state) => ({
      filters: { ...state.filters, categories },
    })),

  setStatuses: (statuses) =>
    set((state) => ({
      filters: { ...state.filters, statuses },
    })),

  setDateRange: (start, end) =>
    set((state) => ({
      filters: {
        ...state.filters,
        dateRange: { start, end },
      },
    })),

  setSearchQuery: (query) =>
    set((state) => ({
      filters: { ...state.filters, searchQuery: query },
    })),

  resetFilters: () =>
    set(() => ({
      filters: initialFilters,
    })),

  // Selection actions
  setSelectedPage: (pageId) => set({ selectedPage: pageId }),
  setSelectedIssue: (issueId) => set({ selectedIssue: issueId }),

  // View actions
  setViewMode: (mode) => set({ viewMode: mode }),
  setDateRangePreset: (preset) => set({ dateRangePreset: preset }),
}));

// Context
const DashboardContext = createContext<DashboardStore | null>(null);

// Provider component
export interface DashboardProviderProps {
  children: ReactNode;
}

export const DashboardProvider: React.FC<DashboardProviderProps> = ({
  children,
}) => {
  const store = useDashboardStore();

  return (
    <DashboardContext.Provider value={store}>
      {children}
    </DashboardContext.Provider>
  );
};

// Custom hook to use dashboard context
export const useDashboardContext = (): DashboardStore => {
  const context = useContext(DashboardContext);

  if (!context) {
    throw new Error(
      'useDashboardContext must be used within a DashboardProvider'
    );
  }

  return context;
};
