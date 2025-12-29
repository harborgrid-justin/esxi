/**
 * useWidgets - Hook for widget management
 */

import { useState, useEffect, useCallback } from 'react';
import axios from 'axios';
import { Widget, WidgetPosition } from '../types';

interface UseWidgetsReturn {
  widgets: Widget[];
  loading: boolean;
  error: Error | null;
  addWidget: (widget: Omit<Widget, 'id' | 'created_at' | 'updated_at'>) => Promise<void>;
  updateWidget: (widgetId: string, updates: Partial<Widget>) => Promise<void>;
  removeWidget: (widgetId: string) => Promise<void>;
  updatePosition: (widgetId: string, position: WidgetPosition) => Promise<void>;
  refresh: () => Promise<void>;
}

export const useWidgets = (dashboardId: string): UseWidgetsReturn => {
  const [widgets, setWidgets] = useState<Widget[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchWidgets = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const response = await axios.get(`/api/dashboards/${dashboardId}/widgets`);
      setWidgets(response.data.widgets || []);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, [dashboardId]);

  useEffect(() => {
    fetchWidgets();
  }, [fetchWidgets]);

  const addWidget = useCallback(
    async (widget: Omit<Widget, 'id' | 'created_at' | 'updated_at'>) => {
      try {
        const response = await axios.post(
          `/api/dashboards/${dashboardId}/widgets`,
          widget
        );
        setWidgets((prev) => [...prev, response.data]);
      } catch (err) {
        setError(err as Error);
        throw err;
      }
    },
    [dashboardId]
  );

  const updateWidget = useCallback(
    async (widgetId: string, updates: Partial<Widget>) => {
      try {
        const response = await axios.put(`/api/widgets/${widgetId}`, updates);
        setWidgets((prev) =>
          prev.map((w) => (w.id === widgetId ? response.data : w))
        );
      } catch (err) {
        setError(err as Error);
        throw err;
      }
    },
    []
  );

  const removeWidget = useCallback(async (widgetId: string) => {
    try {
      await axios.delete(`/api/widgets/${widgetId}`);
      setWidgets((prev) => prev.filter((w) => w.id !== widgetId));
    } catch (err) {
      setError(err as Error);
      throw err;
    }
  }, []);

  const updatePosition = useCallback(
    async (widgetId: string, position: WidgetPosition) => {
      try {
        await axios.put(`/api/widgets/${widgetId}`, { position });
        setWidgets((prev) =>
          prev.map((w) => (w.id === widgetId ? { ...w, position } : w))
        );
      } catch (err) {
        setError(err as Error);
        throw err;
      }
    },
    []
  );

  const refresh = useCallback(async () => {
    await fetchWidgets();
  }, [fetchWidgets]);

  return {
    widgets,
    loading,
    error,
    addWidget,
    updateWidget,
    removeWidget,
    updatePosition,
    refresh,
  };
};
