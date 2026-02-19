import { ApiError } from '../types';

export interface RequestConfig extends RequestInit {
  params?: Record<string, string>;
}

export type TokenProvider = () => Promise<string | null>;

export class HttpClient {
  private baseUrl: string;
  private tokenProvider?: TokenProvider;
  private defaultHeaders: HeadersInit = {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  };

  constructor(baseUrl: string, tokenProvider?: TokenProvider) {
    this.baseUrl = baseUrl.replace(/\/$/, '');
    this.tokenProvider = tokenProvider;
  }

  async request<T>(path: string, config: RequestConfig = {}): Promise<T> {
    const url = new URL(this.baseUrl + path);
    if (config.params) {
      Object.entries(config.params).forEach(([key, value]) => {
        if (value) url.searchParams.append(key, value);
      });
    }

    const headers = new Headers(this.defaultHeaders);
    if (config.headers) {
      new Headers(config.headers).forEach((value, key) => headers.append(key, value));
    }

    if (this.tokenProvider) {
      const token = await this.tokenProvider();
      if (token) {
        headers.set('Authorization', `Bearer ${token}`);
      }
    }

    const response = await fetch(url.toString(), {
      ...config,
      headers,
    });

    if (!response.ok) {
      let errorData: any;
      try {
        errorData = await response.json();
      } catch {
        errorData = { detail: response.statusText };
      }

      const error: ApiError = {
        type: 'api_error',
        title: errorData.title || 'API Error',
        status: response.status,
        detail: errorData.detail || 'Unknown error occurred',
        instance: errorData.instance,
      };
      throw error;
    }

    // Handle 204 No Content
    if (response.status === 204) {
      return {} as T;
    }

    try {
      return await response.json();
    } catch {
      return {} as T;
    }
  }

  async get<T>(path: string, config?: RequestConfig): Promise<T> {
    return this.request<T>(path, { ...config, method: 'GET' });
  }

  async post<T>(path: string, data?: any, config?: RequestConfig): Promise<T> {
    return this.request<T>(path, {
      ...config,
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async put<T>(path: string, data?: any, config?: RequestConfig): Promise<T> {
    return this.request<T>(path, {
      ...config,
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async delete<T>(path: string, config?: RequestConfig): Promise<T> {
    return this.request<T>(path, { ...config, method: 'DELETE' });
  }
}
