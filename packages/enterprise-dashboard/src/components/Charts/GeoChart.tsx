/**
 * Geographic Chart Component
 * Geographic distribution visualization
 */

import React, { useMemo, useRef, useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import clsx from 'clsx';
import * as d3 from 'd3';
import { geoMercator, geoPath } from 'd3-geo';
import type { GeoDataPoint } from '../../types';

export interface GeoChartProps {
  data: GeoDataPoint[];
  metric?: 'users' | 'revenue' | 'requests' | 'latency';
  height?: number;
  className?: string;
  showLabels?: boolean;
  interactive?: boolean;
  theme?: 'light' | 'dark';
}

export const GeoChart: React.FC<GeoChartProps> = ({
  data,
  metric = 'users',
  height = 500,
  className,
  showLabels = true,
  interactive = true,
  theme = 'dark',
}) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const [selectedPoint, setSelectedPoint] = useState<GeoDataPoint | null>(null);
  const [hoveredPoint, setHoveredPoint] = useState<GeoDataPoint | null>(null);

  const isDark = theme === 'dark';

  // Calculate value ranges for color scaling
  const valueRange = useMemo(() => {
    const values = data.map((d) => d[metric]);
    return {
      min: Math.min(...values),
      max: Math.max(...values),
    };
  }, [data, metric]);

  // Color scale
  const colorScale = useMemo(() => {
    return d3.scaleSequential()
      .domain([valueRange.min, valueRange.max])
      .interpolator(d3.interpolateBlues);
  }, [valueRange]);

  // Size scale for bubbles
  const sizeScale = useMemo(() => {
    return d3.scaleSqrt()
      .domain([valueRange.min, valueRange.max])
      .range([5, 40]);
  }, [valueRange]);

  // Top countries by metric
  const topCountries = useMemo(() => {
    return [...data]
      .sort((a, b) => b[metric] - a[metric])
      .slice(0, 10);
  }, [data, metric]);

  // Format value
  const formatValue = (value: number): string => {
    switch (metric) {
      case 'revenue':
        return new Intl.NumberFormat('en-US', {
          style: 'currency',
          currency: 'USD',
          minimumFractionDigits: 0,
          maximumFractionDigits: 0,
        }).format(value);
      case 'latency':
        return `${value.toFixed(0)}ms`;
      case 'requests':
        return value >= 1000000
          ? `${(value / 1000000).toFixed(1)}M`
          : value >= 1000
          ? `${(value / 1000).toFixed(1)}K`
          : value.toString();
      default:
        return new Intl.NumberFormat('en-US').format(value);
    }
  };

  // Render world map with data points
  useEffect(() => {
    if (!svgRef.current) return;

    const svg = d3.select(svgRef.current);
    const width = svgRef.current.clientWidth;

    // Clear previous content
    svg.selectAll('*').remove();

    // Create projection
    const projection = geoMercator()
      .scale(width / 6)
      .translate([width / 2, height / 2]);

    const path = geoPath().projection(projection);

    // Create main group
    const g = svg.append('g');

    // Add world map background (simplified - in production, load GeoJSON)
    // This is a placeholder - you would typically load world-110m.json or similar
    g.append('rect')
      .attr('width', width)
      .attr('height', height)
      .attr('fill', isDark ? '#1f2937' : '#f3f4f6')
      .attr('opacity', 0.3);

    // Add grid lines
    const gridSize = 30;
    for (let i = 0; i < width; i += gridSize) {
      g.append('line')
        .attr('x1', i)
        .attr('y1', 0)
        .attr('x2', i)
        .attr('y2', height)
        .attr('stroke', isDark ? '#374151' : '#d1d5db')
        .attr('stroke-width', 0.5)
        .attr('opacity', 0.3);
    }
    for (let i = 0; i < height; i += gridSize) {
      g.append('line')
        .attr('x1', 0)
        .attr('y1', i)
        .attr('x2', width)
        .attr('y2', i)
        .attr('stroke', isDark ? '#374151' : '#d1d5db')
        .attr('stroke-width', 0.5)
        .attr('opacity', 0.3);
    }

    // Add data points
    const points = g.selectAll('.data-point')
      .data(data)
      .enter()
      .append('g')
      .attr('class', 'data-point')
      .attr('transform', (d) => {
        const [x, y] = projection(d.coordinates) || [0, 0];
        return `translate(${x},${y})`;
      });

    // Add circles
    points.append('circle')
      .attr('r', (d) => sizeScale(d[metric]))
      .attr('fill', (d) => colorScale(d[metric]))
      .attr('stroke', isDark ? '#ffffff' : '#000000')
      .attr('stroke-width', 2)
      .attr('opacity', 0.7)
      .style('cursor', interactive ? 'pointer' : 'default')
      .on('mouseenter', (event, d) => {
        if (interactive) {
          d3.select(event.currentTarget)
            .transition()
            .duration(200)
            .attr('opacity', 1)
            .attr('stroke-width', 3);
          setHoveredPoint(d);
        }
      })
      .on('mouseleave', (event, d) => {
        if (interactive) {
          d3.select(event.currentTarget)
            .transition()
            .duration(200)
            .attr('opacity', 0.7)
            .attr('stroke-width', 2);
          setHoveredPoint(null);
        }
      })
      .on('click', (event, d) => {
        if (interactive) {
          setSelectedPoint(selectedPoint?.country === d.country ? null : d);
        }
      });

    // Add labels for top countries
    if (showLabels) {
      const labelData = topCountries.slice(0, 5);
      points.filter((d) => labelData.includes(d))
        .append('text')
        .attr('dy', (d) => -sizeScale(d[metric]) - 5)
        .attr('text-anchor', 'middle')
        .attr('fill', isDark ? '#e5e7eb' : '#374151')
        .attr('font-size', '12px')
        .attr('font-weight', 'bold')
        .text((d) => d.countryCode);
    }

  }, [data, metric, height, colorScale, sizeScale, isDark, interactive, showLabels, topCountries, selectedPoint]);

  // Calculate totals
  const totals = useMemo(() => {
    return {
      users: data.reduce((sum, d) => sum + d.users, 0),
      revenue: data.reduce((sum, d) => sum + d.revenue, 0),
      requests: data.reduce((sum, d) => sum + d.requests, 0),
      avgLatency: data.reduce((sum, d) => sum + d.latency, 0) / data.length,
    };
  }, [data]);

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      className={clsx('rounded-xl border border-gray-800 bg-gray-900/50 p-6', className)}
    >
      {/* Header */}
      <div className="mb-6">
        <h3 className="text-lg font-semibold text-white mb-4">Geographic Distribution</h3>

        {/* Summary Stats */}
        <div className="grid grid-cols-4 gap-3 mb-4">
          <div className="bg-gray-800/50 rounded-lg p-3">
            <div className="text-xs text-gray-400 mb-1">Total Users</div>
            <div className="text-xl font-bold text-blue-400">
              {(totals.users / 1000).toFixed(1)}K
            </div>
          </div>
          <div className="bg-gray-800/50 rounded-lg p-3">
            <div className="text-xs text-gray-400 mb-1">Total Revenue</div>
            <div className="text-xl font-bold text-green-400">
              ${(totals.revenue / 1000000).toFixed(1)}M
            </div>
          </div>
          <div className="bg-gray-800/50 rounded-lg p-3">
            <div className="text-xs text-gray-400 mb-1">Total Requests</div>
            <div className="text-xl font-bold text-violet-400">
              {(totals.requests / 1000000).toFixed(1)}M
            </div>
          </div>
          <div className="bg-gray-800/50 rounded-lg p-3">
            <div className="text-xs text-gray-400 mb-1">Avg Latency</div>
            <div className="text-xl font-bold text-amber-400">
              {totals.avgLatency.toFixed(0)}ms
            </div>
          </div>
        </div>
      </div>

      {/* Map Container */}
      <div className="relative">
        <svg
          ref={svgRef}
          width="100%"
          height={height}
          className="bg-gray-800/30 rounded-lg"
        />

        {/* Tooltip */}
        {hoveredPoint && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="absolute top-4 right-4 bg-gray-900/95 backdrop-blur-sm border border-gray-700 rounded-lg p-4 shadow-xl"
          >
            <div className="text-sm font-semibold text-white mb-2">
              {hoveredPoint.country} ({hoveredPoint.countryCode})
            </div>
            <div className="space-y-1 text-sm">
              <div className="flex justify-between gap-4">
                <span className="text-gray-400">Users:</span>
                <span className="font-semibold text-white">
                  {formatValue(hoveredPoint.users)}
                </span>
              </div>
              <div className="flex justify-between gap-4">
                <span className="text-gray-400">Revenue:</span>
                <span className="font-semibold text-white">
                  {formatValue(hoveredPoint.revenue)}
                </span>
              </div>
              <div className="flex justify-between gap-4">
                <span className="text-gray-400">Requests:</span>
                <span className="font-semibold text-white">
                  {formatValue(hoveredPoint.requests)}
                </span>
              </div>
              <div className="flex justify-between gap-4">
                <span className="text-gray-400">Latency:</span>
                <span className="font-semibold text-white">
                  {hoveredPoint.latency.toFixed(0)}ms
                </span>
              </div>
            </div>
          </motion.div>
        )}
      </div>

      {/* Top Countries Table */}
      <div className="mt-6">
        <h4 className="text-sm font-semibold text-gray-400 mb-3">Top Countries</h4>
        <div className="space-y-2">
          {topCountries.map((country, index) => (
            <div
              key={country.countryCode}
              className="flex items-center justify-between bg-gray-800/30 rounded-lg p-3 hover:bg-gray-800/50 transition-colors cursor-pointer"
              onClick={() => setSelectedPoint(selectedPoint?.country === country.country ? null : country)}
            >
              <div className="flex items-center gap-3">
                <div className="text-lg font-bold text-gray-500 w-6">
                  #{index + 1}
                </div>
                <div>
                  <div className="font-semibold text-white">{country.country}</div>
                  <div className="text-xs text-gray-500">{country.countryCode}</div>
                </div>
              </div>
              <div className="text-right">
                <div className="font-semibold text-blue-400">
                  {formatValue(country[metric])}
                </div>
                <div className="text-xs text-gray-500">
                  {((country[metric] / valueRange.max) * 100).toFixed(1)}%
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </motion.div>
  );
};

export default GeoChart;
