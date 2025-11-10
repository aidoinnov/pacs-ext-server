import React, { useState } from 'react';
import axios from 'axios';
import './LoginForm.css';

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

interface LoginFormProps {
  onLogin: (user: User, tokens: AuthTokens) => void;
}

const LoginForm: React.FC<LoginFormProps> = ({ onLogin }) => {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [apiUrl, setApiUrl] = useState('http://localhost:8080');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');

    try {
      const response = await axios.post(`${apiUrl}/api/auth/login`, {
        username,
        password,
      });

      const { user_id, keycloak_id, username: userName, email, token, refresh_token, expires_in, refresh_expires_in } = response.data;

      onLogin(
        { user_id, keycloak_id, username: userName, email },
        { token, refresh_token, expires_in, refresh_expires_in }
      );
    } catch (err: any) {
      setError(err.response?.data?.message || err.message || 'Login failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="login-container">
      <div className="login-card">
        <h1>🔐 Auth API Dashboard</h1>
        <p className="subtitle">Test Authentication & Token Management</p>

        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="apiUrl">API URL</label>
            <input
              id="apiUrl"
              type="text"
              value={apiUrl}
              onChange={(e) => setApiUrl(e.target.value)}
              placeholder="http://localhost:8080"
            />
          </div>

          <div className="form-group">
            <label htmlFor="username">Username</label>
            <input
              id="username"
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="Enter username"
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="password">Password</label>
            <input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Enter password"
              required
            />
          </div>

          {error && <div className="error-message">{error}</div>}

          <button type="submit" disabled={loading} className="login-button">
            {loading ? 'Logging in...' : 'Login'}
          </button>
        </form>

        <div className="info-box">
          <h3>ℹ️ Test Credentials</h3>
          <p>Use your Keycloak credentials to test the authentication flow.</p>
          <p className="note">This dashboard tests the new username/password authentication API.</p>
        </div>
      </div>
    </div>
  );
};

export default LoginForm;

