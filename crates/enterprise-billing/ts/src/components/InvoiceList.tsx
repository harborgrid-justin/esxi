/**
 * Invoice List - Display and manage invoices
 */

import React, { useState } from 'react';
import { Invoice, InvoiceStatus } from '../types';
import { format } from 'date-fns';

export interface InvoiceListProps {
  invoices: Invoice[];
  onViewInvoice: (invoice: Invoice) => void;
  onDownloadInvoice: (invoice: Invoice) => void;
  onPayInvoice: (invoice: Invoice) => void;
}

export const InvoiceList: React.FC<InvoiceListProps> = ({
  invoices,
  onViewInvoice,
  onDownloadInvoice,
  onPayInvoice,
}) => {
  const [filter, setFilter] = useState<InvoiceStatus | 'all'>('all');

  const filteredInvoices = filter === 'all'
    ? invoices
    : invoices.filter((inv) => inv.status === filter);

  const getStatusClass = (status: InvoiceStatus): string => {
    switch (status) {
      case InvoiceStatus.PAID:
        return 'status-paid';
      case InvoiceStatus.OPEN:
        return 'status-open';
      case InvoiceStatus.VOID:
        return 'status-void';
      case InvoiceStatus.UNCOLLECTIBLE:
        return 'status-uncollectible';
      default:
        return '';
    }
  };

  return (
    <div className="invoice-list">
      <div className="list-header">
        <h2>Invoices</h2>
        <div className="filters">
          <button
            className={filter === 'all' ? 'active' : ''}
            onClick={() => setFilter('all')}
          >
            All
          </button>
          <button
            className={filter === InvoiceStatus.OPEN ? 'active' : ''}
            onClick={() => setFilter(InvoiceStatus.OPEN)}
          >
            Unpaid
          </button>
          <button
            className={filter === InvoiceStatus.PAID ? 'active' : ''}
            onClick={() => setFilter(InvoiceStatus.PAID)}
          >
            Paid
          </button>
        </div>
      </div>

      <div className="invoices">
        {filteredInvoices.length === 0 ? (
          <div className="no-invoices">No invoices found</div>
        ) : (
          <table>
            <thead>
              <tr>
                <th>Invoice #</th>
                <th>Date</th>
                <th>Amount</th>
                <th>Status</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredInvoices.map((invoice) => (
                <tr key={invoice.id}>
                  <td className="invoice-number" onClick={() => onViewInvoice(invoice)}>
                    {invoice.number}
                  </td>
                  <td>{format(invoice.createdAt, 'MMM d, yyyy')}</td>
                  <td className="amount">${invoice.total.toFixed(2)}</td>
                  <td>
                    <span className={`status ${getStatusClass(invoice.status)}`}>
                      {invoice.status}
                    </span>
                  </td>
                  <td className="actions">
                    <button
                      className="btn-sm"
                      onClick={() => onDownloadInvoice(invoice)}
                    >
                      Download
                    </button>
                    {invoice.status === InvoiceStatus.OPEN && (
                      <button
                        className="btn-sm btn-primary"
                        onClick={() => onPayInvoice(invoice)}
                      >
                        Pay Now
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      <style jsx>{`
        .invoice-list {
          padding: 2rem;
          max-width: 1200px;
          margin: 0 auto;
        }

        .list-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 2rem;
        }

        .filters {
          display: flex;
          gap: 0.5rem;
        }

        .filters button {
          padding: 0.5rem 1rem;
          border: 1px solid #ddd;
          background: white;
          cursor: pointer;
          border-radius: 4px;
        }

        .filters button.active {
          background: #007bff;
          color: white;
          border-color: #007bff;
        }

        table {
          width: 100%;
          background: white;
          border-radius: 8px;
          overflow: hidden;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        thead {
          background: #f8f9fa;
        }

        th,
        td {
          padding: 1rem;
          text-align: left;
        }

        tbody tr {
          border-bottom: 1px solid #f0f0f0;
        }

        tbody tr:hover {
          background: #f8f9fa;
        }

        .invoice-number {
          color: #007bff;
          cursor: pointer;
          font-weight: 600;
        }

        .amount {
          font-weight: 700;
        }

        .status {
          padding: 0.25rem 0.75rem;
          border-radius: 12px;
          font-size: 0.85rem;
          font-weight: 600;
        }

        .status-paid {
          background: #d4edda;
          color: #155724;
        }

        .status-open {
          background: #fff3cd;
          color: #856404;
        }

        .status-void {
          background: #f8d7da;
          color: #721c24;
        }

        .actions {
          display: flex;
          gap: 0.5rem;
        }

        .btn-sm {
          padding: 0.375rem 0.75rem;
          border: 1px solid #007bff;
          background: white;
          color: #007bff;
          border-radius: 4px;
          cursor: pointer;
          font-size: 0.875rem;
        }

        .btn-sm.btn-primary {
          background: #007bff;
          color: white;
        }

        .no-invoices {
          text-align: center;
          padding: 3rem;
          color: #666;
        }
      `}</style>
    </div>
  );
};
