/**
 * Multi-Series Line Chart Component with D3.js
 * Supports multiple series, areas, curves, and animations
 */

import React, { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import {
  TimeSeriesData,
  LineChartConfig,
  BaseChartProps,
  ChartEvent,
} from '../types';

interface LineChartProps extends BaseChartProps<TimeSeriesData[], LineChartConfig> {
  multiSeries?: boolean;
  seriesData?: { [key: string]: TimeSeriesData[] };
}

export const LineChart: React.FC<LineChartProps> = ({
  data,
  config,
  className = '',
  style,
  onEvent,
  multiSeries = false,
  seriesData,
}) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const tooltipRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!svgRef.current || (!data.length && !seriesData)) return;

    const {
      width,
      height,
      margin = { top: 20, right: 30, bottom: 40, left: 50 },
    } = config.dimensions;

    const {
      curve = 'monotone',
      showPoints = true,
      showArea = false,
      showGrid = true,
      theme = {},
      animation = { duration: 1000, enabled: true },
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
      .attr('aria-label', config.accessibility?.ariaLabel || 'Line chart visualization');

    const g = svg
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Color scheme
    const colorScheme = theme.colorScheme || d3.schemeCategory10;
    const colorScale = d3.scaleOrdinal<string>().range(colorScheme);

    // Curve type mapping
    const curveMap: { [key: string]: d3.CurveFactory } = {
      linear: d3.curveLinear,
      monotone: d3.curveMonotoneX,
      step: d3.curveStep,
      basis: d3.curveBasis,
      cardinal: d3.curveCardinal,
    };

    const curveFunction = curveMap[curve] || d3.curveMonotoneX;

    if (multiSeries && seriesData) {
      renderMultiSeriesLineChart(
        g,
        seriesData,
        innerWidth,
        innerHeight,
        curveFunction,
        showPoints,
        showArea,
        showGrid,
        colorScale,
        animation,
        onEvent
      );
    } else {
      renderSingleSeriesLineChart(
        g,
        data,
        innerWidth,
        innerHeight,
        curveFunction,
        showPoints,
        showArea,
        showGrid,
        colorScale,
        animation,
        onEvent
      );
    }
  }, [data, config, multiSeries, seriesData, onEvent]);

  const renderSingleSeriesLineChart = (
    g: d3.Selection<SVGGElement, unknown, null, undefined>,
    data: TimeSeriesData[],
    width: number,
    height: number,
    curveFunction: d3.CurveFactory,
    showPoints: boolean,
    showArea: boolean,
    showGrid: boolean,
    colorScale: d3.ScaleOrdinal<string, string>,
    animation: { duration: number; enabled: boolean },
    onEvent?: (event: ChartEvent<TimeSeriesData>) => void
  ) => {
    // Parse dates
    const parseTime = d3.timeParse('%Y-%m-%d');
    const parsedData = data.map((d) => ({
      ...d,
      timestamp:
        typeof d.timestamp === 'string' ? parseTime(d.timestamp) || new Date() : d.timestamp,
    }));

    // Scales
    const xScale = d3
      .scaleTime()
      .domain(d3.extent(parsedData, (d) => d.timestamp) as [Date, Date])
      .range([0, width]);

    const yScale = d3
      .scaleLinear()
      .domain([0, d3.max(parsedData, (d) => d.value) || 0])
      .range([height, 0])
      .nice();

    // Axes
    g.append('g')
      .attr('class', 'x-axis')
      .attr('transform', `translate(0,${height})`)
      .call(d3.axisBottom(xScale));

    g.append('g').attr('class', 'y-axis').call(d3.axisLeft(yScale));

    // Grid
    if (showGrid) {
      g.append('g')
        .attr('class', 'grid')
        .attr('opacity', 0.1)
        .call(d3.axisLeft(yScale).tickSize(-width).tickFormat(() => ''));

      g.append('g')
        .attr('class', 'grid')
        .attr('opacity', 0.1)
        .attr('transform', `translate(0,${height})`)
        .call(d3.axisBottom(xScale).tickSize(-height).tickFormat(() => ''));
    }

    // Area
    if (showArea) {
      const area = d3
        .area<TimeSeriesData>()
        .x((d) => xScale(d.timestamp as Date))
        .y0(height)
        .y1((d) => yScale(d.value))
        .curve(curveFunction);

      const areaPath = g
        .append('path')
        .datum(parsedData)
        .attr('class', 'area')
        .attr('fill', colorScale('0'))
        .attr('opacity', 0.3)
        .attr('d', area);

      if (animation.enabled) {
        const totalLength = areaPath.node()?.getTotalLength() || 0;
        areaPath
          .attr('stroke-dasharray', `${totalLength} ${totalLength}`)
          .attr('stroke-dashoffset', totalLength)
          .transition()
          .duration(animation.duration)
          .attr('stroke-dashoffset', 0);
      }
    }

    // Line
    const line = d3
      .line<TimeSeriesData>()
      .x((d) => xScale(d.timestamp as Date))
      .y((d) => yScale(d.value))
      .curve(curveFunction);

    const linePath = g
      .append('path')
      .datum(parsedData)
      .attr('class', 'line')
      .attr('fill', 'none')
      .attr('stroke', colorScale('0'))
      .attr('stroke-width', 2)
      .attr('d', line);

    if (animation.enabled) {
      const totalLength = linePath.node()?.getTotalLength() || 0;
      linePath
        .attr('stroke-dasharray', `${totalLength} ${totalLength}`)
        .attr('stroke-dashoffset', totalLength)
        .transition()
        .duration(animation.duration)
        .attr('stroke-dashoffset', 0);
    }

    // Points
    if (showPoints) {
      const points = g
        .selectAll('.point')
        .data(parsedData)
        .enter()
        .append('circle')
        .attr('class', 'point')
        .attr('cx', (d) => xScale(d.timestamp as Date))
        .attr('cy', (d) => yScale(d.value))
        .attr('r', 0)
        .attr('fill', colorScale('0'))
        .attr('stroke', '#fff')
        .attr('stroke-width', 2);

      if (animation.enabled) {
        points
          .transition()
          .duration(animation.duration)
          .delay((d, i) => i * 20)
          .attr('r', 4);
      } else {
        points.attr('r', 4);
      }

      if (onEvent) {
        points
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
            d3.select(this).attr('r', 6);
            onEvent({
              type: 'hover',
              data: d,
              position: { x: event.clientX, y: event.clientY },
              target: event.target,
              originalEvent: event,
            });
          })
          .on('mouseleave', function () {
            d3.select(this).attr('r', 4);
          });
      }
    }
  };

  const renderMultiSeriesLineChart = (
    g: d3.Selection<SVGGElement, unknown, null, undefined>,
    seriesData: { [key: string]: TimeSeriesData[] },
    width: number,
    height: number,
    curveFunction: d3.CurveFactory,
    showPoints: boolean,
    showArea: boolean,
    showGrid: boolean,
    colorScale: d3.ScaleOrdinal<string, string>,
    animation: { duration: number; enabled: boolean },
    onEvent?: (event: ChartEvent<TimeSeriesData>) => void
  ) => {
    const series = Object.keys(seriesData);
    const parseTime = d3.timeParse('%Y-%m-%d');

    // Parse all data
    const allData = series.flatMap((s) =>
      (seriesData[s] || []).map((d) => ({
        ...d,
        timestamp:
          typeof d.timestamp === 'string' ? parseTime(d.timestamp) || new Date() : d.timestamp,
      }))
    );

    // Scales
    const xScale = d3
      .scaleTime()
      .domain(d3.extent(allData, (d) => d.timestamp) as [Date, Date])
      .range([0, width]);

    const yScale = d3
      .scaleLinear()
      .domain([0, d3.max(allData, (d) => d.value) || 0])
      .range([height, 0])
      .nice();

    // Axes
    g.append('g')
      .attr('class', 'x-axis')
      .attr('transform', `translate(0,${height})`)
      .call(d3.axisBottom(xScale));

    g.append('g').attr('class', 'y-axis').call(d3.axisLeft(yScale));

    // Grid
    if (showGrid) {
      g.append('g')
        .attr('class', 'grid')
        .attr('opacity', 0.1)
        .call(d3.axisLeft(yScale).tickSize(-width).tickFormat(() => ''));
    }

    // Render each series
    series.forEach((seriesName, index) => {
      const data = (seriesData[seriesName] || []).map((d) => ({
        ...d,
        timestamp:
          typeof d.timestamp === 'string' ? parseTime(d.timestamp) || new Date() : d.timestamp,
      }));

      const color = colorScale(seriesName);

      // Area
      if (showArea) {
        const area = d3
          .area<TimeSeriesData>()
          .x((d) => xScale(d.timestamp as Date))
          .y0(height)
          .y1((d) => yScale(d.value))
          .curve(curveFunction);

        g.append('path')
          .datum(data)
          .attr('class', `area-${index}`)
          .attr('fill', color)
          .attr('opacity', 0.2)
          .attr('d', area);
      }

      // Line
      const line = d3
        .line<TimeSeriesData>()
        .x((d) => xScale(d.timestamp as Date))
        .y((d) => yScale(d.value))
        .curve(curveFunction);

      const linePath = g
        .append('path')
        .datum(data)
        .attr('class', `line-${index}`)
        .attr('fill', 'none')
        .attr('stroke', color)
        .attr('stroke-width', 2)
        .attr('d', line);

      if (animation.enabled) {
        const totalLength = linePath.node()?.getTotalLength() || 0;
        linePath
          .attr('stroke-dasharray', `${totalLength} ${totalLength}`)
          .attr('stroke-dashoffset', totalLength)
          .transition()
          .duration(animation.duration)
          .delay(index * 200)
          .attr('stroke-dashoffset', 0);
      }

      // Points
      if (showPoints) {
        g.selectAll(`.point-${index}`)
          .data(data)
          .enter()
          .append('circle')
          .attr('class', `point-${index}`)
          .attr('cx', (d) => xScale(d.timestamp as Date))
          .attr('cy', (d) => yScale(d.value))
          .attr('r', animation.enabled ? 0 : 4)
          .attr('fill', color)
          .attr('stroke', '#fff')
          .attr('stroke-width', 2)
          .transition()
          .duration(animation.enabled ? animation.duration : 0)
          .delay((d, i) => index * 200 + i * 20)
          .attr('r', 4);
      }
    });

    // Legend
    const legend = g
      .append('g')
      .attr('class', 'legend')
      .attr('transform', `translate(${width - 100}, 0)`);

    series.forEach((seriesName, i) => {
      const legendRow = legend
        .append('g')
        .attr('transform', `translate(0, ${i * 20})`);

      legendRow
        .append('rect')
        .attr('width', 10)
        .attr('height', 10)
        .attr('fill', colorScale(seriesName));

      legendRow
        .append('text')
        .attr('x', 15)
        .attr('y', 10)
        .attr('text-anchor', 'start')
        .style('font-size', '12px')
        .text(seriesName);
    });
  };

  return (
    <div className={`line-chart ${className}`} style={style}>
      <svg ref={svgRef} />
      <div ref={tooltipRef} className="tooltip" style={{ display: 'none' }} />
    </div>
  );
};

export default LineChart;
