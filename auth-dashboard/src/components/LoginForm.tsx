import React, { useState } from 'react';
import axios from 'axios';
import './LoginForm.css';
import { DEFAULT_API_URL, LOGIN_PAGE } from '../constants/app.constants';

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
  const [apiUrl, setApiUrl] = useState(DEFAULT_API_URL);
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
        <h1>{LOGIN_PAGE.TITLE}</h1>
        <p className="subtitle">{LOGIN_PAGE.SUBTITLE}</p>

        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="apiUrl">{LOGIN_PAGE.LABEL_API_URL}</label>
            <input
              id="apiUrl"
              type="text"
              value={apiUrl}
              onChange={(e) => setApiUrl(e.target.value)}
              placeholder={LOGIN_PAGE.PLACEHOLDER_API_URL}
            />
          </div>

          <div className="form-group">
            <label htmlFor="username">{LOGIN_PAGE.LABEL_USERNAME}</label>
            <input
              id="username"
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder={LOGIN_PAGE.PLACEHOLDER_USERNAME}
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="password">{LOGIN_PAGE.LABEL_PASSWORD}</label>
            <input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder={LOGIN_PAGE.PLACEHOLDER_PASSWORD}
              required
            />
          </div>

          {error && <div className="error-message">{error}</div>}

          <button type="submit" disabled={loading} className="login-button">
            {loading ? LOGIN_PAGE.BUTTON_LOGGING_IN : LOGIN_PAGE.BUTTON_LOGIN}
          </button>
        </form>

        <div className="info-box">
          <h3>{LOGIN_PAGE.INFO_TITLE}</h3>
          <p>{LOGIN_PAGE.INFO_DESCRIPTION}</p>
          <p className="note">{LOGIN_PAGE.INFO_NOTE}</p>
        </div>
      </div>
    </div>
  );
};

export default LoginForm;

