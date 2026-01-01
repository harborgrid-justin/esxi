/**
 * Enterprise API Gateway - Protocol Translator
 *
 * Translate between different API protocols (REST, GraphQL, gRPC)
 */

import type { GatewayRequest, GatewayResponse } from '../types';

export type ProtocolType = 'REST' | 'GraphQL' | 'gRPC' | 'SOAP';

export interface TranslationConfig {
  sourceProtocol: ProtocolType;
  targetProtocol: ProtocolType;
  mapping?: Record<string, string>;
}

export class ProtocolTranslator {
  /**
   * Translate request between protocols
   */
  public translateRequest(
    request: GatewayRequest,
    config: TranslationConfig
  ): GatewayRequest {
    if (config.sourceProtocol === config.targetProtocol) {
      return request;
    }

    const translators: Record<string, (req: GatewayRequest) => GatewayRequest> = {
      'REST->GraphQL': this.restToGraphQL.bind(this),
      'GraphQL->REST': this.graphQLToREST.bind(this),
      'REST->gRPC': this.restToGRPC.bind(this),
      'gRPC->REST': this.gRPCToREST.bind(this),
    };

    const key = `${config.sourceProtocol}->${config.targetProtocol}`;
    const translator = translators[key];

    if (translator) {
      return translator(request);
    }

    return request;
  }

  /**
   * Translate response between protocols
   */
  public translateResponse(
    response: GatewayResponse,
    config: TranslationConfig
  ): GatewayResponse {
    if (config.sourceProtocol === config.targetProtocol) {
      return response;
    }

    // Response translation would mirror request translation
    return response;
  }

  /**
   * Convert REST request to GraphQL
   */
  private restToGraphQL(request: GatewayRequest): GatewayRequest {
    const { method, path, body, query } = request;

    let graphQLQuery = '';
    let variables = {};

    // Convert REST operations to GraphQL
    if (method === 'GET') {
      // GET /users/123 -> query { user(id: 123) { ... } }
      const pathParts = path.split('/').filter(Boolean);
      const resource = pathParts[0];
      const id = pathParts[1];

      if (id) {
        graphQLQuery = `query { ${resource}(id: "${id}") { id } }`;
      } else {
        graphQLQuery = `query { ${resource} { id } }`;
      }
    } else if (method === 'POST') {
      // POST /users -> mutation { createUser(input: {...}) { ... } }
      const pathParts = path.split('/').filter(Boolean);
      const resource = pathParts[0];

      graphQLQuery = `mutation { create${this.capitalize(resource || '')}(input: $input) { id } }`;
      variables = { input: body };
    } else if (method === 'PUT' || method === 'PATCH') {
      // PUT /users/123 -> mutation { updateUser(id: 123, input: {...}) { ... } }
      const pathParts = path.split('/').filter(Boolean);
      const resource = pathParts[0];
      const id = pathParts[1];

      graphQLQuery = `mutation { update${this.capitalize(resource || '')}(id: "${id}", input: $input) { id } }`;
      variables = { input: body };
    } else if (method === 'DELETE') {
      // DELETE /users/123 -> mutation { deleteUser(id: 123) }
      const pathParts = path.split('/').filter(Boolean);
      const resource = pathParts[0];
      const id = pathParts[1];

      graphQLQuery = `mutation { delete${this.capitalize(resource || '')}(id: "${id}") }`;
    }

    return {
      ...request,
      method: 'POST',
      path: '/graphql',
      body: {
        query: graphQLQuery,
        variables,
      },
      headers: {
        ...request.headers,
        'content-type': 'application/json',
      },
    };
  }

  /**
   * Convert GraphQL request to REST
   */
  private graphQLToREST(request: GatewayRequest): GatewayRequest {
    const body = request.body as { query?: string; variables?: Record<string, unknown> };

    if (!body?.query) {
      return request;
    }

    // Parse GraphQL query to determine REST endpoint
    const isQuery = body.query.includes('query');
    const isMutation = body.query.includes('mutation');

    // Extract operation name
    const operationMatch = body.query.match(/\w+\s*\(/);
    const operation = operationMatch ? operationMatch[0].replace('(', '').trim() : '';

    let method: typeof request.method = 'GET';
    let path = '/';

    if (isMutation) {
      if (operation.startsWith('create')) {
        method = 'POST';
        path = `/${operation.replace('create', '').toLowerCase()}s`;
      } else if (operation.startsWith('update')) {
        method = 'PUT';
        const id = body.variables?.id;
        path = `/${operation.replace('update', '').toLowerCase()}s/${id}`;
      } else if (operation.startsWith('delete')) {
        method = 'DELETE';
        const id = body.variables?.id;
        path = `/${operation.replace('delete', '').toLowerCase()}s/${id}`;
      }
    } else if (isQuery) {
      method = 'GET';
      path = `/${operation.toLowerCase()}s`;

      if (body.variables?.id) {
        path += `/${body.variables.id}`;
      }
    }

    return {
      ...request,
      method,
      path,
      body: body.variables?.input || null,
    };
  }

  /**
   * Convert REST request to gRPC
   */
  private restToGRPC(request: GatewayRequest): GatewayRequest {
    // This is a simplified version - real gRPC translation would use protobuf
    const { method, path, body } = request;

    const pathParts = path.split('/').filter(Boolean);
    const service = pathParts[0];
    const id = pathParts[1];

    let grpcMethod = '';

    if (method === 'GET') {
      grpcMethod = id ? 'Get' : 'List';
    } else if (method === 'POST') {
      grpcMethod = 'Create';
    } else if (method === 'PUT' || method === 'PATCH') {
      grpcMethod = 'Update';
    } else if (method === 'DELETE') {
      grpcMethod = 'Delete';
    }

    return {
      ...request,
      method: 'POST',
      path: `/${service}.${this.capitalize(service || '')}Service/${grpcMethod}${this.capitalize(service || '')}`,
      body: id ? { id, ...body } : body,
      headers: {
        ...request.headers,
        'content-type': 'application/grpc+proto',
      },
    };
  }

  /**
   * Convert gRPC request to REST
   */
  private gRPCToREST(request: GatewayRequest): GatewayRequest {
    // This is a simplified version
    const pathParts = request.path.split('/');
    const methodPath = pathParts[pathParts.length - 1];

    if (!methodPath) {
      return request;
    }

    let method: typeof request.method = 'GET';
    let resource = '';

    if (methodPath.startsWith('Get')) {
      method = 'GET';
      resource = methodPath.replace('Get', '').toLowerCase();
    } else if (methodPath.startsWith('List')) {
      method = 'GET';
      resource = methodPath.replace('List', '').toLowerCase() + 's';
    } else if (methodPath.startsWith('Create')) {
      method = 'POST';
      resource = methodPath.replace('Create', '').toLowerCase() + 's';
    } else if (methodPath.startsWith('Update')) {
      method = 'PUT';
      resource = methodPath.replace('Update', '').toLowerCase() + 's';
    } else if (methodPath.startsWith('Delete')) {
      method = 'DELETE';
      resource = methodPath.replace('Delete', '').toLowerCase() + 's';
    }

    const body = request.body as { id?: string };
    const path = body?.id ? `/${resource}/${body.id}` : `/${resource}`;

    return {
      ...request,
      method,
      path,
    };
  }

  /**
   * Capitalize first letter
   */
  private capitalize(str: string): string {
    return str.charAt(0).toUpperCase() + str.slice(1);
  }
}
