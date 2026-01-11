import React, { useState, useEffect, useRef } from 'react';
import axios from 'axios';
import './StudyListViewTests.css';
import { TestAccount } from './types';
import { TEST_ACCOUNTS, DEFAULT_API_URL } from './constants';
import { getTestToken } from './utils';

// ============ 타입 정의 ============
interface SimpleTestCase {
  id: string;
  name: string;
  description: string;
  status: 'pending' | 'running' | 'success' | 'failure';
  result?: string;
  duration?: number;
}

interface StudyListView {
  viewId: string;
  viewName: string;
  isSystem: boolean;
  ownerUserId?: number | null;
  description?: string;
  createdAt?: string;
}

interface ViewField {
  source: 'dicom' | 'extension';  // API 필드
  key: string;                     // API 필드
  label?: string;                  // API 응답에서 받음
  displayLabel?: string;           // 사용자 정의 라벨 (없으면 원본 label 사용)
  displayOrder: number;            // API 필드
  visible: boolean;                // API 필드
  pinned?: boolean;                // API 필드
  width?: number;                  // API 필드
}

interface FieldDef {
  key: string;          // API 응답: key
  label: string;        // API 응답: label
  source: 'dicom' | 'extension';  // API 응답: source
}

// ============ 탭 정의 ============
const TABS = {
  SCENARIO: { id: 'scenario', label: '🧪 시나리오 테스트' },
  CRUD: { id: 'crud', label: '⚙️ View 관리' },
  SELECTION: { id: 'selection', label: '🎬 View Selection' },
} as const;

// ============ View Selection 타입 ============
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

