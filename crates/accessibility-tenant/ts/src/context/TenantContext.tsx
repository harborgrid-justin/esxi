/**
 * Tenant Context
 * Global context for tenant and user state management
 */

import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { Tenant, User, Organization } from '../types';
import TenantService from '../services/TenantService';
import UserService from '../services/UserService';

interface TenantContextState {
  tenant: Tenant | null;
  user: User | null;
  organization: Organization | null;
  isLoading: boolean;
  error: string | null;
  refreshTenant: () => Promise<void>;
  refreshUser: () => Promise<void>;
  setOrganization: (org: Organization | null) => void;
  logout: () => void;
}

const TenantContext = createContext<TenantContextState | undefined>(undefined);

interface TenantProviderProps {
  children: ReactNode;
  apiBaseURL: string;
  getAuthToken: () => string | null;
  onLogout?: () => void;
  initialTenant?: Tenant;
  initialUser?: User;
}

export function TenantProvider({
  children,
  apiBaseURL,
  getAuthToken,
  onLogout,
  initialTenant,
  initialUser,
}: TenantProviderProps) {
  const [tenant, setTenant] = useState<Tenant | null>(initialTenant || null);
  const [user, setUser] = useState<User | null>(initialUser || null);
  const [organization, setOrganization] = useState<Organization | null>(null);
  const [isLoading, setIsLoading] = useState(!initialTenant || !initialUser);
  const [error, setError] = useState<string | null>(null);

  const tenantService = new TenantService(apiBaseURL, getAuthToken);
  const userService = new UserService(apiBaseURL, getAuthToken);

  const refreshTenant = async () => {
    try {
      setIsLoading(true);
      setError(null);
      const response = await tenantService.getCurrentTenant();
      if (response.success && response.data) {
        setTenant(response.data);
      } else {
        setError(response.error?.message || 'Failed to load tenant');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load tenant');
    } finally {
      setIsLoading(false);
    }
  };

  const refreshUser = async () => {
    try {
      setIsLoading(true);
      setError(null);
      const response = await userService.getCurrentUser();
      if (response.success && response.data) {
        setUser(response.data);
      } else {
        setError(response.error?.message || 'Failed to load user');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load user');
    } finally {
      setIsLoading(false);
    }
  };

  const logout = () => {
    setTenant(null);
    setUser(null);
    setOrganization(null);
    if (onLogout) {
      onLogout();
    }
  };

  useEffect(() => {
    if (!initialTenant || !initialUser) {
      const loadInitialData = async () => {
        await Promise.all([refreshTenant(), refreshUser()]);
      };
      loadInitialData();
    }
  }, []);

  useEffect(() => {
    // Listen for unauthorized events
    const handleUnauthorized = () => {
      logout();
    };

    window.addEventListener('auth:unauthorized', handleUnauthorized);

    return () => {
      window.removeEventListener('auth:unauthorized', handleUnauthorized);
    };
  }, []);

  const value: TenantContextState = {
    tenant,
    user,
    organization,
    isLoading,
    error,
    refreshTenant,
    refreshUser,
    setOrganization,
    logout,
  };

  return <TenantContext.Provider value={value}>{children}</TenantContext.Provider>;
}

export function useTenantContext(): TenantContextState {
  const context = useContext(TenantContext);
  if (context === undefined) {
    throw new Error('useTenantContext must be used within a TenantProvider');
  }
  return context;
}

export default TenantContext;
