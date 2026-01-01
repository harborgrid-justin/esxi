/**
 * HTTP Action - REST API calls and HTTP requests
 */

import axios, { AxiosRequestConfig, AxiosResponse } from 'axios';
import { HTTPActionConfig, Context } from '../types';

export class HTTPAction {
  /**
   * Execute HTTP request
   */
  async execute(config: HTTPActionConfig, context: Context): Promise<any> {
    const requestConfig: AxiosRequestConfig = {
      method: config.method,
      url: this.interpolateUrl(config.url, context),
      headers: this.interpolateHeaders(config.headers || {}, context),
      params: config.queryParams,
      timeout: 30000 // 30 second default timeout
    };

    // Add request body for non-GET requests
    if (config.body && config.method !== 'GET') {
      requestConfig.data = this.interpolateBody(config.body, context);
    }

    // Add authentication
    if (config.auth) {
      this.addAuthentication(requestConfig, config.auth);
    }

    try {
      const response: AxiosResponse = await axios(requestConfig);

      return {
        status: response.status,
        statusText: response.statusText,
        headers: response.headers,
        data: response.data
      };

    } catch (error: any) {
      if (error.response) {
        // Server responded with error status
        throw new Error(
          `HTTP ${error.response.status}: ${error.response.statusText} - ${JSON.stringify(error.response.data)}`
        );
      } else if (error.request) {
        // Request was made but no response received
        throw new Error(`No response received: ${error.message}`);
      } else {
        // Error setting up the request
        throw new Error(`Request error: ${error.message}`);
      }
    }
  }

  /**
   * Interpolate URL with context variables
   */
  private interpolateUrl(url: string, context: Context): string {
    return this.interpolateString(url, context);
  }

  /**
   * Interpolate headers with context variables
   */
  private interpolateHeaders(
    headers: Record<string, string>,
    context: Context
  ): Record<string, string> {
    const interpolated: Record<string, string> = {};

    for (const [key, value] of Object.entries(headers)) {
      interpolated[key] = this.interpolateString(value, context);
    }

    return interpolated;
  }

  /**
   * Interpolate request body with context variables
   */
  private interpolateBody(body: any, context: Context): any {
    if (typeof body === 'string') {
      return this.interpolateString(body, context);
    }

    if (typeof body === 'object' && body !== null) {
      return this.interpolateObject(body, context);
    }

    return body;
  }

  /**
   * Interpolate string with variables
   */
  private interpolateString(str: string, context: Context): string {
    return str.replace(/\${([^}]+)}/g, (match, varName) => {
      const value = context.variables.get(varName.trim());
      return value !== undefined ? String(value) : match;
    });
  }

  /**
   * Interpolate object with variables
   */
  private interpolateObject(obj: any, context: Context): any {
    if (Array.isArray(obj)) {
      return obj.map(item => this.interpolateObject(item, context));
    }

    if (typeof obj === 'object' && obj !== null) {
      const result: any = {};
      for (const [key, value] of Object.entries(obj)) {
        result[key] = this.interpolateObject(value, context);
      }
      return result;
    }

    if (typeof obj === 'string') {
      return this.interpolateString(obj, context);
    }

    return obj;
  }

  /**
   * Add authentication to request
   */
  private addAuthentication(
    config: AxiosRequestConfig,
    auth: HTTPActionConfig['auth']
  ): void {
    if (!auth) return;

    switch (auth.type) {
      case 'basic':
        config.auth = {
          username: auth.credentials.username,
          password: auth.credentials.password
        };
        break;

      case 'bearer':
        if (!config.headers) config.headers = {};
        config.headers['Authorization'] = `Bearer ${auth.credentials.token}`;
        break;

      case 'apikey':
        if (!config.headers) config.headers = {};
        const headerName = auth.credentials.headerName || 'X-API-Key';
        config.headers[headerName] = auth.credentials.apiKey;
        break;

      case 'oauth2':
        if (!config.headers) config.headers = {};
        config.headers['Authorization'] = `Bearer ${auth.credentials.accessToken}`;
        break;
    }
  }

  /**
   * Validate HTTP action configuration
   */
  validate(config: HTTPActionConfig): string[] {
    const errors: string[] = [];

    if (!config.method) {
      errors.push('HTTP method is required');
    } else if (!['GET', 'POST', 'PUT', 'PATCH', 'DELETE'].includes(config.method)) {
      errors.push('Invalid HTTP method');
    }

    if (!config.url) {
      errors.push('URL is required');
    } else {
      try {
        new URL(config.url);
      } catch {
        // Check if it's a URL with variables
        if (!config.url.includes('${')) {
          errors.push('Invalid URL format');
        }
      }
    }

    if (config.auth) {
      const authErrors = this.validateAuth(config.auth);
      errors.push(...authErrors);
    }

    return errors;
  }

  /**
   * Validate authentication configuration
   */
  private validateAuth(auth: HTTPActionConfig['auth']): string[] {
    const errors: string[] = [];

    if (!auth) return errors;

    switch (auth.type) {
      case 'basic':
        if (!auth.credentials.username) {
          errors.push('Basic auth requires username');
        }
        if (!auth.credentials.password) {
          errors.push('Basic auth requires password');
        }
        break;

      case 'bearer':
        if (!auth.credentials.token) {
          errors.push('Bearer auth requires token');
        }
        break;

      case 'apikey':
        if (!auth.credentials.apiKey) {
          errors.push('API key auth requires apiKey');
        }
        break;

      case 'oauth2':
        if (!auth.credentials.accessToken) {
          errors.push('OAuth2 requires accessToken');
        }
        break;
    }

    return errors;
  }
}
