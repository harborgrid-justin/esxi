/**
 * Invoice History Component
 * Display and manage invoice history
 */

import React, { useState, useEffect } from 'react';
import { Invoice } from '../../types';
import BillingService from '../../services/BillingService';

interface InvoiceHistoryProps {
  billingService: BillingService;
  tenantId: string;
  className?: string;
}

export const InvoiceHistory: React.FC<InvoiceHistoryProps> = ({
  billingService,
  tenantId,
  className,
}) => {
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string>('');

  useEffect(() => {
    loadInvoices();
  }, [tenantId]);

  const loadInvoices = async () => {
    setIsLoading(true);
    try {
      const response = await billingService.listInvoices(tenantId);
      if (response.success && response.data) {
        setInvoices(response.data);
      }
    } catch (err) {
      setError('Failed to load invoices');
    } finally {
      setIsLoading(false);
    }
  };

  const handleDownload = async (invoiceId: string) => {
    try {
      const blob = await billingService.downloadInvoice(invoiceId);
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `invoice-${invoiceId}.pdf`;
      a.click();
      window.URL.revokeObjectURL(url);
    } catch (err) {
      setError('Failed to download invoice');
    }
  };

  const getStatusBadgeClass = (status: string): string => {
    switch (status) {
      case 'PAID':
        return 'status-paid';
      case 'OPEN':
        return 'status-open';
      case 'VOID':
        return 'status-void';
      default:
        return '';
    }
  };

  if (isLoading) {
    return (
      <div className={className} role="status">
        <div className="loading-spinner">Loading...</div>
      </div>
    );
  }

  return (
    <div className={className}>
      <header>
        <h2>Invoice History</h2>
        <p className="subtitle">{invoices.length} invoice{invoices.length !== 1 ? 's' : ''}</p>
      </header>

      {error && (
        <div className="error-message" role="alert">
          {error}
        </div>
      )}

      <div className="invoice-table-container">
        <table className="invoice-table">
          <thead>
            <tr>
              <th>Invoice ID</th>
              <th>Date</th>
              <th>Amount</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {invoices.map((invoice) => (
              <tr key={invoice.id}>
                <td>{invoice.id}</td>
                <td>{new Date(invoice.createdAt).toLocaleDateString()}</td>
                <td>
                  {invoice.currency} {invoice.amount.toFixed(2)}
                </td>
                <td>
                  <span className={`status-badge ${getStatusBadgeClass(invoice.status)}`}>
                    {invoice.status}
                  </span>
                </td>
                <td>
                  <button
                    type="button"
                    onClick={() => handleDownload(invoice.id)}
                    className="btn btn-sm btn-secondary"
                  >
                    Download
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {invoices.length === 0 && (
          <div className="empty-state">
            <p>No invoices found</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default InvoiceHistory;
