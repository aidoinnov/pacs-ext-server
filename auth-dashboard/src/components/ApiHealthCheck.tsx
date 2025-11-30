import React, { useState } from 'react';
import axios from 'axios';
import './ApiHealthCheck.css';

interface HealthStatus {
  status: 'healthy' | 'unhealthy' | 'checking';
  message?: string;
  timestamp?: string;
}

const ApiHealthCheck: React.FC = () => {
  const [apiUrl] = useState('http://localhost:8080');
  const [healthStatus, setHealthStatus] = useState<HealthStatus>({ status: 'checking' });
  const [isChecking, setIsChecking] = useState(false);

  const checkHealth = async () => {
    setIsChecking(true);
    setHealthStatus({ status: 'checking' });

    try {
      const response = await axios.get(`${apiUrl}/health`, { timeout: 5000 });
      
      if (response.status === 200) {
        setHealthStatus({
          status: 'healthy',
          message: 'API 서버가 정상적으로 동작 중입니다',
          timestamp: new Date().toISOString(),
        });
      } else {
        setHealthStatus({
          status: 'unhealthy',
          message: `예상치 못한 응답: ${response.status}`,
          timestamp: new Date().toISOString(),
        });
      }
    } catch (error: any) {
      setHealthStatus({
        status: 'unhealthy',
        message: error.message || 'API 서버에 연결할 수 없습니다',
        timestamp: new Date().toISOString(),
      });
    } finally {
      setIsChecking(false);
    }
  };

  const getStatusIcon = () => {
    switch (healthStatus.status) {
      case 'healthy': return '✅';
      case 'unhealthy': return '❌';
      case 'checking': return '⏳';
      default: return '⚪';
    }
  };

  const getStatusColor = () => {
    switch (healthStatus.status) {
      case 'healthy': return '#10b981';
      case 'unhealthy': return '#ef4444';
      case 'checking': return '#f59e0b';
      default: return '#6b7280';
    }
  };

  return (
    <div className="api-health-check">
      <div className="health-check-header">
        <h2>🔍 API Health Check</h2>
        <p className="subtitle">API 서버 상태 확인</p>
      </div>

      <div className="health-check-content">
        <div className="health-status-card" style={{ borderColor: getStatusColor() }}>
          <div className="health-status-icon">{getStatusIcon()}</div>
          <div className="health-status-info">
            <h3>서버 상태</h3>
            <p className="status-message">{healthStatus.message || '상태를 확인하려면 버튼을 클릭하세요'}</p>
            {healthStatus.timestamp && (
              <p className="status-timestamp">확인 시간: {new Date(healthStatus.timestamp).toLocaleString()}</p>
            )}
          </div>
        </div>

        <div className="health-check-actions">
          <button
            onClick={checkHealth}
            disabled={isChecking}
            className="check-health-button"
            style={{
              backgroundColor: isChecking ? '#9ca3af' : '#3b82f6',
              color: 'white',
              padding: '12px 24px',
              border: 'none',
              borderRadius: '8px',
              fontSize: '16px',
              fontWeight: '500',
              cursor: isChecking ? 'not-allowed' : 'pointer',
            }}
          >
            {isChecking ? '⏳ 확인 중...' : '🔍 상태 확인'}
          </button>
        </div>

        <div className="health-check-info">
          <h4>API 엔드포인트 정보</h4>
          <ul>
            <li><strong>서버 URL:</strong> {apiUrl}</li>
            <li><strong>Health Check:</strong> GET {apiUrl}/health</li>
          </ul>
        </div>
      </div>
    </div>
  );
};

export default ApiHealthCheck;
