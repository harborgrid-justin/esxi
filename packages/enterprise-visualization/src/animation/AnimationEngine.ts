/**
 * Animation Engine using D3 Transitions
 * Provides advanced animation capabilities for visualizations
 */

import * as d3 from 'd3';
import { AnimationOptions, D3Selection, D3Transition } from '../types';

export class AnimationEngine {
  private activeTransitions: Map<string, d3.Transition<any, any, any, any>> = new Map();

  /**
   * Create a transition with custom easing and callbacks
   */
  public createTransition(
    selection: D3Selection,
    options: AnimationOptions
  ): D3Transition {
    const transition = selection
      .transition()
      .duration(options.duration)
      .delay(options.delay || 0);

    if (options.easing) {
      transition.ease(options.easing);
    }

    if (options.onStart) {
      transition.on('start', options.onStart);
    }

    if (options.onEnd) {
      transition.on('end', options.onEnd);
    }

    return transition as D3Transition;
  }

  /**
   * Animate a numeric attribute
   */
  public animateAttr(
    selection: D3Selection,
    attrName: string,
    targetValue: number | ((d: any, i: number) => number),
    options: AnimationOptions
  ): void {
    const transition = this.createTransition(selection, options);
    transition.attr(attrName, targetValue);
  }

  /**
   * Animate a style property
   */
  public animateStyle(
    selection: D3Selection,
    styleName: string,
    targetValue: string | ((d: any, i: number) => string),
    options: AnimationOptions
  ): void {
    const transition = this.createTransition(selection, options);
    transition.style(styleName, targetValue);
  }

  /**
   * Animate opacity (fade in/out)
   */
  public fade(
    selection: D3Selection,
    targetOpacity: number,
    options: AnimationOptions
  ): void {
    this.animateAttr(selection, 'opacity', targetOpacity, options);
  }

  /**
   * Fade in animation
   */
  public fadeIn(selection: D3Selection, options: AnimationOptions): void {
    selection.attr('opacity', 0);
    this.fade(selection, 1, options);
  }

  /**
   * Fade out animation
   */
  public fadeOut(selection: D3Selection, options: AnimationOptions): void {
    this.fade(selection, 0, options);
  }

  /**
   * Slide animation
   */
  public slide(
    selection: D3Selection,
    fromX: number,
    toX: number,
    fromY: number,
    toY: number,
    options: AnimationOptions
  ): void {
    selection.attr('transform', `translate(${fromX}, ${fromY})`);

    const transition = this.createTransition(selection, options);
    transition.attr('transform', `translate(${toX}, ${toY})`);
  }

  /**
   * Scale animation
   */
  public scale(
    selection: D3Selection,
    fromScale: number,
    toScale: number,
    options: AnimationOptions
  ): void {
    selection.attr('transform', `scale(${fromScale})`);

    const transition = this.createTransition(selection, options);
    transition.attr('transform', `scale(${toScale})`);
  }

  /**
   * Staggered animation (each element delayed progressively)
   */
  public stagger(
    selection: D3Selection,
    callback: (element: any, index: number) => void,
    staggerDelay: number,
    options: AnimationOptions
  ): void {
    selection.each(function (d, i) {
      const delayedOptions = {
        ...options,
        delay: (options.delay || 0) + i * staggerDelay,
      };

      callback.call(this, d, i);
    });
  }

  /**
   * Path animation (draw line from start to end)
   */
  public animatePath(
    selection: d3.Selection<SVGPathElement, any, any, any>,
    options: AnimationOptions
  ): void {
    selection.each(function () {
      const path = d3.select(this);
      const length = (this as SVGPathElement).getTotalLength();

      path
        .attr('stroke-dasharray', `${length} ${length}`)
        .attr('stroke-dashoffset', length)
        .transition()
        .duration(options.duration)
        .delay(options.delay || 0)
        .ease(options.easing || d3.easeCubicInOut)
        .attr('stroke-dashoffset', 0)
        .on('start', options.onStart || null)
        .on('end', options.onEnd || null);
    });
  }

  /**
   * Morph between two paths
   */
  public morphPath(
    selection: d3.Selection<SVGPathElement, any, any, any>,
    targetPath: string,
    options: AnimationOptions
  ): void {
    const transition = selection
      .transition()
      .duration(options.duration)
      .delay(options.delay || 0)
      .ease(options.easing || d3.easeCubicInOut);

    transition.attrTween('d', function () {
      const previous = this.getAttribute('d') || '';
      return d3.interpolateString(previous, targetPath);
    });

    if (options.onStart) transition.on('start', options.onStart);
    if (options.onEnd) transition.on('end', options.onEnd);
  }

  /**
   * Pulse animation (scale up and down)
   */
  public pulse(
    selection: D3Selection,
    scaleAmount: number,
    options: AnimationOptions
  ): void {
    const halfDuration = options.duration / 2;

    // Scale up
    selection
      .transition()
      .duration(halfDuration)
      .delay(options.delay || 0)
      .ease(d3.easeSinInOut)
      .attr('transform', `scale(${scaleAmount})`)
      .transition()
      .duration(halfDuration)
      .ease(d3.easeSinInOut)
      .attr('transform', 'scale(1)')
      .on('end', options.onEnd || null);
  }

