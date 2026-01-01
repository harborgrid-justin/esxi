/**
 * Optimization Settings Component
 * Configuration interface for compression settings
 */

import React, { useState } from 'react';
import {
  CompressionAlgorithm,
  CompressionLevel,
  DataOptimizationProfile,
} from '../types';

interface OptimizationSettingsProps {
  onSave: (profile: DataOptimizationProfile) => void;
  initialProfile?: DataOptimizationProfile;
}

export const OptimizationSettings: React.FC<OptimizationSettingsProps> = ({
  onSave,
  initialProfile,
}) => {
  const [profile, setProfile] = useState<DataOptimizationProfile>(
    initialProfile || {
      name: 'Custom Profile',
      algorithm: CompressionAlgorithm.GZIP,
      level: CompressionLevel.BALANCED,
      enableMinification: true,
      enableDedupe: false,
      enableSchemaOptimization: false,
    }
  );

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave(profile);
  };

  return (
    <div style={styles.container}>
      <h2 style={styles.title}>Optimization Settings</h2>

      <form onSubmit={handleSubmit} style={styles.form}>
        <div style={styles.field}>
          <label style={styles.label}>Profile Name</label>
          <input
            type="text"
            value={profile.name}
            onChange={(e) => setProfile({ ...profile, name: e.target.value })}
            style={styles.input}
          />
        </div>

        <div style={styles.field}>
          <label style={styles.label}>Compression Algorithm</label>
          <select
            value={profile.algorithm}
            onChange={(e) =>
              setProfile({
                ...profile,
                algorithm: e.target.value as CompressionAlgorithm,
              })
            }
            style={styles.select}
          >
            <option value={CompressionAlgorithm.GZIP}>GZIP</option>
            <option value={CompressionAlgorithm.BROTLI}>Brotli</option>
            <option value={CompressionAlgorithm.ZSTD}>Zstandard</option>
            <option value={CompressionAlgorithm.LZ4}>LZ4</option>
            <option value={CompressionAlgorithm.DEFLATE}>Deflate</option>
          </select>
        </div>

        <div style={styles.field}>
          <label style={styles.label}>Compression Level</label>
          <select
            value={profile.level}
            onChange={(e) =>
              setProfile({
                ...profile,
                level: parseInt(e.target.value) as CompressionLevel,
              })
            }
            style={styles.select}
          >
            <option value={CompressionLevel.FASTEST}>Fastest (1)</option>
            <option value={CompressionLevel.FAST}>Fast (3)</option>
            <option value={CompressionLevel.BALANCED}>Balanced (6)</option>
            <option value={CompressionLevel.HIGH}>High (9)</option>
            <option value={CompressionLevel.MAXIMUM}>Maximum (11)</option>
          </select>
        </div>

        <div style={styles.field}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={profile.enableMinification !== false}
              onChange={(e) =>
                setProfile({ ...profile, enableMinification: e.target.checked })
              }
            />
            Enable Minification
          </label>
        </div>

        <div style={styles.field}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={profile.enableDedupe || false}
              onChange={(e) =>
                setProfile({ ...profile, enableDedupe: e.target.checked })
              }
            />
            Enable Deduplication
          </label>
        </div>

        <div style={styles.field}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={profile.enableSchemaOptimization || false}
              onChange={(e) =>
                setProfile({
                  ...profile,
                  enableSchemaOptimization: e.target.checked,
                })
              }
            />
            Enable Schema Optimization
          </label>
        </div>

        <div style={styles.actions}>
          <button type="submit" style={styles.button}>
            Save Profile
          </button>
        </div>
      </form>
    </div>
  );
};

const styles = {
  container: {
    padding: '24px',
    backgroundColor: 'white',
    borderRadius: '8px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
    maxWidth: '600px',
    margin: '0 auto',
  },
  title: {
    margin: '0 0 24px 0',
    fontSize: '24px',
    fontWeight: '600',
    color: '#333',
  },
  form: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '20px',
  },
  field: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
  },
  label: {
    fontSize: '14px',
    fontWeight: '500',
    color: '#333',
  },
  input: {
    padding: '10px',
    fontSize: '14px',
    border: '1px solid #ddd',
    borderRadius: '4px',
  },
  select: {
    padding: '10px',
    fontSize: '14px',
    border: '1px solid #ddd',
    borderRadius: '4px',
    backgroundColor: 'white',
  },
  checkboxLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '14px',
    color: '#333',
  },
  actions: {
    display: 'flex',
    gap: '12px',
    marginTop: '8px',
  },
  button: {
    padding: '12px 24px',
    backgroundColor: '#007bff',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    fontSize: '14px',
    fontWeight: '500',
    cursor: 'pointer',
  },
};
