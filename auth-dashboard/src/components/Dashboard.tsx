import React, { useState, useEffect } from 'react';
import axios from 'axios';
import './Dashboard.css';

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
  const [apiUrl] = useState('http://localhost:8080');
  const [verifyResult, setVerifyResult] = useState<any>(null);
  const [refreshResult, setRefreshResult] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [tokenExpiry, setTokenExpiry] = useState<Date | null>(null);
  const [refreshExpiry, setRefreshExpiry] = useState<Date | null>(null);

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
    alert(`${label} copied to clipboard!`);
  };

  const formatDate = (date: Date | null) => {
    if (!date) return 'N/A';
    return date.toLocaleString();
  };

  const getTimeRemaining = (expiry: Date | null) => {
    if (!expiry) return 'N/A';
    const now = new Date();
    const diff = expiry.getTime() - now.getTime();
    if (diff <= 0) return 'Expired';
    
    const minutes = Math.floor(diff / 60000);
    const seconds = Math.floor((diff % 60000) / 1000);
    return `${minutes}m ${seconds}s`;
  };

  return (
    <div className="dashboard-container">
      <div className="dashboard-header">
        <h1>🎛️ Auth Dashboard</h1>
        <button onClick={onLogout} className="logout-button">Logout</button>
      </div>

      <div className="dashboard-content">
        {/* User Info */}
        <div className="card">
          <h2>👤 User Information</h2>
          <div className="info-grid">
            <div className="info-item">
              <span className="label">User ID:</span>
              <span className="value">{user.user_id}</span>
            </div>
            <div className="info-item">
              <span className="label">Username:</span>
              <span className="value">{user.username}</span>
            </div>
            <div className="info-item">
              <span className="label">Email:</span>
              <span className="value">{user.email}</span>
            </div>
            <div className="info-item">
              <span className="label">Keycloak ID:</span>
              <span className="value code">{user.keycloak_id}</span>
            </div>
          </div>
        </div>

        {/* Token Info */}
        <div className="card">
          <h2>🔑 Token Information</h2>
          
          <div className="token-section">
            <h3>JWT Access Token</h3>
            <div className="token-display">
              <code className="token-value">{tokens.token.substring(0, 50)}...</code>
              <button onClick={() => copyToClipboard(tokens.token, 'JWT Token')} className="copy-button">
                📋 Copy
              </button>
            </div>
            <div className="token-meta">
              <span>Expires: {formatDate(tokenExpiry)}</span>
              <span className="time-remaining">⏱️ {getTimeRemaining(tokenExpiry)}</span>
            </div>
          </div>

          <div className="token-section">
            <h3>Refresh Token</h3>
            <div className="token-display">
              <code className="token-value">{tokens.refresh_token.substring(0, 50)}...</code>
              <button onClick={() => copyToClipboard(tokens.refresh_token, 'Refresh Token')} className="copy-button">
                📋 Copy
              </button>
            </div>
            <div className="token-meta">
              <span>Expires: {formatDate(refreshExpiry)}</span>
              <span className="time-remaining">⏱️ {getTimeRemaining(refreshExpiry)}</span>
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="card">
          <h2>🧪 Test Actions</h2>
          
          <div className="action-section">
            <button 
              onClick={handleVerifyToken} 
              disabled={loading}
              className="action-button verify-button"
            >
              🔍 Verify Token
            </button>
            
            {verifyResult && (
              <div className={`result-box ${verifyResult.success ? 'success' : 'error'}`}>
                <h4>{verifyResult.success ? '✅ Token Valid' : '❌ Token Invalid'}</h4>
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
              🔄 Refresh Tokens
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
          <h2>📡 API Endpoints</h2>
          <div className="endpoint-list">
            <div className="endpoint-item">
              <span className="method post">POST</span>
              <code>/api/auth/login</code>
              <span className="description">Login with username/password</span>
            </div>
            <div className="endpoint-item">
              <span className="method get">GET</span>
              <code>/api/auth/verify/:token</code>
              <span className="description">Verify JWT token</span>
            </div>
            <div className="endpoint-item">
              <span className="method post">POST</span>
              <code>/api/auth/refresh</code>
              <span className="description">Refresh tokens</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Dashboard;

