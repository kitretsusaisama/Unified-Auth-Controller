import { HttpClient } from '../http/client';
import { SessionManager } from '../session/manager';
import { TokenManager } from '../token/lifecycle';
import { EventBus } from '../events/bus';
import { FlowState, AuthTokens, User, AuthConfig } from '../types';
import { generateCodeVerifier, generateCodeChallenge } from '../utils/pkce';

export interface FlowResponse {
  flowId: string;
  step: string;
  data?: any;
}

export class AuthService {
  private httpClient: HttpClient;
  private sessionManager: SessionManager;
  private tokenManager: TokenManager;
  private eventBus: EventBus;
  private config: AuthConfig;

  constructor(
    config: AuthConfig,
    httpClient: HttpClient,
    sessionManager: SessionManager,
    tokenManager: TokenManager,
    eventBus: EventBus
  ) {
    this.config = config;
    this.httpClient = httpClient;
    this.sessionManager = sessionManager;
    this.tokenManager = tokenManager;
    this.eventBus = eventBus;
  }

  async startLoginFlow(email?: string): Promise<FlowResponse> {
    const response = await this.httpClient.post<any>('/auth/flow/start', {
      clientId: this.config.clientId,
      redirectUri: this.config.redirectUri,
      scopes: this.config.scopes || ['openid', 'profile', 'email'],
      email,
    });

    return {
      flowId: response.flowId,
      step: response.step || 'Identify',
      data: response.data,
    };
  }

  async submitStep(flowId: string, action: string, payload: any): Promise<FlowResponse> {
    const response = await this.httpClient.post<any>(`/auth/flow/${flowId}/submit`, {
      action,
      payload,
    });

    if (response.step === 'Success' || response.step === 'Completed') {
        if (response.code) {
             await this.exchangeCodeForTokens(response.code);
        } else if (response.token) {
             await this.tokenManager.setTokens(response.token);
             await this.fetchUser();
        }
    }

    return {
      flowId,
      step: response.step,
      data: response.data,
    };
  }

  async loginWithRedirect(): Promise<void> {
    const verifier = await generateCodeVerifier();
    const challenge = await generateCodeChallenge(verifier);

    sessionStorage.setItem('pkce_verifier', verifier);

    const params = new URLSearchParams({
      response_type: 'code',
      client_id: this.config.clientId,
      redirect_uri: this.config.redirectUri,
      code_challenge: challenge,
      code_challenge_method: 'S256',
      scope: (this.config.scopes || ['openid', 'profile', 'email']).join(' '),
    });

    window.location.href = `${this.config.baseUrl}/auth/authorize?${params.toString()}`;
  }

  async handleRedirectCallback(url?: string): Promise<void> {
    const targetUrl = url || window.location.href;
    const searchParams = new URL(targetUrl).searchParams;
    const code = searchParams.get('code');
    const error = searchParams.get('error');

    if (error) {
      throw new Error(`Auth Error: ${error}`);
    }

    if (!code) {
      throw new Error('No authorization code found in URL');
    }

    await this.exchangeCodeForTokens(code);
    window.history.replaceState({}, document.title, window.location.pathname);
  }

  private async exchangeCodeForTokens(code: string): Promise<void> {
    const verifier = sessionStorage.getItem('pkce_verifier');

    const payload: any = {
      grant_type: 'authorization_code',
      code,
      client_id: this.config.clientId,
      redirect_uri: this.config.redirectUri,
    };

    if (verifier) {
      payload.code_verifier = verifier;
    }

    const tokens = await this.httpClient.post<AuthTokens>('/auth/token', payload);

    await this.tokenManager.setTokens(tokens);
    await this.fetchUser();
    sessionStorage.removeItem('pkce_verifier');
  }

  private async fetchUser(): Promise<void> {
    const user = await this.httpClient.get<User>('/auth/userinfo');
    await this.sessionManager.setUser(user);
    this.eventBus.emit('LOGIN_SUCCESS', { user });
  }

  async logout(): Promise<void> {
    try {
      await this.httpClient.post('/auth/logout');
    } catch (e) {
      console.warn('Logout API failed', e);
    }
    await this.sessionManager.clearSession();
    this.eventBus.emit('LOGOUT');
  }

  async refreshToken(): Promise<void> {
    const refreshToken = this.tokenManager.getRefreshToken();
    if (!refreshToken) throw new Error('No refresh token available');

    try {
      const tokens = await this.httpClient.post<AuthTokens>('/auth/token', {
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: this.config.clientId,
      });

      await this.tokenManager.setTokens(tokens);
      this.eventBus.emit('TOKEN_REFRESHED');
    } catch (e) {
      await this.logout();
      throw e;
    }
  }
}
