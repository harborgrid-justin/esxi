/**
 * Bar Chart Component - Categorical Data Visualization
 * @module @harborgrid/enterprise-analytics/components/charts
 */

import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import type { VisualizationConfig } from '../../types';

export interface BarChartProps<T = Record<string, unknown>> {
  data: T[];
  config: VisualizationConfig;
  width?: number;
  height?: number;
  onBarClick?: (data: T) => void;
  onBarHover?: (data: T) => void;
}

export function BarChart<T = Record<string, unknown>>({
  data,
  config,
  width = 600,
  height = 400,
  onBarClick,
  onBarHover,
}: BarChartProps<T>) {
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

    const margin = { top: 20, right: 30, bottom: 60, left: 60 };
    const chartWidth = width - margin.left - margin.right;
    const chartHeight = height - margin.top - margin.bottom;

    const g = svg
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Create scales
    const xScale = d3
      .scaleBand()
      .domain(data.map((d) => String((d as Record<string, unknown>)[config.xAxis!.field])))
      .range([0, chartWidth])
      .padding(0.2);

    const yScale = d3
      .scaleLinear()
      .domain([0, d3.max(data, (d) => (d as Record<string, unknown>)[config.yAxis!.field] as number) || 100])
      .nice()
      .range([chartHeight, 0]);

    // Add grid lines
    if (config.showGrid) {
      g.append('g')
        .attr('class', 'grid')
        .call(
          d3
            .axisLeft(yScale)
            .ticks(5)
            .tickSize(-chartWidth)
            .tickFormat(() => '')
        )
        .style('stroke', '#e0e0e0')
        .style('stroke-dasharray', '2,2');
    }

    // Add axes
    const xAxis = g
      .append('g')
      .attr('transform', `translate(0,${chartHeight})`)
      .call(d3.axisBottom(xScale));

    xAxis
      .selectAll('text')
      .attr('transform', 'rotate(-45)')
      .style('text-anchor', 'end')
      .attr('dx', '-.8em')
      .attr('dy', '.15em');

    if (config.xAxis.label) {
      xAxis
        .append('text')
        .attr('x', chartWidth / 2)
        .attr('y', 50)
        .attr('fill', '#000')
        .attr('text-anchor', 'middle')
        .text(config.xAxis.label);
    }

    const yAxis = g.append('g').call(d3.axisLeft(yScale));

    if (config.yAxis.label) {
      yAxis
        .append('text')
        .attr('transform', 'rotate(-90)')
        .attr('y', -50)
        .attr('x', -chartHeight / 2)
        .attr('fill', '#000')
        .attr('text-anchor', 'middle')
        .text(config.yAxis.label);
    }

    // Draw bars
    const colorScale = d3
      .scaleOrdinal<string>()
      .domain(data.map((d) => String((d as Record<string, unknown>)[config.xAxis!.field])))
      .range(config.theme?.colors || d3.schemeCategory10);

    g.selectAll('.bar')
      .data(data)
      .enter()
      .append('rect')
      .attr('class', 'bar')
      .attr('x', (d) => xScale(String((d as Record<string, unknown>)[config.xAxis!.field])) || 0)
      .attr('width', xScale.bandwidth())
      .attr('fill', (d) => colorScale(String((d as Record<string, unknown>)[config.xAxis!.field])))
      .attr('opacity', 0.8)
      .style('cursor', 'pointer')
      .attr('y', chartHeight)
      .attr('height', 0)
      .on('click', (event, d) => {
        if (onBarClick) onBarClick(d);
      })
      .on('mouseenter', function (event, d) {
        d3.select(this).attr('opacity', 1);
        const [x, y] = d3.pointer(event);
        setTooltip({ x, y, data: d });
        if (onBarHover) onBarHover(d);
      })
      .on('mouseleave', function () {
        d3.select(this).attr('opacity', 0.8);
        setTooltip(null);
      })
      .transition()
      .duration(800)
      .delay((d, i) => i * 50)
      .attr('y', (d) => yScale((d as Record<string, unknown>)[config.yAxis!.field] as number))
      .attr('height', (d) => chartHeight - yScale((d as Record<string, unknown>)[config.yAxis!.field] as number));

    // Add value labels if configured
    if (config.showValues) {
      g.selectAll('.value-label')
        .data(data)
        .enter()
        .append('text')
        .attr('class', 'value-label')
        .attr('x', (d) => (xScale(String((d as Record<string, unknown>)[config.xAxis!.field])) || 0) + xScale.bandwidth() / 2)
        .attr('y', (d) => yScale((d as Record<string, unknown>)[config.yAxis!.field] as number) - 5)
        .attr('text-anchor', 'middle')
        .attr('font-size', '11px')
        .attr('fill', '#666')
        .text((d) => String((d as Record<string, unknown>)[config.yAxis!.field]))
        .style('opacity', 0)
        .transition()
        .duration(800)
        .delay((d, i) => i * 50)
        .style('opacity', 1);
    }
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
            <div key={key} style={{ marginBottom: '2px' }}>
              <strong>{key}:</strong> {String(value)}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
