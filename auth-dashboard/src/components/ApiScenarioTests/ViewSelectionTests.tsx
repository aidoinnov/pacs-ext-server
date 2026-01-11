import React, { useState } from 'react';
import axios from 'axios';
import './StudyListViewTests.css';
import { DEFAULT_API_URL } from './constants';
import { TEST_ACCOUNTS } from './constants';
import { getTestToken } from './utils';

// ============ 타입 정의 ============
interface SelectedSeries {
  study_uid: string;
  series_uid: string;
}

interface ViewSelectionResponse {
  selection_id: string;
  series: SelectedSeries[];
  created_at: string;
  expires_at: string;
  user_id: number;
}

const ViewSelectionTests: React.FC = () => {
  const [apiUrl] = useState(DEFAULT_API_URL);
  const [testToken, setTestToken] = useState<string | null>(null);

  // ============ View Selection 상태 ============
  const [selectionSeries, setSelectionSeries] = useState<SelectedSeries[]>([
    { study_uid: '1.2.840.113619.2.55.3.604641477.123.1234567890.123', series_uid: '1.2.840.113619.2.55.3.604641477.123.1234567890.124' }
  ]);
  const [createdSelectionId, setCreatedSelectionId] = useState<string | null>(null);
  const [selectionResult, setSelectionResult] = useState<ViewSelectionResponse | null>(null);
  const [selectionMessage, setSelectionMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [newSeriesForm, setNewSeriesForm] = useState({ study_uid: '', series_uid: '' });

  // ============ 토큰 획득 ============
  const getToken = async (): Promise<string> => {
    if (testToken) return testToken;
    return getTestToken(TEST_ACCOUNTS.SUPER_ADMIN, apiUrl, setTestToken, () => {});
  };

  // ============ View Selection 함수 ============
  const handleCreateSelection = async () => {
    if (selectionSeries.length === 0) {
      setSelectionMessage({ type: 'error', text: 'Series를 하나 이상 추가해주세요' });
      return;
    }
    try {
      const token = await getToken();
      const res = await axios.post(`${apiUrl}/api/v1/view-selections`, 
        { series: selectionSeries },
        { headers: { Authorization: `Bearer ${token}` } }
      );
      setCreatedSelectionId(res.data.selection_id);
      setSelectionMessage({ type: 'success', text: `Selection 생성 완료! ID: ${res.data.selection_id}` });
    } catch (e: any) {
      setSelectionMessage({ type: 'error', text: e.response?.data?.message || e.message });
    }
  };

  const handleGetSelection = async () => {
    if (!createdSelectionId) {
      setSelectionMessage({ type: 'error', text: 'Selection ID가 없습니다. 먼저 생성해주세요.' });
      return;
    }
    try {
      const token = await getToken();
      const res = await axios.get(`${apiUrl}/api/v1/view-selections/${createdSelectionId}`,
        { headers: { Authorization: `Bearer ${token}` } }
      );
      setSelectionResult(res.data);
      setSelectionMessage({ type: 'success', text: 'Selection 조회 성공!' });
    } catch (e: any) {
      setSelectionMessage({ type: 'error', text: e.response?.data?.message || e.message });
    }
  };

  const addSeriesToSelection = () => {
    if (!newSeriesForm.study_uid || !newSeriesForm.series_uid) {
      setSelectionMessage({ type: 'error', text: 'Study UID와 Series UID를 입력해주세요' });
      return;
    }
    setSelectionSeries([...selectionSeries, { ...newSeriesForm }]);
    setNewSeriesForm({ study_uid: '', series_uid: '' });
  };

  const removeSeriesFromSelection = (index: number) => {
    setSelectionSeries(selectionSeries.filter((_, i) => i !== index));
  };

  return (
    <div className="slv-container">
      <div className="slv-header">
        <h2>🎬 View Selection API 테스트</h2>
        <p className="slv-subtitle">Viewer에서 여러 Study/Series를 선택하여 세션을 생성합니다.</p>
      </div>

      {selectionMessage && (
        <div className={`slv-message ${selectionMessage.type}`}>
          {selectionMessage.type === 'success' ? '✅' : '❌'} {selectionMessage.text}
        </div>
      )}

      <div className="slv-selection-content">
        {/* 왼쪽: Series 목록 편집 */}
        <div className="slv-selection-left">
          <h4>📋 Series 목록 ({selectionSeries.length}개)</h4>
          
          <div className="series-list">
            {selectionSeries.map((s, idx) => (
              <div key={idx} className="series-item">
                <div className="series-info">
                  <span className="series-label">Study:</span>
                  <span className="series-uid">{s.study_uid.substring(0, 30)}...</span>
                </div>
                <div className="series-info">
                  <span className="series-label">Series:</span>
                  <span className="series-uid">{s.series_uid.substring(0, 30)}...</span>
                </div>
                <button className="btn-remove" onClick={() => removeSeriesFromSelection(idx)}>×</button>
              </div>
            ))}
          </div>

          <div className="add-series-form">
            <h5>➕ Series 추가</h5>
            <input
              type="text"
              placeholder="Study UID"
              value={newSeriesForm.study_uid}
              onChange={e => setNewSeriesForm({ ...newSeriesForm, study_uid: e.target.value })}
            />
            <input
              type="text"
              placeholder="Series UID"
              value={newSeriesForm.series_uid}
              onChange={e => setNewSeriesForm({ ...newSeriesForm, series_uid: e.target.value })}
            />
            <button className="btn-primary" onClick={addSeriesToSelection}>추가</button>
          </div>
        </div>

        {/* 오른쪽: API 테스트 */}
        <div className="slv-selection-right">
          <h4>🔧 API 테스트</h4>

          <div className="api-test-section">
            <div className="api-action">
              <span className="method post">POST</span>
              <span className="endpoint">/api/v1/view-selections</span>
              <button className="btn-primary" onClick={handleCreateSelection}>
                Selection 생성
              </button>
            </div>

            {createdSelectionId && (
              <div className="selection-id-display">
                <span>생성된 ID:</span>
                <code>{createdSelectionId}</code>
              </div>
            )}

            <div className="api-action">
              <span className="method get">GET</span>
              <span className="endpoint">/api/v1/view-selections/{'{id}'}</span>
              <button className="btn-secondary" onClick={handleGetSelection} disabled={!createdSelectionId}>
                Selection 조회
              </button>
            </div>
          </div>

          {selectionResult && (
            <div className="selection-result">
              <h5>📦 조회 결과</h5>
              <pre>{JSON.stringify(selectionResult, null, 2)}</pre>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default ViewSelectionTests;

