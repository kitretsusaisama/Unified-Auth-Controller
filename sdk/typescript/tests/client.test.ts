import { AuthClient } from '../src/core/client';
import { AuthConfig } from '../src/types';

describe('AuthClient', () => {
  const config: AuthConfig = {
    baseUrl: 'http://localhost:3000',
    clientId: 'test-client',
    redirectUri: 'http://localhost:3000/callback',
  };

  it('should initialize correctly', async () => {
    const client = new AuthClient(config);
    expect(client).toBeDefined();
    expect(client.auth).toBeDefined();
    expect(client.session).toBeDefined();
    expect(client.events).toBeDefined();

    await client.init();
    expect(client.session.isAuthenticated()).toBe(false);
  });

  it('should handle session initialization with no storage', async () => {
    const client = new AuthClient(config);
    await client.init();
    const user = client.session.getUser();
    expect(user).toBeNull();
  });
});
