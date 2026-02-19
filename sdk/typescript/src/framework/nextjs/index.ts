import { AuthClient } from '../../core/client';
import { AuthConfig, StorageAdapter } from '../../types';

// Helper to extract token from request
export function getBearerToken(req: Request): string | null {
  const authHeader = req.headers.get('Authorization');
  if (authHeader && authHeader.startsWith('Bearer ')) {
    return authHeader.substring(7);
  }
  return null;
}

// Interface for Next.js cookies (partial)
interface NextCookies {
  get(name: string): { value: string } | undefined;
  set(name: string, value: string, options?: any): void;
  delete(name: string): void;
}

// Adapter for Next.js Server Actions / Route Handlers where cookies() is available
export class NextCookieStorageAdapter implements StorageAdapter {
  private cookies: NextCookies;

  constructor(cookies: NextCookies) {
    this.cookies = cookies;
  }

  getItem(key: string): string | null {
    const cookie = this.cookies.get(key);
    return cookie?.value || null;
  }

  setItem(key: string, value: string): void {
    try {
      this.cookies.set(key, value, { httpOnly: true, secure: true, path: '/' });
    } catch (e) {
      console.warn('Cannot set cookie in this context (e.g. RSC read-only)', e);
    }
  }

  removeItem(key: string): void {
    try {
      this.cookies.delete(key);
    } catch (e) {
      console.warn('Cannot delete cookie in this context', e);
    }
  }
}

// Factory to create client on server side
export function createServerClient(config: AuthConfig, cookies: NextCookies): AuthClient {
  return new AuthClient({
    ...config,
    storage: new NextCookieStorageAdapter(cookies),
  });
}
