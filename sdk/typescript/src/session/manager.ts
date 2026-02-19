import { StorageManager } from '../storage';
import { TokenManager } from '../token/lifecycle';
import { User, AuthSession } from '../types';

export class SessionManager {
  private storage: StorageManager;
  private tokenManager: TokenManager;
  private currentUser: User | null = null;
  private readonly USER_KEY = 'auth_user';

  constructor(storage: StorageManager, tokenManager: TokenManager) {
    this.storage = storage;
    this.tokenManager = tokenManager;
  }

  async initialize(): Promise<void> {
    await this.tokenManager.loadTokens();
    const user = await this.storage.get<User>(this.USER_KEY);
    if (user) {
      this.currentUser = user;
    }
  }

  async setUser(user: User): Promise<void> {
    this.currentUser = user;
    await this.storage.set(this.USER_KEY, user);
  }

  async clearSession(): Promise<void> {
    this.currentUser = null;
    await this.tokenManager.clearTokens();
    await this.storage.remove(this.USER_KEY);
  }

  getUser(): User | null {
    return this.currentUser;
  }

  isAuthenticated(): boolean {
    return !!this.tokenManager.getAccessToken() && !this.tokenManager.isAccessTokenExpired();
  }

  hasRole(role: string): boolean {
    return this.currentUser?.roles.includes(role) || false;
  }

  hasPermission(permission: string): boolean {
    return this.currentUser?.permissions.includes(permission) || false;
  }
}
