/**
 * Geocoding Service
 * Address lookup and reverse geocoding
 */

import { Position, GeocodeRequest, GeocodeResult, Bounds } from '../types';

export class GeocodingService {
  private apiKey?: string;
  private baseUrl: string;

  constructor(options: { apiKey?: string; baseUrl?: string } = {}) {
    this.apiKey = options.apiKey;
    this.baseUrl = options.baseUrl || 'https://nominatim.openstreetmap.org';
  }

  /**
   * Geocode an address to coordinates
   */
  async geocode(request: GeocodeRequest): Promise<GeocodeResult[]> {
    if (!request.address) {
      throw new Error('Address is required for geocoding');
    }

    const params = new URLSearchParams({
      q: request.address,
      format: 'json',
      addressdetails: '1',
      limit: (request.maxResults || 5).toString(),
    });

    if (request.bounds) {
      const { minX, minY, maxX, maxY } = request.bounds;
      params.append('viewbox', `${minX},${minY},${maxX},${maxY}`);
      params.append('bounded', '1');
    }

    if (request.countries && request.countries.length > 0) {
      params.append('countrycodes', request.countries.join(','));
    }

    const url = `${this.baseUrl}/search?${params}`;

    try {
      const response = await fetch(url);
      const data = await response.json();

      return data.map((item: any) => this.parseGeocodeResult(item));
    } catch (error) {
      console.error('Geocoding error:', error);
      return [];
    }
  }

  /**
   * Reverse geocode coordinates to address
   */
  async reverseGeocode(location: Position): Promise<GeocodeResult | null> {
    const [lon, lat] = location;

    const params = new URLSearchParams({
      lat: lat.toString(),
      lon: lon.toString(),
      format: 'json',
      addressdetails: '1',
    });

    const url = `${this.baseUrl}/reverse?${params}`;

    try {
      const response = await fetch(url);
      const data = await response.json();

      return this.parseGeocodeResult(data);
    } catch (error) {
      console.error('Reverse geocoding error:', error);
      return null;
    }
  }

  /**
   * Batch geocode multiple addresses
   */
  async batchGeocode(addresses: string[]): Promise<GeocodeResult[][]> {
    const results: GeocodeResult[][] = [];

    for (const address of addresses) {
      const result = await this.geocode({ address });
      results.push(result);

      // Rate limiting
      await this.delay(1000);
    }

    return results;
  }

  /**
   * Search for places by name
   */
  async searchPlaces(
    query: string,
    options: {
      type?: string;
      bounds?: Bounds;
      maxResults?: number;
    } = {}
  ): Promise<GeocodeResult[]> {
    return this.geocode({
      address: query,
      bounds: options.bounds,
      maxResults: options.maxResults,
    });
  }

  /**
   * Parse geocoding result from API response
   */
  private parseGeocodeResult(data: any): GeocodeResult {
    const lat = parseFloat(data.lat);
    const lon = parseFloat(data.lon);

    const bounds = data.boundingbox
      ? {
          minY: parseFloat(data.boundingbox[0]),
          maxY: parseFloat(data.boundingbox[1]),
          minX: parseFloat(data.boundingbox[2]),
          maxX: parseFloat(data.boundingbox[3]),
        }
      : undefined;

    return {
      address: data.display_name,
      location: [lon, lat],
      confidence: parseFloat(data.importance || 0.5),
      type: data.type || 'unknown',
      bounds,
      attributes: data.address || {},
    };
  }

  /**
   * Delay helper for rate limiting
   */
  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  /**
   * Suggest addresses as user types
   */
  async autocomplete(
    partial: string,
    options: { maxResults?: number } = {}
  ): Promise<string[]> {
    const results = await this.geocode({
      address: partial,
      maxResults: options.maxResults || 5,
    });

    return results.map((r) => r.address);
  }

  /**
   * Get place details by ID
   */
  async getPlaceDetails(placeId: string): Promise<GeocodeResult | null> {
    // Implementation would depend on geocoding provider
    return null;
  }
}
