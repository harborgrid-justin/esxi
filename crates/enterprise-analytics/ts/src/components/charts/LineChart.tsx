/**
 * Line Chart Component - Time Series Visualization
 * @module @harborgrid/enterprise-analytics/components/charts
 */

import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import type { VisualizationConfig } from '../../types';

export interface LineChartProps<T = Record<string, unknown>> {
  data: T[];
  config: VisualizationConfig;
  width?: number;
  height?: number;
  onDataPointClick?: (data: T) => void;
  onDataPointHover?: (data: T) => void;
}

export function LineChart<T = Record<string, unknown>>({
  data,
  config,
  width = 600,
  height = 400,
  onDataPointClick,
  onDataPointHover,
}: LineChartProps<T>) {
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

    const margin = { top: 20, right: 30, bottom: 40, left: 50 };
    const chartWidth = width - margin.left - margin.right;
    const chartHeight = height - margin.top - margin.bottom;

    const g = svg
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Create scales
    const xScale = createXScale(data, config.xAxis, chartWidth);
    const yScale = createYScale(data, config.yAxis, chartHeight);

    // Create axes
    if (config.showGrid) {
      addGridLines(g, xScale, yScale, chartWidth, chartHeight);
    }

    addAxes(g, xScale, yScale, chartWidth, chartHeight, config);

    // Create line generator
    const line = d3
      .line<T>()
      .x((d) => xScale((d as Record<string, unknown>)[config.xAxis!.field] as any) || 0)
      .y((d) => yScale((d as Record<string, unknown>)[config.yAxis!.field] as number) || 0);

    if (config.smooth) {
      line.curve(d3.curveMonotoneX);
    }

    // Draw line
    const path = g
      .append('path')
      .datum(data)
      .attr('fill', 'none')
      .attr('stroke', config.theme?.colors?.[0] || '#1f77b4')
      .attr('stroke-width', 2)
      .attr('d', line);

    // Animate line drawing
    const pathLength = (path.node() as SVGPathElement)?.getTotalLength() || 0;
    path
      .attr('stroke-dasharray', `${pathLength} ${pathLength}`)
      .attr('stroke-dashoffset', pathLength)
      .transition()
      .duration(1000)
      .ease(d3.easeLinear)
      .attr('stroke-dashoffset', 0);

    // Add data points
    if (config.showValues) {
      g.selectAll('.data-point')
        .data(data)
        .enter()
        .append('circle')
        .attr('class', 'data-point')
        .attr('cx', (d) => xScale((d as Record<string, unknown>)[config.xAxis!.field] as any) || 0)
        .attr('cy', (d) => yScale((d as Record<string, unknown>)[config.yAxis!.field] as number) || 0)
        .attr('r', 4)
        .attr('fill', config.theme?.colors?.[0] || '#1f77b4')
        .style('cursor', 'pointer')
        .on('click', (event, d) => {
          if (onDataPointClick) onDataPointClick(d);
        })
        .on('mouseenter', (event, d) => {
          const [x, y] = d3.pointer(event);
          setTooltip({ x, y, data: d });
          if (onDataPointHover) onDataPointHover(d);
        })
        .on('mouseleave', () => {
          setTooltip(null);
        });
    }

    // Add annotations
    if (config.annotations) {
      addAnnotations(g, config.annotations, xScale, yScale, chartHeight);
    }
  };

  const createXScale = (
    data: T[],
    axis: NonNullable<VisualizationConfig['xAxis']>,
    width: number
  ) => {
    if (axis.scale === 'time') {
      const extent = d3.extent(data, (d) => new Date((d as Record<string, unknown>)[axis.field] as string));
      return d3.scaleTime().domain(extent as [Date, Date]).range([0, width]);
    }

    if (axis.scale === 'band') {
      const domain = data.map((d) => String((d as Record<string, unknown>)[axis.field]));
      return d3.scaleBand().domain(domain).range([0, width]).padding(0.1);
    }

    const extent = d3.extent(data, (d) => (d as Record<string, unknown>)[axis.field] as number);
    return d3.scaleLinear().domain(extent as [number, number]).range([0, width]);
  };

  const createYScale = (
    data: T[],
    axis: NonNullable<VisualizationConfig['yAxis']>,
    height: number
  ) => {
    const extent = d3.extent(data, (d) => (d as Record<string, unknown>)[axis.field] as number);
    const domain = axis.zero ? [0, extent[1] || 100] : (extent as [number, number]);
    return d3.scaleLinear().domain(domain).range([height, 0]);
  };

  const addGridLines = (
    g: d3.Selection<SVGGElement, unknown, null, undefined>,
    xScale: any,
    yScale: any,
    width: number,
    height: number
  ) => {
    // Horizontal grid lines
    g.append('g')
      .attr('class', 'grid')
      .call(
        d3
          .axisLeft(yScale)
          .ticks(5)
          .tickSize(-width)
          .tickFormat(() => '')
      )
      .style('stroke', '#e0e0e0')
      .style('stroke-dasharray', '2,2');

    // Vertical grid lines
    g.append('g')
      .attr('class', 'grid')
      .attr('transform', `translate(0,${height})`)
      .call(
        d3
          .axisBottom(xScale)
          .ticks(5)
          .tickSize(-height)
          .tickFormat(() => '')
      )
      .style('stroke', '#e0e0e0')
      .style('stroke-dasharray', '2,2');
  };

  const addAxes = (
    g: d3.Selection<SVGGElement, unknown, null, undefined>,
    xScale: any,
    yScale: any,
    width: number,
    height: number,
    config: VisualizationConfig
  ) => {
    // X axis
    g.append('g')
      .attr('transform', `translate(0,${height})`)
      .call(d3.axisBottom(xScale))
      .append('text')
      .attr('x', width / 2)
      .attr('y', 35)
      .attr('fill', '#000')
      .attr('text-anchor', 'middle')
      .text(config.xAxis?.label || '');

    // Y axis
    g.append('g')
      .call(d3.axisLeft(yScale))
      .append('text')
      .attr('transform', 'rotate(-90)')
      .attr('y', -40)
      .attr('x', -height / 2)
      .attr('fill', '#000')
      .attr('text-anchor', 'middle')
      .text(config.yAxis?.label || '');
  };

  const addAnnotations = (
    g: d3.Selection<SVGGElement, unknown, null, undefined>,
    annotations: NonNullable<VisualizationConfig['annotations']>,
    xScale: any,
    yScale: any,
    height: number
  ) => {
    for (const annotation of annotations) {
      if (annotation.type === 'line' && annotation.axis === 'y' && annotation.value !== undefined) {
        const y = yScale(annotation.value);

        g.append('line')
          .attr('x1', 0)
          .attr('x2', xScale.range()[1])
          .attr('y1', y)
          .attr('y2', y)
          .attr('stroke', annotation.style?.color || '#e74c3c')
          .attr('stroke-width', annotation.style?.strokeWidth || 2)
          .attr('stroke-dasharray', annotation.style?.strokeDash?.join(',') || '5,5');

        if (annotation.label) {
          g.append('text')
            .attr('x', 5)
            .attr('y', y - 5)
            .text(annotation.label)
            .style('font-size', '12px')
            .style('fill', annotation.style?.color || '#e74c3c');
        }
      }
    }
  };

  return (
    <div style={{ position: 'relative' }}>
      <svg ref={svgRef} width={width} height={height} />
      {tooltip && config.tooltip?.show && (
        <div
          style={{
            position: 'absolute',
            left: tooltip.x,
            top: tooltip.y,
            backgroundColor: 'white',
            border: '1px solid #ddd',
            padding: '8px',
            borderRadius: '4px',
            boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
            pointerEvents: 'none',
            fontSize: '12px',
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
