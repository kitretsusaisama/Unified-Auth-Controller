import { StorageManager } from '../storage';
import { HttpClient } from '../http/client';
import { EventBus } from '../events/bus';
import { TokenManager } from '../token/lifecycle';
import { SessionManager } from '../session/manager';
import { AuthService } from '../auth/service';
import { AuthConfig } from '../types';

export class AuthClient {
  private httpClient: HttpClient;
  private storageManager: StorageManager;
  private tokenManager: TokenManager;
  private sessionManager: SessionManager;
  private authService: AuthService;
  private eventBus: EventBus;

  constructor(config: AuthConfig) {
    this.storageManager = new StorageManager(config.storage);
    this.tokenManager = new TokenManager(this.storageManager);

    // Setup HTTP Client with token provider
    this.httpClient = new HttpClient(config.baseUrl, async () => {
      // Refresh token if needed
      if (this.tokenManager.isAccessTokenExpired()) {
        try {
          await this.authService.refreshToken();
        } catch {
          return null;
        }
      }
      return this.tokenManager.getAccessToken();
    });

    this.sessionManager = new SessionManager(this.storageManager, this.tokenManager);
    this.eventBus = new EventBus();

    this.authService = new AuthService(
      config,
      this.httpClient,
      this.sessionManager,
      this.tokenManager,
      this.eventBus
    );
  }

  // Facade Methods

  async init(): Promise<void> {
    await this.sessionManager.initialize();
  }

  get auth() {
    return this.authService;
  }

  get session() {
    return this.sessionManager;
  }

  get events() {
    return this.eventBus;
  }

  get http() {
    return this.httpClient;
  }
}
