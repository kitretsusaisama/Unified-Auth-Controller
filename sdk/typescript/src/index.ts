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

export interface FlowResult {
    next_state: string;
    ui_hints?: Record<string, any>;
    error?: string;
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
        // Generate secure state/nonce
        const state = this.generateRandomString(32);
        const nonce = this.generateRandomString(32);

        // Generate PKCE
        const { verifier, challenge } = await this.generatePKCE();

        // Store state/nonce/verifier in sessionStorage
        if (typeof window !== 'undefined') {
            sessionStorage.setItem('auth_state', state);
            sessionStorage.setItem('auth_nonce', nonce);
            sessionStorage.setItem('auth_verifier', verifier);
        }

        const params = new URLSearchParams({
            response_type: 'code',
            client_id: this.config.clientId,
            redirect_uri: this.config.redirectUri,
            scope: this.config.scope || 'openid profile email',
            state,
            nonce,
            code_challenge: challenge,
            code_challenge_method: 'S256',
        });

        const url = `${this.config.baseUrl}/auth/authorize?${params.toString()}`;

        if (typeof window !== 'undefined') {
            window.location.href = url;
        } else {
            console.log(`[AuthSDK] Redirect URL: ${url}`);
        }
    }

    private generateRandomString(length: number): string {
        const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
        let result = '';
        if (typeof window !== 'undefined' && window.crypto) {
             const array = new Uint8Array(length);
             window.crypto.getRandomValues(array);
             for(let i = 0; i < length; i++) {
                 result += characters.charAt(array[i] % characters.length);
             }
        } else {
             for (let i = 0; i < length; i++) {
                result += characters.charAt(Math.floor(Math.random() * characters.length));
             }
        }
        return result;
    }

    private async generatePKCE() {
        const verifier = this.generateRandomString(43);
        const challenge = await this.generateCodeChallenge(verifier);
        return { verifier, challenge };
    }

    private async generateCodeChallenge(verifier: string) {
        if (typeof window !== 'undefined' && window.crypto && window.crypto.subtle) {
            const encoder = new TextEncoder();
            const data = encoder.encode(verifier);
            const hash = await window.crypto.subtle.digest('SHA-256', data);
            return this.base64UrlEncode(new Uint8Array(hash));
        }
        return verifier; // Fallback for non-browser env (not secure but allows basic test)
    }

    private base64UrlEncode(array: Uint8Array) {
        let str = '';
        for(let i = 0; i < array.length; i++) {
            str += String.fromCharCode(array[i]);
        }
        return btoa(str)
            .replace(/\+/g, '-')
            .replace(/\//g, '_')
            .replace(/=+$/, '');
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
        const verifier = typeof window !== 'undefined' ? sessionStorage.getItem('auth_verifier') : undefined;

        const response = await this.client.post<TokenResponse>('/auth/token', {
            grant_type: 'authorization_code',
            client_id: this.config.clientId,
            code,
            redirect_uri: this.config.redirectUri,
            code_verifier: verifier,
        });

        this.token = response.data.access_token;
        if (typeof window !== 'undefined') {
            localStorage.setItem('access_token', this.token);
            sessionStorage.removeItem('auth_verifier'); // Clean up
            sessionStorage.removeItem('auth_state');
            sessionStorage.removeItem('auth_nonce');
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

    /**
     * Universal Workflow Interaction
     * Submits an action to the current authentication flow.
     * @param flowId The unique ID of the flow (from Start Flow or previous step)
     * @param action The action name (e.g., 'submit_identifier', 'submit_password')
     * @param payload The data payload for the action
     */
    public async submitFlow(flowId: string, action: string, payload: any): Promise<FlowResult> {
        try {
            const response = await this.client.post<FlowResult>(`/auth/flow/${flowId}/submit`, {
                action,
                payload,
            });
            return response.data;
        } catch (error: any) {
            // Handle RFC 7807 Problem Details
            if (error.response && error.response.headers['content-type']?.includes('application/problem+json')) {
                const problem = error.response.data;
                throw new Error(problem.detail || problem.title || 'Unknown flow error');
            }
            throw error;
        }
    }
}
