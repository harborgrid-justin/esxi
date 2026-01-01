/**
 * Lazy Load Manager
 * Progressive image loading with placeholders
 */

import * as sharp from 'sharp';
import { ImageFormat, OptimizationError } from '../types';

export class LazyLoadManager {
  private static readonly BLUR_RADIUS = 10;
  private static readonly PLACEHOLDER_WIDTH = 20;
  private static readonly LQIP_QUALITY = 20;

  /**
   * Generate Low Quality Image Placeholder (LQIP)
   */
  async generateLQIP(
    image: Buffer,
    width: number = LazyLoadManager.PLACEHOLDER_WIDTH
  ): Promise<{
    buffer: Buffer;
    base64: string;
    dataUrl: string;
    width: number;
    height: number;
  }> {
    try {
      const result = await sharp(image)
        .resize(width, null, { withoutEnlargement: true })
        .jpeg({ quality: LazyLoadManager.LQIP_QUALITY })
        .toBuffer({ resolveWithObject: true });

      const base64 = result.data.toString('base64');
      const dataUrl = `data:image/jpeg;base64,${base64}`;

      return {
        buffer: result.data,
        base64,
        dataUrl,
        width: result.info.width,
        height: result.info.height,
      };
    } catch (error) {
      throw new OptimizationError(
        `LQIP generation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'lqip-generation',
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Generate blurred placeholder
   */
  async generateBlurredPlaceholder(
    image: Buffer,
    width: number = 40,
    blurRadius: number = LazyLoadManager.BLUR_RADIUS
  ): Promise<{
    buffer: Buffer;
    base64: string;
    dataUrl: string;
  }> {
    try {
      const blurred = await sharp(image)
        .resize(width, null, { withoutEnlargement: true })
        .blur(blurRadius)
        .jpeg({ quality: 30 })
        .toBuffer();

      const base64 = blurred.toString('base64');
      const dataUrl = `data:image/jpeg;base64,${base64}`;

      return {
        buffer: blurred,
        base64,
        dataUrl,
      };
    } catch (error) {
      throw new OptimizationError(
        `Blurred placeholder generation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'blur-placeholder',
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Generate dominant color placeholder
   */
  async generateColorPlaceholder(image: Buffer): Promise<{
    hex: string;
    rgb: { r: number; g: number; b: number };
    svg: string;
  }> {
    try {
      const { data, info } = await sharp(image)
        .resize(1, 1, { fit: 'cover' })
        .raw()
        .toBuffer({ resolveWithObject: true });

      const r = data[0];
      const g = data[1];
      const b = data[2];

      const hex = `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;

      const metadata = await sharp(image).metadata();
      const width = metadata.width || 1;
      const height = metadata.height || 1;

      const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}"><rect width="100%" height="100%" fill="${hex}"/></svg>`;

      return {
        hex,
        rgb: { r, g, b },
        svg,
      };
    } catch (error) {
      throw new OptimizationError(
        `Color placeholder generation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'color-placeholder',
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Generate progressive loading set
   */
  async generateProgressiveSet(
    image: Buffer,
    stages: number[] = [10, 30, 60, 100]
  ): Promise<Map<number, Buffer>> {
    const results = new Map<number, Buffer>();
    const metadata = await sharp(image).metadata();
    const originalWidth = metadata.width || 1;

    for (const percentage of stages) {
      const width = Math.floor((originalWidth * percentage) / 100);
      const quality = Math.max(20, percentage);

      const progressive = await sharp(image)
        .resize(width, null, { withoutEnlargement: true })
        .jpeg({ quality, progressive: true })
        .toBuffer();

      results.set(percentage, progressive);
    }

    return results;
  }

  /**
   * Generate traced SVG placeholder
   */
  async generateTracedSVG(
    image: Buffer,
    options: {
      threshold?: number;
      blur?: number;
      colors?: number;
    } = {}
  ): Promise<string> {
    try {
      const threshold = options.threshold || 120;
      const blur = options.blur || 0;

      // Simplify to high-contrast black & white
      const processed = await sharp(image)
        .resize(100, null, { withoutEnlargement: true })
        .blur(blur)
        .threshold(threshold)
        .toBuffer({ resolveWithObject: true });

      const metadata = await sharp(image).metadata();
      const width = metadata.width || 1;
      const height = metadata.height || 1;

      // This is a simplified version - a full implementation would
      // use a proper tracing algorithm like potrace
      const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}">
        <rect width="100%" height="100%" fill="#f0f0f0"/>
        <filter id="blur"><feGaussianBlur stdDeviation="5"/></filter>
        <image filter="url(#blur)" width="100%" height="100%" href="data:image/png;base64,${processed.data.toString('base64')}"/>
      </svg>`;

      return svg;
    } catch (error) {
      throw new OptimizationError(
        `Traced SVG generation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'traced-svg',
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Generate lazy loading HTML
   */
  generateLazyHTML(
    src: string,
    placeholder: string,
    alt: string = '',
    className: string = 'lazy-image'
  ): string {
    return `<img
      class="${className}"
      src="${placeholder}"
      data-src="${src}"
      alt="${alt}"
      loading="lazy"
      decoding="async"
    />`;
  }

  /**
   * Generate IntersectionObserver script
   */
  generateObserverScript(): string {
    return `
      const imageObserver = new IntersectionObserver((entries, observer) => {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            const img = entry.target;
            const src = img.dataset.src;

            if (src) {
              img.src = src;
              img.classList.add('loaded');
              observer.unobserve(img);
            }
          }
        });
      }, {
        rootMargin: '50px 0px',
        threshold: 0.01
      });

      document.querySelectorAll('img[data-src]').forEach(img => {
        imageObserver.observe(img);
      });
    `;
  }

  /**
   * Generate complete lazy loading package
   */
  async generateLazyPackage(
    image: Buffer,
    fullImagePath: string
  ): Promise<{
    placeholder: {
      buffer: Buffer;
      dataUrl: string;
    };
    dominantColor: string;
    html: string;
    css: string;
    script: string;
  }> {
    const [placeholder, color] = await Promise.all([
      this.generateLQIP(image),
      this.generateColorPlaceholder(image),
    ]);

    const html = this.generateLazyHTML(fullImagePath, placeholder.dataUrl);

    const css = `
      .lazy-image {
        opacity: 0;
        transition: opacity 0.3s ease-in-out;
        background-color: ${color.hex};
      }

      .lazy-image.loaded {
        opacity: 1;
      }

      .lazy-image:not(.loaded) {
        filter: blur(10px);
        transform: scale(1.05);
      }
    `;

    const script = this.generateObserverScript();

    return {
      placeholder: {
        buffer: placeholder.buffer,
        dataUrl: placeholder.dataUrl,
      },
      dominantColor: color.hex,
      html,
      css,
      script,
    };
  }

  /**
   * Generate responsive lazy loading set
   */
  async generateResponsiveLazy(
    image: Buffer,
    sizes: number[] = [320, 640, 960, 1280, 1920]
  ): Promise<{
    placeholder: string;
    srcset: string;
    sizes: string;
    html: string;
  }> {
    const placeholder = await this.generateLQIP(image, 20);
    const srcsetEntries: string[] = [];

    for (const width of sizes) {
      srcsetEntries.push(`/images/image-${width}.webp ${width}w`);
    }

    const srcset = srcsetEntries.join(', ');
    const sizesAttr = '(max-width: 1920px) 100vw, 1920px';

    const html = `<img
      class="lazy-image responsive"
      src="${placeholder.dataUrl}"
      data-srcset="${srcset}"
      data-sizes="${sizesAttr}"
      alt=""
      loading="lazy"
      decoding="async"
    />`;

    return {
      placeholder: placeholder.dataUrl,
      srcset,
      sizes: sizesAttr,
      html,
    };
  }

  /**
   * Generate fade-in animation CSS
   */
  generateFadeInCSS(): string {
    return `
      @keyframes fadeIn {
        from {
          opacity: 0;
          filter: blur(20px);
        }
        to {
          opacity: 1;
          filter: blur(0);
        }
      }

      .lazy-image.loaded {
        animation: fadeIn 0.5s ease-in-out;
      }
    `;
  }
}