  /**
   * Bounce animation
   */
  public bounce(selection: D3Selection, options: AnimationOptions): void {
    const bounceEasing = (t: number): number => {
      if (t < 1 / 2.75) {
        return 7.5625 * t * t;
      } else if (t < 2 / 2.75) {
        return 7.5625 * (t -= 1.5 / 2.75) * t + 0.75;
      } else if (t < 2.5 / 2.75) {
        return 7.5625 * (t -= 2.25 / 2.75) * t + 0.9375;
      } else {
        return 7.5625 * (t -= 2.625 / 2.75) * t + 0.984375;
      }
    };

    const bounceOptions = {
      ...options,
      easing: bounceEasing,
    };

    this.createTransition(selection, bounceOptions);
  }

  /**
   * Chain multiple animations
   */
  public chain(
    animations: Array<{
      selection: D3Selection;
      animator: (selection: D3Selection, options: AnimationOptions) => void;
      options: AnimationOptions;
    }>
  ): void {
    let totalDelay = 0;

    animations.forEach(({ selection, animator, options }) => {
      const chainedOptions = {
        ...options,
        delay: totalDelay + (options.delay || 0),
      };

      animator(selection, chainedOptions);
      totalDelay += options.duration;
    });
  }

  /**
   * Cancel all active transitions
   */
  public cancelAll(): void {
    this.activeTransitions.forEach((transition) => {
      transition.interrupt();
    });
    this.activeTransitions.clear();
  }

  /**
   * Create a custom interpolator-based animation
   */
  public customInterpolate<T>(
    selection: D3Selection,
    attrName: string,
    interpolator: (t: number) => T,
    options: AnimationOptions
  ): void {
    const transition = this.createTransition(selection, options);

    transition.attrTween(attrName, () => interpolator);
  }

  /**
   * Animate along a path
   */
  public animateAlongPath(
    selection: D3Selection,
    path: SVGPathElement,
    options: AnimationOptions
  ): void {
    const pathLength = path.getTotalLength();

    const transition = this.createTransition(selection, options);

    transition.attrTween('transform', () => {
      return (t: number) => {
        const point = path.getPointAtLength(t * pathLength);
        return `translate(${point.x}, ${point.y})`;
      };
    });
  }

  /**
   * Create entrance animation
   */
  public enter(
    selection: D3Selection,
    animationType: 'fade' | 'slide' | 'scale' | 'bounce',
    options: AnimationOptions
  ): void {
    switch (animationType) {
      case 'fade':
        this.fadeIn(selection, options);
        break;
      case 'slide':
        this.slide(selection, -100, 0, 0, 0, options);
        break;
      case 'scale':
        this.scale(selection, 0, 1, options);
        break;
      case 'bounce':
        selection.attr('transform', 'scale(0)');
        this.bounce(selection, options);
        break;
    }
  }

  /**
   * Create exit animation
   */
  public exit(
    selection: D3Selection,
    animationType: 'fade' | 'slide' | 'scale',
    options: AnimationOptions,
    onComplete?: () => void
  ): void {
    const exitOptions = {
      ...options,
      onEnd: () => {
        selection.remove();
        if (onComplete) onComplete();
        if (options.onEnd) options.onEnd();
      },
    };

    switch (animationType) {
      case 'fade':
        this.fadeOut(selection, exitOptions);
        break;
      case 'slide':
        this.slide(selection, 0, 100, 0, 0, exitOptions);
        break;
      case 'scale':
        this.scale(selection, 1, 0, exitOptions);
        break;
    }
  }

  /**
   * Progress-based animation with callback
   */
  public progressAnimation(
    duration: number,
    onProgress: (progress: number) => void,
    easing?: (t: number) => number
  ): void {
    const startTime = Date.now();
    const easingFunc = easing || d3.easeLinear;

    const tick = () => {
      const elapsed = Date.now() - startTime;
      const rawProgress = Math.min(elapsed / duration, 1);
      const easedProgress = easingFunc(rawProgress);

      onProgress(easedProgress);

      if (rawProgress < 1) {
        requestAnimationFrame(tick);
      }
    };

    requestAnimationFrame(tick);
  }
}

// Export singleton instance
export const animationEngine = new AnimationEngine();

// Export common easing functions
export const Easing = {
  linear: d3.easeLinear,
  quad: d3.easeQuad,
  cubic: d3.easeCubic,
  sin: d3.easeSin,
  exp: d3.easeExp,
  circle: d3.easeCircle,
  elastic: d3.easeElastic,
  back: d3.easeBack,
  bounce: d3.easeBounce,
  quadIn: d3.easeQuadIn,
  quadOut: d3.easeQuadOut,
  quadInOut: d3.easeQuadInOut,
  cubicIn: d3.easeCubicIn,
  cubicOut: d3.easeCubicOut,
  cubicInOut: d3.easeCubicInOut,
};

export default AnimationEngine;
