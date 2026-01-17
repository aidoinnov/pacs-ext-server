import React, { useState } from 'react';
import '../ApiHealthCheck.css';

interface E2ETest {
  name: string;
  description: string;
  script: string;
  status: 'pending' | 'running' | 'success' | 'error';
  output?: string;
  error?: string;
}

const E2E_TESTS: Record<string, E2ETest[]> = {
  'annotation-snapshot': [
    {
      name: 'Annotation Snapshot Upload',
      description: '어노테이션 스냅샷 이미지 업로드 전체 워크플로우 테스트',
      script: 'test_annotation_snapshot_e2e.py',
      status: 'pending',
    },
  ],
  'me-studies': [
    {
      name: 'Me Studies Endpoint',
      description: '/api/me/dicom/studies 엔드포인트 테스트',
      script: 'test_me_studies.py',
      status: 'pending',
    },
  ],
  'keycloak-qido': [
    {
      name: 'Keycloak QIDO Direct',
      description: 'Keycloak 토큰으로 Dcm4chee QIDO 직접 요청 테스트',
      script: 'test_keycloak_qido_direct.py',
      status: 'pending',
    },
  ],
  'all-studies': [
    {
      name: 'All Studies Access',
      description: '전체 Studies 접근 권한 테스트',
      script: 'test_all_studies_access.py',
      status: 'pending',
    },
  ],
};

interface E2ETestsProps {
  testType: string;
}

const E2ETests: React.FC<E2ETestsProps> = ({ testType }) => {
  const [tests, setTests] = useState<E2ETest[]>(E2E_TESTS[testType] || []);
  const [isRunning, setIsRunning] = useState(false);

  const runTest = async (index: number) => {
    const test = tests[index];
    
    // Update test status to running
    setTests(prev => prev.map((t, i) => 
      i === index ? { ...t, status: 'running' as const, output: '', error: '' } : t
    ));
    setIsRunning(true);

    try {
      // Call backend API to run the Python script
      const response = await fetch(`http://localhost:8080/api/e2e/run`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          script: test.script,
        }),
      });

      const result = await response.json();

      if (response.ok && result.success) {
        setTests(prev => prev.map((t, i) =>
          i === index ? {
            ...t,
            status: 'success' as const,
            output: result.stdout || result.output || 'Test completed successfully',
          } : t
        ));
      } else {
        const errorMessage = result.stderr || result.error || 'Test failed';
        const output = result.stdout || '';
        setTests(prev => prev.map((t, i) =>
          i === index ? {
            ...t,
            status: 'error' as const,
            error: errorMessage,
            output: output,
          } : t
        ));
      }
    } catch (error) {
      setTests(prev => prev.map((t, i) => 
        i === index ? { 
          ...t, 
          status: 'error' as const, 
          error: error instanceof Error ? error.message : 'Network error',
        } : t
      ));
    } finally {
      setIsRunning(false);
    }
  };

  const getStatusIcon = (status: E2ETest['status']) => {
    switch (status) {
      case 'pending': return '⏸️';
      case 'running': return '⏳';
      case 'success': return '✅';
      case 'error': return '❌';
    }
  };

  const getStatusClass = (status: E2ETest['status']) => {
    switch (status) {
      case 'pending': return 'status-pending';
      case 'running': return 'status-running';
      case 'success': return 'status-success';
      case 'error': return 'status-error';
    }
  };

  return (
    <div className="api-health-container">
      <div className="card">
        <h2>🧪 E2E 테스트</h2>
        <p className="section-description">
          Python 기반 E2E 테스트 스크립트를 실행합니다.
        </p>

        <div className="test-sections">
          {tests.map((test, index) => (
            <div key={index} className="test-section">
              <div className="test-header">
                <h3>
                  <span className={`status-icon ${getStatusClass(test.status)}`}>
                    {getStatusIcon(test.status)}
                  </span>
                  {test.name}
                </h3>
                <button
                  onClick={() => runTest(index)}
                  disabled={isRunning}
                  className="run-test-button"
                >
                  {test.status === 'running' ? '실행 중...' : '테스트 실행'}
                </button>
              </div>

              <p className="test-description">{test.description}</p>
              <p className="test-script">스크립트: <code>{test.script}</code></p>

              {test.output && (
                <div className="test-output">
                  <h4>출력:</h4>
                  <pre>{test.output}</pre>
                </div>
              )}

              {test.error && (
                <div className="test-error">
                  <h4>에러:</h4>
                  <pre>{test.error}</pre>
                </div>
              )}
            </div>
          ))}
        </div>

        {tests.length === 0 && (
          <div className="no-tests">
            <p>선택한 테스트 타입에 대한 테스트가 없습니다.</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default E2ETests;

