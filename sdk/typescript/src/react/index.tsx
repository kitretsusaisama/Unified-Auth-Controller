import React, { createContext, useContext, useEffect, useState } from 'react';
import { BrowserClient } from '../browser/client';
import { User } from '../types';

interface AuthContextType {
  client: BrowserClient;
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: () => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider: React.FC<{ client: BrowserClient, children: React.ReactNode }> = ({ client, children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Check initial state
    const init = async () => {
        const token = await client.getAccessToken();
        setIsAuthenticated(!!token);
        // Fetch user if token exists...
        setIsLoading(false);
    };
    init();
  }, [client]);

  return (
    <AuthContext.Provider value={{
      client,
      user,
      isAuthenticated,
      isLoading,
      login: () => client.login(),
      logout: () => client.logout()
    }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) throw new Error('useAuth must be used within AuthProvider');
  return context;
};
