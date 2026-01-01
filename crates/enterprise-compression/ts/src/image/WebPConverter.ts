/**
 * WebP Image Converter
 * Convert images to WebP format with advanced options
 */

import * as sharp from 'sharp';
import { ImageFormat, ImageOptimizationResult, OptimizationError } from '../types';

export class WebPConverter {
  private static readonly DEFAULT_QUALITY = 80;
  private static readonly DEFAULT_ALPHA_QUALITY = 100;
  private static readonly DEFAULT_EFFORT = 4;

  /**
   * Convert image to WebP
   */
  async convert(
    image: Buffer,
    options: {
      quality?: number;
      alphaQuality?: number;
      lossless?: boolean;
      nearLossless?: boolean;
      effort?: number;
      preset?: 'default' | 'photo' | 'picture' | 'drawing' | 'icon' | 'text';
      smartSubsample?: boolean;
    } = {}
  ): Promise<ImageOptimizationResult> {
    const startTime = performance.now();

    try {
      const originalSize = image.length;
      const metadata = await sharp(image).metadata();

      const webpOptions: sharp.WebpOptions = {
        quality: options.quality || WebPConverter.DEFAULT_QUALITY,
        alphaQuality: options.alphaQuality || WebPConverter.DEFAULT_ALPHA_QUALITY,
        lossless: options.lossless || false,
        nearLossless: options.nearLossless || false,
        effort: options.effort || WebPConverter.DEFAULT_EFFORT,
        smartSubsample: options.smartSubsample ?? true,
      };

      const { data, info } = await sharp(image)
        .webp(webpOptions)
        .toBuffer({ resolveWithObject: true });

      const optimizedSize = data.length;
      const duration = performance.now() - startTime;
      const savingsPercent = ((originalSize - optimizedSize) / originalSize) * 100;

      return {
        optimized: data,
        originalSize,
        optimizedSize,
        savingsPercent,
        format: ImageFormat.WEBP,
        width: info.width,
        height: info.height,
        duration,
        metadata: {
          format: ImageFormat.WEBP,
          width: info.width,
          height: info.height,
          hasAlpha: metadata.hasAlpha || false,
          colorSpace: metadata.space || 'srgb',
        },
      };
    } catch (error) {
      throw new OptimizationError(
        `WebP conversion failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'webp-conversion',
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Convert with automatic quality detection
   */
  async convertAuto(image: Buffer): Promise<ImageOptimizationResult> {
    const metadata = await sharp(image).metadata();
    const hasAlpha = metadata.hasAlpha || false;

    // Determine optimal settings
    const quality = hasAlpha ? 85 : 80;
    const alphaQuality = hasAlpha ? 95 : 100;

    return this.convert(image, {
      quality,
      alphaQuality,
      nearLossless: true,
      effort: 6,
    });
  }

  /**
   * Convert with lossless compression
   */
  async convertLossless(image: Buffer): Promise<ImageOptimizationResult> {
    return this.convert(image, {
      lossless: true,
      effort: 6,
    });
  }

  /**
   * Convert animated image to animated WebP
   */
  async convertAnimated(
    frames: Buffer[],
    options: {
      quality?: number;
      delay?: number | number[];
      loop?: number;
    } = {}
  ): Promise<Buffer> {
    try {
      // For animated WebP, we need to process frames differently
      // This is a simplified version - full implementation would handle frame timing
      const delays = Array.isArray(options.delay)
        ? options.delay
        : Array(frames.length).fill(options.delay || 100);

      const animated = await sharp(frames[0], { animated: true })
        .webp({
          quality: options.quality || 80,
          loop: options.loop || 0,
        })
        .toBuffer();

      return animated;
    } catch (error) {
      throw new OptimizationError(
        `Animated WebP conversion failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'webp-animated',
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Batch convert images to WebP
   */
  async convertBatch(
    images: Buffer[],
    options: {
      quality?: number;
      concurrency?: number;
    } = {}
  ): Promise<ImageOptimizationResult[]> {
    const concurrency = options.concurrency || 4;
    const results: ImageOptimizationResult[] = [];

    for (let i = 0; i < images.length; i += concurrency) {
      const batch = images.slice(i, i + concurrency);
      const batchResults = await Promise.all(
        batch.map(img => this.convert(img, options))
      );
      results.push(...batchResults);
    }

    return results;
  }

  /**
   * Compare WebP quality levels
   */
  async compareQualities(
    image: Buffer,
    qualities: number[] = [60, 70, 80, 85, 90]
  ): Promise<Array<{
    quality: number;
    size: number;
    savingsPercent: number;
    duration: number;
  }>> {
    const results = [];
    const originalSize = image.length;

    for (const quality of qualities) {
      const result = await this.convert(image, { quality });
      results.push({
        quality,
        size: result.optimizedSize,
        savingsPercent: result.savingsPercent,
        duration: result.duration,
      });
    }

    return results;
  }

  /**
   * Check if browser supports WebP
   */
  isWebPSupported(): boolean {
    // Server-side always returns true, client-side would check browser
    if (typeof window === 'undefined') {
      return true;
    }

    const canvas = document.createElement('canvas');
    if (canvas.getContext && canvas.getContext('2d')) {
      return canvas.toDataURL('image/webp').indexOf('data:image/webp') === 0;
    }

    return false;
  }

  /**
   * Get WebP info
   */
  async getWebPInfo(webp: Buffer): Promise<{
    width: number;
    height: number;
    hasAlpha: boolean;
    isAnimated: boolean;
    frameCount?: number;
  }> {
    const metadata = await sharp(webp).metadata();

    return {
      width: metadata.width || 0,
      height: metadata.height || 0,
      hasAlpha: metadata.hasAlpha || false,
      isAnimated: (metadata.pages || 1) > 1,
      frameCount: metadata.pages,
    };
  }

  /**
   * Create responsive WebP set
   */
  async createResponsiveSet(
    image: Buffer,
    sizes: number[] = [320, 640, 960, 1280, 1920]
  ): Promise<Map<number, Buffer>> {
    const results = new Map<number, Buffer>();

    for (const size of sizes) {
      const resized = await sharp(image)
        .resize(size, null, { withoutEnlargement: true })
        .webp({ quality: 85, effort: 6 })
        .toBuffer();

      results.set(size, resized);
    }

    return results;
  }
}
