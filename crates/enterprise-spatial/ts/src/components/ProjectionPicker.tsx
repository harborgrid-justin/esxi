/**
 * Projection Picker Component
 * Select and manage coordinate reference systems
 */

import React, { useState, useEffect } from 'react';
import { Projection } from '../types';
import { ProjectionRegistry } from '../projection/ProjectionRegistry';

export interface ProjectionPickerProps {
  selected?: string;
  onSelect?: (projection: Projection) => void;
}

export const ProjectionPicker: React.FC<ProjectionPickerProps> = ({
  selected,
  onSelect,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [projections, setProjections] = useState<Projection[]>([]);
  const [filteredProjections, setFilteredProjections] = useState<Projection[]>([]);
  const [selectedProjection, setSelectedProjection] = useState<Projection | null>(null);

  useEffect(() => {
    const allProjections = ProjectionRegistry.search('');
    setProjections(allProjections);
    setFilteredProjections(allProjections);

    if (selected) {
      const proj = allProjections.find((p) => p.code === selected);
      if (proj) setSelectedProjection(proj);
    }
  }, [selected]);

  useEffect(() => {
    if (searchQuery) {
      const results = ProjectionRegistry.search(searchQuery);
      setFilteredProjections(results);
    } else {
      setFilteredProjections(projections);
    }
  }, [searchQuery, projections]);

  const handleSelect = (projection: Projection) => {
    setSelectedProjection(projection);
    onSelect?.(projection);
  };

  return (
    <div className="projection-picker">
      <div className="picker-header">
        <h3>Coordinate Reference System</h3>
      </div>

      <div className="search-box">
        <input
          type="text"
          placeholder="Search EPSG code or name..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>

      {selectedProjection && (
        <div className="selected-projection">
          <div className="projection-title">Selected:</div>
          <div className="projection-code">{selectedProjection.code}</div>
          <div className="projection-name">{selectedProjection.name}</div>
          <div className="projection-details">
            <div className="detail-row">
              <span className="label">Units:</span>
              <span className="value">{selectedProjection.units}</span>
            </div>
            <div className="detail-row">
              <span className="label">Proj4:</span>
              <code className="value">{selectedProjection.proj4def}</code>
            </div>
          </div>
        </div>
      )}

      <div className="projection-list">
        <div className="list-header">
          Available Projections ({filteredProjections.length})
        </div>
        <div className="list-items">
          {filteredProjections.map((projection) => (
            <div
              key={projection.code}
              className={`projection-item ${
                selectedProjection?.code === projection.code ? 'selected' : ''
              }`}
              onClick={() => handleSelect(projection)}
            >
              <div className="item-code">{projection.code}</div>
              <div className="item-name">{projection.name}</div>
              <div className="item-units">{projection.units}</div>
            </div>
          ))}
        </div>
      </div>

      <style jsx>{`
        .projection-picker {
          width: 500px;
          background: white;
          border: 1px solid #ddd;
          border-radius: 4px;
          max-height: 700px;
          display: flex;
          flex-direction: column;
        }

        .picker-header {
          padding: 15px;
          border-bottom: 1px solid #ddd;
        }

        .picker-header h3 {
          margin: 0;
          font-size: 18px;
        }

        .search-box {
          padding: 15px;
          border-bottom: 1px solid #eee;
        }

        .search-box input {
          width: 100%;
          padding: 10px;
          border: 1px solid #ddd;
          border-radius: 3px;
          font-size: 14px;
        }

        .selected-projection {
          padding: 15px;
          background: #e3f2fd;
          border-bottom: 1px solid #ddd;
        }

        .projection-title {
          font-size: 11px;
          text-transform: uppercase;
          color: #666;
          margin-bottom: 5px;
        }

        .projection-code {
          font-size: 16px;
          font-weight: bold;
          color: #007bff;
          margin-bottom: 5px;
        }

        .projection-name {
          font-size: 14px;
          margin-bottom: 10px;
        }

        .projection-details {
          font-size: 12px;
        }

        .detail-row {
          margin-bottom: 5px;
        }

        .detail-row .label {
          font-weight: bold;
          min-width: 60px;
          display: inline-block;
        }

        .detail-row .value {
          color: #555;
        }

        .detail-row code {
          background: #f5f5f5;
          padding: 2px 4px;
          border-radius: 2px;
          font-size: 11px;
          word-break: break-all;
        }

        .projection-list {
          flex: 1;
          overflow: hidden;
          display: flex;
          flex-direction: column;
        }

        .list-header {
          padding: 10px 15px;
          background: #f5f5f5;
          font-weight: bold;
          font-size: 13px;
          border-bottom: 1px solid #ddd;
        }

        .list-items {
          flex: 1;
          overflow-y: auto;
        }

        .projection-item {
          padding: 12px 15px;
          border-bottom: 1px solid #eee;
          cursor: pointer;
          transition: background 0.2s;
        }

        .projection-item:hover {
          background: #f9f9f9;
        }

        .projection-item.selected {
          background: #e3f2fd;
          border-left: 3px solid #007bff;
        }

        .item-code {
          font-weight: bold;
          color: #007bff;
          margin-bottom: 3px;
        }

        .item-name {
          font-size: 13px;
          color: #333;
          margin-bottom: 3px;
        }

        .item-units {
          font-size: 11px;
          color: #666;
        }
      `}</style>
    </div>
  );
};
