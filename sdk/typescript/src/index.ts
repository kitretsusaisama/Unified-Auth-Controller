import axios, { AxiosInstance } from 'axios';
import { jwtDecode } from 'jwt-decode';

export interface AuthConfig {
    baseUrl: string;
    clientId: string;
    redirectUri: string;
    scope?: string;
}

export interface User {
    sub: string;
    email?: string;
    [key: string]: any;
}

export interface TokenResponse {
    access_token: string;
    id_token?: string;
    refresh_token?: string;
    expires_in: number;
}

export class AuthClient {
    private client: AxiosInstance;
    private config: AuthConfig;
    private token?: string;

    constructor(config: AuthConfig) {
        this.config = config;
        this.client = axios.create({
            baseURL: config.baseUrl,
            headers: {
                'Content-Type': 'application/json',
            },
        });
    }

    /**
     * Redirects the browser to the login page
     */
    public async loginWithRedirect(): Promise<void> {
        // Generate simple state/nonce for MVP
        // In prod, use crypto for PKCE verifier/challenge
        const state = Math.random().toString(36).substring(7);
        const nonce = Math.random().toString(36).substring(7);

        // Store state/nonce in sessionStorage
        if (typeof window !== 'undefined') {
            sessionStorage.setItem('auth_state', state);
            sessionStorage.setItem('auth_nonce', nonce);
        }

        const params = new URLSearchParams({
            response_type: 'code',
            client_id: this.config.clientId,
            redirect_uri: this.config.redirectUri,
            scope: this.config.scope || 'openid profile email',
            state,
            nonce,
        });

        const url = `${this.config.baseUrl}/auth/authorize?${params.toString()}`;

        if (typeof window !== 'undefined') {
            window.location.href = url;
        } else {
            console.log(`[AuthSDK] Redirect URL: ${url}`);
        }
    }

    /**
     * Handles the callback from the login redirect
     * Exchanges code for tokens
     */
    public async handleRedirectCallback(): Promise<void> {
        if (typeof window === 'undefined') return;

        const params = new URLSearchParams(window.location.search);
        const code = params.get('code');
        const state = params.get('state');

        const storedState = sessionStorage.getItem('auth_state');
        if (state !== storedState) {
            throw new Error('Invalid state parameter');
        }

        if (code) {
            await this.exchangeCodeForToken(code);
        }
    }

    private async exchangeCodeForToken(code: string): Promise<void> {
        const response = await this.client.post<TokenResponse>('/auth/token', {
            grant_type: 'authorization_code',
            client_id: this.config.clientId,
            code,
            redirect_uri: this.config.redirectUri,
        });

        this.token = response.data.access_token;
        if (typeof window !== 'undefined') {
            localStorage.setItem('access_token', this.token);
        }
    }

    public async getUser(): Promise<User | null> {
        if (!this.token) {
             if (typeof window !== 'undefined') {
                 this.token = localStorage.getItem('access_token') || undefined;
             }
        }

        if (!this.token) return null;

        try {
            // Option A: Decode local ID token
            // Option B: Call UserInfo endpoint
            const response = await this.client.get<User>('/auth/userinfo', {
                headers: { Authorization: `Bearer ${this.token}` }
            });
            return response.data;
        } catch (e) {
            return null;
        }
    }

    public isAuthenticated(): boolean {
        // Basic check
        // In prod, check expiry
        return !!this.token;
    }

    public logout(): void {
        this.token = undefined;
        if (typeof window !== 'undefined') {
            localStorage.removeItem('access_token');
            sessionStorage.clear();
            // Optional: Redirect to logout endpoint
        }
    }
}
