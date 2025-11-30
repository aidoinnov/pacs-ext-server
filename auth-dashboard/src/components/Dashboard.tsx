import React, { useState, useEffect } from 'react';
import axios from 'axios';
import './Dashboard.css';
import ApiHealthCheck from './ApiHealthCheck';
import ApiScenarioTests from './ApiScenarioTests';
import Sidebar from './Sidebar';
import {
  DEFAULT_API_URL,
  DASHBOARD_PAGE,
  AUTH_TEST_SECTION,
  TIME_FORMAT,
  HTTP_METHOD_COLORS,
  SIDEBAR_MENU,
} from '../constants/app.constants';

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

interface DashboardProps {
  user: User;
  tokens: AuthTokens;
  onLogout: () => void;
}

const Dashboard: React.FC<DashboardProps> = ({ user, tokens: initialTokens, onLogout }) => {
  const [tokens, setTokens] = useState(initialTokens);
  const [apiUrl] = useState(DEFAULT_API_URL);
  const [verifyResult, setVerifyResult] = useState<any>(null);
  const [refreshResult, setRefreshResult] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [tokenExpiry, setTokenExpiry] = useState<Date | null>(null);
  const [refreshExpiry, setRefreshExpiry] = useState<Date | null>(null);
  const [activeMenu, setActiveMenu] = useState<string>(SIDEBAR_MENU.AUTH_TEST.id);

  useEffect(() => {
    // Calculate token expiry times
    const now = new Date();
    setTokenExpiry(new Date(now.getTime() + tokens.expires_in * 1000));
    setRefreshExpiry(new Date(now.getTime() + tokens.refresh_expires_in * 1000));
  }, [tokens]);

  const handleVerifyToken = async () => {
    setLoading(true);
    setVerifyResult(null);

    try {
      const response = await axios.get(`${apiUrl}/api/auth/verify/${tokens.token}`);
      setVerifyResult({ success: true, data: response.data });
    } catch (err: any) {
      setVerifyResult({ success: false, error: err.response?.data || err.message });
    } finally {
      setLoading(false);
    }
  };

  const handleRefreshToken = async () => {
    setLoading(true);
    setRefreshResult('');

    try {
      const response = await axios.post(`${apiUrl}/api/auth/refresh`, {
        refresh_token: tokens.refresh_token,
      });

      const newTokens = {
        token: response.data.token,
        refresh_token: response.data.refresh_token,
        expires_in: response.data.expires_in,
        refresh_expires_in: response.data.refresh_expires_in,
      };

      setTokens(newTokens);
      setRefreshResult('✅ Tokens refreshed successfully!');
    } catch (err: any) {
      setRefreshResult(`❌ Refresh failed: ${err.response?.data?.message || err.message}`);
    } finally {
      setLoading(false);
    }
  };

  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text);
    alert(`${label} ${AUTH_TEST_SECTION.TOKEN_INFO.MESSAGE_COPIED}`);
  };

  const formatDate = (date: Date | null) => {
    if (!date) return TIME_FORMAT.NOT_AVAILABLE;
    return date.toLocaleString();
  };

  const getTimeRemaining = (expiry: Date | null) => {
    if (!expiry) return TIME_FORMAT.NOT_AVAILABLE;
    const now = new Date();
    const diff = expiry.getTime() - now.getTime();
    if (diff <= 0) return TIME_FORMAT.EXPIRED;

    const minutes = Math.floor(diff / 60000);
    const seconds = Math.floor((diff % 60000) / 1000);
    return `${minutes}m ${seconds}s`;
  };

  return (
    <div className="dashboard-layout">
      <Sidebar activeMenu={activeMenu} onMenuChange={setActiveMenu} />

      <div className="dashboard-main">
        <div className="dashboard-header">
          <h1>{DASHBOARD_PAGE.TITLE}</h1>
          <button onClick={onLogout} className="logout-button">
            {DASHBOARD_PAGE.BUTTON_LOGOUT}
          </button>
        </div>

        <div className="dashboard-content">
          {activeMenu === SIDEBAR_MENU.AUTH_TEST.id && (
            <>
              {/* User Info */}
              <div className="card">
                <h2>{AUTH_TEST_SECTION.USER_INFO.TITLE}</h2>
                <div className="info-grid">
                  <div className="info-item">
                    <span className="label">{AUTH_TEST_SECTION.USER_INFO.LABEL_USER_ID}:</span>
                    <span className="value">{user.user_id}</span>
                  </div>
                  <div className="info-item">
                    <span className="label">{AUTH_TEST_SECTION.USER_INFO.LABEL_USERNAME}:</span>
                    <span className="value">{user.username}</span>
                  </div>
                  <div className="info-item">
                    <span className="label">{AUTH_TEST_SECTION.USER_INFO.LABEL_EMAIL}:</span>
                    <span className="value">{user.email}</span>
                  </div>
                  <div className="info-item">
                    <span className="label">{AUTH_TEST_SECTION.USER_INFO.LABEL_KEYCLOAK_ID}:</span>
                    <span className="value code">{user.keycloak_id}</span>
                  </div>
                </div>
              </div>

              {/* Token Info */}
              <div className="card">
                <h2>{AUTH_TEST_SECTION.TOKEN_INFO.TITLE}</h2>

                <div className="token-section">
                  <h3>{AUTH_TEST_SECTION.TOKEN_INFO.JWT_TOKEN}</h3>
                  <div className="token-display">
                    <code className="token-value">{tokens.token.substring(0, 50)}...</code>
                    <button onClick={() => copyToClipboard(tokens.token, 'JWT Token')} className="copy-button">
                      {AUTH_TEST_SECTION.TOKEN_INFO.BUTTON_COPY}
                    </button>
                  </div>
                  <div className="token-meta">
                    <span>{AUTH_TEST_SECTION.TOKEN_INFO.LABEL_EXPIRES}: {formatDate(tokenExpiry)}</span>
                    <span className="time-remaining">{AUTH_TEST_SECTION.TOKEN_INFO.LABEL_TIME_REMAINING} {getTimeRemaining(tokenExpiry)}</span>
                  </div>
                </div>

                <div className="token-section">
                  <h3>{AUTH_TEST_SECTION.TOKEN_INFO.REFRESH_TOKEN}</h3>
                  <div className="token-display">
                    <code className="token-value">{tokens.refresh_token.substring(0, 50)}...</code>
                    <button onClick={() => copyToClipboard(tokens.refresh_token, 'Refresh Token')} className="copy-button">
                      {AUTH_TEST_SECTION.TOKEN_INFO.BUTTON_COPY}
                    </button>
                  </div>
                  <div className="token-meta">
                    <span>{AUTH_TEST_SECTION.TOKEN_INFO.LABEL_EXPIRES}: {formatDate(refreshExpiry)}</span>
                    <span className="time-remaining">{AUTH_TEST_SECTION.TOKEN_INFO.LABEL_TIME_REMAINING} {getTimeRemaining(refreshExpiry)}</span>
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div className="card">
                <h2>{AUTH_TEST_SECTION.TEST_ACTIONS.TITLE}</h2>

                <div className="action-section">
                  <button
                    onClick={handleVerifyToken}
                    disabled={loading}
                    className="action-button verify-button"
                  >
                    {AUTH_TEST_SECTION.TEST_ACTIONS.BUTTON_VERIFY}
                  </button>

                  {verifyResult && (
                    <div className={`result-box ${verifyResult.success ? 'success' : 'error'}`}>
                      <h4>{verifyResult.success ? AUTH_TEST_SECTION.TEST_ACTIONS.RESULT_VALID : AUTH_TEST_SECTION.TEST_ACTIONS.RESULT_INVALID}</h4>
                      <pre>{JSON.stringify(verifyResult.success ? verifyResult.data : verifyResult.error, null, 2)}</pre>
                    </div>
                  )}
                </div>

                <div className="action-section">
                  <button
                    onClick={handleRefreshToken}
                    disabled={loading}
                    className="action-button refresh-button"
                  >
                    {AUTH_TEST_SECTION.TEST_ACTIONS.BUTTON_REFRESH}
                  </button>

                  {refreshResult && (
                    <div className={`result-box ${refreshResult.includes('✅') ? 'success' : 'error'}`}>
                      {refreshResult}
                    </div>
                  )}
                </div>
              </div>

              {/* API Endpoints */}
              <div className="card">
                <h2>{AUTH_TEST_SECTION.API_ENDPOINTS.TITLE}</h2>
                <div className="endpoint-list">
                  <div className="endpoint-item">
                    <span className={`method ${HTTP_METHOD_COLORS.POST}`}>{AUTH_TEST_SECTION.API_ENDPOINTS.LOGIN.METHOD}</span>
                    <code>{AUTH_TEST_SECTION.API_ENDPOINTS.LOGIN.PATH}</code>
                    <span className="description">{AUTH_TEST_SECTION.API_ENDPOINTS.LOGIN.DESCRIPTION}</span>
                  </div>
                  <div className="endpoint-item">
                    <span className={`method ${HTTP_METHOD_COLORS.GET}`}>{AUTH_TEST_SECTION.API_ENDPOINTS.VERIFY.METHOD}</span>
                    <code>{AUTH_TEST_SECTION.API_ENDPOINTS.VERIFY.PATH}</code>
                    <span className="description">{AUTH_TEST_SECTION.API_ENDPOINTS.VERIFY.DESCRIPTION}</span>
                  </div>
                  <div className="endpoint-item">
                    <span className={`method ${HTTP_METHOD_COLORS.POST}`}>{AUTH_TEST_SECTION.API_ENDPOINTS.REFRESH.METHOD}</span>
                    <code>{AUTH_TEST_SECTION.API_ENDPOINTS.REFRESH.PATH}</code>
                    <span className="description">{AUTH_TEST_SECTION.API_ENDPOINTS.REFRESH.DESCRIPTION}</span>
                  </div>
                </div>
              </div>
            </>
          )}

          {activeMenu === SIDEBAR_MENU.API_HEALTH.id && (
            <>
              <ApiHealthCheck />
              <ApiScenarioTests />
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default Dashboard;

