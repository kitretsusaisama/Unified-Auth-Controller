import { StorageManager } from '../storage';
import { AuthTokens } from '../types';

export class TokenManager {
  private storage: StorageManager;
  private currentTokens: AuthTokens | null = null;
  private readonly TOKEN_KEY = 'auth_tokens';

  constructor(storage: StorageManager) {
    this.storage = storage;
  }

  async loadTokens(): Promise<AuthTokens | null> {
    const tokens = await this.storage.get<AuthTokens>(this.TOKEN_KEY);
    if (tokens) {
      this.currentTokens = tokens;
      return tokens;
    }
    return null;
  }

  async setTokens(tokens: AuthTokens): Promise<void> {
    this.currentTokens = tokens;
    await this.storage.set(this.TOKEN_KEY, tokens);
  }

  async clearTokens(): Promise<void> {
    this.currentTokens = null;
    await this.storage.remove(this.TOKEN_KEY);
  }

  getAccessToken(): string | null {
    return this.currentTokens?.accessToken || null;
  }

  getRefreshToken(): string | null {
    return this.currentTokens?.refreshToken || null;
  }

  isAccessTokenExpired(): boolean {
    if (!this.currentTokens?.accessToken) return true;
    // Basic check: if we have `expiresIn`, calculate expiry.
    // Usually JWT payload has `exp`. Let's decode it.
    try {
      const payload = JSON.parse(atob(this.currentTokens.accessToken.split('.')[1]));
      const exp = payload.exp * 1000;
      return Date.now() >= exp - 30000; // 30s buffer
    } catch {
      return true; // Assume expired if invalid
    }
  }

  // Helper to decode without verification (client-side only)
  decodeToken(token: string): any {
    try {
      return JSON.parse(atob(token.split('.')[1]));
    } catch {
      return null;
    }
  }
}
