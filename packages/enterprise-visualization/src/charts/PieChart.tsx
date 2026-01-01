/**
 * Interactive Pie/Donut Chart Component with D3.js
 * Supports animations, labels, and interactive selections
 */

import React, { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { DataPoint, PieChartConfig, BaseChartProps, ChartEvent } from '../types';

interface PieChartProps extends BaseChartProps<DataPoint[], PieChartConfig> {}

export const PieChart: React.FC<PieChartProps> = ({
  data,
  config,
  className = '',
  style,
  onEvent,
}) => {
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!svgRef.current || !data.length) return;

    const { width, height } = config.dimensions;

    const {
      innerRadius = 0,
      outerRadius = Math.min(width, height) / 2 - 20,
      padAngle = 0.02,
      startAngle = 0,
      endAngle = 2 * Math.PI,
      showLabels = true,
      theme = {},
      animation = { duration: 750, enabled: true },
    } = config;

    const radius = outerRadius;
    const center = { x: width / 2, y: height / 2 };

    // Clear previous content
    d3.select(svgRef.current).selectAll('*').remove();

    const svg = d3
      .select(svgRef.current)
      .attr('width', width)
      .attr('height', height)
      .attr('role', 'img')
      .attr('aria-label', config.accessibility?.ariaLabel || 'Pie chart visualization');

    const g = svg.append('g').attr('transform', `translate(${center.x},${center.y})`);

    // Color scheme
    const colorScheme = theme.colorScheme || d3.schemeCategory10;
    const color = d3
      .scaleOrdinal<string>()
      .domain(data.map((d, i) => d.label || `Item ${i}`))
      .range(colorScheme);

    // Pie layout
    const pie = d3
      .pie<DataPoint>()
      .value((d) => d.value)
      .startAngle(startAngle)
      .endAngle(endAngle)
      .padAngle(padAngle)
      .sort(null);

    // Arc generator
    const arc = d3
      .arc<d3.PieArcDatum<DataPoint>>()
      .innerRadius(innerRadius)
      .outerRadius(radius);

    // Arc for labels (positioned at centroid)
    const labelArc = d3
      .arc<d3.PieArcDatum<DataPoint>>()
      .innerRadius(radius * 0.6)
      .outerRadius(radius * 0.6);

    // Hover arc (slightly larger)
    const hoverArc = d3
      .arc<d3.PieArcDatum<DataPoint>>()
      .innerRadius(innerRadius)
      .outerRadius(radius + 10);

    const arcs = pie(data);

    // Draw slices
    const slices = g
      .selectAll('.slice')
      .data(arcs)
      .enter()
      .append('g')
      .attr('class', 'slice');

    const paths = slices
      .append('path')
      .attr('fill', (d, i) => color(d.data.label || `Item ${i}`))
      .attr('stroke', '#fff')
      .attr('stroke-width', 2)
      .style('cursor', 'pointer');

    if (animation.enabled) {
      // Animate from center
      paths
        .attr('d', (d) => {
          const arcCopy = { ...d, startAngle: 0, endAngle: 0 };
          return arc(arcCopy);
        })
        .transition()
        .duration(animation.duration)
        .attrTween('d', function (d) {
          const interpolateStart = d3.interpolate(0, d.startAngle);
          const interpolateEnd = d3.interpolate(0, d.endAngle);
          return function (t) {
            const arcData = { ...d };
            arcData.startAngle = interpolateStart(t);
            arcData.endAngle = interpolateEnd(t);
            return arc(arcData) || '';
          };
        });
    } else {
      paths.attr('d', arc);
    }

    // Interaction
    paths
      .on('mouseenter', function (event, d) {
        d3.select(this)
          .transition()
          .duration(200)
          .attr('d', hoverArc(d) || '');

        if (onEvent) {
          onEvent({
            type: 'hover',
            data: d.data,
            position: { x: event.clientX, y: event.clientY },
            target: event.target,
            originalEvent: event,
          });
        }
      })
      .on('mouseleave', function (event, d) {
        d3.select(this)
          .transition()
          .duration(200)
          .attr('d', arc(d) || '');
      })
      .on('click', function (event, d) {
        if (onEvent) {
          onEvent({
            type: 'click',
            data: d.data,
            position: { x: event.clientX, y: event.clientY },
            target: event.target,
            originalEvent: event,
          });
        }
      });

    // Labels
    if (showLabels) {
      const labels = slices
        .append('text')
        .attr('transform', (d) => {
          const [x, y] = labelArc.centroid(d);
          return `translate(${x},${y})`;
        })
        .attr('text-anchor', 'middle')
        .attr('opacity', 0)
        .style('font-size', '12px')
        .style('font-weight', 'bold')
        .style('fill', '#fff')
        .style('pointer-events', 'none');

      labels
        .append('tspan')
        .attr('x', 0)
        .attr('dy', '-0.2em')
        .text((d, i) => d.data.label || `Item ${i}`);

      labels
        .append('tspan')
        .attr('x', 0)
        .attr('dy', '1.2em')
        .style('font-size', '10px')
        .style('font-weight', 'normal')
        .text((d) => {
          const total = d3.sum(data, (item) => item.value);
          const percentage = ((d.data.value / total) * 100).toFixed(1);
          return `${percentage}%`;
        });

      if (animation.enabled) {
        labels.transition().duration(animation.duration).delay(300).attr('opacity', 1);
      } else {
        labels.attr('opacity', 1);
      }
    }

    // Center label for donut charts
    if (innerRadius > 0) {
      const total = d3.sum(data, (d) => d.value);

      const centerLabel = g.append('g').attr('class', 'center-label');

      centerLabel
        .append('text')
        .attr('text-anchor', 'middle')
        .attr('dy', '-0.5em')
        .style('font-size', '24px')
        .style('font-weight', 'bold')
        .text(total.toLocaleString());

      centerLabel
        .append('text')
        .attr('text-anchor', 'middle')
        .attr('dy', '1em')
        .style('font-size', '14px')
        .style('fill', '#666')
        .text('Total');
    }

    // Legend
    const legendWidth = 120;
    const legendItemHeight = 20;
    const legend = svg
      .append('g')
      .attr('class', 'legend')
      .attr('transform', `translate(${width - legendWidth - 10}, 20)`);

    const legendItems = legend
      .selectAll('.legend-item')
      .data(arcs)
      .enter()
      .append('g')
      .attr('class', 'legend-item')
      .attr('transform', (d, i) => `translate(0, ${i * legendItemHeight})`);

    legendItems
      .append('rect')
      .attr('width', 12)
      .attr('height', 12)
      .attr('fill', (d, i) => color(d.data.label || `Item ${i}`));

    legendItems
      .append('text')
      .attr('x', 18)
      .attr('y', 10)
      .style('font-size', '11px')
      .text((d, i) => {
        const label = d.data.label || `Item ${i}`;
        return label.length > 15 ? label.substring(0, 15) + '...' : label;
      });

    legendItems
      .append('text')
      .attr('x', legendWidth)
      .attr('y', 10)
      .attr('text-anchor', 'end')
      .style('font-size', '11px')
      .style('font-weight', 'bold')
      .text((d) => d.data.value.toLocaleString());
  }, [data, config, onEvent]);

  return (
    <div className={`pie-chart ${className}`} style={style}>
      <svg ref={svgRef} />
    </div>
  );
};

export default PieChart;
