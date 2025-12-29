/**
 * useDashboard - Hook for dashboard operations
 */

import { useState, useEffect, useCallback } from 'react';
import axios from 'axios';
import { Dashboard } from '../types';

interface UseDashboardReturn {
  dashboard: Dashboard | null;
  loading: boolean;
  error: Error | null;
  refresh: () => Promise<void>;
  update: (updates: Partial<Dashboard>) => Promise<void>;
  save: () => Promise<void>;
}

export const useDashboard = (dashboardId: string): UseDashboardReturn => {
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchDashboard = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const response = await axios.get(`/api/dashboards/${dashboardId}`);
      setDashboard(response.data);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, [dashboardId]);

  useEffect(() => {
    fetchDashboard();
  }, [fetchDashboard]);

  const refresh = useCallback(async () => {
    await fetchDashboard();
  }, [fetchDashboard]);

  const update = useCallback(
    async (updates: Partial<Dashboard>) => {
      if (!dashboard) return;

      const updated = { ...dashboard, ...updates };
      setDashboard(updated);

      try {
        await axios.put(`/api/dashboards/${dashboardId}`, updates);
      } catch (err) {
        setError(err as Error);
        // Revert on error
        setDashboard(dashboard);
      }
    },
    [dashboard, dashboardId]
  );

  const save = useCallback(async () => {
    if (!dashboard) return;

    try {
      setLoading(true);
      await axios.put(`/api/dashboards/${dashboardId}`, dashboard);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, [dashboard, dashboardId]);

  return {
    dashboard,
    loading,
    error,
    refresh,
    update,
    save,
  };
};
