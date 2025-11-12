/**
 * Project Data Access 테스트 컴포넌트
 * 
 * 기능:
 * 1. 시나리오 구성 (다기관 공동 연구 프로젝트)
 * 2. 접근 제어 매트릭스 조회
 * 3. DICOM API 접근 테스트
 */

import React, { useState } from 'react';
import axios from 'axios';
import './ProjectDataAccess.css';

interface ProjectDataAccessProps {
  apiUrl: string;
  token: string;
}

interface User {
  id: number;
  username: string;
  full_name: string;
  access_records: number;
}

interface Study {
  id: number;
  study_uid: string;
  study_description: string;
  patient_id: string;
  patient_name: string;
}

interface AccessMatrix {
  user: User;
  studies: {
    study: Study;
    status: string | null;
    access_scope: string | null;
    expires_at: string | null;
  }[];
}

const ProjectDataAccess: React.FC<ProjectDataAccessProps> = ({ apiUrl, token }) => {
  const [loading, setLoading] = useState(false);
  const [setupResult, setSetupResult] = useState<string>('');
  const [accessMatrix, setAccessMatrix] = useState<AccessMatrix[]>([]);
  const [projectId, setProjectId] = useState<number | null>(null);
  const [users, setUsers] = useState<User[]>([]);
  const [studies, setStudies] = useState<Study[]>([]);

  // 시나리오 구성
  const handleSetupScenario = async () => {
    setLoading(true);
    setSetupResult('');
    setAccessMatrix([]);

    try {
      const response = await axios.post(
        `${apiUrl}/api/test/project-data-access/setup`,
        {},
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      );

      setProjectId(response.data.project_id);
      setSetupResult(`✅ 시나리오 구성 완료!\n\n` +
        `📁 프로젝트 ID: ${response.data.project_id}\n` +
        `👥 사용자: ${response.data.users.length}명\n` +
        `📊 Study: ${response.data.studies.length}개\n` +
        `🔒 접근 제어: ${response.data.access_records}개 레코드`
      );

      // 접근 제어 매트릭스 자동 조회
      await handleFetchAccessMatrix(response.data.project_id);
    } catch (err: any) {
      setSetupResult(`❌ 시나리오 구성 실패: ${err.response?.data?.message || err.message}`);
    } finally {
      setLoading(false);
    }
  };

  // 접근 제어 매트릭스 조회
  const handleFetchAccessMatrix = async (pid?: number) => {
    const targetProjectId = pid || projectId;
    if (!targetProjectId) {
      alert('먼저 시나리오를 구성해주세요.');
      return;
    }

    setLoading(true);

    try {
      const response = await axios.get(
        `${apiUrl}/api/project-data/${targetProjectId}/data-access/matrix`,
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      );

      setAccessMatrix(response.data.matrix);
      setUsers(response.data.users);
      setStudies(response.data.studies);
    } catch (err: any) {
      alert(`❌ 매트릭스 조회 실패: ${err.response?.data?.message || err.message}`);
    } finally {
      setLoading(false);
    }
  };

  // 접근 아이콘 렌더링
  const renderAccessIcon = (status: string | null, accessScope: string | null, expiresAt: string | null) => {
    if (!status) {
      return <span className="access-icon full" title="전체 접근 (제약 없음)">✅</span>;
    }

    if (status === 'APPROVED') {
      if (accessScope === 'READ_ONLY') {
        const expiry = expiresAt ? new Date(expiresAt).toLocaleString() : '';
        return <span className="access-icon readonly" title={`읽기 전용 (만료: ${expiry})`}>👁️</span>;
      }
      return <span className="access-icon full" title="전체 접근">✅</span>;
    }

    if (status === 'DENIED') {
      return <span className="access-icon denied" title="접근 거부">❌</span>;
    }

    return <span className="access-icon pending" title="승인 대기">⏳</span>;
  };

  // 시나리오 초기화
  const handleCleanup = async () => {
    if (!projectId) {
      alert('초기화할 프로젝트가 없습니다.');
      return;
    }

    if (!window.confirm('시나리오를 초기화하시겠습니까?')) {
      return;
    }

    setLoading(true);

    try {
      await axios.delete(
        `${apiUrl}/api/test/project-data-access/cleanup/${projectId}`,
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      );

      setSetupResult('✅ 시나리오 초기화 완료!');
      setAccessMatrix([]);
      setProjectId(null);
      setUsers([]);
      setStudies([]);
    } catch (err: any) {
      alert(`❌ 초기화 실패: ${err.response?.data?.message || err.message}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="project-data-access">
      <h2>🔒 Project Data Access 테스트</h2>
      <p className="description">
        프로젝트 데이터 접근 제어 기능을 테스트합니다.
        <br />
        다기관 공동 연구 프로젝트 시나리오를 구성하고, 사용자별 접근 권한을 확인할 수 있습니다.
      </p>

      {/* 시나리오 구성 */}
      <div className="section">
        <h3>📋 시나리오 구성</h3>
        <div className="button-group">
          <button
            onClick={handleSetupScenario}
            disabled={loading}
            className="btn btn-primary"
          >
            {loading ? '⏳ 구성 중...' : '🎬 시나리오 구성'}
          </button>
          <button
            onClick={() => handleFetchAccessMatrix()}
            disabled={loading || !projectId}
            className="btn btn-secondary"
          >
            {loading ? '⏳ 조회 중...' : '🔄 매트릭스 새로고침'}
          </button>
          <button
            onClick={handleCleanup}
            disabled={loading || !projectId}
            className="btn btn-danger"
          >
            {loading ? '⏳ 초기화 중...' : '🗑️ 시나리오 초기화'}
          </button>
        </div>

        {setupResult && (
          <div className="result-box">
            <pre>{setupResult}</pre>
          </div>
        )}
      </div>

      {/* 접근 제어 매트릭스 */}
      {accessMatrix.length > 0 && (
        <div className="section">
          <h3>📊 접근 제어 매트릭스</h3>
          <p className="info">
            <strong>프로젝트 ID:</strong> {projectId}
          </p>

          <div className="matrix-container">
            <table className="access-matrix">
              <thead>
                <tr>
                  <th>사용자</th>
                  {studies.map((study) => (
                    <th key={study.id} title={study.study_description}>
                      {study.patient_id}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {accessMatrix.map((row) => (
                  <tr key={row.user.id}>
                    <td className="user-cell">
                      <div className="user-info">
                        <strong>{row.user.full_name}</strong>
                        <small>({row.user.username})</small>
                      </div>
                    </td>
                    {row.studies.map((access, idx) => (
                      <td key={idx} className="access-cell">
                        {renderAccessIcon(access.status, access.access_scope, access.expires_at)}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className="legend">
            <h4>범례:</h4>
            <ul>
              <li><span className="access-icon full">✅</span> 전체 접근 가능</li>
              <li><span className="access-icon readonly">👁️</span> 읽기 전용 접근</li>
              <li><span className="access-icon denied">❌</span> 접근 거부</li>
              <li><span className="access-icon pending">⏳</span> 승인 대기</li>
            </ul>
          </div>
        </div>
      )}

      {/* 시나리오 설명 */}
      <div className="section scenario-info">
        <h3>📖 시나리오 설명</h3>
        <div className="scenario-description">
          <h4>🏥 다기관 공동 연구 프로젝트</h4>
          <ul>
            <li><strong>Dr. Kim (책임연구원)</strong>: 전체 데이터 접근 가능 (제약 없음)</li>
            <li><strong>Dr. Lee (A병원 연구원)</strong>: A병원 Study만 접근 (3개)</li>
            <li><strong>Dr. Park (B병원 연구원)</strong>: B병원 Study만 접근 (3개)</li>
            <li><strong>Dr. Choi (임시 협력자)</strong>: Study 1개만 7일간 읽기 전용 접근</li>
          </ul>

          <h4>📊 Study 데이터</h4>
          <ul>
            <li><strong>A병원</strong>: Study 3개 (CT Chest, MRI Brain, CT Abdomen)</li>
            <li><strong>B병원</strong>: Study 3개 (CT Chest, MRI Spine, CT Heart)</li>
            <li><strong>VIP</strong>: Study 1개 (CT Full Body - 민감 데이터)</li>
          </ul>
        </div>
      </div>
    </div>
  );
};

export default ProjectDataAccess;

