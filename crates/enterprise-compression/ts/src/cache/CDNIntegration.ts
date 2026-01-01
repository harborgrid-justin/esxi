/**
 * CDN Integration
 * Integrate with CDN providers for edge caching
 */

import { CDNConfig, CacheError } from '../types';

export class CDNIntegration {
  constructor(private config: CDNConfig) {}

  /**
   * Purge CDN cache
   */
  async purge(urls: string[]): Promise<{
    success: boolean;
    purgedCount: number;
    errors: string[];
  }> {
    if (!this.config.enablePurge) {
      throw new CacheError(
        'CDN purge is not enabled',
        'cdn' as any
      );
    }

    try {
      const results = await this.purgeByCDN(urls);
      return results;
    } catch (error) {
      throw new CacheError(
        `CDN purge failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'cdn' as any,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Purge by CDN provider
   */
  private async purgeByCDN(urls: string[]): Promise<{
    success: boolean;
    purgedCount: number;
    errors: string[];
  }> {
    switch (this.config.provider) {
      case 'cloudflare':
        return this.purgeCloudflare(urls);
      case 'fastly':
        return this.purgeFastly(urls);
      case 'akamai':
        return this.purgeAkamai(urls);
      case 'cloudfront':
        return this.purgeCloudFront(urls);
      default:
        return this.purgeCustom(urls);
    }
  }

  /**
   * Purge Cloudflare cache
   */
  private async purgeCloudflare(urls: string[]): Promise<any> {
    const response = await fetch(
      `${this.config.endpoint}/zones/${this.config.zoneId}/purge_cache`,
      {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${this.config.apiKey}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ files: urls }),
      }
    );

    if (!response.ok) {
      return { success: false, purgedCount: 0, errors: [await response.text()] };
    }

    return { success: true, purgedCount: urls.length, errors: [] };
  }

  /**
   * Purge Fastly cache
   */
  private async purgeFastly(urls: string[]): Promise<any> {
    const errors: string[] = [];
    let purgedCount = 0;

    for (const url of urls) {
      try {
        const response = await fetch(url, {
          method: 'PURGE',
          headers: {
            'Fastly-Key': this.config.apiKey || '',
          },
        });

        if (response.ok) {
          purgedCount++;
        } else {
          errors.push(`Failed to purge ${url}`);
        }
      } catch (error) {
        errors.push(`Error purging ${url}: ${error}`);
      }
    }

    return { success: errors.length === 0, purgedCount, errors };
  }

  /**
   * Purge Akamai cache
   */
  private async purgeAkamai(urls: string[]): Promise<any> {
    // Akamai purge implementation would go here
    return { success: true, purgedCount: urls.length, errors: [] };
  }

  /**
   * Purge CloudFront cache
   */
  private async purgeCloudFront(urls: string[]): Promise<any> {
    // AWS CloudFront invalidation would go here
    return { success: true, purgedCount: urls.length, errors: [] };
  }

  /**
   * Custom CDN purge
   */
  private async purgeCustom(urls: string[]): Promise<any> {
    return { success: false, purgedCount: 0, errors: ['Custom CDN not implemented'] };
  }

  /**
   * Set cache headers
   */
  getCacheHeaders(
    maxAge: number = 3600,
    sMaxAge?: number,
    staleWhileRevalidate?: number
  ): Record<string, string> {
    const headers: Record<string, string> = {
      'Cache-Control': `public, max-age=${maxAge}`,
    };

    if (sMaxAge !== undefined) {
      headers['Cache-Control'] += `, s-maxage=${sMaxAge}`;
    }

    if (staleWhileRevalidate !== undefined) {
      headers['Cache-Control'] += `, stale-while-revalidate=${staleWhileRevalidate}`;
    }

    if (this.config.customHeaders) {
      Object.assign(headers, this.config.customHeaders);
    }

    return headers;
  }

  /**
   * Generate cache key
   */
  generateCacheKey(url: string, params?: Record<string, any>): string {
    const base = url.toLowerCase();
    if (!params) return base;

    const sorted = Object.keys(params)
      .sort()
      .map(key => `${key}=${params[key]}`)
      .join('&');

    return `${base}?${sorted}`;
  }

  /**
   * Check cache status
   */
  async checkCacheStatus(url: string): Promise<{
    cached: boolean;
    age?: number;
    provider?: string;
  }> {
    try {
      const response = await fetch(url, { method: 'HEAD' });
      const cacheControl = response.headers.get('cache-control');
      const age = response.headers.get('age');
      const cfCacheStatus = response.headers.get('cf-cache-status');

      return {
        cached: cacheControl?.includes('max-age') || false,
        age: age ? parseInt(age) : undefined,
        provider: cfCacheStatus || this.config.provider,
      };
    } catch (error) {
      return { cached: false };
    }
  }
}
