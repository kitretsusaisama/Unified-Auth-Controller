import { CoreClient } from '../core/client';
import { AuthConfig, ClientMode } from '../types';

export class ServerClient extends CoreClient {
  constructor(config: AuthConfig) {
    if (!config.clientSecret) {
        throw new Error('ServerClient requires clientSecret');
    }
    super(config, ClientMode.Server);
  }

  // Server flow typically involves confidential client exchange or on-behalf-of
  async login(): Promise<void> {
      throw new Error('Interactive login not supported in ServerClient directly. Use handleCallback or exchange.');
  }

  async exchangeCode(code: string, redirectUri: string): Promise<void> {
      const response = await fetch(`${this.config.baseUrl}/auth/token`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            grant_type: 'authorization_code',
            code,
            client_id: this.config.clientId,
            client_secret: this.config.clientSecret,
            redirect_uri: redirectUri,
        })
    });

    if (!response.ok) throw new Error('Token exchange failed');
    const tokens = await response.json();
    await this.tokenManager.setTokens(tokens);
  }

  async logout(): Promise<void> {
      await this.tokenManager.clearTokens();
  }
}

export class ServiceClient extends CoreClient {
    constructor(config: AuthConfig) {
        if (!config.clientSecret) throw new Error('ServiceClient requires clientSecret');
        super(config, ClientMode.Service);
    }

    async login(): Promise<void> {
        // Client Credentials Flow
        const response = await fetch(`${this.config.baseUrl}/auth/token`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                grant_type: 'client_credentials',
                client_id: this.config.clientId,
                client_secret: this.config.clientSecret,
                scope: (this.config.scopes || []).join(' ')
            })
        });

        if (!response.ok) throw new Error('Client credentials flow failed');
        const tokens = await response.json();
        await this.tokenManager.setTokens(tokens);
    }

    async logout(): Promise<void> {
        await this.tokenManager.clearTokens();
    }
}