const StudyListViewTests: React.FC = () => {
  const [activeTab, setActiveTab] = useState<string>(TABS.SCENARIO.id);
  const [apiUrl] = useState(DEFAULT_API_URL);
  const [testToken, setTestToken] = useState<string | null>(null);

  // ============ 시나리오 테스트 상태 ============
  const [testCases, setTestCases] = useState<SimpleTestCase[]>([
    { id: 'field-defs', name: '필드 정의 조회', description: 'GET /api/study-list-views/field-defs', status: 'pending' },
    { id: 'dicom-filter', name: 'DICOM 필드 확인', description: 'DICOM 카테고리 필터링', status: 'pending' },
    { id: 'ext-filter', name: 'Extension 필드 확인', description: 'Extension 카테고리 필터링', status: 'pending' },
    { id: 'view-list', name: 'View 목록 조회', description: 'GET /api/study-list-views', status: 'pending' },
    { id: 'view-crud', name: 'View CRUD 테스트', description: '생성 → 조회 → 수정 → 삭제', status: 'pending' },
  ]);
  const [isRunningAll, setIsRunningAll] = useState(false);
  const fieldDefsRef = useRef<FieldDef[]>([]);

  // ============ CRUD 관리 상태 ============
  const [views, setViews] = useState<StudyListView[]>([]);
  const [selectedView, setSelectedView] = useState<StudyListView | null>(null);
  const [viewFields, setViewFields] = useState<ViewField[]>([]);
  const [fieldDefs, setFieldDefs] = useState<FieldDef[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [crudMessage, setCrudMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [newViewForm, setNewViewForm] = useState({ viewId: '', viewName: '', description: '' });
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [editMode, setEditMode] = useState(false);

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

  // ============ 시나리오 테스트 로직 ============
  const runSingleTest = async (testId: string) => {
    const updateTest = (id: string, updates: Partial<SimpleTestCase>) => {
      setTestCases(prev => prev.map(t => (t.id === id ? { ...t, ...updates } : t)));
    };
    updateTest(testId, { status: 'running', result: undefined });
    const startTime = Date.now();

    try {
      let result = '';
      switch (testId) {
        case 'field-defs': {
          const res = await axios.get(`${apiUrl}/api/study-list-views/field-defs`);
          fieldDefsRef.current = res.data.items || [];
          result = `✅ ${res.data.total || fieldDefsRef.current.length}개 필드`;
          break;
        }
        case 'dicom-filter': {
          const dicom = fieldDefsRef.current.filter(f => f.source === 'dicom');
          if (dicom.length === 0) throw new Error('DICOM 필드 없음');
          result = `✅ ${dicom.length}개 DICOM 필드`;
          break;
        }
        case 'ext-filter': {
          const ext = fieldDefsRef.current.filter(f => f.source === 'extension');
          if (ext.length === 0) throw new Error('Extension 필드 없음');
          result = `✅ ${ext.length}개 Extension 필드`;
          break;
        }
        case 'view-list': {
          const res = await axios.get(`${apiUrl}/api/study-list-views`);
          result = `✅ ${res.data.items?.length || 0}개 View`;
          break;
        }
        case 'view-crud': {
          const token = await getToken();
          const viewId = `test_${Date.now()}`;
          await axios.post(`${apiUrl}/api/study-list-views`,
            { viewId, viewName: 'Test', description: 'E2E', fields: [{ source: 'dicom', key: 'PatientName', displayOrder: 1, visible: true }] },
            { headers: { Authorization: `Bearer ${token}` } });
          await axios.get(`${apiUrl}/api/study-list-views/${viewId}`);
          await axios.put(`${apiUrl}/api/study-list-views/${viewId}`,
            { viewName: 'Updated', fields: [{ source: 'dicom', key: 'StudyDate', displayOrder: 1, visible: true }] },
            { headers: { Authorization: `Bearer ${token}` } });
          await axios.delete(`${apiUrl}/api/study-list-views/${viewId}`, { headers: { Authorization: `Bearer ${token}` } });
          result = '✅ CRUD 전체 성공';
          break;
        }
      }
      updateTest(testId, { status: 'success', result, duration: Date.now() - startTime });
    } catch (error: any) {
      updateTest(testId, { status: 'failure', result: `❌ ${error.response?.data?.message || error.message}`, duration: Date.now() - startTime });
    }
  };

  const runAllTests = async () => {
    setIsRunningAll(true);
    setTestCases(prev => prev.map(t => ({ ...t, status: 'pending', result: undefined })));
    for (const test of testCases) {
      await runSingleTest(test.id);
      await new Promise(r => setTimeout(r, 200));
    }
    setIsRunningAll(false);
  };

  const resetTests = () => {
    setTestCases(prev => prev.map(t => ({ ...t, status: 'pending', result: undefined, duration: undefined })));
  };

  const getTestStats = () => {
    const success = testCases.filter(t => t.status === 'success').length;
    const failure = testCases.filter(t => t.status === 'failure').length;
    return { total: testCases.length, success, failure };
  };


  // ============ CRUD 로직 ============
  const loadViews = async () => {
    setIsLoading(true);
    try {
      const res = await axios.get(`${apiUrl}/api/study-list-views`);
      setViews(res.data.items || []);
    } catch (e: any) {
      setCrudMessage({ type: 'error', text: e.message });
    }
    setIsLoading(false);
  };

  const loadFieldDefs = async () => {
    try {
      const res = await axios.get(`${apiUrl}/api/study-list-views/field-defs`);
      setFieldDefs(res.data.items || []);
    } catch (e: any) {
      console.error('필드 정의 로드 실패:', e);
    }
  };

  const loadViewDetail = async (viewId: string) => {
    try {
      const res = await axios.get(`${apiUrl}/api/study-list-views/${viewId}`);
      setSelectedView(res.data);
      setViewFields(res.data.fields || []);
      setEditMode(false);
    } catch (e: any) {
      setCrudMessage({ type: 'error', text: e.message });
    }
  };

  const handleCreateView = async () => {
    if (!newViewForm.viewId || !newViewForm.viewName) {
      setCrudMessage({ type: 'error', text: 'View ID와 이름은 필수입니다' });
      return;
    }
    try {
      const token = await getToken();
      await axios.post(`${apiUrl}/api/study-list-views`, {
        ...newViewForm,
        fields: [{ source: 'dicom', key: 'PatientName', displayOrder: 1, visible: true, width: 200 }]
      }, { headers: { Authorization: `Bearer ${token}` } });
      setCrudMessage({ type: 'success', text: 'View 생성 완료!' });
      setShowCreateForm(false);
      setNewViewForm({ viewId: '', viewName: '', description: '' });
      loadViews();
    } catch (e: any) {
      setCrudMessage({ type: 'error', text: e.response?.data?.message || e.response?.data || e.message });
    }
  };

  const handleUpdateView = async () => {
    if (!selectedView) return;
    try {
      const token = await getToken();
      await axios.put(`${apiUrl}/api/study-list-views/${selectedView.viewId}`, {
        viewName: selectedView.viewName,
        description: selectedView.description,
        fields: viewFields
      }, { headers: { Authorization: `Bearer ${token}` } });
      setCrudMessage({ type: 'success', text: 'View 수정 완료!' });
      setEditMode(false);
      loadViews();
    } catch (e: any) {
      setCrudMessage({ type: 'error', text: e.response?.data?.message || e.message });
    }
  };

  const handleDeleteView = async (viewId: string) => {
    if (!window.confirm(`정말 "${viewId}" View를 삭제하시겠습니까?`)) return;
    try {
      const token = await getToken();
      await axios.delete(`${apiUrl}/api/study-list-views/${viewId}`, {
        headers: { Authorization: `Bearer ${token}` }
      });
      setCrudMessage({ type: 'success', text: 'View 삭제 완료!' });
      setSelectedView(null);
      loadViews();
    } catch (e: any) {
      setCrudMessage({ type: 'error', text: e.response?.data?.message || e.message });
    }
  };

  const addField = (fieldKey: string) => {
    if (viewFields.find(f => f.key === fieldKey)) return;
    const fieldDef = fieldDefs.find(f => f.key === fieldKey);
    if (!fieldDef) return;
    setViewFields([...viewFields, {
      source: fieldDef.source,
      key: fieldKey,
      label: fieldDef.label,
      displayOrder: viewFields.length + 1,
      visible: true,
      width: 150
    }]);
  };

  const removeField = (fieldKey: string) => {
    setViewFields(viewFields.filter(f => f.key !== fieldKey));
  };

  const moveField = (index: number, direction: 'up' | 'down') => {
    const newFields = [...viewFields];
    const targetIndex = direction === 'up' ? index - 1 : index + 1;
    if (targetIndex < 0 || targetIndex >= newFields.length) return;
    [newFields[index], newFields[targetIndex]] = [newFields[targetIndex], newFields[index]];
    // displayOrder 재할당
    setViewFields(newFields.map((f, i) => ({ ...f, displayOrder: i + 1 })));
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

  useEffect(() => {
    if (activeTab === TABS.CRUD.id) {
      loadViews();
      loadFieldDefs();
    }
  }, [activeTab]);

  const stats = getTestStats();


  // ============ 렌더링 ============
  return (
    <div className="study-list-view-tests">
      {/* 탭 네비게이션 */}
      <div className="slv-tabs">
        {Object.values(TABS).map(tab => (
          <button
            key={tab.id}
            className={`slv-tab ${activeTab === tab.id ? 'active' : ''}`}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* 시나리오 테스트 탭 */}
      {activeTab === TABS.SCENARIO.id && (
        <div className="slv-scenario">
          <div className="slv-header">
            <h3>🧪 API 시나리오 테스트</h3>
            <div className="slv-actions">
              <button className="btn-primary" onClick={runAllTests} disabled={isRunningAll}>
                {isRunningAll ? '⏳ 실행 중...' : '▶️ 전체 실행'}
              </button>
              <button className="btn-secondary" onClick={resetTests}>🔄 리셋</button>
            </div>
          </div>

          <div className="slv-stats">
            <span>총 {stats.total}개</span>
            <span className="success">✅ {stats.success}</span>
            <span className="failure">❌ {stats.failure}</span>
          </div>

          <div className="slv-test-list">
            {testCases.map(test => (
              <div key={test.id} className={`slv-test-item ${test.status}`}>
                <div className="test-info">
                  <span className="test-status">
                    {test.status === 'pending' && '⏳'}
                    {test.status === 'running' && '🔄'}
                    {test.status === 'success' && '✅'}
                    {test.status === 'failure' && '❌'}
                  </span>
                  <div className="test-content">
                    <span className="test-name">{test.name}</span>
                    <span className="test-desc">{test.description}</span>
                  </div>
                </div>
                <div className="test-result">
                  {test.result && <span className="result-text">{test.result}</span>}
                  {test.duration && <span className="duration">{test.duration}ms</span>}
                  <button
                    className="btn-run"
                    onClick={() => runSingleTest(test.id)}
                    disabled={test.status === 'running' || isRunningAll}
                  >
                    ▶️
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* CRUD 관리 탭 */}
      {activeTab === TABS.CRUD.id && (
        <div className="slv-crud">
          {crudMessage && (
            <div className={`slv-message ${crudMessage.type}`}>
              {crudMessage.text}
              <button onClick={() => setCrudMessage(null)}>×</button>
            </div>
          )}

          <div className="slv-crud-layout">
            {/* 왼쪽: View 목록 */}
            <div className="slv-view-list">
              <div className="list-header">
                <h4>📋 View 목록</h4>
                <button className="btn-add" onClick={() => setShowCreateForm(true)}>+ 새 View</button>
              </div>

              {isLoading ? (
                <div className="loading">로딩 중...</div>
              ) : (
                <div className="view-items">
                  {views.map(view => (
                    <div
                      key={view.viewId}
                      className={`view-item ${selectedView?.viewId === view.viewId ? 'selected' : ''}`}
                      onClick={() => loadViewDetail(view.viewId)}
                    >
                      <span className="view-name">{view.viewName}</span>
                      <span className="view-id">{view.viewId}</span>
                      {view.isSystem && <span className="badge system">시스템</span>}
                    </div>
                  ))}
                  {views.length === 0 && <div className="empty">View가 없습니다</div>}
                </div>
              )}
            </div>

            {/* 오른쪽: View 상세/편집 */}
            <div className="slv-view-detail">
              {selectedView ? (
                <>
                  <div className="detail-header">
                    <h4>{editMode ? '✏️ View 편집' : '📄 View 상세'}</h4>
                    <div className="detail-actions">
                      {!editMode ? (
                        <>
                          <button className="btn-edit" onClick={() => setEditMode(true)}>✏️ 편집</button>
                          <button className="btn-delete" onClick={() => handleDeleteView(selectedView.viewId)}>🗑️ 삭제</button>
                        </>
                      ) : (
                        <>
                          <button className="btn-save" onClick={handleUpdateView}>💾 저장</button>
                          <button className="btn-cancel" onClick={() => { setEditMode(false); loadViewDetail(selectedView.viewId); }}>취소</button>
                        </>
                      )}
                    </div>
                  </div>

                  <div className="detail-form">
                    <div className="form-row">
                      <label>View ID</label>
                      <input type="text" value={selectedView.viewId} disabled />
                    </div>
                    <div className="form-row">
                      <label>View 이름</label>
                      <input
                        type="text"
                        value={selectedView.viewName}
                        disabled={!editMode}
                        onChange={e => setSelectedView({ ...selectedView, viewName: e.target.value })}
                      />
                    </div>
                    <div className="form-row">
                      <label>설명</label>
                      <input
                        type="text"
                        value={selectedView.description || ''}
                        disabled={!editMode}
                        onChange={e => setSelectedView({ ...selectedView, description: e.target.value })}
                      />
                    </div>

                    <div className="form-section">
                      <label>필드 목록 ({viewFields.length}개) {editMode && <span className="hint">↑↓로 순서 변경</span>}</label>
                      <div className="field-list">
                        {viewFields.map((field, idx) => {
                          const isDicom = field.source === 'dicom';
                          const hasCustomLabel = !!field.displayLabel;
                          return (
                            <div key={field.key} className="field-item">
                              {editMode && (
                                <div className="field-order-btns">
                                  <button disabled={idx === 0} onClick={() => moveField(idx, 'up')}>↑</button>
                                  <button disabled={idx === viewFields.length - 1} onClick={() => moveField(idx, 'down')}>↓</button>
                                </div>
                              )}
                              <span className="field-order">{idx + 1}</span>
                              <span className={`field-badge ${isDicom ? 'dicom' : 'ext'}`}>
                                {isDicom ? 'D' : 'E'}
                              </span>
                              <span className="field-key" title={hasCustomLabel ? `원본: ${field.label}` : undefined}>
                                {hasCustomLabel && <span className="custom-label-badge">✏️</span>}
                                {field.displayLabel || field.label || field.key}
                              </span>
                              {editMode && (
                                <input
                                  type="text"
                                  className="display-label-input"
                                  placeholder="표시명"
                                  value={field.displayLabel || ''}
                                  onChange={e => {
                                    const newFields = [...viewFields];
                                    newFields[idx] = { ...field, displayLabel: e.target.value || undefined };
                                    setViewFields(newFields);
                                  }}
                                  title="커스텀 표시명 (비우면 원본 라벨 사용)"
                                />
                              )}
                              <span className="field-width">W:{field.width || 150}</span>
                              {editMode && (
                                <button className="btn-remove" onClick={() => removeField(field.key)}>×</button>
                              )}
                            </div>
                          );
                        })}
                      </div>

                      {editMode && (
                        <div className="add-field">
                          <select onChange={e => { if (e.target.value) addField(e.target.value); e.target.value = ''; }}>
                            <option value="">+ 필드 추가...</option>
                            <optgroup label="📊 DICOM 필드">
                              {fieldDefs
                                .filter(f => f.source === 'dicom' && !viewFields.find(vf => vf.key === f.key))
                                .map(f => (
                                  <option key={f.key} value={f.key}>{f.label}</option>
                                ))}
                            </optgroup>
                            <optgroup label="🔧 Extension 필드">
                              {fieldDefs
                                .filter(f => f.source === 'extension' && !viewFields.find(vf => vf.key === f.key))
                                .map(f => (
                                  <option key={f.key} value={f.key}>{f.label}</option>
                                ))}
                            </optgroup>
                          </select>
                        </div>
                      )}
                    </div>
                  </div>
                </>
              ) : (
                <div className="no-selection">
                  <p>👈 왼쪽에서 View를 선택하세요</p>
                </div>
              )}
            </div>
          </div>

          {/* 새 View 생성 모달 */}
          {showCreateForm && (
            <div className="slv-modal-overlay">
              <div className="slv-modal">
                <h4>➕ 새 View 생성</h4>
                <div className="form-row">
                  <label>View ID *</label>
                  <input
                    type="text"
                    value={newViewForm.viewId}
                    onChange={e => setNewViewForm({ ...newViewForm, viewId: e.target.value })}
                    placeholder="예: my_custom_view"
                  />
                </div>
                <div className="form-row">
                  <label>View 이름 *</label>
                  <input
                    type="text"
                    value={newViewForm.viewName}
                    onChange={e => setNewViewForm({ ...newViewForm, viewName: e.target.value })}
                    placeholder="예: 내 커스텀 뷰"
                  />
                </div>
                <div className="form-row">
                  <label>설명</label>
                  <input
                    type="text"
                    value={newViewForm.description}
                    onChange={e => setNewViewForm({ ...newViewForm, description: e.target.value })}
                    placeholder="설명 (선택)"
                  />
                </div>
                <div className="modal-actions">
                  <button className="btn-primary" onClick={handleCreateView}>생성</button>
                  <button className="btn-secondary" onClick={() => setShowCreateForm(false)}>취소</button>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* ============ View Selection 탭 ============ */}
      {activeTab === TABS.SELECTION.id && (
        <div className="slv-selection-tab">
          <div className="slv-section-header">
            <h3>🎬 View Selection API 테스트</h3>
            <p className="slv-desc">Viewer에서 여러 Study/Series를 선택하여 세션을 생성합니다.</p>
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
      )}
    </div>
  );
};

export default StudyListViewTests;