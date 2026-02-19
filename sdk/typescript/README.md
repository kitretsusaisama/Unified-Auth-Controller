# @upflame/sso

Production-ready Identity SDK for the Upflame Platform. Fully compatible with Browser, Server (Node.js), Next.js, and Edge environments.

## Features

- **Universal Auth**: Supports Browser (PKCE), Server (Confidential), and Service (Client Credentials) flows.
- **Environment Agnostic**: Automatically detects environment and selects optimal storage/flow.
- **Framework Integration**: Built-in helpers for Next.js (App Router, Middleware) and React.
- **Advanced Lifecycle**: Automatic token refresh, refresh locking, and session management.
- **Type Safety**: Full TypeScript support.

## Installation

```bash
npm install @upflame/sso
```

## Usage

### Browser (React/SPA)

```typescript
import { AuthClient, AuthProvider, useAuth } from '@upflame/sso';

// 1. Initialize Client
const authClient = AuthClient.createBrowserClient({
  baseUrl: 'https://auth.example.com',
  clientId: 'your-client-id',
  redirectUri: window.location.origin + '/callback',
});

// 2. Wrap App
function App() {
  return (
    <AuthProvider client={authClient}>
      <Main />
    </AuthProvider>
  );
}

// 3. Use Hook
function Main() {
  const { user, login, logout } = useAuth();
  if (!user) return <button onClick={() => login()}>Login</button>;
  return <div>Welcome {user.email} <button onClick={logout}>Logout</button></div>;
}
```

### Server (Node.js)

```typescript
import { AuthClient } from '@upflame/sso';

const client = AuthClient.createServerClient({
  baseUrl: process.env.AUTH_URL,
  clientId: process.env.CLIENT_ID,
  clientSecret: process.env.CLIENT_SECRET, // Required for server
});

// Exchange code from callback
await client.exchangeCode(code, redirectUri);
```

### Next.js (App Router)

```typescript
// app/auth.ts
import { AuthClient } from '@upflame/sso';

export const auth = AuthClient.autoDetect({
  baseUrl: process.env.AUTH_URL,
  clientId: process.env.CLIENT_ID,
  clientSecret: process.env.CLIENT_SECRET,
});
```

## Security

- **Browser**: Uses Authorization Code Flow with PKCE. Tokens stored in secure storage (defaults to LocalStorage, configurable to Cookie/Memory).
- **Server**: Uses Confidential Client flow. `clientSecret` never exposed to browser.
