/**
 * Hierarchical Tree Map Visualization
 * Supports multiple tiling algorithms and nested hierarchies
 */

import React, { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { HierarchicalData, TreeMapConfig, BaseChartProps, ChartEvent } from '../types';

interface TreeMapProps extends BaseChartProps<HierarchicalData, TreeMapConfig> {}

export const TreeMap: React.FC<TreeMapProps> = ({
  data,
  config,
  className = '',
  style,
  onEvent,
}) => {
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!svgRef.current || !data) return;

    const {
      width,
      height,
      margin = { top: 10, right: 10, bottom: 10, left: 10 },
    } = config.dimensions;

    const {
      paddingInner = 2,
      paddingOuter = 2,
      tile = 'squarify',
      showLabels = true,
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
      .attr('aria-label', config.accessibility?.ariaLabel || 'Tree map visualization');

    const g = svg
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Tile method mapping
    const tileMap: {
      [key: string]: (
        node: d3.HierarchyRectangularNode<HierarchicalData>,
        x0: number,
        y0: number,
        x1: number,
        y1: number
      ) => void;
    } = {
      binary: d3.treemapBinary,
      squarify: d3.treemapSquarify,
      slice: d3.treemapSlice,
      dice: d3.treemapDice,
    };

    const tileMethod = tileMap[tile] || d3.treemapSquarify;

    // Create hierarchy
    const root = d3
      .hierarchy(data)
      .sum((d) => d.value || 0)
      .sort((a, b) => (b.value || 0) - (a.value || 0));

    // Create treemap layout
    const treemap = d3
      .treemap<HierarchicalData>()
      .size([innerWidth, innerHeight])
      .paddingInner(paddingInner)
      .paddingOuter(paddingOuter)
      .tile(tileMethod);

    const nodes = treemap(root);

    // Color scheme
    const colorScheme = theme.colorScheme || d3.schemeCategory10;
    const color = d3.scaleOrdinal<string>().range(colorScheme);

    // Get all leaf nodes
    const leaves = nodes.leaves();

    // Calculate opacity based on depth
    const maxDepth = d3.max(leaves, (d) => d.depth) || 1;
    const opacityScale = d3.scaleLinear().domain([0, maxDepth]).range([1, 0.6]);

    // Create cells
    const cells = g
      .selectAll('.cell')
      .data(leaves)
      .enter()
      .append('g')
      .attr('class', 'cell')
      .attr('transform', (d) => `translate(${d.x0},${d.y0})`);

    // Rectangles
    const rects = cells
      .append('rect')
      .attr('width', (d) => d.x1 - d.x0)
      .attr('height', (d) => d.y1 - d.y0)
      .attr('fill', (d) => {
        // Color by top-level parent
        let node = d;
        while (node.depth > 1 && node.parent) {
          node = node.parent;
        }
        return color(node.data.name);
      })
      .attr('opacity', (d) => opacityScale(d.depth))
      .attr('stroke', '#fff')
      .attr('stroke-width', 1)
      .style('cursor', 'pointer');

    if (animation.enabled) {
      rects
        .attr('width', 0)
        .attr('height', 0)
        .transition()
        .duration(animation.duration)
        .attr('width', (d) => d.x1 - d.x0)
        .attr('height', (d) => d.y1 - d.y0);
    }

    // Labels
    if (showLabels) {
      const labels = cells
        .append('text')
        .attr('x', 4)
        .attr('y', 16)
        .style('font-size', '11px')
        .style('font-weight', '500')
        .style('fill', '#fff')
        .style('text-shadow', '0 1px 2px rgba(0,0,0,0.5)')
        .style('pointer-events', 'none')
        .each(function (d) {
          const width = d.x1 - d.x0;
          const height = d.y1 - d.y0;

          // Only show label if cell is large enough
          if (width > 50 && height > 20) {
            const text = d3.select(this);

            // Add name
            text
              .append('tspan')
              .attr('x', 4)
              .attr('dy', 0)
              .text(() => {
                const name = d.data.name;
                const maxLength = Math.floor(width / 6);
                return name.length > maxLength ? name.substring(0, maxLength - 3) + '...' : name;
              });

            // Add value if there's space
            if (height > 35 && d.data.value) {
              text
                .append('tspan')
                .attr('x', 4)
                .attr('dy', '1.2em')
                .style('font-size', '10px')
                .style('font-weight', '400')
                .text(formatValue(d.data.value));
            }
          }
        });

      if (animation.enabled) {
        labels.attr('opacity', 0).transition().duration(animation.duration).delay(300).attr('opacity', 1);
      }
    }

    // Interaction
    rects
      .on('mouseenter', function (event, d) {
        d3.select(this)
          .transition()
          .duration(200)
          .attr('opacity', 1)
          .attr('stroke-width', 2)
          .attr('stroke', '#333');

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
          .attr('opacity', opacityScale(d.depth))
          .attr('stroke-width', 1)
          .attr('stroke', '#fff');
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

    // Breadcrumb trail
    const breadcrumb = svg
      .append('g')
      .attr('class', 'breadcrumb')
      .attr('transform', `translate(${margin.left}, 5)`);

    breadcrumb
      .append('text')
      .style('font-size', '12px')
      .style('font-weight', '600')
      .text(data.name);
  }, [data, config, onEvent]);

  const formatValue = (value: number): string => {
    if (value >= 1e9) return (value / 1e9).toFixed(1) + 'B';
    if (value >= 1e6) return (value / 1e6).toFixed(1) + 'M';
    if (value >= 1e3) return (value / 1e3).toFixed(1) + 'K';
    return value.toFixed(0);
  };

  return (
    <div className={`tree-map ${className}`} style={style}>
      <svg ref={svgRef} />
    </div>
  );
};

export default TreeMap;
