import React, { useState, useEffect } from 'react';
import axios from 'axios';
import './SyncMonitor.css';

interface SyncStatus {
  is_running: boolean;
  last_run: string | null;
  next_run: string | null;
  interval_sec: number;
}

interface SyncResult {
  success: boolean;
  processed: number;
  duration_ms: number;
  error: string | null;
}

interface SyncHistory {
  timestamp: string;
  result: SyncResult;
}

const SyncMonitor: React.FC = () => {
  const [apiUrl] = useState('http://localhost:8080');
  const [status, setStatus] = useState<SyncStatus | null>(null);
  const [lastResult, setLastResult] = useState<SyncResult | null>(null);
  const [history, setHistory] = useState<SyncHistory[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [intervalSec, setIntervalSec] = useState<number>(300);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 상태 조회
  const fetchStatus = async () => {
    try {
      const response = await axios.get(`${apiUrl}/api/sync/status`);
      setStatus(response.data);
      setError(null);
    } catch (err: any) {
      setError(err.message || '상태 조회 실패');
    }
  };

  // 스케줄 조회
  const fetchSchedule = async () => {
    try {
      const response = await axios.get(`${apiUrl}/api/sync/schedule`);
      setIntervalSec(response.data.interval_sec);
    } catch (err: any) {
      console.error('스케줄 조회 실패:', err);
    }
  };

  // 초기 로드 및 자동 갱신
  useEffect(() => {
    fetchStatus();
    fetchSchedule();
    
    const interval = setInterval(() => {
      fetchStatus();
    }, 5000); // 5초마다 갱신

    return () => clearInterval(interval);
  }, []);

  // 수동 동기화 실행
  const handleRunSync = async () => {
    setIsRunning(true);
    setLoading(true);
    setError(null);

    try {
      const response = await axios.post(`${apiUrl}/api/sync/run`, {}, { timeout: 65000 });
      const result: SyncResult = response.data;
      
      setLastResult(result);
      
      // 히스토리에 추가
      setHistory(prev => [{
        timestamp: new Date().toISOString(),
        result
      }, ...prev].slice(0, 10)); // 최근 10개만 유지

      await fetchStatus();
    } catch (err: any) {
      setError(err.message || '동기화 실행 실패');
      setLastResult({
        success: false,
        processed: 0,
        duration_ms: 0,
        error: err.message
      });
    } finally {
      setIsRunning(false);
      setLoading(false);
    }
  };

  // 일시 중지
  const handlePause = async () => {
    try {
      await axios.post(`${apiUrl}/api/sync/pause`);
      await fetchStatus();
    } catch (err: any) {
      setError(err.message || '일시 중지 실패');
    }
  };

  // 재개
  const handleResume = async () => {
    try {
      await axios.post(`${apiUrl}/api/sync/resume`);
      await fetchStatus();
    } catch (err: any) {
      setError(err.message || '재개 실패');
    }
  };

  // 스케줄 변경
  const handleUpdateSchedule = async () => {
    try {
      await axios.put(`${apiUrl}/api/sync/schedule`, {
        interval_sec: intervalSec
      });
      await fetchSchedule();
      alert(`동기화 간격이 ${intervalSec}초로 변경되었습니다.`);
    } catch (err: any) {
      setError(err.message || '스케줄 변경 실패');
    }
  };

  const formatDateTime = (dateStr: string | null) => {
    if (!dateStr) return 'N/A';
    return new Date(dateStr).toLocaleString('ko-KR');
  };

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}초`;
  };

  return (
    <div className="sync-monitor">
      <div className="sync-header">
        <h2>🔄 동기화 모니터링</h2>
        <p className="subtitle">PACS 데이터 동기화 상태 및 제어</p>
      </div>

      {error && (
        <div className="error-banner">
          ❌ {error}
        </div>
      )}

      {/* 현재 상태 카드 */}
      <div className="sync-status-card">
        <h3>📊 현재 상태</h3>
        {status ? (
          <div className="status-grid">
            <div className="status-item">
              <span className="label">실행 상태:</span>
              <span className={`value ${status.is_running ? 'running' : 'idle'}`}>
                {status.is_running ? '🔄 실행 중' : '⏸️ 대기 중'}
              </span>
            </div>
            <div className="status-item">
              <span className="label">마지막 실행:</span>
              <span className="value">{formatDateTime(status.last_run)}</span>
            </div>
            <div className="status-item">
              <span className="label">다음 실행:</span>
              <span className="value">{formatDateTime(status.next_run)}</span>
            </div>
            <div className="status-item">
              <span className="label">실행 간격:</span>
              <span className="value">{status.interval_sec}초 ({Math.floor(status.interval_sec / 60)}분)</span>
            </div>
          </div>
        ) : (
          <p>상태 로딩 중...</p>
        )}
      </div>

      {/* 제어 버튼 */}
      <div className="sync-controls">
        <h3>🎮 동기화 제어</h3>
        <div className="control-buttons">
          <button
            onClick={handleRunSync}
            disabled={isRunning || loading || status?.is_running}
            className="btn btn-primary"
          >
            {isRunning ? '⏳ 실행 중...' : '▶️ 수동 실행'}
          </button>

          <button
            onClick={handlePause}
            disabled={loading}
            className="btn btn-warning"
          >
            ⏸️ 일시 중지
          </button>

          <button
            onClick={handleResume}
            disabled={loading}
            className="btn btn-success"
          >
            ▶️ 재개
          </button>

          <button
            onClick={fetchStatus}
            disabled={loading}
            className="btn btn-secondary"
          >
            🔄 상태 새로고침
          </button>
        </div>
      </div>

      {/* 스케줄 설정 */}
      <div className="sync-schedule">
        <h3>⏰ 스케줄 설정</h3>
        <div className="schedule-controls">
          <label>
            실행 간격 (초):
            <input
              type="number"
              value={intervalSec}
              onChange={(e) => setIntervalSec(Number(e.target.value))}
              min="60"
              step="60"
              className="interval-input"
            />
          </label>
          <button
            onClick={handleUpdateSchedule}
            disabled={loading}
            className="btn btn-primary"
          >
            💾 간격 변경
          </button>
          <div className="schedule-presets">
            <button onClick={() => setIntervalSec(300)} className="preset-btn">5분</button>
            <button onClick={() => setIntervalSec(600)} className="preset-btn">10분</button>
            <button onClick={() => setIntervalSec(1800)} className="preset-btn">30분</button>
            <button onClick={() => setIntervalSec(3600)} className="preset-btn">1시간</button>
          </div>
        </div>
      </div>

      {/* 마지막 실행 결과 */}
      {lastResult && (
        <div className={`sync-result ${lastResult.success ? 'success' : 'error'}`}>
          <h3>📋 마지막 실행 결과</h3>
          <div className="result-grid">
            <div className="result-item">
              <span className="label">상태:</span>
              <span className={`value ${lastResult.success ? 'success' : 'error'}`}>
                {lastResult.success ? '✅ 성공' : '❌ 실패'}
              </span>
            </div>
            <div className="result-item">
              <span className="label">처리 항목:</span>
              <span className="value">{lastResult.processed.toLocaleString()}개</span>
            </div>
            <div className="result-item">
              <span className="label">소요 시간:</span>
              <span className="value">{formatDuration(lastResult.duration_ms)}</span>
            </div>
            {lastResult.error && (
              <div className="result-item full-width">
                <span className="label">에러:</span>
                <span className="value error">{lastResult.error}</span>
              </div>
            )}
          </div>
        </div>
      )}

      {/* 실행 히스토리 */}
      {history.length > 0 && (
        <div className="sync-history">
          <h3>📜 실행 히스토리 (최근 10개)</h3>
          <div className="history-table">
            <table>
              <thead>
                <tr>
                  <th>시간</th>
                  <th>상태</th>
                  <th>처리 항목</th>
                  <th>소요 시간</th>
                  <th>에러</th>
                </tr>
              </thead>
              <tbody>
                {history.map((item, index) => (
                  <tr key={index}>
                    <td>{formatDateTime(item.timestamp)}</td>
                    <td className={item.result.success ? 'success' : 'error'}>
                      {item.result.success ? '✅ 성공' : '❌ 실패'}
                    </td>
                    <td>{item.result.processed.toLocaleString()}</td>
                    <td>{formatDuration(item.result.duration_ms)}</td>
                    <td className="error-cell">{item.result.error || '-'}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* 도움말 */}
      <div className="sync-help">
        <h3>ℹ️ 도움말</h3>
        <ul>
          <li><strong>수동 실행:</strong> 즉시 동기화를 실행합니다 (최대 60초 소요)</li>
          <li><strong>일시 중지:</strong> 자동 동기화를 중지합니다 (수동 실행은 가능)</li>
          <li><strong>재개:</strong> 일시 중지된 자동 동기화를 재개합니다</li>
          <li><strong>실행 간격:</strong> 자동 동기화 주기를 설정합니다 (최소 60초)</li>
          <li><strong>처리 항목:</strong> Study + Series + Instance의 총 처리 개수</li>
        </ul>
      </div>
    </div>
  );
};

export default SyncMonitor;

