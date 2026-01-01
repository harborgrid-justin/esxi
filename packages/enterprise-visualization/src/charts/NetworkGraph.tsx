/**
 * Force-Directed Network Graph Component
 * Visualizes nodes and relationships with physics simulation
 */

import React, { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { NetworkData, NetworkGraphConfig, BaseChartProps, ChartEvent, NetworkNode, NetworkLink } from '../types';

interface NetworkGraphProps extends BaseChartProps<NetworkData, NetworkGraphConfig> {}

export const NetworkGraph: React.FC<NetworkGraphProps> = ({
  data,
  config,
  className = '',
  style,
  onEvent,
}) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const simulationRef = useRef<d3.Simulation<NetworkNode, NetworkLink> | null>(null);

  useEffect(() => {
    if (!svgRef.current || !data.nodes.length) return;

    const {
      width,
      height,
      margin = { top: 20, right: 20, bottom: 20, left: 20 },
    } = config.dimensions;

    const {
      linkDistance = 100,
      linkStrength = 0.5,
      chargeStrength = -300,
      showLabels = true,
      showArrows = true,
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
      .attr('aria-label', config.accessibility?.ariaLabel || 'Network graph visualization');

    const g = svg
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Color scheme
    const colorScheme = theme.colorScheme || d3.schemeCategory10;
    const color = d3.scaleOrdinal<string>().range(colorScheme);

    // Clone data to avoid mutations
    const nodes = data.nodes.map((d) => ({ ...d }));
    const links = data.links.map((d) => ({ ...d }));

    // Arrow marker definition
    if (showArrows) {
      const defs = svg.append('defs');

      colorScheme.forEach((c, i) => {
        defs
          .append('marker')
          .attr('id', `arrow-${i}`)
          .attr('viewBox', '0 -5 10 10')
          .attr('refX', 20)
          .attr('refY', 0)
          .attr('markerWidth', 6)
          .attr('markerHeight', 6)
          .attr('orient', 'auto')
          .append('path')
          .attr('d', 'M0,-5L10,0L0,5')
          .attr('fill', c);
      });

      defs
        .append('marker')
        .attr('id', 'arrow-default')
        .attr('viewBox', '0 -5 10 10')
        .attr('refX', 20)
        .attr('refY', 0)
        .attr('markerWidth', 6)
        .attr('markerHeight', 6)
        .attr('orient', 'auto')
        .append('path')
        .attr('d', 'M0,-5L10,0L0,5')
        .attr('fill', '#999');
    }

    // Create force simulation
    const simulation = d3
      .forceSimulation<NetworkNode>(nodes)
      .force(
        'link',
        d3
          .forceLink<NetworkNode, NetworkLink>(links)
          .id((d) => d.id)
          .distance(linkDistance)
          .strength(linkStrength)
      )
      .force('charge', d3.forceManyBody().strength(chargeStrength))
      .force('center', d3.forceCenter(innerWidth / 2, innerHeight / 2))
      .force('collision', d3.forceCollide().radius(30));

    simulationRef.current = simulation;

    // Draw links
    const link = g
      .append('g')
      .attr('class', 'links')
      .selectAll('line')
      .data(links)
      .enter()
      .append('line')
      .attr('stroke', theme.gridColor || '#999')
      .attr('stroke-opacity', 0.6)
      .attr('stroke-width', (d) => Math.sqrt(d.value || 1))
      .attr('marker-end', (d, i) =>
        showArrows ? `url(#arrow-${i % colorScheme.length})` : ''
      );

    // Draw nodes
    const node = g
      .append('g')
      .attr('class', 'nodes')
      .selectAll('g')
      .data(nodes)
      .enter()
      .append('g')
      .attr('class', 'node')
      .call(
        d3
          .drag<SVGGElement, NetworkNode>()
          .on('start', dragstarted)
          .on('drag', dragged)
          .on('end', dragended)
      );

    const circles = node
      .append('circle')
      .attr('r', (d) => Math.sqrt((d.value || 10)) + 5)
      .attr('fill', (d) => color(d.group || '0'))
      .attr('stroke', '#fff')
      .attr('stroke-width', 2)
      .style('cursor', 'pointer');

    if (animation.enabled) {
      circles.attr('r', 0).transition().duration(animation.duration).attr('r', (d) => Math.sqrt((d.value || 10)) + 5);
    }

    // Node labels
    if (showLabels) {
      const labels = node
        .append('text')
        .text((d) => d.label || d.id.toString())
        .attr('x', 0)
        .attr('y', (d) => Math.sqrt((d.value || 10)) + 15)
        .attr('text-anchor', 'middle')
        .style('font-size', '10px')
        .style('font-weight', '500')
        .style('pointer-events', 'none')
        .style('fill', theme.textColor || '#000');

      if (animation.enabled) {
        labels.attr('opacity', 0).transition().duration(animation.duration).delay(300).attr('opacity', 1);
      }
    }

    // Node interaction
    circles
      .on('mouseenter', function (event, d) {
        d3.select(this)
          .transition()
          .duration(200)
          .attr('r', Math.sqrt((d.value || 10)) + 8)
          .attr('stroke-width', 3);

        // Highlight connected links
        link.attr('stroke-opacity', (l) => {
          const sourceId = typeof l.source === 'object' ? l.source.id : l.source;
          const targetId = typeof l.target === 'object' ? l.target.id : l.target;
          return sourceId === d.id || targetId === d.id ? 1 : 0.1;
        });

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
        d3.select(this)
          .transition()
          .duration(200)
          .attr('r', Math.sqrt((d.value || 10)) + 5)
          .attr('stroke-width', 2);

        link.attr('stroke-opacity', 0.6);
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

    // Update positions on each tick
    simulation.on('tick', () => {
      link
        .attr('x1', (d) => (d.source as NetworkNode).x || 0)
        .attr('y1', (d) => (d.source as NetworkNode).y || 0)
        .attr('x2', (d) => (d.target as NetworkNode).x || 0)
        .attr('y2', (d) => (d.target as NetworkNode).y || 0);

      node.attr('transform', (d) => `translate(${d.x || 0},${d.y || 0})`);
    });

    // Drag functions
    function dragstarted(event: d3.D3DragEvent<SVGGElement, NetworkNode, NetworkNode>) {
      if (!event.active && simulationRef.current) {
        simulationRef.current.alphaTarget(0.3).restart();
      }
      event.subject.fx = event.subject.x;
      event.subject.fy = event.subject.y;
    }

    function dragged(event: d3.D3DragEvent<SVGGElement, NetworkNode, NetworkNode>) {
      event.subject.fx = event.x;
      event.subject.fy = event.y;
    }

    function dragended(event: d3.D3DragEvent<SVGGElement, NetworkNode, NetworkNode>) {
      if (!event.active && simulationRef.current) {
        simulationRef.current.alphaTarget(0);
      }
      event.subject.fx = null;
      event.subject.fy = null;
    }

    // Zoom behavior
    const zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.5, 5])
      .on('zoom', (event) => {
        g.attr('transform', event.transform);
      });

    svg.call(zoom);

    // Legend
    const groups = Array.from(new Set(nodes.map((d) => d.group).filter(Boolean)));
    if (groups.length > 0) {
      const legend = svg
        .append('g')
        .attr('class', 'legend')
        .attr('transform', `translate(${width - 120}, 20)`);

      groups.forEach((group, i) => {
        const legendRow = legend.append('g').attr('transform', `translate(0, ${i * 20})`);

        legendRow.append('circle').attr('cx', 5).attr('cy', 5).attr('r', 5).attr('fill', color(group!));

        legendRow
          .append('text')
          .attr('x', 15)
          .attr('y', 10)
          .style('font-size', '11px')
          .text(group!);
      });
    }

    // Cleanup
    return () => {
      if (simulationRef.current) {
        simulationRef.current.stop();
      }
    };
  }, [data, config, onEvent]);

  return (
    <div className={`network-graph ${className}`} style={style}>
      <svg ref={svgRef} />
    </div>
  );
};

export default NetworkGraph;
