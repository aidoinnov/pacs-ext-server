import React, { useState } from 'react';
import axios from 'axios';
import '../ApiHealthCheck.css';

const DEFAULT_API_URL = 'http://localhost:8080';
const DEFAULT_USERNAME = 'iaid-pacs-admin';
const DEFAULT_PASSWORD = 'Qlalfqjsgh1!';

interface SnapshotData {
  annotation_id: number;
  image_key: string;
  upload_url: string;
  download_url: string;
  status: string;
  uploaded_at?: string;
}

const AnnotationSnapshotTests: React.FC = () => {
  const [apiUrl] = useState(DEFAULT_API_URL);
  const [username] = useState(DEFAULT_USERNAME);
  const [password] = useState(DEFAULT_PASSWORD);
  
  const [testRunning, setTestRunning] = useState(false);
  const [testOutput, setTestOutput] = useState<string>('');
  
  // CRUD State
  const [annotationId, setAnnotationId] = useState<string>('');
  const [snapshotData, setSnapshotData] = useState<SnapshotData | null>(null);
  const [crudMessage, setCrudMessage] = useState<{ type: 'success' | 'error', text: string } | null>(null);

  const getToken = async (): Promise<string> => {
    const response = await axios.post(`${apiUrl}/api/auth/login`, {
      username,
      password,
    });
    return response.data.token;
  };

  const runE2ETest = async () => {
    setTestRunning(true);
    setTestOutput('🚀 E2E 테스트 시작...\n\n');
    
    try {
      const response = await axios.get(`${apiUrl}/api/test/annotation-snapshot-e2e`);
      setTestOutput(prev => prev + response.data);
    } catch (error: any) {
      setTestOutput(prev => prev + `\n❌ 에러: ${error.message}\n`);
      if (error.response?.data) {
        setTestOutput(prev => prev + `응답: ${JSON.stringify(error.response.data, null, 2)}\n`);
      }
    } finally {
      setTestRunning(false);
    }
  };

  const createAnnotation = async () => {
    setCrudMessage(null);
    try {
      const token = await getToken();
      const response = await axios.post(
        `${apiUrl}/api/annotations`,
        {
          project_id: 1,
          study_instance_uid: '1.2.3.4.5',
          series_instance_uid: '1.2.3.4.5.6',
          sop_instance_uid: '1.2.3.4.5.6.7',
          tool_name: 'Test Tool',
          annotation_data: { type: 'circle', x: 100, y: 100, radius: 50 },
          label: 'Test Annotation',
        },
        { headers: { Authorization: `Bearer ${token}` } }
      );
      setAnnotationId(response.data.id.toString());
      setCrudMessage({ type: 'success', text: `어노테이션 생성 완료! ID: ${response.data.id}` });
    } catch (error: any) {
      setCrudMessage({ type: 'error', text: error.response?.data?.message || error.message });
    }
  };

  const requestUploadUrl = async () => {
    if (!annotationId) {
      setCrudMessage({ type: 'error', text: '어노테이션 ID를 입력하세요' });
      return;
    }
    
    setCrudMessage(null);
    try {
      const token = await getToken();
      const response = await axios.post(
        `${apiUrl}/api/annotations/${annotationId}/snapshot/upload-url`,
        { filename: 'test_snapshot.png' },
        { headers: { Authorization: `Bearer ${token}` } }
      );
      setSnapshotData({
        annotation_id: parseInt(annotationId),
        image_key: response.data.image_key,
        upload_url: response.data.upload_url,
        download_url: response.data.download_url,
        status: 'pending',
      });
      setCrudMessage({ type: 'success', text: '업로드 URL 생성 완료!' });
    } catch (error: any) {
      setCrudMessage({ type: 'error', text: error.response?.data?.message || error.message });
    }
  };

  const completeUpload = async () => {
    if (!annotationId || !snapshotData) {
      setCrudMessage({ type: 'error', text: '먼저 업로드 URL을 요청하세요' });
      return;
    }
    
    setCrudMessage(null);
    try {
      const token = await getToken();
      const response = await axios.post(
        `${apiUrl}/api/annotations/${annotationId}/snapshot/complete-upload`,
        {
          image_key: snapshotData.image_key,
          success: true,
        },
        { headers: { Authorization: `Bearer ${token}` } }
      );
      setSnapshotData({
        ...snapshotData,
        status: response.data.snapshot_status || 'completed',
        uploaded_at: response.data.snapshot_uploaded_at,
      });
      setCrudMessage({ type: 'success', text: '업로드 완료 처리 성공!' });
    } catch (error: any) {
      setCrudMessage({ type: 'error', text: error.response?.data?.message || error.message });
    }
  };

  const getSnapshotStatus = async () => {
    if (!annotationId) {
      setCrudMessage({ type: 'error', text: '어노테이션 ID를 입력하세요' });
      return;
    }

    setCrudMessage(null);
    try {
      const token = await getToken();
      const response = await axios.get(
        `${apiUrl}/api/annotations/${annotationId}/snapshot/status`,
        { headers: { Authorization: `Bearer ${token}` } }
      );
      setSnapshotData({
        annotation_id: response.data.annotation_id,
        image_key: response.data.image_key,
        upload_url: '',
        download_url: '',
        status: response.data.status,
        uploaded_at: response.data.uploaded_at,
      });
      setCrudMessage({ type: 'success', text: '상태 조회 완료!' });
    } catch (error: any) {
      setCrudMessage({ type: 'error', text: error.response?.data?.message || error.message });
    }
  };

  return (
    <div className="annotation-snapshot-tests">
      <div className="test-section">
        <h2>📸 Annotation Snapshot 테스트</h2>

        {/* E2E 테스트 실행 */}
        <div className="test-card">
          <h3>🚀 E2E 테스트 실행</h3>
          <p>Python 테스트 스크립트를 실행합니다 (서버에서 실행)</p>
          <button
            onClick={runE2ETest}
            disabled={testRunning}
            className="btn-primary"
          >
            {testRunning ? '실행 중...' : 'E2E 테스트 실행'}
          </button>

          {testOutput && (
            <pre className="test-output">{testOutput}</pre>
          )}
        </div>

        {/* CRUD 인터페이스 */}
        <div className="test-card">
          <h3>🔧 CRUD 인터페이스</h3>

          {crudMessage && (
            <div className={`message ${crudMessage.type}`}>
              {crudMessage.text}
            </div>
          )}

          {/* 1. 어노테이션 생성 */}
          <div className="crud-section">
            <h4>1️⃣ 어노테이션 생성</h4>
            <button onClick={createAnnotation} className="btn-secondary">
              테스트 어노테이션 생성
            </button>
          </div>

          {/* 2. 어노테이션 ID 입력 */}
          <div className="crud-section">
            <h4>2️⃣ 어노테이션 ID</h4>
            <input
              type="text"
              value={annotationId}
              onChange={(e) => setAnnotationId(e.target.value)}
              placeholder="어노테이션 ID 입력"
              className="input-field"
            />
          </div>

          {/* 3. 업로드 URL 요청 */}
          <div className="crud-section">
            <h4>3️⃣ 업로드 URL 요청</h4>
            <button onClick={requestUploadUrl} className="btn-secondary">
              업로드 URL 생성
            </button>
          </div>

          {/* 4. 스냅샷 데이터 표시 */}
          {snapshotData && (
            <div className="crud-section">
              <h4>📊 스냅샷 데이터</h4>
              <div className="data-display">
                <div className="data-row">
                  <span className="label">Annotation ID:</span>
                  <span className="value">{snapshotData.annotation_id}</span>
                </div>
                <div className="data-row">
                  <span className="label">Image Key:</span>
                  <span className="value">{snapshotData.image_key}</span>
                </div>
                <div className="data-row">
                  <span className="label">Status:</span>
                  <span className={`value status-${snapshotData.status}`}>
                    {snapshotData.status}
                  </span>
                </div>
                {snapshotData.uploaded_at && (
                  <div className="data-row">
                    <span className="label">Uploaded At:</span>
                    <span className="value">{snapshotData.uploaded_at}</span>
                  </div>
                )}
                {snapshotData.upload_url && (
                  <div className="data-row">
                    <span className="label">Upload URL:</span>
                    <span className="value url">{snapshotData.upload_url.substring(0, 80)}...</span>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* 5. 업로드 완료 처리 */}
          <div className="crud-section">
            <h4>4️⃣ 업로드 완료 처리</h4>
            <button onClick={completeUpload} className="btn-secondary">
              업로드 완료 알림
            </button>
          </div>

          {/* 6. 상태 조회 */}
          <div className="crud-section">
            <h4>5️⃣ 상태 조회</h4>
            <button onClick={getSnapshotStatus} className="btn-secondary">
              스냅샷 상태 조회
            </button>
          </div>
        </div>
      </div>

      <style>{`
        .annotation-snapshot-tests {
          padding: 20px;
        }

        .test-section {
          max-width: 1200px;
          margin: 0 auto;
        }

        .test-card {
          background: white;
          border-radius: 8px;
          padding: 24px;
          margin-bottom: 24px;
          box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        }

        .test-card h3 {
          margin-top: 0;
          color: #1e293b;
          font-size: 20px;
          margin-bottom: 12px;
        }

        .test-card p {
          color: #64748b;
          margin-bottom: 16px;
        }

        .btn-primary, .btn-secondary {
          padding: 10px 20px;
          border: none;
          border-radius: 6px;
          font-size: 14px;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s;
        }

        .btn-primary {
          background: #3b82f6;
          color: white;
        }

        .btn-primary:hover:not(:disabled) {
          background: #2563eb;
        }

        .btn-primary:disabled {
          background: #94a3b8;
          cursor: not-allowed;
        }

        .btn-secondary {
          background: #10b981;
          color: white;
          margin-right: 8px;
          margin-bottom: 8px;
        }

        .btn-secondary:hover {
          background: #059669;
        }

        .test-output {
          background: #1e293b;
          color: #e2e8f0;
          padding: 16px;
          border-radius: 6px;
          overflow-x: auto;
          font-family: 'Courier New', monospace;
          font-size: 13px;
          line-height: 1.6;
          margin-top: 16px;
          max-height: 500px;
          overflow-y: auto;
        }

        .message {
          padding: 12px 16px;
          border-radius: 6px;
          margin-bottom: 16px;
          font-weight: 500;
        }

        .message.success {
          background: #d1fae5;
          color: #065f46;
          border: 1px solid #10b981;
        }

        .message.error {
          background: #fee2e2;
          color: #991b1b;
          border: 1px solid #ef4444;
        }

        .crud-section {
          margin-bottom: 20px;
          padding-bottom: 20px;
          border-bottom: 1px solid #e5e7eb;
        }

        .crud-section:last-child {
          border-bottom: none;
        }

        .crud-section h4 {
          color: #475569;
          font-size: 16px;
          margin-bottom: 12px;
        }

        .input-field {
          width: 100%;
          max-width: 400px;
          padding: 10px 12px;
          border: 1px solid #cbd5e1;
          border-radius: 6px;
          font-size: 14px;
        }

        .input-field:focus {
          outline: none;
          border-color: #3b82f6;
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        }

        .data-display {
          background: #f8fafc;
          border: 1px solid #e2e8f0;
          border-radius: 6px;
          padding: 16px;
        }

        .data-row {
          display: flex;
          padding: 8px 0;
          border-bottom: 1px solid #e2e8f0;
        }

        .data-row:last-child {
          border-bottom: none;
        }

        .data-row .label {
          font-weight: 600;
          color: #475569;
          min-width: 150px;
        }

        .data-row .value {
          color: #1e293b;
          flex: 1;
          word-break: break-all;
        }

        .data-row .value.url {
          font-family: 'Courier New', monospace;
          font-size: 12px;
        }

        .status-pending {
          color: #f59e0b;
          font-weight: 600;
        }

        .status-uploading {
          color: #3b82f6;
          font-weight: 600;
        }

        .status-completed {
          color: #10b981;
          font-weight: 600;
        }

        .status-failed {
          color: #ef4444;
          font-weight: 600;
        }
      `}</style>
    </div>
  );
};

export default AnnotationSnapshotTests;

