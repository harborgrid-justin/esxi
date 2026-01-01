/**
 * Asset Optimizer Component
 * UI for optimizing images and assets
 */

import React, { useState } from 'react';
import { ImageFormat, ImageOptimizationConfig } from '../types';

interface AssetOptimizerProps {
  onOptimize: (file: File, config: ImageOptimizationConfig) => Promise<void>;
}

export const AssetOptimizer: React.FC<AssetOptimizerProps> = ({ onOptimize }) => {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [config, setConfig] = useState<ImageOptimizationConfig>({
    format: ImageFormat.WEBP,
    quality: 85,
    progressive: true,
    stripMetadata: true,
  });
  const [optimizing, setOptimizing] = useState(false);

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setSelectedFile(file);
    }
  };

  const handleOptimize = async () => {
    if (!selectedFile) return;

    setOptimizing(true);
    try {
      await onOptimize(selectedFile, config);
    } finally {
      setOptimizing(false);
    }
  };

  return (
    <div style={styles.container}>
      <h2 style={styles.title}>Asset Optimizer</h2>

      <div style={styles.uploader}>
        <input
          type="file"
          accept="image/*"
          onChange={handleFileSelect}
          style={styles.fileInput}
        />
        {selectedFile && (
          <div style={styles.fileName}>{selectedFile.name}</div>
        )}
      </div>

      <div style={styles.config}>
        <div style={styles.field}>
          <label style={styles.label}>Output Format</label>
          <select
            value={config.format}
            onChange={(e) =>
              setConfig({ ...config, format: e.target.value as ImageFormat })
            }
            style={styles.select}
          >
            <option value={ImageFormat.WEBP}>WebP</option>
            <option value={ImageFormat.AVIF}>AVIF</option>
            <option value={ImageFormat.JPEG}>JPEG</option>
            <option value={ImageFormat.PNG}>PNG</option>
          </select>
        </div>

        <div style={styles.field}>
          <label style={styles.label}>Quality: {config.quality}</label>
          <input
            type="range"
            min="1"
            max="100"
            value={config.quality}
            onChange={(e) =>
              setConfig({ ...config, quality: parseInt(e.target.value) })
            }
            style={styles.slider}
          />
        </div>

        <div style={styles.field}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={config.progressive !== false}
              onChange={(e) =>
                setConfig({ ...config, progressive: e.target.checked })
              }
            />
            Progressive
          </label>
        </div>

        <div style={styles.field}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={config.stripMetadata !== false}
              onChange={(e) =>
                setConfig({ ...config, stripMetadata: e.target.checked })
              }
            />
            Strip Metadata
          </label>
        </div>
      </div>

      <button
        onClick={handleOptimize}
        disabled={!selectedFile || optimizing}
        style={{
          ...styles.button,
          ...((!selectedFile || optimizing) && styles.buttonDisabled),
        }}
      >
        {optimizing ? 'Optimizing...' : 'Optimize Image'}
      </button>
    </div>
  );
};

const styles = {
  container: {
    padding: '24px',
    backgroundColor: 'white',
    borderRadius: '8px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
  },
  title: {
    margin: '0 0 24px 0',
    fontSize: '24px',
    fontWeight: '600',
    color: '#333',
  },
  uploader: {
    marginBottom: '24px',
  },
  fileInput: {
    width: '100%',
    padding: '12px',
    border: '2px dashed #ddd',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  fileName: {
    marginTop: '8px',
    fontSize: '14px',
    color: '#666',
  },
  config: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '16px',
    marginBottom: '24px',
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
  select: {
    padding: '10px',
    fontSize: '14px',
    border: '1px solid #ddd',
    borderRadius: '4px',
  },
  slider: {
    width: '100%',
  },
  checkboxLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '14px',
    color: '#333',
  },
  button: {
    width: '100%',
    padding: '12px',
    backgroundColor: '#007bff',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    fontSize: '16px',
    fontWeight: '500',
    cursor: 'pointer',
  },
  buttonDisabled: {
    backgroundColor: '#ccc',
    cursor: 'not-allowed',
  },
};
