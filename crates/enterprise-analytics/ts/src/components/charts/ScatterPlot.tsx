/**
 * Scatter Plot Component - Correlation Visualization
 * @module @harborgrid/enterprise-analytics/components/charts
 */

import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import type { VisualizationConfig } from '../../types';

export interface ScatterPlotProps<T = Record<string, unknown>> {
  data: T[];
  config: VisualizationConfig;
  width?: number;
  height?: number;
  onPointClick?: (data: T) => void;
}

export function ScatterPlot<T = Record<string, unknown>>({
  data,
  config,
  width = 600,
  height = 400,
  onPointClick,
}: ScatterPlotProps<T>) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [tooltip, setTooltip] = useState<{ x: number; y: number; data: T } | null>(null);

  useEffect(() => {
    if (!svgRef.current || !data || data.length === 0) return;
    renderChart();
  }, [data, config, width, height]);

  const renderChart = () => {
    if (!svgRef.current || !config.xAxis || !config.yAxis) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll('*').remove();

    const margin = { top: 20, right: 30, bottom: 50, left: 60 };
    const chartWidth = width - margin.left - margin.right;
    const chartHeight = height - margin.top - margin.bottom;

    const g = svg.append('g').attr('transform', `translate(${margin.left},${margin.top})`);

    // Create scales
    const xExtent = d3.extent(data, (d) => (d as Record<string, unknown>)[config.xAxis!.field] as number) as [number, number];
    const yExtent = d3.extent(data, (d) => (d as Record<string, unknown>)[config.yAxis!.field] as number) as [number, number];

    const xScale = d3.scaleLinear().domain(xExtent).nice().range([0, chartWidth]);

    const yScale = d3.scaleLinear().domain(yExtent).nice().range([chartHeight, 0]);

    // Size scale if configured
    let sizeScale: d3.ScaleLinear<number, number> | undefined;
    if (config.zAxis) {
      const sizeExtent = d3.extent(data, (d) => (d as Record<string, unknown>)[config.zAxis!.field] as number) as [number, number];
      sizeScale = d3.scaleLinear().domain(sizeExtent).range([3, 20]);
    }

    // Color scale
    const colorScale = d3
      .scaleOrdinal<string>()
      .range(config.theme?.colors || d3.schemeCategory10);

    // Add grid
    if (config.showGrid) {
      g.append('g')
        .attr('class', 'grid')
        .call(d3.axisLeft(yScale).ticks(5).tickSize(-chartWidth).tickFormat(() => ''))
        .style('stroke', '#e0e0e0')
        .style('stroke-dasharray', '2,2');

      g.append('g')
        .attr('class', 'grid')
        .attr('transform', `translate(0,${chartHeight})`)
        .call(d3.axisBottom(xScale).ticks(5).tickSize(-chartHeight).tickFormat(() => ''))
        .style('stroke', '#e0e0e0')
        .style('stroke-dasharray', '2,2');
    }

    // Add axes
    g.append('g')
      .attr('transform', `translate(0,${chartHeight})`)
      .call(d3.axisBottom(xScale))
      .append('text')
      .attr('x', chartWidth / 2)
      .attr('y', 40)
      .attr('fill', '#000')
      .attr('text-anchor', 'middle')
      .text(config.xAxis.label || '');

    g.append('g')
      .call(d3.axisLeft(yScale))
      .append('text')
      .attr('transform', 'rotate(-90)')
      .attr('y', -50)
      .attr('x', -chartHeight / 2)
      .attr('fill', '#000')
      .attr('text-anchor', 'middle')
      .text(config.yAxis.label || '');

    // Draw points
    g.selectAll('.point')
      .data(data)
      .enter()
      .append('circle')
      .attr('class', 'point')
      .attr('cx', (d) => xScale((d as Record<string, unknown>)[config.xAxis!.field] as number))
      .attr('cy', (d) => yScale((d as Record<string, unknown>)[config.yAxis!.field] as number))
      .attr('r', (d) => {
        if (sizeScale && config.zAxis) {
          return sizeScale((d as Record<string, unknown>)[config.zAxis.field] as number);
        }
        return 5;
      })
      .attr('fill', (d, i) => colorScale(String(i)))
      .attr('stroke', 'white')
      .attr('stroke-width', 1.5)
      .attr('opacity', 0.7)
      .style('cursor', 'pointer')
      .on('click', (event, d) => {
        if (onPointClick) onPointClick(d);
      })
      .on('mouseenter', function (event, d) {
        d3.select(this)
          .transition()
          .duration(200)
          .attr('opacity', 1)
          .attr('r', function () {
            const currentR = parseFloat(d3.select(this).attr('r'));
            return currentR * 1.5;
          });

        const [x, y] = d3.pointer(event);
        setTooltip({ x, y, data: d });
      })
      .on('mouseleave', function (event, d) {
        d3.select(this)
          .transition()
          .duration(200)
          .attr('opacity', 0.7)
          .attr('r', () => {
            if (sizeScale && config.zAxis) {
              return sizeScale((d as Record<string, unknown>)[config.zAxis.field] as number);
            }
            return 5;
          });
        setTooltip(null);
      })
      .style('opacity', 0)
      .transition()
      .duration(800)
      .delay((d, i) => i * 10)
      .style('opacity', 0.7);

    // Add trend line if enabled
    if (config.custom?.showTrendLine) {
      const xValues = data.map((d) => (d as Record<string, unknown>)[config.xAxis!.field] as number);
      const yValues = data.map((d) => (d as Record<string, unknown>)[config.yAxis!.field] as number);

      const slope = calculateSlope(xValues, yValues);
      const intercept = calculateIntercept(xValues, yValues, slope);

      const trendLine = d3.line<[number, number]>()
        .x((d) => xScale(d[0]))
        .y((d) => yScale(d[1]));

      const trendData: [number, number][] = [
        [xExtent[0], slope * xExtent[0] + intercept],
        [xExtent[1], slope * xExtent[1] + intercept],
      ];

      g.append('path')
        .datum(trendData)
        .attr('d', trendLine)
        .attr('fill', 'none')
        .attr('stroke', '#e74c3c')
        .attr('stroke-width', 2)
        .attr('stroke-dasharray', '5,5');
    }
  };

  const calculateSlope = (xValues: number[], yValues: number[]): number => {
    const n = xValues.length;
    const sumX = d3.sum(xValues);
    const sumY = d3.sum(yValues);
    const sumXY = d3.sum(xValues.map((x, i) => x * yValues[i]!));
    const sumX2 = d3.sum(xValues.map((x) => x * x));

    return (n * sumXY - sumX * sumY) / (n * sumX2 - sumX * sumX);
  };

  const calculateIntercept = (xValues: number[], yValues: number[], slope: number): number => {
    const n = xValues.length;
    const sumX = d3.sum(xValues);
    const sumY = d3.sum(yValues);

    return (sumY - slope * sumX) / n;
  };

  return (
    <div style={{ position: 'relative' }}>
      <svg ref={svgRef} width={width} height={height} />
      {tooltip && config.tooltip?.show && (
        <div
          style={{
            position: 'absolute',
            left: tooltip.x + 10,
            top: tooltip.y - 10,
            backgroundColor: 'white',
            border: '1px solid #ddd',
            padding: '8px 12px',
            borderRadius: '4px',
            boxShadow: '0 2px 8px rgba(0,0,0,0.15)',
            pointerEvents: 'none',
            fontSize: '12px',
            zIndex: 1000,
          }}
        >
          {Object.entries(tooltip.data as Record<string, unknown>).map(([key, value]) => (
            <div key={key}>
              <strong>{key}:</strong> {typeof value === 'number' ? value.toFixed(2) : String(value)}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
