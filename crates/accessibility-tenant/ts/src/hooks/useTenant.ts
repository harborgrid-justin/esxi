/**
 * useTenant Hook
 * Custom hook for tenant management operations
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Tenant, TenantSettings, TenantBranding } from '../types';
import TenantService from '../services/TenantService';

export function useTenant(
  tenantService: TenantService,
  tenantId?: string
) {
  const queryClient = useQueryClient();

  // Query current tenant
  const {
    data: tenant,
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['tenant', tenantId || 'current'],
    queryFn: async () => {
      const response = tenantId
        ? await tenantService.getTenant(tenantId)
        : await tenantService.getCurrentTenant();
      return response.data;
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
  });

  // Update tenant mutation
  const updateTenant = useMutation({
    mutationFn: async (updates: Partial<Tenant>) => {
      if (!tenant?.id) throw new Error('Tenant ID not available');
      const response = await tenantService.updateTenant(tenant.id, updates);
      return response.data;
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['tenant', data?.id || 'current'], data);
      queryClient.invalidateQueries({ queryKey: ['tenant'] });
    },
  });

  // Update settings mutation
  const updateSettings = useMutation({
    mutationFn: async (settings: Partial<TenantSettings>) => {
      if (!tenant?.id) throw new Error('Tenant ID not available');
      const response = await tenantService.updateSettings(tenant.id, settings);
      return response.data;
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['tenant', data?.id || 'current'], data);
      queryClient.invalidateQueries({ queryKey: ['tenant'] });
    },
  });

  // Update branding mutation
  const updateBranding = useMutation({
    mutationFn: async (branding: Partial<TenantBranding>) => {
      if (!tenant?.id) throw new Error('Tenant ID not available');
      const response = await tenantService.updateBranding(tenant.id, branding);
      return response.data;
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['tenant', data?.id || 'current'], data);
      queryClient.invalidateQueries({ queryKey: ['tenant'] });
    },
  });

  // Upload logo mutation
  const uploadLogo = useMutation({
    mutationFn: async (file: File) => {
      if (!tenant?.id) throw new Error('Tenant ID not available');
      const response = await tenantService.uploadLogo(tenant.id, file);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tenant'] });
    },
  });

  // Set custom domain mutation
  const setCustomDomain = useMutation({
    mutationFn: async (domain: string) => {
      if (!tenant?.id) throw new Error('Tenant ID not available');
      const response = await tenantService.setCustomDomain(tenant.id, domain);
      return response.data;
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['tenant', data?.id || 'current'], data);
      queryClient.invalidateQueries({ queryKey: ['tenant'] });
    },
  });

  // Verify custom domain mutation
  const verifyCustomDomain = useMutation({
    mutationFn: async (domain: string) => {
      if (!tenant?.id) throw new Error('Tenant ID not available');
      const response = await tenantService.verifyCustomDomain(tenant.id, domain);
      return response.data;
    },
  });

  // Remove custom domain mutation
  const removeCustomDomain = useMutation({
    mutationFn: async () => {
      if (!tenant?.id) throw new Error('Tenant ID not available');
      const response = await tenantService.removeCustomDomain(tenant.id);
      return response.data;
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['tenant', data?.id || 'current'], data);
      queryClient.invalidateQueries({ queryKey: ['tenant'] });
    },
  });

  // Get usage metrics query
  const {
    data: usageMetrics,
    isLoading: isLoadingMetrics,
    refetch: refetchMetrics,
  } = useQuery({
    queryKey: ['tenant-usage', tenant?.id],
    queryFn: async () => {
      if (!tenant?.id) return null;
      const response = await tenantService.getUsageMetrics(tenant.id);
      return response.data;
    },
    enabled: !!tenant?.id,
    staleTime: 1 * 60 * 1000, // 1 minute
  });

  return {
    tenant,
    isLoading,
    error,
    refetch,
    updateTenant,
    updateSettings,
    updateBranding,
    uploadLogo,
    setCustomDomain,
    verifyCustomDomain,
    removeCustomDomain,
    usageMetrics,
    isLoadingMetrics,
    refetchMetrics,
  };
}

export default useTenant;
