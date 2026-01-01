/**
 * Enterprise API Gateway - Plugin Manager Component
 */

import React, { useState } from 'react';
import type { Plugin, PluginPhase } from '../types';

export interface PluginManagerProps {
  plugins: Plugin[];
  onAdd?: (plugin: Omit<Plugin, 'id'>) => void;
  onUpdate?: (id: string, updates: Partial<Plugin>) => void;
  onDelete?: (id: string) => void;
}

export const PluginManager: React.FC<PluginManagerProps> = ({
  plugins,
  onAdd,
  onUpdate,
  onDelete,
}) => {
  const [showForm, setShowForm] = useState(false);

  const groupedPlugins = plugins.reduce((acc, plugin) => {
    const phase = plugin.phase;
    if (!acc[phase]) acc[phase] = [];
    acc[phase]!.push(plugin);
    return acc;
  }, {} as Record<PluginPhase, Plugin[]>);

  return (
    <div className="plugin-manager">
      <h2>Plugins</h2>

      {(['pre-route', 'route', 'post-route', 'error'] as PluginPhase[]).map((phase) => (
        <div key={phase} className="plugin-phase">
          <h3>{phase.toUpperCase()}</h3>
          <div className="plugins-grid">
            {(groupedPlugins[phase] || []).map((plugin) => (
              <div key={plugin.id} className="plugin-card">
                <h4>{plugin.name}</h4>
                <p>Priority: {plugin.priority}</p>
                <button onClick={() => onUpdate?.(plugin.id, { enabled: !plugin.enabled })}>
                  {plugin.enabled ? 'Disable' : 'Enable'}
                </button>
              </div>
            ))}
          </div>
        </div>
      ))}

      <style>{`
        .plugin-manager {
          padding: 20px;
        }

        .plugin-phase {
          margin-bottom: 30px;
        }

        .plugins-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
          gap: 15px;
        }

        .plugin-card {
          background: white;
          padding: 15px;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        .plugin-card h4 {
          margin-top: 0;
        }

        .plugin-card button {
          width: 100%;
          padding: 8px;
          margin-top: 10px;
          background: #007bff;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }
      `}</style>
    </div>
  );
};
