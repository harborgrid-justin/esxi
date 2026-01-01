/**
 * Thumbnail Generator
 * Generate responsive thumbnails with multiple sizes
 */

import * as sharp from 'sharp';
import {
  ThumbnailConfig,
  ThumbnailSize,
  ResponsiveImageSet,
  ImageFormat,
  ImageMetadata,
  OptimizationError,
} from '../types';

export class ThumbnailGenerator {
  private static readonly DEFAULT_QUALITY = 85;
  private static readonly DEFAULT_FORMAT = ImageFormat.WEBP;

  /**
   * Generate thumbnails from image
   */
  async generate(
    image: Buffer,
    config: ThumbnailConfig
  ): Promise<Map<string, Buffer>> {
    try {
      const thumbnails = new Map<string, Buffer>();

      for (const size of config.sizes) {
        const thumbnail = await this.generateSingle(image, size, config);
        thumbnails.set(size.suffix, thumbnail);

        // Generate retina version if requested
        if (config.retina) {
          const retinaThumbnail = await this.generateRetina(image, size, config);
          thumbnails.set(`${size.suffix}@2x`, retinaThumbnail);
        }
      }

      return thumbnails;
    } catch (error) {
      throw new OptimizationError(
        `Thumbnail generation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'thumbnail-generation',
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Generate single thumbnail
   */
  private async generateSingle(
    image: Buffer,
    size: ThumbnailSize,
    config: ThumbnailConfig
  ): Promise<Buffer> {
    let pipeline = sharp(image);

    // Resize
    pipeline = pipeline.resize(size.width, size.height, {
      fit: size.fit || 'cover',
      position: 'center',
      withoutEnlargement: true,
    });

    // Apply format
    pipeline = this.applyFormat(pipeline, config.format, config.quality);

    return pipeline.toBuffer();
  }

  /**
   * Generate retina thumbnail (2x resolution)
   */
  private async generateRetina(
    image: Buffer,
    size: ThumbnailSize,
    config: ThumbnailConfig
  ): Promise<Buffer> {
    let pipeline = sharp(image);

    const retinaSize: ThumbnailSize = {
      ...size,
      width: size.width * 2,
      height: size.height ? size.height * 2 : undefined,
    };

    pipeline = pipeline.resize(retinaSize.width, retinaSize.height, {
      fit: size.fit || 'cover',
      position: 'center',
      withoutEnlargement: true,
    });

    pipeline = this.applyFormat(pipeline, config.format, config.quality);

    return pipeline.toBuffer();
  }

  /**
   * Apply format-specific options
   */
  private applyFormat(
    pipeline: sharp.Sharp,
    format: ImageFormat,
    quality: number
  ): sharp.Sharp {
    switch (format) {
      case ImageFormat.JPEG:
        return pipeline.jpeg({ quality, progressive: true });
      case ImageFormat.PNG:
        return pipeline.png({ compressionLevel: 9, quality });
      case ImageFormat.WEBP:
        return pipeline.webp({ quality, effort: 4 });
      case ImageFormat.AVIF:
        return pipeline.avif({ quality, effort: 4 });
      default:
        return pipeline;
    }
  }

  /**
   * Generate responsive image set with srcset
   */
  async generateResponsive(
    image: Buffer,
    sizes: number[] = [320, 640, 960, 1280, 1920],
    format: ImageFormat = ImageFormat.WEBP
  ): Promise<ResponsiveImageSet> {
    const thumbnails = new Map<string, Buffer>();
    const srcsetEntries: string[] = [];
    const metadata = await sharp(image).metadata();

    for (const width of sizes) {
      const thumbnail = await sharp(image)
        .resize(width, null, { withoutEnlargement: true })
        .toFormat(format, { quality: ThumbnailGenerator.DEFAULT_QUALITY })
        .toBuffer();

      const suffix = `${width}w`;
      thumbnails.set(suffix, thumbnail);
      srcsetEntries.push(`image-${width}.${format} ${width}w`);
    }

    const srcset = srcsetEntries.join(', ');
    const sizesAttr = `(max-width: ${Math.max(...sizes)}px) 100vw, ${Math.max(...sizes)}px`;

    return {
      original: image,
      thumbnails,
      srcset,
      sizes: sizesAttr,
      metadata: {
        format: (metadata.format as ImageFormat) || format,
        width: metadata.width || 0,
        height: metadata.height || 0,
        hasAlpha: metadata.hasAlpha || false,
        colorSpace: metadata.space || 'srgb',
      },
    };
  }

  /**
   * Generate art-directed responsive set (different crops)
   */
  async generateArtDirected(
    image: Buffer,
    configs: Array<{
      size: number;
      crop: { x: number; y: number; width: number; height: number };
      format?: ImageFormat;
    }>
  ): Promise<Map<string, Buffer>> {
    const results = new Map<string, Buffer>();

    for (const config of configs) {
      const cropped = await sharp(image)
        .extract(config.crop)
        .resize(config.size, null, { withoutEnlargement: true })
        .toFormat(config.format || ThumbnailGenerator.DEFAULT_FORMAT, {
          quality: ThumbnailGenerator.DEFAULT_QUALITY,
        })
        .toBuffer();

      results.set(`${config.size}w`, cropped);
    }

    return results;
  }

  /**
   * Generate smart crop thumbnails (focus on important regions)
   */
  async generateSmartCrop(
    image: Buffer,
    sizes: ThumbnailSize[]
  ): Promise<Map<string, Buffer>> {
    const results = new Map<string, Buffer>();

    for (const size of sizes) {
      const thumbnail = await sharp(image)
        .resize(size.width, size.height, {
          fit: 'cover',
          position: sharp.strategy.attention, // Focus on high-contrast areas
        })
        .toBuffer();

      results.set(size.suffix, thumbnail);
    }

    return results;
  }

  /**
   * Generate placeholder (tiny blurred image)
   */
  async generatePlaceholder(
    image: Buffer,
    width: number = 20
  ): Promise<{
    buffer: Buffer;
    base64: string;
    width: number;
    height: number;
  }> {
    const placeholder = await sharp(image)
      .resize(width, null, { withoutEnlargement: true })
      .blur(5)
      .jpeg({ quality: 50 })
      .toBuffer({ resolveWithObject: true });

    return {
      buffer: placeholder.data,
      base64: `data:image/jpeg;base64,${placeholder.data.toString('base64')}`,
      width: placeholder.info.width,
      height: placeholder.info.height,
    };
  }

  /**
   * Generate thumbnail with watermark
   */
  async generateWithWatermark(
    image: Buffer,
    watermark: Buffer,
    size: ThumbnailSize,
    options: {
      gravity?: 'northwest' | 'northeast' | 'southwest' | 'southeast' | 'center';
      opacity?: number;
    } = {}
  ): Promise<Buffer> {
    const resized = await sharp(image)
      .resize(size.width, size.height, { fit: size.fit || 'cover' })
      .toBuffer();

    const watermarkResized = await sharp(watermark)
      .resize(Math.floor(size.width * 0.3), null, { withoutEnlargement: true })
      .composite([
        {
          input: await sharp(watermark)
            .resize(Math.floor(size.width * 0.3))
            .toBuffer(),
          blend: 'over',
          gravity: options.gravity || 'southeast',
        },
      ])
      .toBuffer();

    return sharp(resized)
      .composite([
        {
          input: watermarkResized,
          gravity: options.gravity || 'southeast',
          blend: 'over',
        },
      ])
      .toBuffer();
  }

  /**
   * Batch generate thumbnails for multiple images
   */
  async generateBatch(
    images: Buffer[],
    config: ThumbnailConfig,
    concurrency: number = 4
  ): Promise<Array<Map<string, Buffer>>> {
    const results: Array<Map<string, Buffer>> = [];

    for (let i = 0; i < images.length; i += concurrency) {
      const batch = images.slice(i, i + concurrency);
      const batchResults = await Promise.all(
        batch.map(img => this.generate(img, config))
      );
      results.push(...batchResults);
    }

    return results;
  }

  /**
   * Generate thumbnail grid (sprite)
   */
  async generateGrid(
    images: Buffer[],
    columns: number,
    thumbnailSize: number
  ): Promise<Buffer> {
    const rows = Math.ceil(images.length / columns);

    // Resize all images to thumbnail size
    const thumbnails = await Promise.all(
      images.map(img =>
        sharp(img)
          .resize(thumbnailSize, thumbnailSize, { fit: 'cover' })
          .toBuffer()
      )
    );

    // Create composite
    const composites: sharp.OverlayOptions[] = [];
    for (let i = 0; i < thumbnails.length; i++) {
      const row = Math.floor(i / columns);
      const col = i % columns;

      composites.push({
        input: thumbnails[i],
        left: col * thumbnailSize,
        top: row * thumbnailSize,
      });
    }

    return sharp({
      create: {
        width: columns * thumbnailSize,
        height: rows * thumbnailSize,
        channels: 4,
        background: { r: 255, g: 255, b: 255, alpha: 1 },
      },
    })
      .composite(composites)
      .png()
      .toBuffer();
  }
}
