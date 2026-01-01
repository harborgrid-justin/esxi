/**
 * D3.js Integration Layer
 * @module @harborgrid/enterprise-analytics/visualization
 */

import * as d3 from 'd3';
import type { AxisConfig, ColorScaleConfig } from '../types';

export class D3Integration {
  // ============================================================================
  // Scale Creation
  // ============================================================================

  createScale(config: AxisConfig, data: unknown[]): d3.ScaleLinear<number, number> | d3.ScaleTime<number, number> | d3.ScaleBand<string> {
    switch (config.scale) {
      case 'linear':
        return this.createLinearScale(config, data);
      case 'log':
        return this.createLogScale(config, data);
      case 'time':
        return this.createTimeScale(config, data);
      case 'ordinal':
      case 'band':
        return this.createBandScale(config, data);
      default:
        return this.createLinearScale(config, data);
    }
  }

  private createLinearScale(config: AxisConfig, data: unknown[]): d3.ScaleLinear<number, number> {
    const values = data.map((d) => (d as Record<string, unknown>)[config.field] as number);
    const domain = config.domain || [d3.min(values) || 0, d3.max(values) || 100];
    const range = config.range || [0, 100];

    const scale = d3.scaleLinear().domain(domain as [number, number]).range(range as [number, number]);

    if (config.zero) {
      scale.domain([0, d3.max(values) || 100]);
    }

    return scale;
  }

  private createLogScale(config: AxisConfig, data: unknown[]): d3.ScaleLinear<number, number> {
    const values = data.map((d) => (d as Record<string, unknown>)[config.field] as number);
    const domain = config.domain || [d3.min(values) || 1, d3.max(values) || 100];
    const range = config.range || [0, 100];

    return d3.scaleLog().domain(domain as [number, number]).range(range as [number, number]) as any;
  }

  private createTimeScale(config: AxisConfig, data: unknown[]): d3.ScaleTime<number, number> {
    const values = data.map((d) => new Date((d as Record<string, unknown>)[config.field] as string));
    const domain = config.domain || [d3.min(values) || new Date(), d3.max(values) || new Date()];
    const range = config.range || [0, 100];

    return d3.scaleTime().domain(domain as [Date, Date]).range(range as [number, number]);
  }

  private createBandScale(config: AxisConfig, data: unknown[]): d3.ScaleBand<string> {
    const values = data.map((d) => String((d as Record<string, unknown>)[config.field]));
    const domain = config.domain || Array.from(new Set(values));
    const range = config.range || [0, 100];

    return d3.scaleBand().domain(domain as string[]).range(range as [number, number]).padding(0.1);
  }

  // ============================================================================
  // Color Scale Creation
  // ============================================================================

  createColorScale(config: ColorScaleConfig, data?: unknown[]): d3.ScaleOrdinal<string, string> | d3.ScaleSequential<string> {
    switch (config.type) {
      case 'categorical':
        return this.createCategoricalColorScale(config, data);
      case 'sequential':
        return this.createSequentialColorScale(config, data);
      case 'diverging':
        return this.createDivergingColorScale(config, data);
      default:
        return this.createCategoricalColorScale(config, data);
    }
  }

  private createCategoricalColorScale(
    config: ColorScaleConfig,
    data?: unknown[]
  ): d3.ScaleOrdinal<string, string> {
    const domain = config.domain || [];
    const range = config.range || this.getColorScheme(config.scheme || 'category10');

    const scale = d3.scaleOrdinal<string, string>().domain(domain as string[]).range(range);

    // Apply custom colors if provided
    if (config.customColors) {
      for (const [key, color] of Object.entries(config.customColors)) {
        scale.domain([...scale.domain(), key]);
        scale.range([...scale.range(), color]);
      }
    }

    return scale;
  }

  private createSequentialColorScale(
    config: ColorScaleConfig,
    data?: unknown[]
  ): d3.ScaleSequential<string> {
    const interpolator = this.getColorInterpolator(config.scheme || 'Blues');
    return d3.scaleSequential(interpolator);
  }

  private createDivergingColorScale(
    config: ColorScaleConfig,
    data?: unknown[]
  ): d3.ScaleSequential<string> {
    const interpolator = this.getColorInterpolator(config.scheme || 'RdYlBu');
    return d3.scaleSequential(interpolator);
  }

  // ============================================================================
  // Axis Creation
  // ============================================================================

