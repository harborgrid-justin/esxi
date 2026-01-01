/**
 * Image Optimization Engine
 * Comprehensive image optimization with multiple format support
 */

import * as sharp from 'sharp';
import { createHash } from 'crypto';
import {
  ImageOptimizationConfig,
  ImageOptimizationResult,
  ImageFormat,
  ImageMetadata,
  OptimizationError,
} from '../types';

export class ImageOptimizer {
  private static readonly VERSION = '1.0.0';
  private static readonly DEFAULT_QUALITY = 85;
  private static readonly DEFAULT_EFFORT = 4;

  /**
   * Optimize image with comprehensive options
   */
  async optimize(
    image: Buffer,
    config: ImageOptimizationConfig
  ): Promise<ImageOptimizationResult> {
    const startTime = performance.now();

    try {
      if (!image || image.length === 0) {
        throw new Error('Input image is empty');
      }

      const originalSize = image.length;
      let pipeline = sharp(image);

      // Get original metadata
      const metadata = await pipeline.metadata();

      // Apply transformations
      if (config.width || config.height) {
        pipeline = pipeline.resize(config.width, config.height, {
          fit: config.fit || 'cover',
          withoutEnlargement: true,
        });
      }

      // Progressive loading
      if (config.progressive) {
        pipeline = pipeline.progressive();
      }

      // Strip metadata if requested
      if (config.stripMetadata) {
        pipeline = pipeline.withMetadata({
          orientation: metadata.orientation,
        });
      }

      // Apply format-specific optimization
      pipeline = this.applyFormatOptimization(
        pipeline,
        config.format || ImageFormat.JPEG,
        config
      );

      // Generate optimized image
      const { data, info } = await pipeline.toBuffer({ resolveWithObject: true });

      const optimizedSize = data.length;
      const duration = performance.now() - startTime;
      const savingsPercent = ((originalSize - optimizedSize) / originalSize) * 100;

      return {
        optimized: data,
        originalSize,
        optimizedSize,
        savingsPercent,
        format: config.format || ImageFormat.JPEG,
        width: info.width,
        height: info.height,
        duration,
        metadata: {
          format: info.format as ImageFormat,
          width: info.width,
          height: info.height,
          hasAlpha: info.hasAlpha,
          colorSpace: metadata.space || 'srgb',
          density: metadata.density,
          orientation: metadata.orientation,
        },
      };
    } catch (error) {
      throw new OptimizationError(
        `Image optimization failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'image-optimization',
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Apply format-specific optimization
   */
  private applyFormatOptimization(
    pipeline: sharp.Sharp,
    format: ImageFormat,
    config: ImageOptimizationConfig
  ): sharp.Sharp {
    const quality = config.quality || ImageOptimizer.DEFAULT_QUALITY;
    const effort = config.effort || ImageOptimizer.DEFAULT_EFFORT;

    switch (format) {
      case ImageFormat.JPEG:
        return pipeline.jpeg({
          quality,
          progressive: config.progressive !== false,
          mozjpeg: true,
          chromaSubsampling: quality >= 90 ? '4:4:4' : '4:2:0',
        });

      case ImageFormat.PNG:
        return pipeline.png({
          compressionLevel: 9,
          adaptiveFiltering: true,
          palette: !config.lossless,
          quality,
          effort,
        });

      case ImageFormat.WEBP:
        return pipeline.webp({
          quality,
          lossless: config.lossless || false,
          nearLossless: !config.lossless,
          effort,
          smartSubsample: true,
        });

      case ImageFormat.AVIF:
        return pipeline.avif({
          quality,
          lossless: config.lossless || false,
          effort,
          chromaSubsampling: '4:2:0',
        });

      default:
        return pipeline;
    }
  }

  /**
   * Batch optimize multiple images
   */
  async optimizeBatch(
    images: Array<{ buffer: Buffer; config: ImageOptimizationConfig }>,
    concurrency: number = 4
  ): Promise<ImageOptimizationResult[]> {
    const results: ImageOptimizationResult[] = [];
    const batches = this.createBatches(images, concurrency);

    for (const batch of batches) {
      const batchResults = await Promise.all(
        batch.map(img => this.optimize(img.buffer, img.config))
      );
      results.push(...batchResults);
    }

    return results;
  }

  /**
   * Auto-optimize with best settings detection
   */
  async autoOptimize(image: Buffer): Promise<ImageOptimizationResult> {
    const metadata = await sharp(image).metadata();

    // Determine best format
    const hasAlpha = metadata.hasAlpha || false;
    const isPhoto = this.isPhotographic(image);

    let format: ImageFormat;
    if (hasAlpha) {
      format = ImageFormat.WEBP; // WebP supports alpha
    } else if (isPhoto) {
      format = ImageFormat.JPEG; // JPEG for photos
    } else {
      format = ImageFormat.PNG; // PNG for graphics
    }

    // Determine quality based on image characteristics
    const quality = this.determineOptimalQuality(image, format);

    return this.optimize(image, {
      format,
      quality,
      progressive: true,
      stripMetadata: true,
    });
  }

  /**
   * Check if image is photographic
   */
  private async isPhotographic(image: Buffer): Promise<boolean> {
    const { data, info } = await sharp(image)
      .resize(100, 100, { fit: 'inside' })
      .raw()
      .toBuffer({ resolveWithObject: true });

    // Calculate color variance
    const pixels = data.length / info.channels;
    let variance = 0;

    for (let i = 0; i < data.length; i += info.channels) {
      const r = data[i];
      const g = data[i + 1];
      const b = data[i + 2];
      const avg = (r + g + b) / 3;
      variance += Math.pow(r - avg, 2) + Math.pow(g - avg, 2) + Math.pow(b - avg, 2);
    }

    variance /= pixels * 3;

    // High variance suggests photographic content
    return variance > 1000;
  }

  /**
   * Determine optimal quality setting
   */
  private async determineOptimalQuality(
    image: Buffer,
    format: ImageFormat
  ): Promise<number> {
    const qualities = [60, 70, 80, 85, 90];
    const results: Array<{ quality: number; size: number; quality_score: number }> = [];

    for (const quality of qualities) {
      const optimized = await this.optimize(image, { format, quality });
      const qualityScore = this.calculateQualityScore(
        image.length,
        optimized.optimizedSize,
        quality
      );

      results.push({
        quality,
        size: optimized.optimizedSize,
        quality_score: qualityScore,
      });
    }

    // Find quality with best quality/size tradeoff
    results.sort((a, b) => b.quality_score - a.quality_score);
    return results[0].quality;
  }

  /**
   * Calculate quality score (balance between size and quality)
   */
  private calculateQualityScore(
    originalSize: number,
    optimizedSize: number,
    quality: number
  ): number {
    const compressionRatio = originalSize / optimizedSize;
    const qualityWeight = quality / 100;

    // Balance compression and quality (70% compression, 30% quality)
    return compressionRatio * 0.7 + qualityWeight * 0.3;
  }

  /**
   * Generate responsive images
   */
  async generateResponsive(
    image: Buffer,
    sizes: number[],
    format: ImageFormat = ImageFormat.WEBP
  ): Promise<Map<number, Buffer>> {
    const results = new Map<number, Buffer>();

    for (const size of sizes) {
      const optimized = await this.optimize(image, {
        format,
        width: size,
        quality: 85,
        progressive: true,
      });

      results.set(size, optimized.optimized);
    }

    return results;
  }

  /**
   * Compare formats for same image
   */
  async compareFormats(image: Buffer): Promise<Array<{
    format: ImageFormat;
    size: number;
    savingsPercent: number;
    duration: number;
  }>> {
    const formats = [
      ImageFormat.JPEG,
      ImageFormat.PNG,
      ImageFormat.WEBP,
      ImageFormat.AVIF,
    ];

    const results = [];

    for (const format of formats) {
      try {
        const result = await this.optimize(image, {
          format,
          quality: 85,
        });

        results.push({
          format,
          size: result.optimizedSize,
          savingsPercent: result.savingsPercent,
          duration: result.duration,
        });
      } catch (error) {
        // Skip formats that fail
        continue;
      }
    }

    return results.sort((a, b) => a.size - b.size);
  }

  /**
   * Extract dominant colors
   */
  async extractDominantColors(image: Buffer, count: number = 5): Promise<Array<{
    r: number;
    g: number;
    b: number;
    hex: string;
    percentage: number;
  }>> {
    const { data, info } = await sharp(image)
      .resize(100, 100, { fit: 'inside' })
      .raw()
      .toBuffer({ resolveWithObject: true });

    const colorMap = new Map<string, number>();
    const totalPixels = data.length / info.channels;

    for (let i = 0; i < data.length; i += info.channels) {
      const r = Math.round(data[i] / 16) * 16;
      const g = Math.round(data[i + 1] / 16) * 16;
      const b = Math.round(data[i + 2] / 16) * 16;
      const key = `${r},${g},${b}`;

      colorMap.set(key, (colorMap.get(key) || 0) + 1);
    }

    const sorted = Array.from(colorMap.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, count);

    return sorted.map(([key, count]) => {
      const [r, g, b] = key.split(',').map(Number);
      const hex = `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;

      return {
        r,
        g,
        b,
        hex,
        percentage: (count / totalPixels) * 100,
      };
    });
  }

  /**
   * Create batches for parallel processing
   */
  private createBatches<T>(items: T[], batchSize: number): T[][] {
    const batches: T[][] = [];
    for (let i = 0; i < items.length; i += batchSize) {
      batches.push(items.slice(i, i + batchSize));
    }
    return batches;
  }

  /**
   * Get image information
   */
  async getInfo(image: Buffer): Promise<ImageMetadata> {
    const metadata = await sharp(image).metadata();

    return {
      format: (metadata.format as ImageFormat) || ImageFormat.JPEG,
      width: metadata.width || 0,
      height: metadata.height || 0,
      hasAlpha: metadata.hasAlpha || false,
      colorSpace: metadata.space || 'srgb',
      density: metadata.density,
      orientation: metadata.orientation,
      originalMetadata: metadata,
    };
  }
}
