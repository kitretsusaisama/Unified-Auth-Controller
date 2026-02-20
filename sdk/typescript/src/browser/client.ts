import { CoreClient } from '../core/client';
import { AuthConfig, ClientMode } from '../types';
import { generateCodeVerifier, generateCodeChallenge } from '../utils/pkce';

export class BrowserClient extends CoreClient {
  constructor(config: AuthConfig) {
    super(config, ClientMode.Browser);
  }

  async login(options?: { redirectUri?: string, scopes?: string[] }): Promise<void> {
    const verifier = await generateCodeVerifier();
    const challenge = await generateCodeChallenge(verifier);

    // Store verifier in sessionStorage (browser specific)
    sessionStorage.setItem('pkce_verifier', verifier);

    const params = new URLSearchParams({
      response_type: 'code',
      client_id: this.config.clientId,
      redirect_uri: options?.redirectUri || this.config.redirectUri || window.location.href,
      code_challenge: challenge,
      code_challenge_method: 'S256',
      scope: (options?.scopes || this.config.scopes || ['openid', 'profile', 'email']).join(' '),
    });

    window.location.href = `${this.config.baseUrl}/auth/authorize?${params.toString()}`;
  }

  async handleCallback(): Promise<void> {
    const params = new URLSearchParams(window.location.search);
    const code = params.get('code');
    if (!code) return;

    const verifier = sessionStorage.getItem('pkce_verifier');
    if (!verifier) throw new Error('No PKCE verifier found');

    // Exchange code
    const response = await fetch(`${this.config.baseUrl}/auth/token`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            grant_type: 'authorization_code',
            code,
            client_id: this.config.clientId,
            redirect_uri: this.config.redirectUri, // Must match original
            code_verifier: verifier
        })
    });

    const tokens = await response.json();
    await this.tokenManager.setTokens(tokens);
    sessionStorage.removeItem('pkce_verifier');

    // Clean URL
    window.history.replaceState({}, document.title, window.location.pathname);
  }

  async logout(): Promise<void> {
    await this.tokenManager.clearTokens();
    // Optional: Call server logout endpoint
  }
}