  createAxis(
    orientation: 'top' | 'right' | 'bottom' | 'left',
    scale: d3.AxisScale<d3.AxisDomain>
  ): d3.Axis<d3.AxisDomain> {
    switch (orientation) {
      case 'top':
        return d3.axisTop(scale);
      case 'right':
        return d3.axisRight(scale);
      case 'bottom':
        return d3.axisBottom(scale);
      case 'left':
        return d3.axisLeft(scale);
    }
  }

  // ============================================================================
  // Shape Generators
  // ============================================================================

  createLineGenerator<T>(
    xScale: d3.ScaleLinear<number, number> | d3.ScaleTime<number, number>,
    yScale: d3.ScaleLinear<number, number>,
    xField: string,
    yField: string,
    smooth: boolean = false
  ): d3.Line<T> {
    const line = d3
      .line<T>()
      .x((d) => xScale((d as Record<string, unknown>)[xField] as number))
      .y((d) => yScale((d as Record<string, unknown>)[yField] as number));

    if (smooth) {
      line.curve(d3.curveMonotoneX);
    }

    return line;
  }

  createAreaGenerator<T>(
    xScale: d3.ScaleLinear<number, number> | d3.ScaleTime<number, number>,
    yScale: d3.ScaleLinear<number, number>,
    xField: string,
    yField: string,
    smooth: boolean = false
  ): d3.Area<T> {
    const area = d3
      .area<T>()
      .x((d) => xScale((d as Record<string, unknown>)[xField] as number))
      .y0(yScale(0))
      .y1((d) => yScale((d as Record<string, unknown>)[yField] as number));

    if (smooth) {
      area.curve(d3.curveMonotoneX);
    }

    return area;
  }

  createPieGenerator<T>(): d3.Pie<unknown, T> {
    return d3.pie<T>().sort(null);
  }

  createArcGenerator(innerRadius: number = 0, outerRadius: number = 100): d3.Arc<unknown, d3.DefaultArcObject> {
    return d3.arc().innerRadius(innerRadius).outerRadius(outerRadius);
  }

  // ============================================================================
  // Layout Generators
  // ============================================================================

  createTreemapLayout<T>(width: number, height: number): d3.TreemapLayout<T> {
    return d3.treemap<T>().size([width, height]).padding(1).round(true);
  }

  createPackLayout<T>(width: number, height: number): d3.PackLayout<T> {
    return d3.pack<T>().size([width, height]).padding(3);
  }

  createHierarchyFromData<T>(data: T[], childrenAccessor?: (d: T) => T[]): d3.HierarchyNode<T> {
    return d3.hierarchy(data as any, childrenAccessor);
  }

  // ============================================================================
  // Interpolators and Transitions
  // ============================================================================

  createTransition(duration: number = 300, ease: string = 'cubic'): d3.Transition<d3.BaseType, unknown, null, undefined> {
    const easing = this.getEasingFunction(ease);
    return d3.transition().duration(duration).ease(easing);
  }

  private getEasingFunction(ease: string): (normalizedTime: number) => number {
    switch (ease) {
      case 'linear':
        return d3.easeLinear;
      case 'quad':
        return d3.easeQuad;
      case 'cubic':
        return d3.easeCubic;
      case 'sin':
        return d3.easeSin;
      case 'exp':
        return d3.easeExp;
      case 'circle':
        return d3.easeCircle;
      case 'bounce':
        return d3.easeBounce;
      case 'elastic':
        return d3.easeElastic;
      default:
        return d3.easeCubic;
    }
  }

  // ============================================================================
  // Data Transformations
  // ============================================================================

  stack<T>(data: T[], keys: string[]): d3.Series<T, string>[] {
    return d3.stack<T>().keys(keys)(data);
  }

  bin<T>(data: T[], accessor: (d: T) => number, thresholds?: number): d3.Bin<T, number>[] {
    return d3
      .bin<T, number>()
      .value(accessor)
      .thresholds(thresholds || 10)(data);
  }

  group<T, K>(data: T[], accessor: (d: T) => K): d3.InternMap<K, T[]> {
    return d3.group(data, accessor);
  }

  rollup<T, R>(
    data: T[],
    reducer: (values: T[]) => R,
    ...accessors: Array<(d: T) => unknown>
  ): d3.InternMap<unknown, R> {
    return d3.rollup(data, reducer, ...accessors) as d3.InternMap<unknown, R>;
  }

