/**
 * Heat Map Visualization Component
 * Supports color scales, tooltips, and interactive cells
 */

import React, { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { HeatMapConfig, BaseChartProps, ChartEvent } from '../types';

interface HeatMapDataPoint {
  row: string;
  col: string;
  value: number;
}

interface HeatMapProps extends BaseChartProps<HeatMapDataPoint[], HeatMapConfig> {}

export const HeatMap: React.FC<HeatMapProps> = ({
  data,
  config,
  className = '',
  style,
  onEvent,
}) => {
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!svgRef.current || !data.length) return;

    const {
      width,
      height,
      margin = { top: 60, right: 100, bottom: 60, left: 100 },
    } = config.dimensions;

    const {
      colorScale: colorScheme = ['#f7fbff', '#08519c'],
      cellPadding = 2,
      showValues = true,
      showLegend = true,
      theme = {},
      animation = { duration: 500, enabled: true },
    } = config;

    const innerWidth = width - margin.left - margin.right;
    const innerHeight = height - margin.top - margin.bottom;

    // Clear previous content
    d3.select(svgRef.current).selectAll('*').remove();

    const svg = d3
      .select(svgRef.current)
      .attr('width', width)
      .attr('height', height)
      .attr('role', 'img')
      .attr('aria-label', config.accessibility?.ariaLabel || 'Heat map visualization');

    const g = svg
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Get unique rows and columns
    const rows = Array.from(new Set(data.map((d) => d.row)));
    const cols = Array.from(new Set(data.map((d) => d.col)));

    // Scales
    const xScale = d3.scaleBand().domain(cols).range([0, innerWidth]).padding(0.05);

    const yScale = d3.scaleBand().domain(rows).range([0, innerHeight]).padding(0.05);

    const [minValue, maxValue] = d3.extent(data, (d) => d.value) as [number, number];

    const colorScaleFunc = d3
      .scaleSequential()
      .domain([minValue, maxValue])
      .interpolator(
        d3.interpolateRgbBasis(colorScheme.length > 2 ? colorScheme : ['#f7fbff', '#08519c'])
      );

    // Axes
    const xAxis = g
      .append('g')
      .attr('class', 'x-axis')
      .attr('transform', `translate(0, -5)`)
      .selectAll('text')
      .data(cols)
      .enter()
      .append('text')
      .attr('x', (d) => (xScale(d) || 0) + xScale.bandwidth() / 2)
      .attr('y', 0)
      .attr('text-anchor', 'end')
      .attr('transform', (d) => {
        const x = (xScale(d) || 0) + xScale.bandwidth() / 2;
        return `rotate(-45, ${x}, 0)`;
      })
      .style('font-size', '11px')
      .text((d) => d);

    const yAxis = g
      .append('g')
      .attr('class', 'y-axis')
      .selectAll('text')
      .data(rows)
      .enter()
      .append('text')
      .attr('x', -5)
      .attr('y', (d) => (yScale(d) || 0) + yScale.bandwidth() / 2)
      .attr('text-anchor', 'end')
      .attr('dominant-baseline', 'middle')
      .style('font-size', '11px')
      .text((d) => d);

    // Cells
    const cells = g
      .selectAll('.cell')
      .data(data)
      .enter()
      .append('g')
      .attr('class', 'cell')
      .attr('transform', (d) => `translate(${xScale(d.col) || 0},${yScale(d.row) || 0})`);

    const rects = cells
      .append('rect')
      .attr('width', xScale.bandwidth())
      .attr('height', yScale.bandwidth())
      .attr('rx', 2)
      .attr('ry', 2)
      .attr('fill', (d) => colorScaleFunc(d.value))
      .attr('stroke', theme.backgroundColor || '#fff')
      .attr('stroke-width', cellPadding)
      .style('cursor', 'pointer');

    if (animation.enabled) {
      rects.attr('opacity', 0).transition().duration(animation.duration).attr('opacity', 1);
    }

    // Cell values
    if (showValues) {
      const texts = cells
        .append('text')
        .attr('x', xScale.bandwidth() / 2)
        .attr('y', yScale.bandwidth() / 2)
        .attr('text-anchor', 'middle')
        .attr('dominant-baseline', 'middle')
        .style('font-size', '10px')
        .style('font-weight', 'bold')
        .style('fill', (d) => {
          // Choose text color based on cell background brightness
          const rgb = d3.rgb(colorScaleFunc(d.value));
          const brightness = (rgb.r * 299 + rgb.g * 587 + rgb.b * 114) / 1000;
          return brightness > 128 ? '#000' : '#fff';
        })
        .style('pointer-events', 'none')
        .text((d) => {
          if (d.value >= 1000) {
            return (d.value / 1000).toFixed(1) + 'K';
          } else if (d.value < 1) {
            return d.value.toFixed(2);
          }
          return d.value.toFixed(0);
        });

      if (animation.enabled) {
        texts.attr('opacity', 0).transition().duration(animation.duration).delay(200).attr('opacity', 1);
      }
    }

    // Interaction
    rects
      .on('mouseenter', function (event, d) {
        d3.select(this).attr('stroke-width', 3).attr('stroke', '#333');

        if (onEvent) {
          onEvent({
            type: 'hover',
            data: d,
            position: { x: event.clientX, y: event.clientY },
            target: event.target,
            originalEvent: event,
          });
        }
      })
      .on('mouseleave', function (event, d) {
        d3.select(this).attr('stroke-width', cellPadding).attr('stroke', theme.backgroundColor || '#fff');
      })
      .on('click', function (event, d) {
        if (onEvent) {
          onEvent({
            type: 'click',
            data: d,
            position: { x: event.clientX, y: event.clientY },
            target: event.target,
            originalEvent: event,
          });
        }
      });

    // Legend
    if (showLegend) {
      const legendWidth = 20;
      const legendHeight = innerHeight;

      const legendScale = d3
        .scaleLinear()
        .domain([minValue, maxValue])
        .range([legendHeight, 0]);

      const legendAxis = d3.axisRight(legendScale).ticks(5);

      const legend = svg
        .append('g')
        .attr('class', 'legend')
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

      const numStops = 10;
      for (let i = 0; i <= numStops; i++) {
        const offset = (i / numStops) * 100;
        const value = minValue + (maxValue - minValue) * (i / numStops);
        gradient
          .append('stop')
          .attr('offset', `${offset}%`)
          .attr('stop-color', colorScaleFunc(value));
      }

      legend
        .append('rect')
        .attr('width', legendWidth)
        .attr('height', legendHeight)
        .style('fill', 'url(#heatmap-gradient)');

      legend.append('g').attr('transform', `translate(${legendWidth}, 0)`).call(legendAxis);

      legend
        .append('text')
        .attr('transform', 'rotate(-90)')
        .attr('x', -legendHeight / 2)
        .attr('y', -30)
        .attr('text-anchor', 'middle')
        .style('font-size', '12px')
        .text('Value');
    }
  }, [data, config, onEvent]);

  return (
    <div className={`heat-map ${className}`} style={style}>
      <svg ref={svgRef} />
    </div>
  );
};

export default HeatMap;
