import React, { useState, useCallback } from 'react';
import axios from 'axios';
import './QidoEnhancedTests.css';

const API_URL = 'http://localhost:8080';

interface QueryParams {
  view?: string;
  project_id?: number;
  page?: number;
  page_size?: number;
  report_status?: string;
  PatientName?: string;
  StudyDate?: string;
  Modality?: string;
  includefield?: string[];
}

interface E2ETestCase {
  name: string;
  status: 'pending' | 'running' | 'passed' | 'failed';
  detail?: string;
}

interface TestResult {
  status: 'idle' | 'running' | 'success' | 'error';
  output: string;
  duration?: number;
}

interface ApiResponse {
  status: number;
  headers: Record<string, string>;
  data: any;
  duration: number;
}

// DICOM includefield 옵션들
const INCLUDE_FIELDS = [
  { key: 'PatientName', label: '환자명', tag: '00100010' },
  { key: 'PatientID', label: '환자ID', tag: '00100020' },
  { key: 'PatientBirthDate', label: '생년월일', tag: '00100030' },
  { key: 'PatientSex', label: '성별', tag: '00100040' },
  { key: 'StudyDate', label: 'Study일자', tag: '00080020' },
  { key: 'StudyTime', label: 'Study시간', tag: '00080030' },
  { key: 'StudyDescription', label: 'Study설명', tag: '00081030' },
  { key: 'AccessionNumber', label: 'Accession', tag: '00080050' },
  { key: 'Modality', label: 'Modality', tag: '00080060' },
  { key: 'ModalitiesInStudy', label: 'Modalities', tag: '00080061' },
  { key: 'ReferringPhysicianName', label: '의뢰의', tag: '00080090' },
  { key: 'InstitutionName', label: '기관명', tag: '00080080' },
  { key: 'NumberOfStudyRelatedSeries', label: 'Series수', tag: '00201206' },
  { key: 'NumberOfStudyRelatedInstances', label: 'Instance수', tag: '00201208' },
];

// Extension 필드 옵션들 (_ext에 포함되는 확장 필드)
const EXTENSION_FIELDS = [
  { key: 'projects', label: '📁 Projects', description: '프로젝트 목록 (id, name, role_name)' },
  { key: 'report_status', label: '📄 Report Status', description: '리포트 상태 (unread, approval, unapproval)' },
  { key: 'review', label: '🔄 Review/Annotations', description: 'reviewStage, availableStages, annotationSummary' },
];

