import React, { createContext, useContext, useEffect, useState } from 'react';
import { AuthClient } from '../../core/client';
import { User, AuthEvent } from '../../types';

interface AuthContextType {
  client: AuthClient;
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (email?: string) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export interface AuthProviderProps {
  client: AuthClient;
  children: React.ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ client, children }) => {
  const [user, setUser] = useState<User | null>(client.session.getUser());
  const [isAuthenticated, setIsAuthenticated] = useState<boolean>(client.session.isAuthenticated());
  const [isLoading, setIsLoading] = useState<boolean>(true);

  useEffect(() => {
    let mounted = true;
    const init = async () => {
      await client.init();
      if (mounted) {
        setUser(client.session.getUser());
        setIsAuthenticated(client.session.isAuthenticated());
        setIsLoading(false);
      }
    };

    init();

    const sub = client.events.subscribe((event: AuthEvent) => {
      if (!mounted) return;

      if (event.type === 'LOGIN_SUCCESS') {
        setUser(event.payload.user);
        setIsAuthenticated(true);
      } else if (event.type === 'LOGOUT' || event.type === 'SESSION_EXPIRED') {
        setUser(null);
        setIsAuthenticated(false);
      }
    });

    return () => {
      mounted = false;
      sub();
    };
  }, [client]);

  const login = async (email?: string) => {
    // Start login flow - this might redirect or just start API flow
    // For this hook, we assume initiating redirect flow is common or returning promise
    await client.auth.startLoginFlow(email);
  };

  const logout = async () => {
    await client.auth.logout();
  };

  return (
    <AuthContext.Provider value={{ client, user, isAuthenticated, isLoading, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};
