/**
 * Sankey Flow Diagram Component
 * Visualizes flows and relationships between nodes
 */

import React, { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import {
  sankey as d3Sankey,
  sankeyLinkHorizontal,
  SankeyGraph,
  SankeyNode,
  SankeyLink,
} from 'd3-sankey';
import { FlowData, ChartConfig, BaseChartProps, ChartEvent } from '../types';

interface SankeyProps extends BaseChartProps<FlowData, ChartConfig> {
  nodeWidth?: number;
  nodePadding?: number;
}

export const SankeyDiagram: React.FC<SankeyProps> = ({
  data,
  config,
  className = '',
  style,
  onEvent,
  nodeWidth = 20,
  nodePadding = 10,
}) => {
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!svgRef.current || !data.nodes.length || !data.links.length) return;

    const {
      width,
      height,
      margin = { top: 20, right: 20, bottom: 20, left: 20 },
    } = config.dimensions;

    const {
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
      .attr('aria-label', config.accessibility?.ariaLabel || 'Sankey diagram visualization');

    const g = svg
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Color scheme
    const colorScheme = theme.colorScheme || d3.schemeCategory10;
    const color = d3.scaleOrdinal<string>().range(colorScheme);

    // Create Sankey layout
    const sankeyLayout = d3Sankey<FlowData['nodes'][0], FlowData['links'][0]>()
      .nodeWidth(nodeWidth)
      .nodePadding(nodePadding)
      .extent([
        [0, 0],
        [innerWidth, innerHeight],
      ]);

    // Process data
    const graph: SankeyGraph<FlowData['nodes'][0], FlowData['links'][0]> = {
      nodes: data.nodes.map((d) => ({ ...d })),
      links: data.links.map((d) => ({ ...d })),
    };

    sankeyLayout(graph);

    // Create gradient definitions
    const defs = svg.append('defs');

    graph.links.forEach((link, i) => {
      const gradient = defs
        .append('linearGradient')
        .attr('id', `gradient-${i}`)
        .attr('gradientUnits', 'userSpaceOnUse')
        .attr('x1', (link.source as SankeyNode<FlowData['nodes'][0], FlowData['links'][0]>).x1)
        .attr('x2', (link.target as SankeyNode<FlowData['nodes'][0], FlowData['links'][0]>).x0);

      gradient
        .append('stop')
        .attr('offset', '0%')
        .attr(
          'stop-color',
          color(
            (link.source as SankeyNode<FlowData['nodes'][0], FlowData['links'][0]>).id?.toString() ||
              '0'
          )
        );

      gradient
        .append('stop')
        .attr('offset', '100%')
        .attr(
          'stop-color',
          color(
            (link.target as SankeyNode<FlowData['nodes'][0], FlowData['links'][0]>).id?.toString() ||
              '0'
          )
        );
    });

    // Draw links
    const links = g
      .append('g')
      .attr('class', 'links')
      .selectAll('path')
      .data(graph.links)
      .enter()
      .append('path')
      .attr('d', sankeyLinkHorizontal())
      .attr('fill', 'none')
      .attr('stroke', (d, i) => `url(#gradient-${i})`)
      .attr('stroke-width', (d) => Math.max(1, d.width || 0))
      .attr('opacity', 0.5)
      .style('cursor', 'pointer');

    if (animation.enabled) {
      links
        .attr('stroke-dasharray', function () {
          const length = (this as SVGPathElement).getTotalLength();
          return `${length} ${length}`;
        })
        .attr('stroke-dashoffset', function () {
          return (this as SVGPathElement).getTotalLength();
        })
        .transition()
        .duration(animation.duration)
        .attr('stroke-dashoffset', 0);
    }

    // Link interaction
    links
      .on('mouseenter', function (event, d) {
        d3.select(this).transition().duration(200).attr('opacity', 0.8).attr('stroke-width', (d) => Math.max(1, (d.width || 0) + 2));

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
        d3.select(this).transition().duration(200).attr('opacity', 0.5).attr('stroke-width', (d) => Math.max(1, d.width || 0));
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

    // Draw nodes
    const nodes = g
      .append('g')
      .attr('class', 'nodes')
      .selectAll('g')
      .data(graph.nodes)
      .enter()
      .append('g')
      .attr('class', 'node');

    const nodeRects = nodes
      .append('rect')
      .attr('x', (d) => d.x0 || 0)
      .attr('y', (d) => d.y0 || 0)
      .attr('width', (d) => (d.x1 || 0) - (d.x0 || 0))
      .attr('height', 0)
      .attr('fill', (d) => color(d.id?.toString() || '0'))
      .attr('stroke', '#fff')
      .attr('stroke-width', 2)
      .style('cursor', 'pointer');

    if (animation.enabled) {
      nodeRects
        .transition()
        .duration(animation.duration)
        .delay(animation.duration / 2)
        .attr('height', (d) => (d.y1 || 0) - (d.y0 || 0));
    } else {
      nodeRects.attr('height', (d) => (d.y1 || 0) - (d.y0 || 0));
    }

    // Node interaction
    nodeRects
      .on('mouseenter', function (event, d) {
        d3.select(this).transition().duration(200).attr('opacity', 0.8);

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
        d3.select(this).transition().duration(200).attr('opacity', 1);
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

    // Node labels
    const labels = nodes
      .append('text')
      .attr('x', (d) => ((d.x0 || 0) < innerWidth / 2 ? (d.x1 || 0) + 6 : (d.x0 || 0) - 6))
      .attr('y', (d) => ((d.y1 || 0) + (d.y0 || 0)) / 2)
      .attr('dy', '0.35em')
      .attr('text-anchor', (d) => ((d.x0 || 0) < innerWidth / 2 ? 'start' : 'end'))
      .style('font-size', '12px')
      .style('font-weight', '500')
      .style('pointer-events', 'none');

    labels
      .append('tspan')
      .text((d) => d.label || d.id?.toString() || '')
      .attr('fill', '#000');

    labels
      .append('tspan')
      .attr('x', (d) => ((d.x0 || 0) < innerWidth / 2 ? (d.x1 || 0) + 6 : (d.x0 || 0) - 6))
      .attr('dy', '1.2em')
      .style('font-size', '10px')
      .style('font-weight', '400')
      .attr('fill', '#666')
      .text((d) => {
        const value = d.value || 0;
        return formatValue(value);
      });

    if (animation.enabled) {
      labels
        .attr('opacity', 0)
        .transition()
        .duration(animation.duration)
        .delay(animation.duration)
        .attr('opacity', 1);
    }
  }, [data, config, onEvent, nodeWidth, nodePadding]);

  const formatValue = (value: number): string => {
    if (value >= 1e9) return (value / 1e9).toFixed(1) + 'B';
    if (value >= 1e6) return (value / 1e6).toFixed(1) + 'M';
    if (value >= 1e3) return (value / 1e3).toFixed(1) + 'K';
    return value.toFixed(0);
  };

  return (
    <div className={`sankey-diagram ${className}`} style={style}>
      <svg ref={svgRef} />
    </div>
  );
};

export default SankeyDiagram;
