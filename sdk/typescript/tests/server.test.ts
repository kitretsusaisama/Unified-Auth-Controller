import { ServerClient, ServiceClient } from '../src/server/client';
import { AuthConfig } from '../src/types';

global.fetch = jest.fn();

describe('ServerClient', () => {
  const config: AuthConfig = {
    baseUrl: 'https://auth.example.com',
    clientId: 'client-id',
    clientSecret: 'secret',
  };

  it('should exchange code for tokens', async () => {
    const client = new ServerClient(config);

    (global.fetch as jest.Mock).mockResolvedValue({
        json: async () => ({ accessToken: 'access', expiresIn: 3600 }),
        ok: true
    });

    await client.exchangeCode('code', 'http://cb');

    expect(fetch).toHaveBeenCalledWith(
        'https://auth.example.com/auth/token',
        expect.objectContaining({
            body: expect.stringContaining('"grant_type":"authorization_code"')
        })
    );
  });

  it('should throw if login called directly', async () => {
      const client = new ServerClient(config);
      await expect(client.login()).rejects.toThrow();
  });
});

describe('ServiceClient', () => {
    const config: AuthConfig = {
      baseUrl: 'https://auth.example.com',
      clientId: 'client-id',
      clientSecret: 'secret',
    };

    it('should use client credentials flow', async () => {
      const client = new ServiceClient(config);

      (global.fetch as jest.Mock).mockResolvedValue({
          json: async () => ({ accessToken: 'm2m-token', expiresIn: 3600 }),
          ok: true
      });

      await client.login();

      expect(fetch).toHaveBeenCalledWith(
          'https://auth.example.com/auth/token',
          expect.objectContaining({
              body: expect.stringContaining('"grant_type":"client_credentials"')
          })
      );
    });
  });
