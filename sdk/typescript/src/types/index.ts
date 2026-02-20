
export interface AuthConfig {
  baseUrl: string;
  clientId: string;
  redirectUri?: string;
  clientSecret?: string; // For confidential clients only
  scopes?: string[];
  autoRefresh?: boolean;
  storage?: StorageAdapter;
}

export interface User {
  id: string;
  email?: string;
  roles: string[];
  permissions: string[];
  metadata?: Record<string, any>;
}

export interface AuthTokens {
  accessToken: string;
  refreshToken?: string;
  idToken?: string;
  expiresIn: number;
  tokenType: string;
  scope?: string;
}

export interface StorageAdapter {
  getItem(key: string): Promise<string | null> | string | null;
  setItem(key: string, value: string): Promise<void> | void;
  removeItem(key: string): Promise<void> | void;
}

export interface AuthEvent {
  type: 'LOGIN_SUCCESS' | 'LOGIN_FAILURE' | 'LOGOUT' | 'TOKEN_REFRESHED' | 'SESSION_EXPIRED' | 'ERROR';
  payload?: any;
}

export enum ClientMode {
  Browser = 'browser',
  Server = 'server',
  Service = 'service',
}
