import { TokenManager } from '../src/core/token';
import { MemoryStorageAdapter } from '../src/storage';
import { AuthTokens } from '../src/types';

describe('TokenManager', () => {
  let storage: MemoryStorageAdapter;
  let manager: TokenManager;

  const validToken = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjk5OTk5OTk5OTksInN1YiI6InRlc3QifQ.sig';
  const expiredToken = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE1MTYyMzkwMjIsInN1YiI6InRlc3QifQ.sig';

  const tokens: AuthTokens = {
    accessToken: validToken,
    refreshToken: 'refresh-token',
    idToken: 'id-token',
    expiresIn: 3600,
    tokenType: 'Bearer',
  };

  beforeEach(() => {
    storage = new MemoryStorageAdapter();
    manager = new TokenManager(storage);
  });

  it('should store and retrieve tokens', async () => {
    await manager.setTokens(tokens);
    const retrieved = await manager.loadTokens();
    expect(retrieved).toEqual(tokens);
    expect(manager.getAccessToken()).toBe(tokens.accessToken);
  });

  it('should clear tokens', async () => {
    await manager.setTokens(tokens);
    await manager.clearTokens();
    const retrieved = await manager.loadTokens();
    expect(retrieved).toBeNull();
    expect(manager.getAccessToken()).toBeNull();
  });

  it('should detect expired tokens', async () => {
    await manager.setTokens({ ...tokens, accessToken: expiredToken });
    expect(manager.isAccessTokenExpired()).toBe(true);
  });

  it('should not consider valid tokens expired', async () => {
    await manager.setTokens(tokens);
    expect(manager.isAccessTokenExpired()).toBe(false);
  });

  it('should handle refresh locks', async () => {
    expect(await manager.getRefreshLock()).toBe(true);

    // Simulate refresh in progress
    const refreshPromise = new Promise<AuthTokens | null>(() => {}); // Never resolves
    manager.setRefreshPromise(refreshPromise);

    expect(await manager.getRefreshLock()).toBe(false);
    expect(manager.getExistingRefreshPromise()).toBe(refreshPromise);
  });
});