const QidoEnhancedTests: React.FC = () => {
  // Query Parameters State
  const [params, setParams] = useState<QueryParams>({
    page: 1,
    page_size: 10,
  });

  // Include fields State (DICOM)
  const [selectedFields, setSelectedFields] = useState<string[]>([]);

  // Extension fields State (_ext)
  const [selectedExtFields, setSelectedExtFields] = useState<string[]>([]);

  // E2E Test State
  const [e2eTests, setE2eTests] = useState<E2ETestCase[]>([]);
  const [e2eRunning, setE2eRunning] = useState(false);

  // API Response State
  const [apiResponse, setApiResponse] = useState<ApiResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  // Token 가져오기
  const getToken = useCallback(async (): Promise<string> => {
    const response = await axios.post(`${API_URL}/api/auth/login`, {
      username: 'iaid-pacs-admin',
      password: 'Qlalfqjsgh1!',
    });
    return response.data.token;
  }, []);

  // Parameter Toggle Handlers
  const toggleParam = (key: keyof QueryParams, value: any) => {
    setParams(prev => {
      if (prev[key] === value) {
        const { [key]: _, ...rest } = prev;
        return rest as QueryParams;
      }
      return { ...prev, [key]: value };
    });
  };

  // Include field toggle
  const toggleIncludeField = (fieldKey: string) => {
    setSelectedFields(prev =>
      prev.includes(fieldKey)
        ? prev.filter(f => f !== fieldKey)
        : [...prev, fieldKey]
    );
  };

  // Extension field toggle
  const toggleExtField = (fieldKey: string) => {
    setSelectedExtFields(prev =>
      prev.includes(fieldKey)
        ? prev.filter(f => f !== fieldKey)
        : [...prev, fieldKey]
    );
  };

  // Quick API Call
  const callApi = async () => {
    setIsLoading(true);
    setApiResponse(null);
    const startTime = Date.now();

    try {
      const token = await getToken();

      // Build query params
      const queryParts: string[] = [];
      Object.entries(params).forEach(([k, v]) => {
        if (v !== undefined && v !== '') {
          queryParts.push(`${k}=${encodeURIComponent(String(v))}`);
        }
      });

      // Add includefield params (DICOM fields)
      selectedFields.forEach(field => {
        queryParts.push(`includefield=${encodeURIComponent(field)}`);
      });

      // Add extension fields (_ext)
      selectedExtFields.forEach(field => {
        queryParts.push(`_ext=${encodeURIComponent(field)}`);
      });

      const queryString = queryParts.join('&');
      const url = `${API_URL}/api/me/dicom/studies${queryString ? `?${queryString}` : ''}`;

      const response = await axios.get(url, {
        headers: { Authorization: `Bearer ${token}` },
      });

      const duration = Date.now() - startTime;
      const h = response.headers;
      setApiResponse({
        status: response.status,
        headers: {
          'X-Total-Count': h['x-total-count'] ?? h['X-Total-Count'] ?? 'N/A',
          'X-Page': h['x-page'] ?? h['X-Page'] ?? 'N/A',
          'X-Page-Size': h['x-page-size'] ?? h['X-Page-Size'] ?? 'N/A',
          'X-Total-Pages': h['x-total-pages'] ?? h['X-Total-Pages'] ?? 'N/A',
        },
        data: response.data,
        duration,
      });
    } catch (err: any) {
      const duration = Date.now() - startTime;
      setApiResponse({
        status: err.response?.status || 0,
        headers: {},
        data: err.response?.data || err.message,
        duration,
      });
    } finally {
      setIsLoading(false);
    }
  };

  // E2E Test Runner - 프론트엔드에서 직접 실행
  const runE2ETest = async () => {
    setE2eRunning(true);
    const tests: E2ETestCase[] = [
      { name: '1. 기본 조회 (view 없음, _ext 없어야 함)', status: 'pending' },
      { name: '2. view=default (_ext 필드 포함, 결과 수 동일)', status: 'pending' },
      { name: '3. project_id=2 필터링', status: 'pending' },
      { name: '4. 페이지네이션 헤더 (X-Total-Count 등)', status: 'pending' },
      { name: '5. report_status=approval 필터링', status: 'pending' },
      { name: '6. 복합: view + project_id', status: 'pending' },
    ];
    setE2eTests([...tests]);

    try {
      const token = await getToken();
      const headers = { Authorization: `Bearer ${token}` };

      // Test 1: 기본 조회 (view 없음)
      tests[0].status = 'running';
      setE2eTests([...tests]);
      let res = await axios.get(`${API_URL}/api/me/dicom/studies?page=1&page_size=5`, { headers });
      const baseCount = res.data?.length || 0;
      const hasNoExt = res.data?.length > 0 ? !('_ext' in res.data[0]) : true;
      tests[0].status = res.status === 200 ? 'passed' : 'failed';
      tests[0].detail = `${baseCount}개 (view 없음 → _ext: ${hasNoExt ? '없음 ✓' : '있음?'})`;
      setE2eTests([...tests]);

      // Test 2: view=default (확장 필드 포함)
      tests[1].status = 'running';
      setE2eTests([...tests]);
      res = await axios.get(`${API_URL}/api/me/dicom/studies?view=default&page=1&page_size=5`, { headers });
      const viewCount = res.data?.length || 0;
      const hasExt = res.data?.length > 0 && '_ext' in res.data[0];
      const extKeys = hasExt ? Object.keys(res.data[0]._ext || {}).join(', ') : '';
      // view 유무와 관계없이 결과 수는 동일해야 함
      const countMatch = viewCount === baseCount;
      tests[1].status = res.status === 200 && hasExt ? 'passed' : 'failed';
      tests[1].detail = `${viewCount}개, _ext: ${hasExt ? `✓ [${extKeys}]` : '없음 ✗'} ${countMatch ? '' : `⚠️ 수 불일치(${baseCount}→${viewCount})`}`;
      setE2eTests([...tests]);

      // Test 3: project_id 필터링
      tests[2].status = 'running';
      setE2eTests([...tests]);
      res = await axios.get(`${API_URL}/api/me/dicom/studies?project_id=2&page=1&page_size=5`, { headers });
      tests[2].status = res.status === 200 ? 'passed' : 'failed';
      tests[2].detail = `project_id=2 → ${res.data?.length || 0}개`;
      setE2eTests([...tests]);

      // Test 4: 페이지네이션 헤더
      tests[3].status = 'running';
      setE2eTests([...tests]);
      const res1 = await axios.get(`${API_URL}/api/me/dicom/studies?page=1&page_size=2`, { headers });
      const h = res1.headers;
      const total = h['x-total-count'] ?? h['X-Total-Count'] ?? 'N/A';
      const page = h['x-page'] ?? h['X-Page'] ?? 'N/A';
      const pages = h['x-total-pages'] ?? h['X-Total-Pages'] ?? 'N/A';
      tests[3].status = res1.status === 200 && total !== 'N/A' ? 'passed' : 'failed';
      tests[3].detail = `Total: ${total}, Page: ${page}/${pages}, 데이터: ${res1.data?.length || 0}개`;
      setE2eTests([...tests]);

      // Test 5: report_status 필터링
      tests[4].status = 'running';
      setE2eTests([...tests]);
      res = await axios.get(`${API_URL}/api/me/dicom/studies?report_status=approval&page=1&page_size=5`, { headers });
      tests[4].status = res.status === 200 ? 'passed' : 'failed';
      tests[4].detail = `report_status=approval → ${res.data?.length || 0}개`;
      setE2eTests([...tests]);

      // Test 6: 복합 필터 (view + project_id)
      tests[5].status = 'running';
      setE2eTests([...tests]);
      res = await axios.get(`${API_URL}/api/me/dicom/studies?view=default&project_id=2&page=1&page_size=10`, { headers });
      const hasExtCombo = res.data?.length > 0 && '_ext' in res.data[0];
      tests[5].status = res.status === 200 ? 'passed' : 'failed';
      tests[5].detail = `view+project_id=2 → ${res.data?.length || 0}개, _ext: ${hasExtCombo ? '✓' : '-'}`;
      setE2eTests([...tests]);

    } catch (err: any) {
      const failedIdx = tests.findIndex(t => t.status === 'running');
      if (failedIdx >= 0) {
        tests[failedIdx].status = 'failed';
        tests[failedIdx].detail = err.message;
        setE2eTests([...tests]);
      }
    } finally {
      setE2eRunning(false);
    }
  };

  // Parameter Chips
  const paramChips = [
    { key: 'view', label: 'View', values: ['default', 'compact', 'detailed'] },
    { key: 'page_size', label: 'Page Size', values: [5, 10, 20, 50] },
    { key: 'report_status', label: 'Report Status', values: ['unread', 'approval', 'unapproval'] },
    { key: 'Modality', label: 'Modality', values: ['CT', 'MR', 'CR', 'DX', 'US'] },
  ];

  return (
    <div className="qido-enhanced-tests">
      <div className="page-header">
        <h2>🚀 QIDO Enhanced API 테스트</h2>
        <p>GET /api/me/dicom/studies - 확장된 Study 목록 조회 API</p>
      </div>

      {/* E2E Test Section */}
      <div className="section e2e-section">
        <div className="section-header">
          <h3>🧪 E2E 시나리오 테스트</h3>
          <button
            className={`btn-run ${e2eRunning ? 'running' : ''}`}
            onClick={runE2ETest}
            disabled={e2eRunning}
          >
            {e2eRunning ? '실행 중...' : '▶ 테스트 실행'}
          </button>
        </div>

        {e2eTests.length > 0 && (
          <div className="e2e-tests-list">
            {e2eTests.map((test, idx) => (
              <div key={idx} className={`e2e-test-item ${test.status}`}>
                <span className="test-icon">
                  {test.status === 'pending' && '⏳'}
                  {test.status === 'running' && '🔄'}
                  {test.status === 'passed' && '✅'}
                  {test.status === 'failed' && '❌'}
                </span>
                <span className="test-name">{test.name}</span>
                {test.detail && <span className="test-detail">{test.detail}</span>}
              </div>
            ))}
            <div className="e2e-summary">
              {!e2eRunning && e2eTests.length > 0 && (
                <>
                  <span className="passed">✅ {e2eTests.filter(t => t.status === 'passed').length}</span>
                  <span className="failed">❌ {e2eTests.filter(t => t.status === 'failed').length}</span>
                </>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Quick API Tester */}
      <div className="section api-tester-section">
        <div className="section-header">
          <h3>⚡ 빠른 API 테스트</h3>
          <button className="btn-call" onClick={callApi} disabled={isLoading}>
            {isLoading ? '호출 중...' : '🔍 API 호출'}
          </button>
        </div>

        {/* Parameter Chips */}
        <div className="param-chips">
          {paramChips.map(chip => (
            <div key={chip.key} className="chip-group">
              <span className="chip-label">{chip.label}:</span>
              <div className="chip-values">
                {chip.values.map(val => (
                  <button
                    key={String(val)}
                    className={`chip ${params[chip.key as keyof QueryParams] === val ? 'active' : ''}`}
                    onClick={() => toggleParam(chip.key as keyof QueryParams, val)}
                  >
                    {String(val)}
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>

        {/* Extension Fields (_ext) */}
        <div className="include-fields-section ext-section">
          <div className="section-label">
            <span>🔌 _ext (확장 필드):</span>
            <button
              className="btn-small"
              onClick={() => setSelectedExtFields(EXTENSION_FIELDS.map(f => f.key))}
            >
              전체선택
            </button>
            <button
              className="btn-small"
              onClick={() => setSelectedExtFields([])}
            >
              초기화
            </button>
          </div>
          <div className="ext-fields-grid">
            {EXTENSION_FIELDS.map(field => (
              <button
                key={field.key}
                className={`ext-chip ${selectedExtFields.includes(field.key) ? 'active' : ''}`}
                onClick={() => toggleExtField(field.key)}
                title={field.description}
              >
                {field.label}
              </button>
            ))}
          </div>
        </div>

        {/* Include Fields (DICOM) */}
        <div className="include-fields-section">
          <div className="section-label">
            <span>📋 includefield (DICOM):</span>
            <button
              className="btn-small"
              onClick={() => setSelectedFields(INCLUDE_FIELDS.map(f => f.key))}
            >
              전체선택
            </button>
            <button
              className="btn-small"
              onClick={() => setSelectedFields([])}
            >
              초기화
            </button>
          </div>
          <div className="include-fields-grid">
            {INCLUDE_FIELDS.map(field => (
              <button
                key={field.key}
                className={`field-chip ${selectedFields.includes(field.key) ? 'active' : ''}`}
                onClick={() => toggleIncludeField(field.key)}
                title={`${field.tag} - ${field.key}`}
              >
                {field.label}
              </button>
            ))}
          </div>
        </div>

        {/* Custom Inputs */}
        <div className="custom-params">
          <div className="param-input">
            <label>project_id:</label>
            <input
              type="number"
              value={params.project_id || ''}
              onChange={e => setParams(prev => ({
                ...prev,
                project_id: e.target.value ? Number(e.target.value) : undefined
              }))}
              placeholder="선택 안함"
            />
          </div>
          <div className="param-input">
            <label>page:</label>
            <input
              type="number"
              value={params.page || 1}
              onChange={e => setParams(prev => ({
                ...prev,
                page: Number(e.target.value) || 1
              }))}
            />
          </div>
          <div className="param-input">
            <label>PatientName:</label>
            <input
              type="text"
              value={params.PatientName || ''}
              onChange={e => setParams(prev => ({
                ...prev,
                PatientName: e.target.value || undefined
              }))}
              placeholder="예: Kim*"
            />
          </div>
        </div>

        {/* Current Query Display */}
        <div className="query-preview">
          <code>GET /api/me/dicom/studies?{[
            ...Object.entries(params)
              .filter(([_, v]) => v !== undefined && v !== '')
              .map(([k, v]) => `${k}=${v}`),
            ...selectedFields.map(f => `includefield=${f}`),
            ...selectedExtFields.map(f => `_ext=${f}`)
          ].join('&') || '(no params)'}</code>
        </div>
      </div>

      {/* API Response */}
      {apiResponse && (
        <div className="section response-section">
          <div className="section-header">
            <h3>📦 응답 결과</h3>
            <span className={`status-badge ${apiResponse.status >= 200 && apiResponse.status < 300 ? 'success' : 'error'}`}>
              {apiResponse.status} | {apiResponse.duration}ms
            </span>
          </div>

          {/* Pagination Headers */}
          <div className="response-headers">
            {Object.entries(apiResponse.headers).map(([k, v]) => (
              <span key={k} className="header-item">{k}: <strong>{v}</strong></span>
            ))}
          </div>

          {/* Response Data */}
          <div className="response-data">
            <div className="data-summary">
              {Array.isArray(apiResponse.data) && (
                <span>📋 {apiResponse.data.length}개 Study 반환</span>
              )}
            </div>

            {/* Study Cards */}
            {Array.isArray(apiResponse.data) && apiResponse.data.slice(0, 5).map((study: any, idx: number) => (
              <div key={idx} className="study-card">
                <div className="study-main">
                  <span className="study-uid">{study['0020000D']?.Value?.[0]?.substring(0, 30) || 'N/A'}...</span>
                  <span className="study-date">{study['00080020']?.Value?.[0] || 'N/A'}</span>
                </div>
                {study._ext && (
                  <div className="study-ext">
                    <span className="ext-label">_ext:</span>
                    {study._ext.projects && (
                      <span className="ext-projects">
                        Projects: {study._ext.projects.map((p: any) => p.name || p.id).join(', ')}
                      </span>
                    )}
                    {study._ext.report_status && (
                      <span className={`ext-status status-${study._ext.report_status}`}>
                        📄 {study._ext.report_status}
                      </span>
                    )}
                    {study._ext.review && (
                      <span className="ext-review">
                        🔄 {study._ext.review.reviewStage}
                      </span>
                    )}
                  </div>
                )}
              </div>
            ))}

            {/* Raw JSON Toggle */}
            <details className="raw-json">
              <summary>Raw JSON 보기</summary>
              <pre>{JSON.stringify(apiResponse.data, null, 2)}</pre>
            </details>
          </div>
        </div>
      )}
    </div>
  );
};

export default QidoEnhancedTests;

