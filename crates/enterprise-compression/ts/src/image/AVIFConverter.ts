/**
 * AVIF Image Converter
 * Convert images to AVIF format with advanced compression
 */

import * as sharp from 'sharp';
import { ImageFormat, ImageOptimizationResult, OptimizationError } from '../types';

export class AVIFConverter {
  private static readonly DEFAULT_QUALITY = 75;
  private static readonly DEFAULT_EFFORT = 4;

  /**
   * Convert image to AVIF
   */
  async convert(
    image: Buffer,
    options: {
      quality?: number;
      lossless?: boolean;
      effort?: number;
      chromaSubsampling?: '4:2:0' | '4:4:4';
      bitdepth?: 8 | 10 | 12;
    } = {}
  ): Promise<ImageOptimizationResult> {
    const startTime = performance.now();

    try {
      const originalSize = image.length;
      const metadata = await sharp(image).metadata();

      const avifOptions: sharp.AvifOptions = {
        quality: options.quality || AVIFConverter.DEFAULT_QUALITY,
        lossless: options.lossless || false,
        effort: options.effort || AVIFConverter.DEFAULT_EFFORT,
        chromaSubsampling: options.chromaSubsampling || '4:2:0',
      };

      const { data, info } = await sharp(image)
        .avif(avifOptions)
        .toBuffer({ resolveWithObject: true });

      const optimizedSize = data.length;
      const duration = performance.now() - startTime;
      const savingsPercent = ((originalSize - optimizedSize) / originalSize) * 100;

      return {
        optimized: data,
        originalSize,
        optimizedSize,
        savingsPercent,
        format: ImageFormat.AVIF,
        width: info.width,
        height: info.height,
        duration,
        metadata: {
          format: ImageFormat.AVIF,
          width: info.width,
          height: info.height,
          hasAlpha: metadata.hasAlpha || false,
          colorSpace: metadata.space || 'srgb',
        },
      };
    } catch (error) {
      throw new OptimizationError(
        `AVIF conversion failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'avif-conversion',
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

    return this.convert(image, {
      quality: hasAlpha ? 80 : 75,
      chromaSubsampling: hasAlpha ? '4:4:4' : '4:2:0',
      effort: 6,
    });
  }

  /**
   * Convert with maximum quality
   */
  async convertHighQuality(image: Buffer): Promise<ImageOptimizationResult> {
    return this.convert(image, {
      quality: 90,
      chromaSubsampling: '4:4:4',
      effort: 9,
    });
  }

  /**
   * Convert with lossless compression
   */
  async convertLossless(image: Buffer): Promise<ImageOptimizationResult> {
    return this.convert(image, {
      lossless: true,
      effort: 9,
    });
  }

  /**
   * Batch convert images to AVIF
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
   * Compare AVIF vs WebP vs JPEG
   */
  async compareFormats(image: Buffer): Promise<{
    avif: ImageOptimizationResult;
    webp: ImageOptimizationResult;
    jpeg: ImageOptimizationResult;
    recommendation: ImageFormat;
  }> {
    const [avif, webp, jpeg] = await Promise.all([
      sharp(image).avif({ quality: 75 }).toBuffer({ resolveWithObject: true }),
      sharp(image).webp({ quality: 80 }).toBuffer({ resolveWithObject: true }),
      sharp(image).jpeg({ quality: 85 }).toBuffer({ resolveWithObject: true }),
    ]);

    const results = {
      avif: this.createResult(image.length, avif, ImageFormat.AVIF),
      webp: this.createResult(image.length, webp, ImageFormat.WEBP),
      jpeg: this.createResult(image.length, jpeg, ImageFormat.JPEG),
    };

    // Recommend smallest format
    const sizes = [
      { format: ImageFormat.AVIF, size: avif.data.length },
      { format: ImageFormat.WEBP, size: webp.data.length },
      { format: ImageFormat.JPEG, size: jpeg.data.length },
    ];

    sizes.sort((a, b) => a.size - b.size);

    return {
      ...results,
      recommendation: sizes[0].format,
    };
  }

  /**
   * Create optimization result
   */
  private createResult(
    originalSize: number,
    result: { data: Buffer; info: sharp.OutputInfo },
    format: ImageFormat
  ): ImageOptimizationResult {
    const optimizedSize = result.data.length;
    const savingsPercent = ((originalSize - optimizedSize) / originalSize) * 100;

    return {
      optimized: result.data,
      originalSize,
      optimizedSize,
      savingsPercent,
      format,
      width: result.info.width,
      height: result.info.height,
      duration: 0,
      metadata: {
        format,
        width: result.info.width,
        height: result.info.height,
        hasAlpha: false,
        colorSpace: 'srgb',
      },
    };
  }

  /**
   * Check AVIF support
   */
  isAVIFSupported(): boolean {
    // Server-side check
    if (typeof window === 'undefined') {
      return true;
    }

    // Client-side browser check
    const avif = new Image();
    avif.src = 'data:image/avif;base64,AAAAIGZ0eXBhdmlmAAAAAGF2aWZtaWYxbWlhZk1BMUIAAADybWV0YQAAAAAAAAAoaGRscgAAAAAAAAAAcGljdAAAAAAAAAAAAAAAAGxpYmF2aWYAAAAADnBpdG0AAAAAAAEAAAAeaWxvYwAAAABEAAABAAEAAAABAAABGgAAAB0AAAAoaWluZgAAAAAAAQAAABppbmZlAgAAAAABAABhdjAxQ29sb3IAAAAAamlwcnAAAABLaXBjbwAAABRpc3BlAAAAAAAAAAIAAAACAAAAEHBpeGkAAAAAAwgICAAAAAxhdjFDgQ0MAAAAABNjb2xybmNseAACAAIAAYAAAAAXaXBtYQAAAAAAAAABAAEEAQKDBAAAACVtZGF0EgAKCBgANogQEAwgMg8f8D///8WfhwB8+ErK42A=';

    return avif.complete || avif.width > 0;
  }

  /**
   * Create responsive AVIF set
   */
  async createResponsiveSet(
    image: Buffer,
    sizes: number[] = [320, 640, 960, 1280, 1920]
  ): Promise<Map<number, Buffer>> {
    const results = new Map<number, Buffer>();

    for (const size of sizes) {
      const resized = await sharp(image)
        .resize(size, null, { withoutEnlargement: true })
        .avif({ quality: 75, effort: 6 })
        .toBuffer();

      results.set(size, resized);
    }

    return results;
  }

  /**
   * Optimize AVIF with progressive quality
   */
  async optimizeProgressive(
    image: Buffer,
    targetSize: number
  ): Promise<ImageOptimizationResult> {
    let quality = 90;
    let result: ImageOptimizationResult;

    do {
      result = await this.convert(image, { quality });
      quality -= 5;
    } while (result.optimizedSize > targetSize && quality > 20);

    return result;
  }

  /**
   * Get AVIF metadata
   */
  async getAVIFInfo(avif: Buffer): Promise<{
    width: number;
    height: number;
    hasAlpha: boolean;
    bitDepth: number;
    colorSpace: string;
  }> {
    const metadata = await sharp(avif).metadata();

    return {
      width: metadata.width || 0,
      height: metadata.height || 0,
      hasAlpha: metadata.hasAlpha || false,
      bitDepth: metadata.depth ? parseInt(metadata.depth.replace('uchar', '8')) : 8,
      colorSpace: metadata.space || 'srgb',
    };
  }
}
