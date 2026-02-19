# @auth-core/sdk

A Universal Microsoft-Level TypeScript SDK for the Auth Core platform.

## Features

- **Universal Auth Flow**: Handle any authentication method (Password, Passkey, Magic Link, OIDC) through a unified API.
- **Framework Agnostic**: Core logic is pure TypeScript.
- **React Integration**: `AuthProvider` and `useAuth` hook included.
- **Next.js Integration**: Server-side helpers and cookie storage adapter.
- **Token Management**: Automatic token refresh, storage, and expiration handling.
- **Type Safety**: Fully typed with TypeScript.

## Installation

```bash
npm install @auth-core/sdk
```

## Basic Usage

```typescript
import { AuthClient } from '@auth-core/sdk';

const client = new AuthClient({
  baseUrl: 'https://auth.example.com',
  clientId: 'your-client-id',
  redirectUri: 'https://app.example.com/callback',
});

// Initialize session
await client.init();

// Check if logged in
if (client.session.isAuthenticated()) {
  console.log('User:', client.session.getUser());
} else {
  // Start login flow
  const flow = await client.auth.startLoginFlow();
  console.log('Next step:', flow.step);
}
```

## React Usage

Wrap your app with `AuthProvider`:

```tsx
import { AuthClient } from '@auth-core/sdk';
import { AuthProvider } from '@auth-core/sdk/dist/framework/react';

const client = new AuthClient({ ... });

function App() {
  return (
    <AuthProvider client={client}>
      <Main />
    </AuthProvider>
  );
}
```

Use the hook:

```tsx
import { useAuth } from '@auth-core/sdk/dist/framework/react';

function Main() {
  const { user, login, logout } = useAuth();

  if (!user) return <button onClick={() => login()}>Login</button>;
  return <button onClick={logout}>Logout {user.email}</button>;
}
```

## Next.js (App Router)

In `layout.tsx` or Server Component:

```tsx
import { cookies } from 'next/headers';
import { createServerClient } from '@auth-core/sdk/dist/framework/nextjs';

export default async function Layout({ children }) {
  const cookieStore = await cookies();
  const client = createServerClient({ ... }, cookieStore);
  // ...
}
```
