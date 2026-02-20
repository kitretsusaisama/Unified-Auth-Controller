import { AuthEvent } from '../types';

export type AuthEventListener = (event: AuthEvent) => void;

export class EventBus {
  private listeners: Set<AuthEventListener> = new Set();

  subscribe(listener: AuthEventListener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  emit(type: AuthEvent['type'], payload?: any): void {
    const event: AuthEvent = { type, payload };
    this.listeners.forEach((listener) => {
      try {
        listener(event);
      } catch (err) {
        console.error('Error in auth event listener:', err);
      }
    });
  }
}
