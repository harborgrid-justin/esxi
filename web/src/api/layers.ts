import { apiClient } from './client';
import type { Layer, FeatureCollection, Feature } from '@/types';

export const layersApi = {
  // Get all layers
  getLayers: async (): Promise<Layer[]> => {
    return apiClient.get<Layer[]>('/layers');
  },

  // Get single layer
  getLayer: async (id: string): Promise<Layer> => {
    return apiClient.get<Layer>(`/layers/${id}`);
  },

  // Create new layer
  createLayer: async (layer: Omit<Layer, 'id'>): Promise<Layer> => {
    return apiClient.post<Layer>('/layers', layer);
  },

  // Update layer
  updateLayer: async (id: string, updates: Partial<Layer>): Promise<Layer> => {
    return apiClient.patch<Layer>(`/layers/${id}`, updates);
  },

  // Delete layer
  deleteLayer: async (id: string): Promise<void> => {
    return apiClient.delete<void>(`/layers/${id}`);
  },

  // Get layer features
  getFeatures: async (layerId: string, bbox?: [number, number, number, number]): Promise<FeatureCollection> => {
    const params = bbox ? { bbox: bbox.join(',') } : undefined;
    return apiClient.get<FeatureCollection>(`/layers/${layerId}/features`, params);
  },

  // Get single feature
  getFeature: async (layerId: string, featureId: string): Promise<Feature> => {
    return apiClient.get<Feature>(`/layers/${layerId}/features/${featureId}`);
  },

  // Create feature
  createFeature: async (layerId: string, feature: Omit<Feature, 'id'>): Promise<Feature> => {
    return apiClient.post<Feature>(`/layers/${layerId}/features`, feature);
  },

  // Update feature
  updateFeature: async (layerId: string, featureId: string, updates: Partial<Feature>): Promise<Feature> => {
    return apiClient.patch<Feature>(`/layers/${layerId}/features/${featureId}`, updates);
  },

  // Delete feature
  deleteFeature: async (layerId: string, featureId: string): Promise<void> => {
    return apiClient.delete<void>(`/layers/${layerId}/features/${featureId}`);
  },

  // Get layer style
  getStyle: async (layerId: string): Promise<unknown> => {
    return apiClient.get<unknown>(`/layers/${layerId}/style`);
  },

  // Update layer style
  updateStyle: async (layerId: string, style: unknown): Promise<unknown> => {
    return apiClient.put<unknown>(`/layers/${layerId}/style`, style);
  },

  // Get tiles
  getTileUrl: (layerId: string): string => {
    return `/api/layers/${layerId}/tiles/{z}/{x}/{y}`;
  },
};
