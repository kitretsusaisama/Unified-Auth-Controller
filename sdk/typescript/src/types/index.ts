/**
 * Configuration for the Auth SDK.
 */
export interface AuthConfig {
  /** The base URL of the Auth API (e.g., https://auth.example.com) */
  baseUrl: string;
  /** The Client ID for your application */
  clientId: string;
  /** The Redirect URI for OIDC callbacks */
  redirectUri: string;
  /** List of scopes to request (default: openid, profile, email) */
  scopes?: string[];
  /** Custom storage adapter (default: LocalStorage or Memory) */
  storage?: StorageAdapter;
  /** Whether to automatically refresh tokens (default: true) */
  autoRefresh?: boolean;
}

/**
 * Represents an authenticated user.
 */
export interface User {
  id: string;
  email: string;
  roles: string[];
  permissions: string[];
  metadata?: Record<string, any>;
}

/**
 * Authentication tokens returned by the API.
 */
export interface AuthTokens {
  accessToken: string;
  refreshToken?: string;
  idToken?: string;
  expiresIn: number;
  tokenType: string;
}

/**
 * Current session state.
 */
export interface AuthSession {
  user: User | null;
  tokens: AuthTokens | null;
  isAuthenticated: boolean;
}

/**
 * Interface for storage adapters.
 * Can be synchronous or asynchronous.
 */
export interface StorageAdapter {
  getItem(key: string): Promise<string | null> | string | null;
  setItem(key: string, value: string): Promise<void> | void;
  removeItem(key: string): Promise<void> | void;
}

/**
 * Events emitted by the Auth Client.
 */
export interface AuthEvent {
  type: 'LOGIN_SUCCESS' | 'LOGIN_FAILURE' | 'LOGOUT' | 'TOKEN_REFRESHED' | 'SESSION_EXPIRED' | 'ERROR' | 'FLOW_UPDATE';
  payload?: any;
}

/**
 * State of an authentication flow step.
 */
export interface FlowState {
  flowId: string;
  step: string;
  data: any;
  requiredAction?: string;
}

/**
 * Standard API Error structure (RFC 7807).
 */
export interface ApiError {
  type: string;
  title: string;
  status: number;
  detail: string;
  instance?: string;
}
