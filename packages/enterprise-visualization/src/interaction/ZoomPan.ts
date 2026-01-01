/**
 * Zoom and Pan Controls for Visualizations
 * Provides interactive zoom and pan functionality using D3
 */

import * as d3 from 'd3';
import { ZoomPanConfig, ZoomPanState } from '../types';

export class ZoomPan {
  private zoom: d3.ZoomBehavior<SVGSVGElement, unknown> | null = null;
  private state: ZoomPanState = { scale: 1, translateX: 0, translateY: 0 };
  private listeners: Map<string, (state: ZoomPanState) => void> = new Map();

  /**
   * Initialize zoom and pan on an SVG element
   */
  public initialize(
    svg: SVGSVGElement,
    target: SVGGElement,
    config: ZoomPanConfig = {}
  ): void {
    const {
      minZoom = 0.1,
      maxZoom = 10,
      enableZoom = true,
      enablePan = true,
      wheelSensitivity = 1,
      panExtent,
    } = config;

    this.zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([minZoom, maxZoom])
      .on('zoom', (event) => {
        this.handleZoom(event, target);
      });

    // Configure zoom behavior
    if (!enableZoom) {
      this.zoom.filter((event) => {
        // Disable zoom but allow pan
        return enablePan && event.type !== 'wheel' && event.type !== 'dblclick';
      });
    }

    if (!enablePan) {
      this.zoom.filter((event) => {
        // Disable pan but allow zoom
        return enableZoom && (event.type === 'wheel' || event.type === 'dblclick');
      });
    }

    // Set wheel delta (sensitivity)
    this.zoom.wheelDelta((event) => {
      return (-event.deltaY * (event.deltaMode === 1 ? 0.05 : event.deltaMode ? 1 : 0.002)) * wheelSensitivity;
    });

    // Set pan extent if provided
    if (panExtent) {
      this.zoom.translateExtent(panExtent);
    }

    // Apply zoom behavior to SVG
    d3.select(svg).call(this.zoom);
  }

  /**
   * Handle zoom event
   */
  private handleZoom(event: d3.D3ZoomEvent<SVGSVGElement, unknown>, target: SVGGElement): void {
    const transform = event.transform;

    // Update target transform
    d3.select(target).attr('transform', transform.toString());

    // Update state
    this.state = {
      scale: transform.k,
      translateX: transform.x,
      translateY: transform.y,
    };

    // Notify listeners
    this.notifyListeners();
  }

  /**
   * Programmatically zoom to a specific level
   */
  public zoomTo(
    svg: SVGSVGElement,
    scale: number,
    duration: number = 750,
    center?: { x: number; y: number }
  ): void {
    if (!this.zoom) return;

    const selection = d3.select(svg);

    if (center) {
      const transform = d3.zoomIdentity.translate(center.x, center.y).scale(scale).translate(-center.x, -center.y);

      selection
        .transition()
        .duration(duration)
        .call(this.zoom.transform, transform);
    } else {
      selection
        .transition()
        .duration(duration)
        .call(this.zoom.scaleTo, scale);
    }
  }

  /**
   * Programmatically pan to a specific position
   */
  public panTo(
    svg: SVGSVGElement,
    x: number,
    y: number,
    duration: number = 750
  ): void {
    if (!this.zoom) return;

    const transform = d3.zoomIdentity.translate(x, y).scale(this.state.scale);

    d3.select(svg)
      .transition()
      .duration(duration)
      .call(this.zoom.transform, transform);
  }

  /**
   * Zoom in
   */
  public zoomIn(svg: SVGSVGElement, factor: number = 1.5, duration: number = 300): void {
    if (!this.zoom) return;

    d3.select(svg)
      .transition()
      .duration(duration)
      .call(this.zoom.scaleBy, factor);
  }

  /**
   * Zoom out
   */
  public zoomOut(svg: SVGSVGElement, factor: number = 1.5, duration: number = 300): void {
    if (!this.zoom) return;

    d3.select(svg)
      .transition()
      .duration(duration)
      .call(this.zoom.scaleBy, 1 / factor);
  }

  /**
   * Reset zoom and pan to default
   */
  public reset(svg: SVGSVGElement, duration: number = 750): void {
    if (!this.zoom) return;

    d3.select(svg)
      .transition()
      .duration(duration)
      .call(this.zoom.transform, d3.zoomIdentity);
  }

