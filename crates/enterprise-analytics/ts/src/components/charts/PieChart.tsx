/**
 * Pie Chart Component - Proportional Data Visualization
 * @module @harborgrid/enterprise-analytics/components/charts
 */

import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import type { VisualizationConfig } from '../../types';

export interface PieChartProps<T = Record<string, unknown>> {
  data: T[];
  config: VisualizationConfig;
  width?: number;
  height?: number;
  onSliceClick?: (data: T) => void;
  onSliceHover?: (data: T) => void;
}

export function PieChart<T = Record<string, unknown>>({
  data,
  config,
  width = 400,
  height = 400,
  onSliceClick,
  onSliceHover,
}: PieChartProps<T>) {
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

    const radius = Math.min(width, height) / 2 - 40;
    const innerRadius = config.type === 'donut_chart' ? radius * 0.6 : 0;

    const g = svg
      .append('g')
      .attr('transform', `translate(${width / 2},${height / 2})`);

    // Create pie layout
    const pie = d3
      .pie<T>()
      .value((d) => (d as Record<string, unknown>)[config.yAxis!.field] as number)
      .sort(null);

    // Create arc generator
    const arc = d3
      .arc<d3.PieArcDatum<T>>()
      .innerRadius(innerRadius)
      .outerRadius(radius);

    // Create arc for hover effect
    const arcHover = d3
      .arc<d3.PieArcDatum<T>>()
      .innerRadius(innerRadius)
      .outerRadius(radius + 10);

    // Color scale
    const colorScale = d3
      .scaleOrdinal<string>()
      .domain(data.map((d) => String((d as Record<string, unknown>)[config.xAxis!.field])))
      .range(config.theme?.colors || d3.schemeCategory10);

    // Draw slices
    const slices = g
      .selectAll('.slice')
      .data(pie(data))
      .enter()
      .append('g')
      .attr('class', 'slice');

    slices
      .append('path')
      .attr('d', arc)
      .attr('fill', (d) => colorScale(String((d.data as Record<string, unknown>)[config.xAxis!.field])))
      .attr('stroke', 'white')
      .attr('stroke-width', 2)
      .style('cursor', 'pointer')
      .style('opacity', 0)
      .on('click', (event, d) => {
        if (onSliceClick) onSliceClick(d.data);
      })
      .on('mouseenter', function (event, d) {
        d3.select(this)
          .transition()
          .duration(200)
          .attr('d', arcHover);

        const [x, y] = d3.pointer(event, svg.node());
        setTooltip({ x, y, data: d.data });
        if (onSliceHover) onSliceHover(d.data);
      })
      .on('mouseleave', function () {
        d3.select(this)
          .transition()
          .duration(200)
          .attr('d', arc);
        setTooltip(null);
      })
      .transition()
      .duration(800)
      .delay((d, i) => i * 100)
      .style('opacity', 0.9)
      .attrTween('d', function (d) {
        const interpolate = d3.interpolate({ startAngle: 0, endAngle: 0 }, d);
        return function (t) {
          return arc(interpolate(t)) || '';
        };
      });

    // Add labels
    if (config.showValues) {
      const labelArc = d3
        .arc<d3.PieArcDatum<T>>()
        .innerRadius(radius * 0.7)
        .outerRadius(radius * 0.7);

      slices
        .append('text')
        .attr('transform', (d) => `translate(${labelArc.centroid(d)})`)
        .attr('text-anchor', 'middle')
        .attr('font-size', '12px')
        .attr('fill', 'white')
        .attr('font-weight', 'bold')
        .style('opacity', 0)
        .text((d) => {
          const percentage = ((d.endAngle - d.startAngle) / (2 * Math.PI)) * 100;
          return percentage > 5 ? `${percentage.toFixed(1)}%` : '';
        })
        .transition()
        .duration(800)
        .delay((d, i) => i * 100 + 400)
        .style('opacity', 1);
    }

    // Add center text for donut charts
    if (config.type === 'donut_chart') {
      const total = d3.sum(data, (d) => (d as Record<string, unknown>)[config.yAxis!.field] as number);

      g.append('text')
        .attr('text-anchor', 'middle')
        .attr('dy', '-0.5em')
        .attr('font-size', '24px')
        .attr('font-weight', 'bold')
        .text(total.toLocaleString());

      g.append('text')
        .attr('text-anchor', 'middle')
        .attr('dy', '1em')
        .attr('font-size', '14px')
        .attr('fill', '#666')
        .text('Total');
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
            backgroundColor: 'rgba(0, 0, 0, 0.8)',
            color: 'white',
            border: '1px solid #333',
            padding: '8px 12px',
            borderRadius: '4px',
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
