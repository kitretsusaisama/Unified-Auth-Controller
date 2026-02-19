import { AuthClient } from '../src/index';
import { AuthConfig } from '../src/types';

describe('AuthClient', () => {
  const config: AuthConfig = {
    baseUrl: 'http://localhost:3000',
    clientId: 'test-client',
    redirectUri: 'http://localhost:3000/callback',
  };

  it('should autoDetect browser environment', () => {
    // JSDOM environment => window is defined => BrowserClient
    const client = AuthClient.autoDetect(config);
    expect(client.constructor.name).toBe('BrowserClient');
  });

  it('should create ServerClient manually', () => {
    const serverConfig = { ...config, clientSecret: 'secret' };
    const client = AuthClient.createServerClient(serverConfig);
    expect(client.constructor.name).toBe('ServerClient');
  });
});
