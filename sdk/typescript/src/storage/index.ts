import { StorageAdapter } from '../types';

export class LocalStorageAdapter implements StorageAdapter {
  getItem(key: string): string | null {
    if (typeof window === 'undefined') return null;
    return localStorage.getItem(key);
  }

  setItem(key: string, value: string): void {
    if (typeof window === 'undefined') return;
    localStorage.setItem(key, value);
  }

  removeItem(key: string): void {
    if (typeof window === 'undefined') return;
    localStorage.removeItem(key);
  }
}

export class SessionStorageAdapter implements StorageAdapter {
  getItem(key: string): string | null {
    if (typeof window === 'undefined') return null;
    return sessionStorage.getItem(key);
  }

  setItem(key: string, value: string): void {
    if (typeof window === 'undefined') return;
    sessionStorage.setItem(key, value);
  }

  removeItem(key: string): void {
    if (typeof window === 'undefined') return;
    sessionStorage.removeItem(key);
  }
}

export class MemoryStorageAdapter implements StorageAdapter {
  private store: Map<string, string> = new Map();

  getItem(key: string): string | null {
    return this.store.get(key) || null;
  }

  setItem(key: string, value: string): void {
    this.store.set(key, value);
  }

  removeItem(key: string): void {
    this.store.delete(key);
  }
}

export class StorageManager {
  private adapter: StorageAdapter;
  private prefix: string = 'auth_core:';

  constructor(adapter?: StorageAdapter) {
    this.adapter = adapter || (typeof window !== 'undefined' ? new LocalStorageAdapter() : new MemoryStorageAdapter());
  }

  async get<T>(key: string): Promise<T | null> {
    const raw = await this.adapter.getItem(this.prefix + key);
    if (!raw) return null;
    try {
      return JSON.parse(raw) as T;
    } catch {
      return null;
    }
  }

  async set<T>(key: string, value: T): Promise<void> {
    const raw = JSON.stringify(value);
    await this.adapter.setItem(this.prefix + key, raw);
  }

  async remove(key: string): Promise<void> {
    await this.adapter.removeItem(this.prefix + key);
  }

  async clear(): Promise<void> {
    // Note: Most adapters don't support clear by prefix easily without scanning keys.
    // For now we assume clearing specific keys manually or implementing a clear method if critical.
    // We'll add common keys here:
    await this.remove('tokens');
    await this.remove('user');
    await this.remove('flow_id');
  }
}