  // ============================================================================
  // Statistical Functions
  // ============================================================================

  extent<T>(data: T[], accessor: (d: T) => number): [number, number] | [undefined, undefined] {
    return d3.extent(data, accessor);
  }

  mean<T>(data: T[], accessor: (d: T) => number): number | undefined {
    return d3.mean(data, accessor);
  }

  median<T>(data: T[], accessor: (d: T) => number): number | undefined {
    return d3.median(data, accessor);
  }

  quantile<T>(data: T[], p: number, accessor: (d: T) => number): number | undefined {
    return d3.quantile(data.map(accessor), p);
  }

  variance<T>(data: T[], accessor: (d: T) => number): number | undefined {
    return d3.variance(data, accessor);
  }

  deviation<T>(data: T[], accessor: (d: T) => number): number | undefined {
    return d3.deviation(data, accessor);
  }

  // ============================================================================
  // Format Functions
  // ============================================================================

  createNumberFormat(specifier: string): (n: number | { valueOf(): number }) => string {
    return d3.format(specifier);
  }

  createTimeFormat(specifier: string): (date: Date) => string {
    return d3.timeFormat(specifier);
  }

  // ============================================================================
  // Color Schemes and Interpolators
  // ============================================================================

  private getColorScheme(scheme: string): string[] {
    const schemes: Record<string, string[]> = {
      category10: d3.schemeCategory10,
      accent: d3.schemeAccent,
      dark2: d3.schemeDark2,
      paired: d3.schemePaired,
      pastel1: d3.schemePastel1,
      pastel2: d3.schemePastel2,
      set1: d3.schemeSet1,
      set2: d3.schemeSet2,
      set3: d3.schemeSet3,
      tableau10: d3.schemeTableau10,
    };

    return schemes[scheme] || d3.schemeCategory10;
  }

  private getColorInterpolator(scheme: string): (t: number) => string {
    const interpolators: Record<string, (t: number) => string> = {
      Blues: d3.interpolateBlues,
      Greens: d3.interpolateGreens,
      Greys: d3.interpolateGreys,
      Oranges: d3.interpolateOranges,
      Purples: d3.interpolatePurples,
      Reds: d3.interpolateReds,
      BuGn: d3.interpolateBuGn,
      BuPu: d3.interpolateBuPu,
      GnBu: d3.interpolateGnBu,
      OrRd: d3.interpolateOrRd,
      PuBuGn: d3.interpolatePuBuGn,
      PuBu: d3.interpolatePuBu,
      PuRd: d3.interpolatePuRd,
      RdPu: d3.interpolateRdPu,
      YlGnBu: d3.interpolateYlGnBu,
      YlGn: d3.interpolateYlGn,
      YlOrBr: d3.interpolateYlOrBr,
      YlOrRd: d3.interpolateYlOrRd,
      BrBG: d3.interpolateBrBG,
      PRGn: d3.interpolatePRGn,
      PiYG: d3.interpolatePiYG,
      PuOr: d3.interpolatePuOr,
      RdBu: d3.interpolateRdBu,
      RdGy: d3.interpolateRdGy,
      RdYlBu: d3.interpolateRdYlBu,
      RdYlGn: d3.interpolateRdYlGn,
      Spectral: d3.interpolateSpectral,
    };

    return interpolators[scheme] || d3.interpolateBlues;
  }

  // ============================================================================
  // Utility Functions
  // ============================================================================

  selectAll(selector: string): d3.Selection<d3.BaseType, unknown, HTMLElement, unknown> {
    return d3.selectAll(selector);
  }

  select(selector: string): d3.Selection<d3.BaseType, unknown, HTMLElement, unknown> {
    return d3.select(selector);
  }

  createSVG(container: HTMLElement, width: number, height: number): d3.Selection<SVGSVGElement, unknown, null, undefined> {
    return d3
      .select(container)
      .append('svg')
      .attr('width', width)
      .attr('height', height);
  }

  zoom(): d3.ZoomBehavior<Element, unknown> {
    return d3.zoom();
  }

  drag(): d3.DragBehavior<Element, unknown, unknown> {
    return d3.drag();
  }

  brush(): d3.BrushBehavior<unknown> {
    return d3.brush();
  }
}

// Factory function
export function createD3Integration(): D3Integration {
  return new D3Integration();
}
