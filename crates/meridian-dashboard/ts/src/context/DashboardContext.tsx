/**
 * DashboardContext - Global dashboard state management
 */

import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';
import { Dashboard, DashboardFilter } from '../types';

interface DashboardContextValue {
  dashboard: Dashboard | null;
  setDashboard: (dashboard: Dashboard | null) => void;
  filters: DashboardFilter[];
  applyFilter: (filter: DashboardFilter) => void;
  removeFilter: (filterId: string) => void;
  clearFilters: () => void;
  isEditing: boolean;
  setIsEditing: (editing: boolean) => void;
  refreshInterval: number | null;
  setRefreshInterval: (interval: number | null) => void;
}

const DashboardContext = createContext<DashboardContextValue | undefined>(
  undefined
);

export interface DashboardProviderProps {
  children: ReactNode;
  initialDashboard?: Dashboard;
}

export const DashboardProvider: React.FC<DashboardProviderProps> = ({
  children,
  initialDashboard,
}) => {
  const [dashboard, setDashboard] = useState<Dashboard | null>(
    initialDashboard || null
  );
  const [filters, setFilters] = useState<DashboardFilter[]>([]);
  const [isEditing, setIsEditing] = useState(false);
  const [refreshInterval, setRefreshInterval] = useState<number | null>(null);

  const applyFilter = useCallback((filter: DashboardFilter) => {
    setFilters((prev) => {
      // Replace existing filter for the same field or add new one
      const existing = prev.findIndex((f) => f.field === filter.field);
      if (existing >= 0) {
        const updated = [...prev];
        updated[existing] = filter;
        return updated;
      }
      return [...prev, filter];
    });
  }, []);

  const removeFilter = useCallback((filterId: string) => {
    setFilters((prev) => prev.filter((f) => f.id !== filterId));
  }, []);

  const clearFilters = useCallback(() => {
    setFilters([]);
  }, []);

  const value: DashboardContextValue = {
    dashboard,
    setDashboard,
    filters,
    applyFilter,
    removeFilter,
    clearFilters,
    isEditing,
    setIsEditing,
    refreshInterval,
    setRefreshInterval,
  };

  return (
    <DashboardContext.Provider value={value}>
      {children}
    </DashboardContext.Provider>
  );
};

export const useDashboardContext = (): DashboardContextValue => {
  const context = useContext(DashboardContext);
  if (!context) {
    throw new Error(
      'useDashboardContext must be used within a DashboardProvider'
    );
  }
  return context;
};
