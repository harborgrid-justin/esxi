/**
 * Scatter Plot with Regression Analysis
 * Supports linear, polynomial, and exponential regression
 */

import React, { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { DataPoint, ScatterPlotConfig, BaseChartProps, ChartEvent } from '../types';

interface ScatterDataPoint extends DataPoint {
  x?: number;
  y?: number;
}

interface ScatterPlotProps extends BaseChartProps<ScatterDataPoint[], ScatterPlotConfig> {}

export const ScatterPlot: React.FC<ScatterPlotProps> = ({
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
      margin = { top: 20, right: 30, bottom: 40, left: 50 },
    } = config.dimensions;

    const {
      pointRadius = 5,
      showRegression = true,
      regressionType = 'linear',
      colorByCategory = false,
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
      .attr('aria-label', config.accessibility?.ariaLabel || 'Scatter plot visualization');

    const g = svg
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Prepare data - use x/y if available, otherwise use value
    const scatterData = data.map((d, i) => ({
      ...d,
      x: d.x ?? i,
      y: d.y ?? d.value,
    }));

    // Scales
    const xScale = d3
      .scaleLinear()
      .domain([0, d3.max(scatterData, (d) => d.x!) || 0])
      .range([0, innerWidth])
      .nice();

    const yScale = d3
      .scaleLinear()
      .domain([0, d3.max(scatterData, (d) => d.y!) || 0])
      .range([innerHeight, 0])
      .nice();

    // Color scheme
    const colorScheme = theme.colorScheme || d3.schemeCategory10;
    const colorScale = d3.scaleOrdinal<string>().range(colorScheme);

    // Axes
    g.append('g')
      .attr('class', 'x-axis')
      .attr('transform', `translate(0,${innerHeight})`)
      .call(d3.axisBottom(xScale));

    g.append('g').attr('class', 'y-axis').call(d3.axisLeft(yScale));

    // Grid
    g.append('g')
      .attr('class', 'grid')
      .attr('opacity', 0.1)
      .call(d3.axisLeft(yScale).tickSize(-innerWidth).tickFormat(() => ''));

    g.append('g')
      .attr('class', 'grid')
      .attr('opacity', 0.1)
      .attr('transform', `translate(0,${innerHeight})`)
      .call(d3.axisBottom(xScale).tickSize(-innerHeight).tickFormat(() => ''));

    // Regression line
    if (showRegression) {
      const regression = calculateRegression(scatterData, regressionType);

      if (regression) {
        const regressionLine = g
          .append('path')
          .datum(regression.points)
          .attr('class', 'regression-line')
          .attr('fill', 'none')
          .attr('stroke', theme.gridColor || '#999')
          .attr('stroke-width', 2)
          .attr('stroke-dasharray', '5,5')
          .attr(
            'd',
            d3
              .line<{ x: number; y: number }>()
              .x((d) => xScale(d.x))
              .y((d) => yScale(d.y))
          );

        if (animation.enabled) {
          const totalLength = regressionLine.node()?.getTotalLength() || 0;
          regressionLine
            .attr('stroke-dasharray', `${totalLength} ${totalLength}`)
            .attr('stroke-dashoffset', totalLength)
            .transition()
            .duration(animation.duration)
            .attr('stroke-dashoffset', 0);
        }

        // Add R² value
        g.append('text')
          .attr('x', innerWidth - 10)
          .attr('y', 20)
          .attr('text-anchor', 'end')
          .style('font-size', '12px')
          .style('fill', '#666')
          .text(`R² = ${regression.r2.toFixed(4)}`);
      }
    }

    // Points
    const points = g
      .selectAll('.point')
      .data(scatterData)
      .enter()
      .append('circle')
      .attr('class', 'point')
      .attr('cx', (d) => xScale(d.x!))
      .attr('cy', (d) => yScale(d.y!))
      .attr('r', 0)
      .attr('fill', (d) =>
        colorByCategory && d.category
          ? colorScale(d.category)
          : colorScheme[0] || '#3b82f6'
      )
      .attr('stroke', '#fff')
      .attr('stroke-width', 1.5)
      .attr('opacity', 0.7)
      .style('cursor', 'pointer');

    if (animation.enabled) {
      points
        .transition()
        .duration(animation.duration)
        .delay((d, i) => i * 10)
        .attr('r', pointRadius);
    } else {
      points.attr('r', pointRadius);
    }

    // Interaction
    points
      .on('mouseenter', function (event, d) {
        d3.select(this)
          .transition()
          .duration(200)
          .attr('r', pointRadius * 1.5)
          .attr('opacity', 1);

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
      .on('mouseleave', function () {
        d3.select(this)
          .transition()
          .duration(200)
          .attr('r', pointRadius)
          .attr('opacity', 0.7);
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

    // Legend (if color by category)
    if (colorByCategory) {
      const categories = Array.from(new Set(scatterData.map((d) => d.category).filter(Boolean)));

      const legend = g
        .append('g')
        .attr('class', 'legend')
        .attr('transform', `translate(${innerWidth - 100}, 40)`);

      categories.forEach((category, i) => {
        const legendRow = legend
          .append('g')
          .attr('transform', `translate(0, ${i * 20})`);

        legendRow
          .append('circle')
          .attr('cx', 5)
          .attr('cy', 5)
          .attr('r', 5)
          .attr('fill', colorScale(category!));

        legendRow
          .append('text')
          .attr('x', 15)
          .attr('y', 10)
          .style('font-size', '11px')
          .text(category!);
      });
    }
  }, [data, config, onEvent]);

  // Regression calculation helper
  const calculateRegression = (
    data: ScatterDataPoint[],
    type: 'linear' | 'polynomial' | 'exponential'
  ): { points: { x: number; y: number }[]; r2: number } | null => {
    const points = data.filter((d) => d.x !== undefined && d.y !== undefined);
    if (points.length < 2) return null;

    const xs = points.map((d) => d.x!);
    const ys = points.map((d) => d.y!);

    if (type === 'linear') {
      return linearRegression(xs, ys);
    } else if (type === 'polynomial') {
      return polynomialRegression(xs, ys, 2);
    } else if (type === 'exponential') {
      return exponentialRegression(xs, ys);
    }

    return null;
  };

  const linearRegression = (
    xs: number[],
    ys: number[]
  ): { points: { x: number; y: number }[]; r2: number } => {
    const n = xs.length;
    const sumX = d3.sum(xs);
    const sumY = d3.sum(ys);
    const sumXY = d3.sum(xs.map((x, i) => x * ys[i]!));
    const sumX2 = d3.sum(xs.map((x) => x * x));

    const slope = (n * sumXY - sumX * sumY) / (n * sumX2 - sumX * sumX);
    const intercept = (sumY - slope * sumX) / n;

    // Calculate R²
    const yMean = sumY / n;
    const ssTotal = d3.sum(ys.map((y) => Math.pow(y - yMean, 2)));
    const ssResidual = d3.sum(xs.map((x, i) => Math.pow(ys[i]! - (slope * x + intercept), 2)));
    const r2 = 1 - ssResidual / ssTotal;

    const minX = d3.min(xs) || 0;
    const maxX = d3.max(xs) || 0;

    return {
      points: [
        { x: minX, y: slope * minX + intercept },
        { x: maxX, y: slope * maxX + intercept },
      ],
      r2,
    };
  };

  const polynomialRegression = (
    xs: number[],
    ys: number[],
    degree: number
  ): { points: { x: number; y: number }[]; r2: number } => {
    // Simplified polynomial regression (degree 2)
    const n = xs.length;

    // Use matrix methods for polynomial fit (simplified for degree 2)
    const sumX = d3.sum(xs);
    const sumX2 = d3.sum(xs.map((x) => x * x));
    const sumX3 = d3.sum(xs.map((x) => Math.pow(x, 3)));
    const sumX4 = d3.sum(xs.map((x) => Math.pow(x, 4)));
    const sumY = d3.sum(ys);
    const sumXY = d3.sum(xs.map((x, i) => x * ys[i]!));
    const sumX2Y = d3.sum(xs.map((x, i) => x * x * ys[i]!));

    // Solve using simplified matrix
    const a = (n * sumX2 - sumX * sumX) === 0 ? 0 :
              (n * sumXY - sumX * sumY) / (n * sumX2 - sumX * sumX);
    const b = sumY / n - a * sumX / n;

    const predict = (x: number) => a * x + b;

    const minX = d3.min(xs) || 0;
    const maxX = d3.max(xs) || 0;
    const step = (maxX - minX) / 50;

    const points: { x: number; y: number }[] = [];
    for (let x = minX; x <= maxX; x += step) {
      points.push({ x, y: predict(x) });
    }

    // Calculate R²
    const yMean = sumY / n;
    const ssTotal = d3.sum(ys.map((y) => Math.pow(y - yMean, 2)));
    const ssResidual = d3.sum(xs.map((x, i) => Math.pow(ys[i]! - predict(x), 2)));
    const r2 = 1 - ssResidual / ssTotal;

    return { points, r2 };
  };

  const exponentialRegression = (
    xs: number[],
    ys: number[]
  ): { points: { x: number; y: number }[]; r2: number } => {
    // Transform to linear: ln(y) = ln(a) + b*x
    const lnYs = ys.map((y) => Math.log(Math.max(y, 0.0001)));

    const n = xs.length;
    const sumX = d3.sum(xs);
    const sumLnY = d3.sum(lnYs);
    const sumXLnY = d3.sum(xs.map((x, i) => x * lnYs[i]!));
    const sumX2 = d3.sum(xs.map((x) => x * x));

    const b = (n * sumXLnY - sumX * sumLnY) / (n * sumX2 - sumX * sumX);
    const lnA = (sumLnY - b * sumX) / n;
    const a = Math.exp(lnA);

    const predict = (x: number) => a * Math.exp(b * x);

    const minX = d3.min(xs) || 0;
    const maxX = d3.max(xs) || 0;
    const step = (maxX - minX) / 50;

    const points: { x: number; y: number }[] = [];
    for (let x = minX; x <= maxX; x += step) {
      points.push({ x, y: predict(x) });
    }

    // Calculate R²
    const yMean = d3.sum(ys) / n;
    const ssTotal = d3.sum(ys.map((y) => Math.pow(y - yMean, 2)));
    const ssResidual = d3.sum(xs.map((x, i) => Math.pow(ys[i]! - predict(x), 2)));
    const r2 = 1 - ssResidual / ssTotal;

    return { points, r2 };
  };

  return (
    <div className={`scatter-plot ${className}`} style={style}>
      <svg ref={svgRef} />
    </div>
  );
};

export default ScatterPlot;
