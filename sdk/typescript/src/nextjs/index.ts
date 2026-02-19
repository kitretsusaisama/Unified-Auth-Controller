import { AuthConfig } from '../types';
import { ServerClient } from '../server/client';
import { CookieStorageAdapter } from '../storage';

// Next.js specific helpers
export function createNextServerClient(config: AuthConfig, context: { cookies: () => any }): ServerClient {
    // We wrap Next.js cookies in our StorageAdapter
    const nextCookies = context.cookies();
    const cookieAdapter = new CookieStorageAdapter();

    // Override methods to use nextCookies if available (Server Components)
    // Note: This is simplified. Real implementation needs to handle async/sync nature of Next.js cookies() in recent versions

    // For now, return standard ServerClient, assuming config handles storage or we pass a custom one
    // Ideally we'd wrap `nextCookies` in a `NextCookieAdapter`
    return new ServerClient(config);
}
