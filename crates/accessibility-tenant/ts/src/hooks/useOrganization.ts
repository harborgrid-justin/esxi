/**
 * useOrganization Hook
 * Custom hook for organization management operations
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Organization, OrganizationSettings, OrganizationBranding, PaginationParams, FilterParams } from '../types';
import OrganizationService from '../services/OrganizationService';

export function useOrganization(
  organizationService: OrganizationService,
  organizationId?: string
) {
  const queryClient = useQueryClient();

  // Query single organization
  const {
    data: organization,
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['organization', organizationId],
    queryFn: async () => {
      if (!organizationId) return null;
      const response = await organizationService.getOrganization(organizationId);
      return response.data;
    },
    enabled: !!organizationId,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });

  // Create organization mutation
  const createOrganization = useMutation({
    mutationFn: async (data: Partial<Organization>) => {
      const response = await organizationService.createOrganization(data);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });

  // Update organization mutation
  const updateOrganization = useMutation({
    mutationFn: async (updates: Partial<Organization>) => {
      if (!organizationId) throw new Error('Organization ID not available');
      const response = await organizationService.updateOrganization(organizationId, updates);
      return response.data;
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['organization', data?.id], data);
      queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });

  // Delete organization mutation
  const deleteOrganization = useMutation({
    mutationFn: async (id: string) => {
      const response = await organizationService.deleteOrganization(id);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });

  // Update settings mutation
  const updateSettings = useMutation({
    mutationFn: async (settings: Partial<OrganizationSettings>) => {
      if (!organizationId) throw new Error('Organization ID not available');
      const response = await organizationService.updateSettings(organizationId, settings);
      return response.data;
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['organization', data?.id], data);
      queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });

  // Update branding mutation
  const updateBranding = useMutation({
    mutationFn: async (branding: Partial<OrganizationBranding>) => {
      if (!organizationId) throw new Error('Organization ID not available');
      const response = await organizationService.updateBranding(organizationId, branding);
      return response.data;
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['organization', data?.id], data);
      queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });

  // Upload logo mutation
  const uploadLogo = useMutation({
    mutationFn: async (file: File) => {
      if (!organizationId) throw new Error('Organization ID not available');
      const response = await organizationService.uploadLogo(organizationId, file);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['organization', organizationId] });
    },
  });

  // Set custom domain mutation
  const setCustomDomain = useMutation({
    mutationFn: async (domain: string) => {
      if (!organizationId) throw new Error('Organization ID not available');
      const response = await organizationService.setCustomDomain(organizationId, domain);
      return response.data;
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['organization', data?.id], data);
      queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });

  // Verify custom domain mutation
  const verifyCustomDomain = useMutation({
    mutationFn: async (domain: string) => {
      if (!organizationId) throw new Error('Organization ID not available');
      const response = await organizationService.verifyCustomDomain(organizationId, domain);
      return response.data;
    },
  });

  // Remove custom domain mutation
  const removeCustomDomain = useMutation({
    mutationFn: async () => {
      if (!organizationId) throw new Error('Organization ID not available');
      const response = await organizationService.removeCustomDomain(organizationId);
      return response.data;
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['organization', data?.id], data);
      queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });

  // Activate organization mutation
  const activateOrganization = useMutation({
    mutationFn: async (id: string) => {
      const response = await organizationService.activateOrganization(id);
      return response.data;
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['organization', data?.id], data);
      queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });

  // Deactivate organization mutation
  const deactivateOrganization = useMutation({
    mutationFn: async (id: string) => {
      const response = await organizationService.deactivateOrganization(id);
      return response.data;
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['organization', data?.id], data);
      queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });

  // Get organization statistics
  const {
    data: statistics,
    isLoading: isLoadingStats,
    refetch: refetchStats,
  } = useQuery({
    queryKey: ['organization-stats', organizationId],
    queryFn: async () => {
      if (!organizationId) return null;
      const response = await organizationService.getStatistics(organizationId);
      return response.data;
    },
    enabled: !!organizationId,
    staleTime: 1 * 60 * 1000, // 1 minute
  });

  return {
    organization,
    isLoading,
    error,
    refetch,
    createOrganization,
    updateOrganization,
    deleteOrganization,
    updateSettings,
    updateBranding,
    uploadLogo,
    setCustomDomain,
    verifyCustomDomain,
    removeCustomDomain,
    activateOrganization,
    deactivateOrganization,
    statistics,
    isLoadingStats,
    refetchStats,
  };
}

export function useOrganizationList(
  organizationService: OrganizationService,
  params?: PaginationParams & FilterParams
) {
  const {
    data: organizations,
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['organizations', params],
    queryFn: async () => {
      const response = await organizationService.listOrganizations(params);
      return response;
    },
    staleTime: 2 * 60 * 1000, // 2 minutes
  });

  return {
    organizations: organizations?.data || [],
    metadata: organizations?.metadata,
    isLoading,
    error,
    refetch,
  };
}

export default useOrganization;
