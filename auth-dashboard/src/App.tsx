import React, { useState } from 'react';
import './App.css';
import LoginForm from './components/LoginForm';
import Dashboard from './components/Dashboard';

interface User {
  user_id: number;
  keycloak_id: string;
  username: string;
  email: string;
}

interface AuthTokens {
  token: string;
  refresh_token: string;
  expires_in: number;
  refresh_expires_in: number;
}

function App() {
  const [user, setUser] = useState<User | null>(null);
  const [tokens, setTokens] = useState<AuthTokens | null>(null);

  const handleLogin = (userData: User, authTokens: AuthTokens) => {
    setUser(userData);
    setTokens(authTokens);
  };

  const handleLogout = () => {
    setUser(null);
    setTokens(null);
  };

  return (
    <div className="App">
      {!user ? (
        <LoginForm onLogin={handleLogin} />
      ) : (
        <Dashboard user={user} tokens={tokens!} onLogout={handleLogout} />
      )}
    </div>
  );
}

export default App;
