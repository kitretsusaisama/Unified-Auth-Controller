import { AuthConfig, ClientMode, StorageAdapter } from '../types';
import { TokenManager } from './token';
import { LocalStorageAdapter, MemoryStorageAdapter, CookieStorageAdapter } from '../storage';

export abstract class CoreClient {
  protected config: AuthConfig;
  protected tokenManager: TokenManager;
  protected mode: ClientMode;

  constructor(config: AuthConfig, mode: ClientMode) {
    this.config = config;
    this.mode = mode;

    // Auto-select storage if not provided
    const storage = config.storage || this.getDefaultStorage(mode);
    this.tokenManager = new TokenManager(storage);
  }

  private getDefaultStorage(mode: ClientMode): StorageAdapter {
    if (typeof window !== 'undefined') {
        // Browser environment
        return new LocalStorageAdapter();
    } else {
        // Server environment
        return new MemoryStorageAdapter(); // Or default to None if state is ephemeral per request
    }
  }

  abstract login(options?: any): Promise<void>;
  abstract logout(options?: any): Promise<void>;

  async getAccessToken(): Promise<string | null> {
    // Check expiry and refresh if needed
    if (this.tokenManager.isAccessTokenExpired()) {
        await this.refreshToken();
    }
    return this.tokenManager.getAccessToken();
  }

  async refreshToken(): Promise<void> {
      // Base implementation, might be overridden or use shared logic
      const existingPromise = this.tokenManager.getExistingRefreshPromise();
      if (existingPromise) {
          await existingPromise;
          return;
      }

      const refreshLogic = async () => {
          // Actual refresh logic (HTTP call)
          // This should be implemented by subclasses or a shared HTTP helper
          // For now returning null to satisfy types in abstract base
          return null;
      };

      this.tokenManager.setRefreshPromise(refreshLogic());
      await this.tokenManager.getExistingRefreshPromise();
  }
}
