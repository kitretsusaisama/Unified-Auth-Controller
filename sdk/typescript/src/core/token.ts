import { StorageAdapter, AuthTokens } from '../types';

export class TokenManager {
  private storage: StorageAdapter;
  private currentTokens: AuthTokens | null = null;
  private readonly TOKEN_KEY = 'upflame_tokens';
  private refreshPromise: Promise<AuthTokens | null> | null = null;

  constructor(storage: StorageAdapter) {
    this.storage = storage;
  }

  async loadTokens(): Promise<AuthTokens | null> {
    const raw = await this.storage.getItem(this.TOKEN_KEY);
    if (!raw) return null;
    try {
      this.currentTokens = JSON.parse(raw);
      return this.currentTokens;
    } catch {
      return null;
    }
  }

  async setTokens(tokens: AuthTokens): Promise<void> {
    this.currentTokens = tokens;
    await this.storage.setItem(this.TOKEN_KEY, JSON.stringify(tokens));
  }

  async clearTokens(): Promise<void> {
    this.currentTokens = null;
    await this.storage.removeItem(this.TOKEN_KEY);
  }

  getAccessToken(): string | null {
    return this.currentTokens?.accessToken || null;
  }

  getRefreshToken(): string | null {
    return this.currentTokens?.refreshToken || null;
  }

  isAccessTokenExpired(bufferSeconds = 30): boolean {
    if (!this.currentTokens?.accessToken) return true;
    try {
      const payload = this.decodeToken(this.currentTokens.accessToken);
      if (!payload.exp) return true;
      return Date.now() >= (payload.exp * 1000) - (bufferSeconds * 1000);
    } catch {
      return true;
    }
  }

  decodeToken(token: string): any {
    try {
      const base64Url = token.split('.')[1];
      const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
      const jsonPayload = decodeURIComponent(atob(base64).split('').map(function(c) {
          return '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2);
      }).join(''));
      return JSON.parse(jsonPayload);
    } catch {
      return {};
    }
  }

  // Basic locking mechanism for refresh
  async getRefreshLock(): Promise<boolean> {
    if (this.refreshPromise) return false;
    return true;
  }

  setRefreshPromise(promise: Promise<AuthTokens | null>): void {
    this.refreshPromise = promise;
    promise.finally(() => {
        this.refreshPromise = null;
    });
  }

  getExistingRefreshPromise(): Promise<AuthTokens | null> | null {
      return this.refreshPromise;
  }
}
