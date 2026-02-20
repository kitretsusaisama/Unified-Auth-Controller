import { CookieStorageAdapter, MemoryStorageAdapter } from '../src/storage';

describe('Storage Adapters', () => {
  describe('MemoryStorageAdapter', () => {
    it('should store and retrieve items', async () => {
      const storage = new MemoryStorageAdapter();
      await storage.setItem('key', 'value');
      expect(await storage.getItem('key')).toBe('value');
    });

    it('should remove items', async () => {
      const storage = new MemoryStorageAdapter();
      await storage.setItem('key', 'value');
      await storage.removeItem('key');
      expect(await storage.getItem('key')).toBeNull();
    });
  });

  describe('CookieStorageAdapter', () => {
    it('should behave like storage', async () => {
        // JSDOM has poor cookie support by default sometimes depending on version or config
        // We can manually shim document.cookie if it fails, or rely on 'cookie' package behavior if used internally
        // The implementation uses document.cookie directly.

        // Let's use Object.defineProperty to ensure basic getter/setter behavior for test if JSDOM is flaky
        let cookieStore = '';
        Object.defineProperty(document, 'cookie', {
            get: () => cookieStore,
            set: (val) => {
                // Simple append for test
                const key = val.split('=')[0];
                // Remove existing
                const parts = cookieStore.split('; ').filter(c => !c.startsWith(key + '='));
                if (!val.includes('max-age=0') && !val.includes('expires=')) {
                     parts.push(val.split(';')[0]);
                }
                cookieStore = parts.join('; ');
            },
            configurable: true
        });

        const storage = new CookieStorageAdapter();

        storage.setItem('test_cookie', 'value');
        expect(storage.getItem('test_cookie')).toBe('value');

        storage.removeItem('test_cookie');
        expect(storage.getItem('test_cookie')).toBeNull();
    });
  });
});
