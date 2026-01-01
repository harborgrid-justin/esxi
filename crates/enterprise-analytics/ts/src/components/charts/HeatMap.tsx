/**
 * Heat Map Component - Density Visualization
 * @module @harborgrid/enterprise-analytics/components/charts
 */

import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import type { VisualizationConfig } from '../../types';

export interface HeatMapProps<T = Record<string, unknown>> {
  data: T[];
  config: VisualizationConfig;
  width?: number;
  height?: number;
  onCellClick?: (data: T) => void;
}

export function HeatMap<T = Record<string, unknown>>({
  data,
  config,
  width = 600,
  height = 400,
  onCellClick,
}: HeatMapProps<T>) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [tooltip, setTooltip] = useState<{ x: number; y: number; data: T } | null>(null);

  useEffect(() => {
    if (!svgRef.current || !data || data.length === 0) return;
    renderChart();
  }, [data, config, width, height]);

  const renderChart = () => {
    if (!svgRef.current || !config.xAxis || !config.yAxis || !config.colorScale) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll('*').remove();

    const margin = { top: 40, right: 100, bottom: 60, left: 80 };
    const chartWidth = width - margin.left - margin.right;
    const chartHeight = height - margin.top - margin.bottom;

    const g = svg.append('g').attr('transform', `translate(${margin.left},${margin.top})`);

    // Get unique x and y values
    const xValues = Array.from(new Set(data.map((d) => String((d as Record<string, unknown>)[config.xAxis!.field]))));
    const yValues = Array.from(new Set(data.map((d) => String((d as Record<string, unknown>)[config.yAxis!.field]))));

    // Create scales
    const xScale = d3.scaleBand().domain(xValues).range([0, chartWidth]).padding(0.05);

    const yScale = d3.scaleBand().domain(yValues).range([0, chartHeight]).padding(0.05);

    const valueField = config.colorScale.domain?.[0] as string || 'value';
    const values = data.map((d) => (d as Record<string, unknown>)[valueField] as number);
    const colorScale = d3
      .scaleSequential()
      .interpolator(d3.interpolateYlOrRd)
      .domain([d3.min(values) || 0, d3.max(values) || 100]);

    // Add x-axis
    g.append('g')
      .attr('transform', `translate(0,${chartHeight})`)
      .call(d3.axisBottom(xScale))
      .selectAll('text')
      .attr('transform', 'rotate(-45)')
      .style('text-anchor', 'end');

    // Add y-axis
    g.append('g').call(d3.axisLeft(yScale));

    // Draw cells
    g.selectAll('.cell')
      .data(data)
      .enter()
      .append('rect')
      .attr('class', 'cell')
      .attr('x', (d) => xScale(String((d as Record<string, unknown>)[config.xAxis!.field])) || 0)
      .attr('y', (d) => yScale(String((d as Record<string, unknown>)[config.yAxis!.field])) || 0)
      .attr('width', xScale.bandwidth())
      .attr('height', yScale.bandwidth())
      .attr('fill', (d) => colorScale((d as Record<string, unknown>)[valueField] as number))
      .attr('stroke', 'white')
      .attr('stroke-width', 1)
      .style('cursor', 'pointer')
      .style('opacity', 0)
      .on('click', (event, d) => {
        if (onCellClick) onCellClick(d);
      })
      .on('mouseenter', function (event, d) {
        d3.select(this).attr('stroke', '#333').attr('stroke-width', 2);
        const [x, y] = d3.pointer(event);
        setTooltip({ x, y, data: d });
      })
      .on('mouseleave', function () {
        d3.select(this).attr('stroke', 'white').attr('stroke-width', 1);
        setTooltip(null);
      })
      .transition()
      .duration(500)
      .delay((d, i) => i * 10)
      .style('opacity', 1);

    // Add legend
    const legendWidth = 20;
    const legendHeight = chartHeight;

    const legendScale = d3
      .scaleLinear()
      .domain([d3.min(values) || 0, d3.max(values) || 100])
      .range([legendHeight, 0]);

    const legendAxis = d3.axisRight(legendScale).ticks(5);

    const legend = svg
      .append('g')
      .attr('transform', `translate(${width - margin.right + 20},${margin.top})`);

    // Create gradient
    const defs = svg.append('defs');
    const gradient = defs
      .append('linearGradient')
      .attr('id', 'heatmap-gradient')
      .attr('x1', '0%')
      .attr('y1', '100%')
      .attr('x2', '0%')
      .attr('y2', '0%');

    gradient
      .append('stop')
      .attr('offset', '0%')
      .attr('stop-color', colorScale(d3.min(values) || 0));

    gradient
      .append('stop')
      .attr('offset', '100%')
      .attr('stop-color', colorScale(d3.max(values) || 100));

    legend
      .append('rect')
      .attr('width', legendWidth)
      .attr('height', legendHeight)
      .style('fill', 'url(#heatmap-gradient)');

    legend.append('g').attr('transform', `translate(${legendWidth}, 0)`).call(legendAxis);
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
              <strong>{key}:</strong> {String(value)}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
