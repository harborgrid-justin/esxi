/**
 * useDataSource - Hook for fetching widget data
 */

import { useState, useEffect, useCallback } from 'react';
import axios from 'axios';
import { DataSourceConfig } from '../types';

interface UseDataSourceReturn {
  data: any;
  loading: boolean;
  error: Error | null;
  refresh: () => Promise<void>;
}

export const useDataSource = (
  dataSource: DataSourceConfig,
  refreshInterval?: number
): UseDataSourceReturn => {
  const [data, setData] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      let result: any;

      switch (dataSource.type) {
        case 'Sql': {
          const response = await axios.post('/api/data-source/sql', {
            connection_id: dataSource.connection_id,
            query: dataSource.query,
            parameters: dataSource.parameters,
          });
          result = response.data;
          break;
        }

        case 'Api': {
          const response = await axios({
            method: dataSource.method.toLowerCase(),
            url: dataSource.url,
            headers: dataSource.headers,
            data: dataSource.body ? JSON.parse(dataSource.body) : undefined,
          });
          result = response.data;
          break;
        }

        case 'Static': {
          result = dataSource.data;
          break;
        }

        default:
          throw new Error('Unsupported data source type');
      }

      setData(result);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, [dataSource]);

  useEffect(() => {
    fetchData();

    // Set up auto-refresh if interval is provided
    if (refreshInterval && refreshInterval > 0) {
      const intervalId = setInterval(fetchData, refreshInterval * 1000);
      return () => clearInterval(intervalId);
    }
  }, [fetchData, refreshInterval]);

  const refresh = useCallback(async () => {
    await fetchData();
  }, [fetchData]);

  return {
    data,
    loading,
    error,
    refresh,
  };
};
