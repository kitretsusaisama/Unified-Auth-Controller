import { AuthConfig } from './types';
import { BrowserClient } from './browser/client';
import { ServerClient, ServiceClient } from './server/client';

export class AuthClient {
  static createBrowserClient(config: AuthConfig): BrowserClient {
    return new BrowserClient(config);
  }

  static createServerClient(config: AuthConfig): ServerClient {
    return new ServerClient(config);
  }

  static createServiceClient(config: AuthConfig): ServiceClient {
    return new ServiceClient(config);
  }

  static autoDetect(config: AuthConfig): BrowserClient | ServerClient {
    if (typeof window !== 'undefined') {
      return new BrowserClient(config);
    }
    // In Node/Server environment, we default to ServerClient but strictly check requirements
    if (config.clientSecret) {
        return new ServerClient(config);
    }
    throw new Error('Detected server environment but no clientSecret provided for ServerClient');
  }
}

export * from './types';
export * from './browser/client';
export * from './server/client';
export * from './react';
export * from './nextjs';