  /**
   * Fit content to view
   */
  public fitToView(
    svg: SVGSVGElement,
    bounds: { x: number; y: number; width: number; height: number },
    padding: number = 20,
    duration: number = 750
  ): void {
    if (!this.zoom) return;

    const svgRect = svg.getBoundingClientRect();
    const svgWidth = svgRect.width;
    const svgHeight = svgRect.height;

    // Calculate scale to fit content
    const scaleX = (svgWidth - padding * 2) / bounds.width;
    const scaleY = (svgHeight - padding * 2) / bounds.height;
    const scale = Math.min(scaleX, scaleY);

    // Calculate translation to center content
    const translateX = (svgWidth - bounds.width * scale) / 2 - bounds.x * scale;
    const translateY = (svgHeight - bounds.height * scale) / 2 - bounds.y * scale;

    const transform = d3.zoomIdentity.translate(translateX, translateY).scale(scale);

    d3.select(svg)
      .transition()
      .duration(duration)
      .call(this.zoom.transform, transform);
  }

  /**
   * Zoom to specific element
   */
  public zoomToElement(
    svg: SVGSVGElement,
    element: SVGGraphicsElement,
    padding: number = 20,
    duration: number = 750
  ): void {
    const bbox = element.getBBox();
    this.fitToView(
      svg,
      { x: bbox.x, y: bbox.y, width: bbox.width, height: bbox.height },
      padding,
      duration
    );
  }

  /**
   * Get current zoom/pan state
   */
  public getState(): ZoomPanState {
    return { ...this.state };
  }

  /**
   * Set zoom/pan state programmatically
   */
  public setState(svg: SVGSVGElement, state: ZoomPanState, duration: number = 0): void {
    if (!this.zoom) return;

    const transform = d3.zoomIdentity
      .translate(state.translateX, state.translateY)
      .scale(state.scale);

    if (duration > 0) {
      d3.select(svg)
        .transition()
        .duration(duration)
        .call(this.zoom.transform, transform);
    } else {
      d3.select(svg).call(this.zoom.transform, transform);
    }
  }

  /**
   * Add listener for zoom/pan state changes
   */
  public addListener(id: string, callback: (state: ZoomPanState) => void): void {
    this.listeners.set(id, callback);
  }

  /**
   * Remove listener
   */
  public removeListener(id: string): void {
    this.listeners.delete(id);
  }

  /**
   * Notify all listeners of state change
   */
  private notifyListeners(): void {
    this.listeners.forEach((callback) => {
      callback(this.state);
    });
  }

  /**
   * Enable or disable zoom
   */
  public setZoomEnabled(enabled: boolean): void {
    if (!this.zoom) return;

    if (enabled) {
      this.zoom.filter((event) => {
        return event.type === 'wheel' || event.type === 'dblclick' || event.button === 0;
      });
    } else {
      this.zoom.filter((event) => {
        return event.type !== 'wheel' && event.type !== 'dblclick';
      });
    }
  }

  /**
   * Enable or disable pan
   */
  public setPanEnabled(enabled: boolean): void {
    if (!this.zoom) return;

    if (enabled) {
      this.zoom.filter((event) => {
        return event.button === 0;
      });
    } else {
      this.zoom.filter((event) => {
        return event.type === 'wheel' || event.type === 'dblclick';
      });
    }
  }

  /**
   * Create zoom controls UI
   */
  public createControls(
    container: HTMLElement,
    svg: SVGSVGElement
  ): HTMLDivElement {
    const controls = document.createElement('div');
    controls.className = 'zoom-controls';
    controls.style.cssText = `
      position: absolute;
      top: 10px;
      right: 10px;
      display: flex;
      flex-direction: column;
      gap: 5px;
      background: white;
      border-radius: 4px;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
      padding: 5px;
    `;

    const createButton = (text: string, onClick: () => void): HTMLButtonElement => {
      const button = document.createElement('button');
      button.textContent = text;
      button.style.cssText = `
        padding: 8px 12px;
        border: none;
        background: #f0f0f0;
        cursor: pointer;
        border-radius: 3px;
        font-size: 14px;
        font-weight: 500;
      `;
      button.addEventListener('mouseenter', () => {
        button.style.background = '#e0e0e0';
      });
      button.addEventListener('mouseleave', () => {
        button.style.background = '#f0f0f0';
      });
      button.addEventListener('click', onClick);
      return button;
    };

    controls.appendChild(createButton('+', () => this.zoomIn(svg)));
    controls.appendChild(createButton('−', () => this.zoomOut(svg)));
    controls.appendChild(createButton('⟲', () => this.reset(svg)));

    container.style.position = 'relative';
    container.appendChild(controls);

    return controls;
  }

  /**
   * Destroy zoom/pan instance
   */
  public destroy(svg: SVGSVGElement): void {
    if (this.zoom) {
      d3.select(svg).on('.zoom', null);
      this.zoom = null;
    }
    this.listeners.clear();
    this.state = { scale: 1, translateX: 0, translateY: 0 };
  }
}

// Export singleton instance
export const zoomPan = new ZoomPan();

export default ZoomPan;
