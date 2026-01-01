/**
 * Advanced Bar Chart Component with D3.js
 * Supports vertical/horizontal, grouped, and stacked bars with animations
 */

import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import {
  DataPoint,
  BarChartConfig,
  BaseChartProps,
  ChartEvent,
} from '../types';

interface BarChartProps extends BaseChartProps<DataPoint[], BarChartConfig> {
  multiSeries?: boolean;
  seriesData?: { [key: string]: DataPoint[] };
}

export const BarChart: React.FC<BarChartProps> = ({
  data,
  config,
  className = '',
  style,
  onEvent,
  multiSeries = false,
  seriesData,
}) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState(config.dimensions);

  useEffect(() => {
    if (!svgRef.current || !data.length) return;

    const {
      width,
      height,
      margin = { top: 20, right: 30, bottom: 40, left: 50 },
    } = dimensions;

    const {
      orientation = 'vertical',
      barPadding = 0.1,
      grouped = false,
      stacked = false,
      theme = {},
      animation = { duration: 750, enabled: true },
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
      .attr('aria-label', config.accessibility?.ariaLabel || 'Bar chart visualization');

    const g = svg
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Color scheme
    const colorScheme = theme.colorScheme || d3.schemeCategory10;
    const colorScale = d3.scaleOrdinal<string>().range(colorScheme);

    if (multiSeries && seriesData) {
      renderMultiSeriesBarChart(
        g,
        seriesData,
        innerWidth,
        innerHeight,
        orientation,
        barPadding,
        grouped,
        stacked,
        colorScale,
        animation
      );
    } else {
      renderSingleSeriesBarChart(
        g,
        data,
        innerWidth,
        innerHeight,
        orientation,
        barPadding,
        colorScale,
        animation,
        onEvent
      );
    }
  }, [data, config, dimensions, multiSeries, seriesData, onEvent]);

  const renderSingleSeriesBarChart = (
    g: d3.Selection<SVGGElement, unknown, null, undefined>,
    data: DataPoint[],
    width: number,
    height: number,
    orientation: 'vertical' | 'horizontal',
    barPadding: number,
    colorScale: d3.ScaleOrdinal<string, string>,
    animation: { duration: number; enabled: boolean },
    onEvent?: (event: ChartEvent<DataPoint>) => void
  ) => {
    const isVertical = orientation === 'vertical';

    // Scales
    const xScale = isVertical
      ? d3
          .scaleBand()
          .domain(data.map((d, i) => d.label || `Item ${i}`))
          .range([0, width])
          .padding(barPadding)
      : d3
          .scaleLinear()
          .domain([0, d3.max(data, (d) => d.value) || 0])
          .range([0, width]);

    const yScale = isVertical
      ? d3
          .scaleLinear()
          .domain([0, d3.max(data, (d) => d.value) || 0])
          .range([height, 0])
      : d3
          .scaleBand()
          .domain(data.map((d, i) => d.label || `Item ${i}`))
          .range([0, height])
          .padding(barPadding);

    // Axes
    const xAxis = isVertical
      ? d3.axisBottom(xScale as d3.ScaleBand<string>)
      : d3.axisBottom(xScale as d3.ScaleLinear<number, number>);

    const yAxis = isVertical
      ? d3.axisLeft(yScale as d3.ScaleLinear<number, number>)
      : d3.axisLeft(yScale as d3.ScaleBand<string>);

    g.append('g')
      .attr('class', 'x-axis')
      .attr('transform', `translate(0,${height})`)
      .call(xAxis)
      .selectAll('text')
      .attr('transform', isVertical ? 'rotate(-45)' : '')
      .style('text-anchor', isVertical ? 'end' : 'start');

    g.append('g').attr('class', 'y-axis').call(yAxis);

    // Grid lines
    g.append('g')
      .attr('class', 'grid')
      .attr('opacity', 0.1)
      .call(
        d3
          .axisLeft(yScale as d3.ScaleLinear<number, number>)
          .tickSize(-width)
          .tickFormat(() => '')
      );

    // Bars
    const bars = g
      .selectAll('.bar')
      .data(data)
      .enter()
      .append('rect')
      .attr('class', 'bar')
      .attr('fill', (d, i) => colorScale(d.category || `${i}`))
      .attr('stroke', '#fff')
      .attr('stroke-width', 1);

    if (isVertical) {
      bars
        .attr('x', (d, i) => (xScale as d3.ScaleBand<string>)(d.label || `Item ${i}`) || 0)
        .attr('width', (xScale as d3.ScaleBand<string>).bandwidth())
        .attr('y', height)
        .attr('height', 0);

      if (animation.enabled) {
        bars
          .transition()
          .duration(animation.duration)
          .delay((d, i) => i * 50)
          .attr('y', (d) => (yScale as d3.ScaleLinear<number, number>)(d.value))
          .attr('height', (d) => height - (yScale as d3.ScaleLinear<number, number>)(d.value));
      } else {
        bars
          .attr('y', (d) => (yScale as d3.ScaleLinear<number, number>)(d.value))
          .attr('height', (d) => height - (yScale as d3.ScaleLinear<number, number>)(d.value));
      }
    } else {
      bars
        .attr('y', (d, i) => (yScale as d3.ScaleBand<string>)(d.label || `Item ${i}`) || 0)
        .attr('height', (yScale as d3.ScaleBand<string>).bandwidth())
        .attr('x', 0)
        .attr('width', 0);

      if (animation.enabled) {
        bars
          .transition()
          .duration(animation.duration)
          .delay((d, i) => i * 50)
          .attr('width', (d) => (xScale as d3.ScaleLinear<number, number>)(d.value));
      } else {
        bars.attr('width', (d) => (xScale as d3.ScaleLinear<number, number>)(d.value));
      }
    }

    // Interaction
    if (onEvent) {
      bars
        .on('click', function (event, d) {
          onEvent({
            type: 'click',
            data: d,
            position: { x: event.clientX, y: event.clientY },
            target: event.target,
            originalEvent: event,
          });
        })
        .on('mouseenter', function (event, d) {
          d3.select(this).attr('opacity', 0.7);
          onEvent({
            type: 'hover',
            data: d,
            position: { x: event.clientX, y: event.clientY },
            target: event.target,
            originalEvent: event,
          });
        })
        .on('mouseleave', function () {
          d3.select(this).attr('opacity', 1);
        });
    }
  };

  const renderMultiSeriesBarChart = (
    g: d3.Selection<SVGGElement, unknown, null, undefined>,
    seriesData: { [key: string]: DataPoint[] },
    width: number,
    height: number,
    orientation: 'vertical' | 'horizontal',
    barPadding: number,
    grouped: boolean,
    stacked: boolean,
    colorScale: d3.ScaleOrdinal<string, string>,
    animation: { duration: number; enabled: boolean }
  ) => {
    const series = Object.keys(seriesData);
    const allLabels = Array.from(
      new Set(
        series.flatMap((s) =>
          seriesData[s]?.map((d, i) => d.label || `Item ${i}`) || []
        )
      )
    );

    if (grouped) {
      // Grouped bar chart implementation
      const x0Scale = d3
        .scaleBand()
        .domain(allLabels)
        .range([0, width])
        .padding(barPadding);

      const x1Scale = d3
        .scaleBand()
        .domain(series)
        .range([0, x0Scale.bandwidth()])
        .padding(0.05);

      const yScale = d3
        .scaleLinear()
        .domain([
          0,
          d3.max(series, (s) => d3.max(seriesData[s] || [], (d) => d.value) || 0) || 0,
        ])
        .range([height, 0]);

      // Axes
      g.append('g')
        .attr('transform', `translate(0,${height})`)
        .call(d3.axisBottom(x0Scale));

      g.append('g').call(d3.axisLeft(yScale));

      // Grouped bars
      const groups = g
        .selectAll('.group')
        .data(allLabels)
        .enter()
        .append('g')
        .attr('class', 'group')
        .attr('transform', (d) => `translate(${x0Scale(d) || 0},0)`);

      series.forEach((seriesName) => {
        const seriesDataArray = seriesData[seriesName] || [];
        groups
          .selectAll(`.bar-${seriesName}`)
          .data((label) => {
            const dataPoint = seriesDataArray.find((d, i) => (d.label || `Item ${i}`) === label);
            return dataPoint ? [{ ...dataPoint, series: seriesName }] : [];
          })
          .enter()
          .append('rect')
          .attr('class', `bar-${seriesName}`)
          .attr('x', () => x1Scale(seriesName) || 0)
          .attr('width', x1Scale.bandwidth())
          .attr('fill', colorScale(seriesName))
          .attr('y', height)
          .attr('height', 0)
          .transition()
          .duration(animation.enabled ? animation.duration : 0)
          .attr('y', (d) => yScale(d.value))
          .attr('height', (d) => height - yScale(d.value));
      });
    } else if (stacked) {
      // Stacked bar chart implementation
      const stack = d3.stack<{ [key: string]: number }>().keys(series);

      const stackData = allLabels.map((label) => {
        const entry: { [key: string]: number } = { label };
        series.forEach((s) => {
          const dataPoint = seriesData[s]?.find((d, i) => (d.label || `Item ${i}`) === label);
          entry[s] = dataPoint?.value || 0;
        });
        return entry;
      });

      const stackedData = stack(stackData);

      const xScale = d3.scaleBand().domain(allLabels).range([0, width]).padding(barPadding);

      const yScale = d3
        .scaleLinear()
        .domain([0, d3.max(stackedData, (d) => d3.max(d, (d) => d[1]) || 0) || 0])
        .range([height, 0]);

      g.append('g')
        .attr('transform', `translate(0,${height})`)
        .call(d3.axisBottom(xScale));

      g.append('g').call(d3.axisLeft(yScale));

      stackedData.forEach((seriesData, i) => {
        g.selectAll(`.bar-stack-${i}`)
          .data(seriesData)
          .enter()
          .append('rect')
          .attr('class', `bar-stack-${i}`)
          .attr('x', (d) => xScale((d.data as { label: string }).label) || 0)
          .attr('width', xScale.bandwidth())
          .attr('fill', colorScale(series[i] || ''))
          .attr('y', height)
          .attr('height', 0)
          .transition()
          .duration(animation.enabled ? animation.duration : 0)
          .attr('y', (d) => yScale(d[1]))
          .attr('height', (d) => yScale(d[0]) - yScale(d[1]));
      });
    }
  };

  return (
    <div ref={containerRef} className={`bar-chart ${className}`} style={style}>
      <svg ref={svgRef} />
    </div>
  );
};

export default BarChart;
