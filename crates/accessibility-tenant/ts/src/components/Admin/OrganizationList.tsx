/**
 * Organization List Component
 * List and manage all organizations in the tenant
 */

import React, { useState } from 'react';
import { useOrganizationList } from '../../hooks/useOrganization';
import { usePermissions } from '../../hooks/usePermissions';
import { Organization } from '../../types';
import OrganizationService from '../../services/OrganizationService';

interface OrganizationListProps {
  organizationService: OrganizationService;
  onOrganizationSelect?: (organization: Organization) => void;
  onCreateOrganization?: () => void;
  className?: string;
}

export const OrganizationList: React.FC<OrganizationListProps> = ({
  organizationService,
  onOrganizationSelect,
  onCreateOrganization,
  className,
}) => {
  const [page, setPage] = useState(1);
  const [perPage] = useState(20);
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('');

  const { organizations, metadata, isLoading, error, refetch } = useOrganizationList(
    organizationService,
    {
      page,
      perPage,
      search: search || undefined,
      status: statusFilter || undefined,
    }
  );

  const { can } = usePermissions();
  const canManageOrgs = can.manageOrganization();

  const handleSearch = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSearch(e.target.value);
    setPage(1);
  };

  const handleStatusFilter = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setStatusFilter(e.target.value);
    setPage(1);
  };

  const handleRefresh = () => {
    refetch();
  };

  if (error) {
    return (
      <div className={className} role="alert">
        <p className="error-message">Error loading organizations: {error.toString()}</p>
        <button onClick={handleRefresh} type="button" className="btn btn-secondary">
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className={className}>
      <header className="list-header">
        <div className="header-content">
          <h1>Organizations</h1>
          <p className="subtitle">
            {metadata?.total || 0} organization{metadata?.total !== 1 ? 's' : ''}
          </p>
        </div>
        {canManageOrgs && onCreateOrganization && (
          <button
            type="button"
            onClick={onCreateOrganization}
            className="btn btn-primary"
            aria-label="Create new organization"
          >
            Create Organization
          </button>
        )}
      </header>

      {/* Filters */}
      <div className="filters" role="search">
        <div className="filter-group">
          <label htmlFor="search-input" className="filter-label">
            Search
          </label>
          <input
            id="search-input"
            type="search"
            value={search}
            onChange={handleSearch}
            placeholder="Search organizations..."
            className="filter-input"
            aria-label="Search organizations"
          />
        </div>

        <div className="filter-group">
          <label htmlFor="status-filter" className="filter-label">
            Status
          </label>
          <select
            id="status-filter"
            value={statusFilter}
            onChange={handleStatusFilter}
            className="filter-select"
            aria-label="Filter by status"
          >
            <option value="">All</option>
            <option value="active">Active</option>
            <option value="inactive">Inactive</option>
          </select>
        </div>

        <button
          type="button"
          onClick={handleRefresh}
          className="btn btn-secondary"
          aria-label="Refresh list"
          disabled={isLoading}
        >
          {isLoading ? 'Refreshing...' : 'Refresh'}
        </button>
      </div>

      {/* Organization List */}
      {isLoading && organizations.length === 0 ? (
        <div className="loading" role="status" aria-label="Loading organizations">
          <div className="loading-spinner">Loading...</div>
        </div>
      ) : organizations.length === 0 ? (
        <div className="empty-state">
          <p>No organizations found</p>
          {canManageOrgs && onCreateOrganization && (
            <button
              type="button"
              onClick={onCreateOrganization}
              className="btn btn-primary"
            >
              Create First Organization
            </button>
          )}
        </div>
      ) : (
        <div className="organization-grid">
          {organizations.map((org) => (
            <article
              key={org.id}
              className={`organization-card ${!org.isActive ? 'inactive' : ''}`}
              onClick={() => onOrganizationSelect?.(org)}
              role="button"
              tabIndex={0}
              aria-label={`Organization: ${org.name}`}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  onOrganizationSelect?.(org);
                }
              }}
            >
              <div className="card-header">
                {org.branding.logoUrl && (
                  <img
                    src={org.branding.logoUrl}
                    alt={`${org.name} logo`}
                    className="org-logo"
                  />
                )}
                <div className="org-info">
                  <h3 className="org-name">{org.name}</h3>
                  <p className="org-slug">{org.slug}</p>
                </div>
                <span
                  className={`status-badge ${org.isActive ? 'active' : 'inactive'}`}
                  aria-label={`Status: ${org.isActive ? 'Active' : 'Inactive'}`}
                >
                  {org.isActive ? 'Active' : 'Inactive'}
                </span>
              </div>

              {org.description && (
                <p className="org-description">{org.description}</p>
              )}

              <div className="card-footer">
                <div className="org-metadata">
                  {org.domain && (
                    <span className="metadata-item">
                      <span className="metadata-label">Domain:</span>
                      <span className="metadata-value">{org.domain}</span>
                    </span>
                  )}
                  {org.customDomain && (
                    <span className="metadata-item">
                      <span className="metadata-label">Custom Domain:</span>
                      <span className="metadata-value">{org.customDomain}</span>
                    </span>
                  )}
                  <span className="metadata-item">
                    <span className="metadata-label">Created:</span>
                    <span className="metadata-value">
                      {new Date(org.createdAt).toLocaleDateString()}
                    </span>
                  </span>
                </div>
              </div>
            </article>
          ))}
        </div>
      )}

      {/* Pagination */}
      {metadata && metadata.totalPages > 1 && (
        <nav className="pagination" aria-label="Pagination navigation">
          <button
            type="button"
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            disabled={page === 1}
            className="btn btn-secondary"
            aria-label="Previous page"
          >
            Previous
          </button>
          <span className="pagination-info" aria-live="polite">
            Page {page} of {metadata.totalPages}
          </span>
          <button
            type="button"
            onClick={() => setPage((p) => Math.min(metadata.totalPages, p + 1))}
            disabled={page === metadata.totalPages}
            className="btn btn-secondary"
            aria-label="Next page"
          >
            Next
          </button>
        </nav>
      )}
    </div>
  );
};

export default OrganizationList;
