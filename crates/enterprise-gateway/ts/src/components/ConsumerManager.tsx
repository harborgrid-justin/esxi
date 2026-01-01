/**
 * Enterprise API Gateway - Consumer Manager Component
 */

import React, { useState } from 'react';
import type { Consumer, APIKey } from '../types';

export interface ConsumerManagerProps {
  consumers: Consumer[];
  onAddConsumer?: (consumer: Omit<Consumer, 'id' | 'createdAt' | 'updatedAt'>) => void;
  onGenerateAPIKey?: (consumerId: string, name: string) => void;
  onRevokeAPIKey?: (consumerId: string, keyId: string) => void;
}

export const ConsumerManager: React.FC<ConsumerManagerProps> = ({
  consumers,
  onAddConsumer,
  onGenerateAPIKey,
  onRevokeAPIKey,
}) => {
  const [showAddForm, setShowAddForm] = useState(false);
  const [selectedConsumer, setSelectedConsumer] = useState<string | null>(null);
  const [formData, setFormData] = useState({ username: '', customId: '', tags: '' });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onAddConsumer?.({
      username: formData.username,
      customId: formData.customId || undefined,
      apiKeys: [],
      tags: formData.tags.split(',').map((t) => t.trim()).filter(Boolean),
      enabled: true,
    });
    setFormData({ username: '', customId: '', tags: '' });
    setShowAddForm(false);
  };

  return (
    <div className="consumer-manager">
      <div className="manager-header">
        <h2>Consumers</h2>
        <button onClick={() => setShowAddForm(!showAddForm)}>
          {showAddForm ? 'Cancel' : '+ Add Consumer'}
        </button>
      </div>

      {showAddForm && (
        <form onSubmit={handleSubmit} className="add-form">
          <input
            placeholder="Username"
            value={formData.username}
            onChange={(e) => setFormData({ ...formData, username: e.target.value })}
            required
          />
          <input
            placeholder="Custom ID (optional)"
            value={formData.customId}
            onChange={(e) => setFormData({ ...formData, customId: e.target.value })}
          />
          <input
            placeholder="Tags (comma-separated)"
            value={formData.tags}
            onChange={(e) => setFormData({ ...formData, tags: e.target.value })}
          />
          <button type="submit">Create Consumer</button>
        </form>
      )}

      <div className="consumers-list">
        {consumers.map((consumer) => (
          <div key={consumer.id} className="consumer-card">
            <h3>{consumer.username}</h3>
            <p>API Keys: {consumer.apiKeys.length}</p>
            <p>Tags: {consumer.tags.join(', ') || 'None'}</p>
            <button onClick={() => onGenerateAPIKey?.(consumer.id, 'default')}>
              Generate API Key
            </button>
          </div>
        ))}
      </div>
    </div>
  );
};
