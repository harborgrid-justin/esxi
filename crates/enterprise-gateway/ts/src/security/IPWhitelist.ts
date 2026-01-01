/**
 * Enterprise API Gateway - IP Whitelist/Blacklist
 *
 * IP-based access control with CIDR support
 */

import type { IPFilter, GatewayRequest } from '../types';
import { AuthorizationError } from '../types';

export class IPWhitelist {
  private filter: IPFilter;
  private compiledRanges: Array<{ min: bigint; max: bigint }> = [];

  constructor(filter: IPFilter) {
    this.filter = filter;
    this.compileAddresses();
  }

  /**
   * Compile IP addresses and CIDR ranges for efficient matching
   */
  private compileAddresses(): void {
    this.compiledRanges = [];

    for (const address of this.filter.addresses) {
      if (address.includes('/')) {
        // CIDR notation
        const range = this.parseCIDR(address);
        if (range) {
          this.compiledRanges.push(range);
        }
      } else {
        // Single IP address
        const ip = this.ipToNumber(address);
        if (ip !== null) {
          this.compiledRanges.push({ min: ip, max: ip });
        }
      }
    }
  }

  /**
   * Check if IP is allowed
   */
  public isAllowed(request: GatewayRequest): boolean {
    if (!this.filter.enabled) {
      return true;
    }

    const ip = request.ip;
    const ipNum = this.ipToNumber(ip);

    if (ipNum === null) {
      return this.filter.mode === 'blacklist'; // Unknown IPs blocked in whitelist mode
    }

    const isInList = this.compiledRanges.some(
      (range) => ipNum >= range.min && ipNum <= range.max
    );

    if (this.filter.mode === 'whitelist') {
      return isInList;
    } else {
      return !isInList;
    }
  }

  /**
   * Validate IP or throw error
   */
  public validate(request: GatewayRequest): void {
    if (!this.isAllowed(request)) {
      throw new AuthorizationError(`IP address ${request.ip} is not allowed`);
    }
  }

  /**
   * Parse CIDR notation
   */
  private parseCIDR(cidr: string): { min: bigint; max: bigint } | null {
    const [ip, prefixStr] = cidr.split('/');
    const prefix = parseInt(prefixStr || '32', 10);

    if (!ip || isNaN(prefix) || prefix < 0 || prefix > 32) {
      return null;
    }

    const ipNum = this.ipToNumber(ip);
    if (ipNum === null) {
      return null;
    }

    // Calculate network mask
    const mask = BigInt(0xffffffff) << BigInt(32 - prefix);
    const networkAddr = ipNum & mask;
    const broadcastAddr = networkAddr | (~mask & BigInt(0xffffffff));

    return {
      min: networkAddr,
      max: broadcastAddr,
    };
  }

  /**
   * Convert IP address to number
   */
  private ipToNumber(ip: string): bigint | null {
    const parts = ip.split('.');

    if (parts.length !== 4) {
      return null;
    }

    let result = BigInt(0);

    for (let i = 0; i < 4; i++) {
      const part = parseInt(parts[i]!, 10);

      if (isNaN(part) || part < 0 || part > 255) {
        return null;
      }

      result = (result << BigInt(8)) | BigInt(part);
    }

    return result;
  }

  /**
   * Convert number to IP address
   */
  private numberToIp(num: bigint): string {
    const parts: number[] = [];

    for (let i = 0; i < 4; i++) {
      parts.unshift(Number((num >> BigInt(i * 8)) & BigInt(0xff)));
    }

    return parts.join('.');
  }

  /**
   * Add IP address or CIDR range
   */
  public addAddress(address: string): void {
    if (!this.filter.addresses.includes(address)) {
      this.filter.addresses.push(address);
      this.compileAddresses();
    }
  }

  /**
   * Remove IP address or CIDR range
   */
  public removeAddress(address: string): void {
    const index = this.filter.addresses.indexOf(address);
    if (index >= 0) {
      this.filter.addresses.splice(index, 1);
      this.compileAddresses();
    }
  }

  /**
   * Update filter mode
   */
  public setMode(mode: 'whitelist' | 'blacklist'): void {
    this.filter.mode = mode;
  }

  /**
   * Enable/disable filter
   */
  public setEnabled(enabled: boolean): void {
    this.filter.enabled = enabled;
  }

  /**
   * Get filter configuration
   */
  public getFilter(): IPFilter {
    return { ...this.filter };
  }

  /**
   * Get statistics
   */
  public getStatistics(): {
    mode: string;
    enabled: boolean;
    addressCount: number;
    rangeCount: number;
  } {
    return {
      mode: this.filter.mode,
      enabled: this.filter.enabled,
      addressCount: this.filter.addresses.length,
      rangeCount: this.compiledRanges.length,
    };
  }
}
