import React, { useState, useRef } from 'react';
import axios from 'axios';
import './ApiHealthCheck.css';

interface TestResult {
  name: string;
  status: 'pending' | 'running' | 'success' | 'failure' | 'skipped';
  duration?: number;
  request?: any;
  response?: any;
  error?: string;
  dependencies?: string[]; // 의존하는 테스트 이름들
  isSequential?: boolean; // 순차 실행 필요 여부
  cleanup?: boolean; // 정리(cleanup) 테스트 여부
  indentLevel?: number; // 들여쓰기 레벨 (의존성 트리 시각화)
  delayAfter?: number; // 이 테스트 후 대기 시간 (ms)
}

interface TestSection {
  title: string;
  description: string;
  tests: TestResult[];
  isSequential?: boolean; // 섹션 전체가 순차 실행되어야 하는지
}

const ApiHealthCheck: React.FC = () => {
  const [apiUrl] = useState('http://localhost:8080');
  const [sections, setSections] = useState<TestSection[]>([
    {
      title: '📊 프로젝트 메타데이터',
      description: '프로젝트 상태 메타데이터 조회 테스트 (순서 무관)',
      isSequential: false, // 순서 무관
      tests: [
        { name: '메타데이터 조회', status: 'pending' },
        { name: '메타데이터 구조 검증', status: 'pending' },
        { name: '5개 상태 존재 확인', status: 'pending' },
      ],
    },
    {
      title: '🔄 프로젝트 생명주기',
      description: '프로젝트 생성 및 상태 변경 테스트 (순차 실행)',
      isSequential: true, // 순차 실행 필수
      tests: [
        {
          name: '프로젝트 생성 (PREPARING)',
          status: 'pending',
          isSequential: true,
          indentLevel: 0,
          delayAfter: 1500, // 프로젝트 생성 후 1.5초 대기 (DB 커밋 완료 대기)
        },
        {
          name: '프로젝트 조회',
          status: 'pending',
          dependencies: ['프로젝트 생성 (PREPARING)'],
          isSequential: true,
          indentLevel: 1,
        },
        {
          name: 'PREPARING → IN_PROGRESS',
          status: 'pending',
          dependencies: ['프로젝트 생성 (PREPARING)'],
          indentLevel: 1,
        },
        {
          name: 'IN_PROGRESS → ON_HOLD',
          status: 'pending',
          dependencies: ['PREPARING → IN_PROGRESS'],
          indentLevel: 2,
        },
        {
          name: 'ON_HOLD → IN_PROGRESS',
          status: 'pending',
          dependencies: ['IN_PROGRESS → ON_HOLD'],
          indentLevel: 3,
        },
        {
          name: 'IN_PROGRESS → COMPLETED',
          status: 'pending',
          dependencies: ['ON_HOLD → IN_PROGRESS'],
          indentLevel: 4,
        },
        {
          name: '잘못된 상태 값 처리',
          status: 'pending',
          dependencies: ['IN_PROGRESS → COMPLETED'],
          indentLevel: 1,
        },
        {
          name: '존재하지 않는 프로젝트 조회',
          status: 'pending',
          dependencies: ['잘못된 상태 값 처리'],
          indentLevel: 1,
        },
        {
          name: '테스트 프로젝트 삭제',
          status: 'pending',
          dependencies: ['존재하지 않는 프로젝트 조회'],
          cleanup: true,
          isSequential: true,
          indentLevel: 1,
        },
      ],
    },
    {
      title: '📦 프로젝트 데이터 할당/제거',
      description: 'DICOM Study/Series 할당 및 조회 테스트 (순차 실행)',
      isSequential: true,
      tests: [
        {
          name: '데이터 테스트용 프로젝트 생성',
          status: 'pending',
          isSequential: true,
          indentLevel: 0,
          delayAfter: 1500,
        },
        {
          name: 'Study 할당',
          status: 'pending',
          dependencies: ['데이터 테스트용 프로젝트 생성'],
          isSequential: true,
          indentLevel: 1,
        },
        {
          name: 'Series 할당 (3개)',
          status: 'pending',
          dependencies: ['Study 할당'],
          isSequential: true,
          indentLevel: 1,
        },
        {
          name: '프로젝트 Study 목록 조회',
          status: 'pending',
          dependencies: ['Series 할당 (3개)'],
          indentLevel: 2,
        },
        {
          name: '프로젝트 Series 목록 조회',
          status: 'pending',
          dependencies: ['프로젝트 Study 목록 조회'],
          indentLevel: 2,
        },
        {
          name: 'Series 중복 할당 시도 (409 에러)',
          status: 'pending',
          dependencies: ['프로젝트 Series 목록 조회'],
          indentLevel: 2,
        },
        {
          name: '존재하지 않는 프로젝트에 할당 (404 에러)',
          status: 'pending',
          dependencies: ['Series 중복 할당 시도 (409 에러)'],
          indentLevel: 2,
        },
        {
          name: '다른 프로젝트 생성 (격리 테스트)',
          status: 'pending',
          dependencies: ['존재하지 않는 프로젝트에 할당 (404 에러)'],
          indentLevel: 1,
        },
        {
          name: '다른 프로젝트 데이터 조회 (빈 목록)',
          status: 'pending',
          dependencies: ['다른 프로젝트 생성 (격리 테스트)'],
          indentLevel: 2,
        },
        {
          name: '데이터 테스트 프로젝트 삭제',
          status: 'pending',
          dependencies: ['다른 프로젝트 데이터 조회 (빈 목록)'],
          cleanup: true,
          isSequential: true,
          indentLevel: 1,
        },
      ],
    },
  ]);

  const [expandedTest, setExpandedTest] = useState<string | null>(null);
  const [isRunningAll, setIsRunningAll] = useState(false);
  const [createdProjectId, setCreatedProjectId] = useState<number | null>(null);

  // useRef로 즉시 접근 가능한 프로젝트 ID 및 데이터 ID 관리
  const createdProjectIdRef = useRef<number | null>(null);
  const createdStudyIdRef = useRef<number | null>(null);
  const createdStudyUidRef = useRef<string | null>(null);
  const createdSeriesIdsRef = useRef<number[]>([]);
  const createdSeriesUidsRef = useRef<string[]>([]);

  // 통계 계산
  const getStats = () => {
    const allTests = sections.flatMap(s => s.tests);
    return {
      total: allTests.length,
      success: allTests.filter(t => t.status === 'success').length,
      failure: allTests.filter(t => t.status === 'failure').length,
      running: allTests.filter(t => t.status === 'running').length,
      pending: allTests.filter(t => t.status === 'pending').length,
      skipped: allTests.filter(t => t.status === 'skipped').length,
    };
  };

  // 의존성 체크: 특정 테스트를 실행할 수 있는지 확인
  const canRunTest = (sectionIndex: number, testIndex: number): { canRun: boolean; reason?: string } => {
    const test = sections[sectionIndex].tests[testIndex];

    if (!test.dependencies || test.dependencies.length === 0) {
      return { canRun: true };
    }

    // 모든 섹션의 모든 테스트를 검색
    const allTests = sections.flatMap(s => s.tests);

    for (const depName of test.dependencies) {
      const depTest = allTests.find(t => t.name === depName);

      if (!depTest) {
        return { canRun: false, reason: `의존 테스트를 찾을 수 없습니다: ${depName}` };
      }

      if (depTest.status !== 'success') {
        return { canRun: false, reason: `먼저 "${depName}" 테스트를 성공시켜야 합니다` };
      }
    }

    return { canRun: true };
  };



  // 개별 테스트 실행
  const runTest = async (sectionIndex: number, testIndex: number) => {
    // 의존성 체크
    const dependencyCheck = canRunTest(sectionIndex, testIndex);
    if (!dependencyCheck.canRun) {
      alert(`⚠️ ${dependencyCheck.reason}`);
      return;
    }

    const newSections = [...sections];
    const test = newSections[sectionIndex].tests[testIndex];

    test.status = 'running';
    test.request = undefined;
    test.response = undefined;
    test.error = undefined;
    setSections(newSections);

    const startTime = Date.now();

    try {
      let result;

      // 섹션별 테스트 로직
      if (sectionIndex === 0) {
        // 메타데이터 섹션
        result = await runMetadataTest(testIndex);
      } else if (sectionIndex === 1) {
        // 프로젝트 생명주기 섹션
        result = await runLifecycleTest(testIndex);
      } else if (sectionIndex === 2) {
        // 프로젝트 데이터 할당/제거 섹션
        result = await runDataAssignmentTest(testIndex);
      }

      test.status = 'success';
      test.request = result?.request;
      test.response = result?.response;
      test.duration = Date.now() - startTime;
    } catch (error: any) {
      test.status = 'failure';
      test.error = error.message || 'Unknown error';

      // 요청 정보 추출
      if (error.config) {
        const method = error.config.method?.toUpperCase() || 'UNKNOWN';
        const url = error.config.url || 'unknown';
        let body = undefined;

        if (error.config.data) {
          try {
            body = typeof error.config.data === 'string'
              ? JSON.parse(error.config.data)
              : error.config.data;
          } catch {
            body = error.config.data;
          }
        }

        test.request = {
          method,
          url,
          ...(body && { body }),
        };
      }

      test.response = error.response?.data;
      test.duration = Date.now() - startTime;
    }

    setSections([...newSections]);
  };

  // 메타데이터 테스트
  const runMetadataTest = async (testIndex: number) => {
    const requestInfo = { method: 'GET', url: '/api/projects/meta' };

    try {
      if (testIndex === 0) {
        // 메타데이터 조회
        const response = await axios.get(`${apiUrl}/api/projects/meta`);
        return {
          request: requestInfo,
          response: response.data,
        };
      } else if (testIndex === 1) {
        // 메타데이터 구조 검증
        const response = await axios.get(`${apiUrl}/api/projects/meta`);
        const data = response.data;

        if (!data.available_statuses || !Array.isArray(data.available_statuses)) {
          throw new Error('available_statuses가 배열이 아닙니다');
        }

        const firstStatus = data.available_statuses[0];
        if (!firstStatus.value || !firstStatus.label || !firstStatus.description) {
          throw new Error('상태 객체에 필수 필드가 없습니다');
        }

        return {
          request: requestInfo,
          response: data,
        };
      } else if (testIndex === 2) {
        // 5개 상태 존재 확인
        const response = await axios.get(`${apiUrl}/api/projects/meta`);
        const data = response.data;

        if (data.available_statuses.length !== 5) {
          throw new Error(`5개의 상태가 있어야 하는데 ${data.available_statuses.length}개가 있습니다`);
        }

        const expectedStatuses = ['PREPARING', 'IN_PROGRESS', 'COMPLETED', 'ON_HOLD', 'CANCELLED'];
        const actualStatuses = data.available_statuses.map((s: any) => s.value);

        for (const expected of expectedStatuses) {
          if (!actualStatuses.includes(expected)) {
            throw new Error(`${expected} 상태가 없습니다`);
          }
        }

        return {
          request: requestInfo,
          response: data,
        };
      }
    } catch (error: any) {
      // 에러에 요청 정보 포함
      if (!error.config) {
        error.config = {
          method: 'get',
          url: `${apiUrl}/api/projects/meta`,
        };
      }
      throw error;
    }
  };

  // 프로젝트 생명주기 테스트
  const runLifecycleTest = async (testIndex: number) => {
    if (testIndex === 0) {
      // 프로젝트 생성
      const projectData = {
        name: `E2E Test ${Date.now()}`,
        description: 'API Health Check Test',
        sponsor: 'Test Sponsor',
        start_date: '2025-01-01',
        end_date: '2025-12-31',
        auto_complete: false,
      };

      try {
        const response = await axios.post(`${apiUrl}/api/projects`, projectData);

        console.log(`  ✅ 프로젝트 생성 성공:`, response.data);
        console.log(`  📝 생성된 프로젝트 ID: ${response.data.id}`);

        if (response.data.status !== 'Preparing') {
          throw new Error(`초기 상태가 Preparing이 아닙니다: ${response.data.status}`);
        }

        // 상태와 ref 모두 업데이트
        const projectId = response.data.id;
        setCreatedProjectId(projectId);
        createdProjectIdRef.current = projectId; // ref는 즉시 업데이트됨
        console.log(`  💾 createdProjectId 저장 완료: ${projectId} (state + ref)`);

        return {
          request: { method: 'POST', url: '/api/projects', body: projectData },
          response: response.data,
          createdId: projectId, // 생성된 ID를 반환값에 포함
        };
      } catch (error: any) {
        // 에러에 요청 정보 포함
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects`,
            data: projectData,
          };
        }
        throw error;
      }
    } else if (testIndex === 1) {
      // 프로젝트 조회 (재시도 로직 포함)
      // ref를 사용하여 즉시 업데이트된 값 확인
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        console.log(`  ❌ createdProjectIdRef.current: ${createdProjectIdRef.current}`);
        console.log(`  ❌ createdProjectId state: ${createdProjectId}`);
        throw new Error('프로젝트가 생성되지 않았습니다. 먼저 프로젝트 생성 테스트를 실행하세요.');
      }

      console.log(`  ✅ 프로젝트 ID 확인: ${projectId} (ref 사용)`);

      // 최대 3번 재시도 (DB 커밋 대기)
      let lastError: any = null;
      const requestInfo = { method: 'GET', url: `/api/projects/${projectId}` };

      for (let attempt = 1; attempt <= 3; attempt++) {
        try {
          console.log(`  🔍 프로젝트 조회 시도 ${attempt}/3...`);
          const response = await axios.get(`${apiUrl}/api/projects/${projectId}`);

          console.log(`  ✅ 프로젝트 조회 성공 (시도 ${attempt})`);
          return {
            request: requestInfo,
            response: response.data,
          };
        } catch (error: any) {
          lastError = error;
          console.log(`  ⚠️ 프로젝트 조회 실패 (시도 ${attempt}/3): ${error.message}`);

          if (attempt < 3) {
            // 재시도 전 대기 (500ms)
            console.log(`  ⏱️ 500ms 후 재시도...`);
            await new Promise(resolve => setTimeout(resolve, 500));
          }
        }
      }

      // 모든 재시도 실패 - 에러에 요청 정보 포함
      const errorMessage = `프로젝트 조회 실패 (3회 시도): ${lastError?.response?.data?.error || lastError?.message || '알 수 없는 오류'}`;
      const error: any = new Error(errorMessage);
      error.config = {
        method: 'get',
        url: `${apiUrl}/api/projects/${projectId}`,
      };
      error.response = lastError?.response;
      throw error;
    } else if (testIndex >= 2 && testIndex <= 5) {
      // 상태 변경 테스트
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트가 생성되지 않았습니다. 먼저 프로젝트 생성 테스트를 실행하세요.');
      }

      const transitions = [
        { from: 'Preparing', to: 'IN_PROGRESS', expected: 'InProgress' },
        { from: 'InProgress', to: 'ON_HOLD', expected: 'OnHold' },
        { from: 'OnHold', to: 'IN_PROGRESS', expected: 'InProgress' },
        { from: 'InProgress', to: 'COMPLETED', expected: 'Completed' },
      ];

      const transition = transitions[testIndex - 2];
      const updateData = {
        status: transition.to,
        end_date: '',
      };

      try {
        const response = await axios.put(`${apiUrl}/api/projects/${projectId}`, updateData);

        if (response.data.status !== transition.expected) {
          throw new Error(`상태가 ${transition.expected}로 변경되지 않았습니다: ${response.data.status}`);
        }

        return {
          request: { method: 'PUT', url: `/api/projects/${projectId}`, body: updateData },
          response: response.data,
        };
      } catch (error: any) {
        // 에러에 요청 정보 포함
        if (!error.config) {
          error.config = {
            method: 'put',
            url: `${apiUrl}/api/projects/${projectId}`,
            data: updateData,
          };
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // 잘못된 상태 값 처리
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트가 생성되지 않았습니다. 먼저 프로젝트 생성 테스트를 실행하세요.');
      }

      const updateData = {
        status: 'INVALID_STATUS',
        end_date: '',
      };

      try {
        const response = await axios.put(`${apiUrl}/api/projects/${projectId}`, updateData);

        // 잘못된 상태는 무시되고 200 응답이 와야 함
        if (response.status !== 200) {
          throw new Error(`예상치 못한 응답 코드: ${response.status}`);
        }

        return {
          request: { method: 'PUT', url: `/api/projects/${projectId}`, body: updateData },
          response: response.data,
        };
      } catch (error: any) {
        // 에러에 요청 정보 포함
        if (!error.config) {
          error.config = {
            method: 'put',
            url: `${apiUrl}/api/projects/${projectId}`,
            data: updateData,
          };
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // 존재하지 않는 프로젝트 조회
      const requestInfo = { method: 'GET', url: '/api/projects/999999' };

      try {
        await axios.get(`${apiUrl}/api/projects/999999`);
        throw new Error('404 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 404) {
          return {
            request: requestInfo,
            response: error.response.data || { status: 404, message: 'Not Found' },
          };
        }

        // 404가 아닌 다른 에러 - 에러에 요청 정보 포함
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/projects/999999`,
          };
        }
        throw error;
      }
    } else if (testIndex === 8) {
      // 테스트 프로젝트 삭제 (cleanup)
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('삭제할 프로젝트가 없습니다.');
      }

      const deletedId = projectId;

      try {
        const response = await axios.delete(`${apiUrl}/api/projects/${projectId}`);

        // 삭제 후 ID 초기화 (state + ref)
        setCreatedProjectId(null);
        createdProjectIdRef.current = null;

        return {
          request: { method: 'DELETE', url: `/api/projects/${deletedId}` },
          response: response.data || { message: '프로젝트가 삭제되었습니다' },
        };
      } catch (error: any) {
        // 삭제 실패해도 ID 초기화 (다음 테스트를 위해)
        setCreatedProjectId(null);
        createdProjectIdRef.current = null;

        // 에러에 요청 정보 포함
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/projects/${deletedId}`,
          };
        }
        throw error;
      }
    }
  };

  // 프로젝트 데이터 할당/제거 테스트
  const runDataAssignmentTest = async (testIndex: number) => {
    if (testIndex === 0) {
      // 데이터 테스트용 프로젝트 생성
      const projectData = {
        name: `Data Test ${Date.now()}`,
        description: 'DICOM Data Assignment Test',
        sponsor: 'Test Sponsor',
        start_date: '2025-01-01',
        end_date: '2025-12-31',
        auto_complete: false,
      };

      try {
        const response = await axios.post(`${apiUrl}/api/projects`, projectData);

        console.log(`  ✅ 데이터 테스트용 프로젝트 생성 성공:`, response.data);
        console.log(`  📝 생성된 프로젝트 ID: ${response.data.id}`);

        // 상태와 ref 모두 업데이트
        const projectId = response.data.id;
        setCreatedProjectId(projectId);
        createdProjectIdRef.current = projectId;
        console.log(`  💾 createdProjectId 저장 완료: ${projectId} (state + ref)`);

        return {
          request: { method: 'POST', url: '/api/projects', body: projectData },
          response: response.data,
          createdId: projectId,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects`,
            data: projectData,
          };
        }
        throw error;
      }
    } else if (testIndex === 1) {
      // Study 할당
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트가 생성되지 않았습니다. 먼저 프로젝트 생성 테스트를 실행하세요.');
      }

      const studyUid = `1.2.840.113619.2.1.1.${Date.now()}`;
      const studyData = {
        study_uid: studyUid,
        study_description: 'Test Study for E2E',
        patient_id: 'TEST001',
        patient_name: 'Test Patient',
        study_date: '2025-01-15',
      };

      try {
        const response = await axios.post(
          `${apiUrl}/api/projects/${projectId}/studies/assign`,
          studyData
        );

        console.log(`  ✅ Study 할당 성공:`, response.data);
        console.log(`  📝 생성된 Study ID: ${response.data.study_id}`);

        // Study ID 및 UID 저장
        createdStudyIdRef.current = response.data.study_id;
        createdStudyUidRef.current = studyUid;

        return {
          request: { method: 'POST', url: `/api/projects/${projectId}/studies/assign`, body: studyData },
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects/${projectId}/studies/assign`,
            data: studyData,
          };
        }
        throw error;
      }
    } else if (testIndex === 2) {
      // Series 할당 (3개)
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트가 생성되지 않았습니다.');
      }

      const studyId = createdStudyIdRef.current;
      if (!studyId) {
        throw new Error('Study가 할당되지 않았습니다.');
      }

      // 3개의 Series 할당
      const studyUid = createdStudyUidRef.current;
      if (!studyUid) {
        throw new Error('Study UID가 저장되지 않았습니다.');
      }

      const timestamp = Date.now();
      const seriesIds: number[] = [];
      const seriesUids: string[] = [];
      const seriesDataList = [
        {
          study_uid: studyUid,
          series_uid: `1.2.840.113619.2.1.2.${timestamp}.1`,
          series_description: 'Axial CT 5mm',
          modality: 'CT',
          series_number: 1,
        },
        {
          study_uid: studyUid,
          series_uid: `1.2.840.113619.2.1.2.${timestamp}.2`,
          series_description: 'Coronal CT 5mm',
          modality: 'CT',
          series_number: 2,
        },
        {
          study_uid: studyUid,
          series_uid: `1.2.840.113619.2.1.2.${timestamp}.3`,
          series_description: 'Sagittal CT 5mm',
          modality: 'CT',
          series_number: 3,
        },
      ];

      try {
        for (let i = 0; i < seriesDataList.length; i++) {
          const response = await axios.post(
            `${apiUrl}/api/projects/${projectId}/series/assign`,
            seriesDataList[i]
          );

          console.log(`  ✅ Series ${i + 1}/3 할당 성공:`, response.data);
          seriesIds.push(response.data.series_id);
          seriesUids.push(seriesDataList[i].series_uid);
        }

        // Series IDs 및 UIDs 저장
        createdSeriesIdsRef.current = seriesIds;
        createdSeriesUidsRef.current = seriesUids;

        return {
          request: { method: 'POST', url: `/api/projects/${projectId}/series/assign`, body: seriesDataList },
          response: { message: `${seriesIds.length}개의 Series 할당 완료`, series_ids: seriesIds },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects/${projectId}/series/assign`,
            data: seriesDataList,
          };
        }
        throw error;
      }
    } else if (testIndex === 3) {
      // 프로젝트 Study 목록 조회
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트가 생성되지 않았습니다.');
      }

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/studies`);

        console.log(`  ✅ Study 목록 조회 성공:`, response.data);

        // 할당한 Study가 목록에 있는지 확인
        if (!response.data.studies || response.data.studies.length === 0) {
          throw new Error('할당한 Study가 목록에 없습니다.');
        }

        return {
          request: { method: 'GET', url: `/api/project-data/${projectId}/studies` },
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/studies`,
          };
        }
        throw error;
      }
    } else if (testIndex === 4) {
      // 프로젝트 Series 목록 조회
      const projectId = createdProjectIdRef.current;
      const studyId = createdStudyIdRef.current;

      if (!projectId || !studyId) {
        throw new Error('프로젝트 또는 Study가 생성되지 않았습니다.');
      }

      try {
        const response = await axios.get(
          `${apiUrl}/api/project-data/${projectId}/studies/${studyId}/series`
        );

        console.log(`  ✅ Series 목록 조회 성공:`, response.data);

        // 할당한 Series가 목록에 있는지 확인
        if (!response.data.series || response.data.series.length !== 3) {
          throw new Error(`할당한 3개의 Series가 목록에 없습니다. (실제: ${response.data.series?.length || 0}개)`);
        }

        return {
          request: { method: 'GET', url: `/api/project-data/${projectId}/studies/${studyId}/series` },
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/studies/${studyId}/series`,
          };
        }
        throw error;
      }
    } else if (testIndex === 5) {
      // Series 중복 할당 시도 (409 에러)
      const projectId = createdProjectIdRef.current;
      const studyUid = createdStudyUidRef.current;
      const seriesUids = createdSeriesUidsRef.current;

      if (!projectId) {
        throw new Error('프로젝트가 생성되지 않았습니다.');
      }

      if (!studyUid || seriesUids.length === 0) {
        throw new Error('Study 또는 Series가 할당되지 않았습니다.');
      }

      // 첫 번째 Series와 동일한 UID 사용
      const duplicateSeriesData = {
        study_uid: studyUid,
        series_uid: seriesUids[0], // 첫 번째 Series UID 재사용
        series_description: 'Duplicate Series',
        modality: 'CT',
        series_number: 1,
      };

      try {
        await axios.post(
          `${apiUrl}/api/projects/${projectId}/series/assign`,
          duplicateSeriesData
        );
        throw new Error('409 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 409) {
          console.log(`  ✅ 중복 할당 시 409 에러 정상 반환`);
          return {
            request: { method: 'POST', url: `/api/projects/${projectId}/series/assign`, body: duplicateSeriesData },
            response: error.response.data || { status: 409, message: 'Conflict' },
          };
        }

        // 409가 아닌 다른 에러
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects/${projectId}/series/assign`,
            data: duplicateSeriesData,
          };
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // 존재하지 않는 프로젝트에 할당 (404 에러)
      const nonExistentProjectId = 999999;
      const seriesData = {
        study_uid: `1.2.840.113619.2.1.1.${Date.now()}`,
        series_uid: `1.2.840.113619.2.1.2.${Date.now()}.999`,
        series_description: 'Test Series',
        modality: 'CT',
        series_number: 1,
      };

      try {
        await axios.post(
          `${apiUrl}/api/projects/${nonExistentProjectId}/series/assign`,
          seriesData
        );
        throw new Error('404 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 404) {
          console.log(`  ✅ 존재하지 않는 프로젝트에 할당 시 404 에러 정상 반환`);
          return {
            request: { method: 'POST', url: `/api/projects/${nonExistentProjectId}/series/assign`, body: seriesData },
            response: error.response.data || { status: 404, message: 'Not Found' },
          };
        }

        // 404가 아닌 다른 에러
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects/${nonExistentProjectId}/series/assign`,
            data: seriesData,
          };
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // 다른 프로젝트 생성 (격리 테스트)
      const projectData = {
        name: `Isolation Test ${Date.now()}`,
        description: 'Project Isolation Test',
        sponsor: 'Test Sponsor',
        start_date: '2025-01-01',
        end_date: '2025-12-31',
        auto_complete: false,
      };

      try {
        const response = await axios.post(`${apiUrl}/api/projects`, projectData);

        console.log(`  ✅ 격리 테스트용 프로젝트 생성 성공:`, response.data);
        console.log(`  📝 생성된 프로젝트 ID: ${response.data.id}`);

        return {
          request: { method: 'POST', url: '/api/projects', body: projectData },
          response: response.data,
          isolationProjectId: response.data.id,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects`,
            data: projectData,
          };
        }
        throw error;
      }
    } else if (testIndex === 8) {
      // 다른 프로젝트 데이터 조회 (빈 목록)
      // 이전 테스트에서 생성한 격리 테스트용 프로젝트 ID를 가져와야 함
      // 임시로 sections에서 이전 테스트 결과를 찾아서 ID 추출
      const allTests = sections.flatMap(s => s.tests);
      const isolationTest = allTests.find(t => t.name === '다른 프로젝트 생성 (격리 테스트)');

      if (!isolationTest || !isolationTest.response) {
        throw new Error('격리 테스트용 프로젝트가 생성되지 않았습니다.');
      }

      const isolationProjectId = (isolationTest.response as any).id;

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${isolationProjectId}/studies`);

        console.log(`  ✅ 다른 프로젝트 데이터 조회 성공:`, response.data);

        // 빈 목록이어야 함 (다른 프로젝트의 데이터는 보이지 않아야 함)
        if (response.data.studies && response.data.studies.length > 0) {
          throw new Error(`다른 프로젝트의 데이터가 조회되었습니다. (${response.data.studies.length}개)`);
        }

        return {
          request: { method: 'GET', url: `/api/project-data/${isolationProjectId}/studies` },
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${isolationProjectId}/studies`,
          };
        }
        throw error;
      }
    } else if (testIndex === 9) {
      // 데이터 테스트 프로젝트 삭제 (cleanup)
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('삭제할 프로젝트가 없습니다.');
      }

      const deletedId = projectId;

      try {
        const response = await axios.delete(`${apiUrl}/api/projects/${projectId}`);

        // 삭제 후 ID 초기화 (state + ref)
        setCreatedProjectId(null);
        createdProjectIdRef.current = null;
        createdStudyIdRef.current = null;
        createdStudyUidRef.current = null;
        createdSeriesIdsRef.current = [];
        createdSeriesUidsRef.current = [];

        console.log(`  ✅ 데이터 테스트 프로젝트 삭제 완료`);

        return {
          request: { method: 'DELETE', url: `/api/projects/${deletedId}` },
          response: response.data || { message: '프로젝트가 삭제되었습니다' },
        };
      } catch (error: any) {
        // 삭제 실패해도 ID 초기화 (다음 테스트를 위해)
        setCreatedProjectId(null);
        createdProjectIdRef.current = null;
        createdStudyIdRef.current = null;
        createdStudyUidRef.current = null;
        createdSeriesIdsRef.current = [];
        createdSeriesUidsRef.current = [];

        // 에러에 요청 정보 포함
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/projects/${deletedId}`,
          };
        }
        throw error;
      }
    }
  };

  // 모든 테스트 실행 (의존성 및 순차 실행 고려)
  const runAllTests = async () => {
    setIsRunningAll(true);
    setCreatedProjectId(null);
    createdProjectIdRef.current = null; // ref도 초기화
    createdStudyIdRef.current = null;
    createdStudyUidRef.current = null;
    createdSeriesIdsRef.current = [];
    createdSeriesUidsRef.current = [];

    // 모든 테스트를 pending으로 초기화
    const newSections = sections.map(section => ({
      ...section,
      tests: section.tests.map(test => ({
        ...test,
        status: 'pending' as const,
        request: undefined,
        response: undefined,
        error: undefined,
        duration: undefined,
      })),
    }));
    setSections(newSections);

    // 섹션별로 실행
    for (let sectionIndex = 0; sectionIndex < sections.length; sectionIndex++) {
      const section = sections[sectionIndex];

      if (section.isSequential) {
        // 순차 실행 섹션: 하나씩 순서대로 실행
        console.log(`📋 순차 실행 섹션: ${section.title}`);
        for (let testIndex = 0; testIndex < section.tests.length; testIndex++) {
          const test = section.tests[testIndex];
          console.log(`  ▶️ ${test.name} 실행 중...`);
          await runTest(sectionIndex, testIndex);

          // 테스트별 커스텀 딜레이 또는 기본 딜레이
          const delay = test.delayAfter !== undefined ? test.delayAfter : 300;
          if (delay > 0) {
            console.log(`  ⏱️ ${delay}ms 대기 중...`);
            await new Promise(resolve => setTimeout(resolve, delay));
          }
        }
      } else {
        // 병렬 실행 가능 섹션: 의존성만 체크하고 실행
        console.log(`🔀 병렬 실행 섹션: ${section.title}`);
        for (let testIndex = 0; testIndex < section.tests.length; testIndex++) {
          const test = section.tests[testIndex];

          // 의존성 체크
          const dependencyCheck = canRunTest(sectionIndex, testIndex);
          if (dependencyCheck.canRun) {
            console.log(`  ▶️ ${test.name} 실행 중...`);
            await runTest(sectionIndex, testIndex);
          } else {
            console.log(`  ⏭️ ${test.name} 건너뜀: ${dependencyCheck.reason}`);
            // 의존성 미충족 시 스킵
            const newSections = [...sections];
            newSections[sectionIndex].tests[testIndex].status = 'skipped';
            newSections[sectionIndex].tests[testIndex].error = dependencyCheck.reason;
            setSections(newSections);
          }

          // 테스트별 커스텀 딜레이 또는 기본 딜레이
          const delay = test.delayAfter !== undefined ? test.delayAfter : 200;
          if (delay > 0) {
            await new Promise(resolve => setTimeout(resolve, delay));
          }
        }
      }
    }

    setIsRunningAll(false);
  };

  const toggleTestDetails = (testName: string) => {
    setExpandedTest(expandedTest === testName ? null : testName);
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'success': return '✅';
      case 'failure': return '❌';
      case 'running': return '⏳';
      case 'skipped': return '⏭️';
      case 'pending': return '⚪';
      default: return '⚪';
    }
  };



  const stats = getStats();

  return (
    <div className="api-health-check">
      <div className="health-check-header">
        <h2>🔍 API 점검</h2>
        <p className="subtitle">프로젝트 상태 관리 API E2E 테스트</p>
      </div>

      {/* 통계 */}
      <div className="stats-bar">
        <div className="stat-item total">
          <span className="stat-label">전체</span>
          <span className="stat-value">{stats.total}</span>
        </div>
        <div className="stat-item success">
          <span className="stat-label">성공</span>
          <span className="stat-value">{stats.success}</span>
        </div>
        <div className="stat-item failure">
          <span className="stat-label">실패</span>
          <span className="stat-value">{stats.failure}</span>
        </div>
        <div className="stat-item running">
          <span className="stat-label">진행중</span>
          <span className="stat-value">{stats.running}</span>
        </div>
        <div className="stat-item pending">
          <span className="stat-label">대기</span>
          <span className="stat-value">{stats.pending}</span>
        </div>
        <div className="stat-item skipped">
          <span className="stat-label">건너뜀</span>
          <span className="stat-value">{stats.skipped}</span>
        </div>
      </div>

      {/* 일괄 테스트 버튼 */}
      <div className="bulk-actions">
        <button 
          onClick={runAllTests} 
          disabled={isRunningAll}
          className="run-all-button"
        >
          {isRunningAll ? '🔄 테스트 실행 중...' : '▶️ 모든 테스트 실행'}
        </button>
      </div>

      {/* 테스트 섹션 */}
      {sections.map((section, sectionIndex) => (
        <div key={sectionIndex} className="test-section">
          <div className="section-header">
            <h3>{section.title}</h3>
            <p className="section-description">{section.description}</p>
          </div>

          <div className="test-list">
            {section.tests.map((test, testIndex) => {
              const testKey = `${sectionIndex}-${testIndex}`;
              const isExpanded = expandedTest === testKey;

              const indentLevel = test.indentLevel || 0;
              const indentStyle = {
                marginLeft: `${indentLevel * 30}px`,
                borderLeft: indentLevel > 0 ? '3px solid #e5e7eb' : 'none',
                paddingLeft: indentLevel > 0 ? '15px' : '0',
              };

              return (
                <div
                  key={testIndex}
                  className={`test-item ${test.status}`}
                  style={indentStyle}
                >
                  <div className="test-header">
                    <div className="test-info">
                      {indentLevel > 0 && (
                        <span className="indent-connector">└─</span>
                      )}
                      <span className="test-icon">{getStatusIcon(test.status)}</span>
                      <div className="test-name-container">
                        <span className="test-name">{test.name}</span>
                        {test.dependencies && test.dependencies.length > 0 && (
                          <span className="test-dependencies" title={`의존: ${test.dependencies.join(', ')}`}>
                            🔗 {test.dependencies.length}개 의존
                          </span>
                        )}
                        {test.isSequential && (
                          <span className="test-sequential" title="순차 실행 필요">
                            ⏩ 순차
                          </span>
                        )}
                        {test.cleanup && (
                          <span className="test-cleanup" title="정리 작업">
                            🧹 정리
                          </span>
                        )}
                        {test.delayAfter !== undefined && test.delayAfter > 0 && (
                          <span className="test-delay" title={`이 테스트 후 ${test.delayAfter}ms 대기`}>
                            ⏱️ {test.delayAfter}ms
                          </span>
                        )}
                      </div>
                      {test.duration && (
                        <span className="test-duration">{test.duration}ms</span>
                      )}
                    </div>
                    <div className="test-actions">
                      <button
                        onClick={() => runTest(sectionIndex, testIndex)}
                        disabled={test.status === 'running' || isRunningAll}
                        className="run-test-button"
                      >
                        {test.status === 'running' ? '⏳' : '▶️'}
                      </button>
                      {(test.request || test.response || test.error) && (
                        <button
                          onClick={() => toggleTestDetails(testKey)}
                          className="toggle-details-button"
                        >
                          {isExpanded ? '▼' : '▶'}
                        </button>
                      )}
                    </div>
                  </div>

                  {isExpanded && (test.request || test.response || test.error) && (
                    <div className="test-details">
                      {test.request && (
                        <div className="detail-section">
                          <h4>📤 요청</h4>
                          <pre>{JSON.stringify(test.request, null, 2)}</pre>
                        </div>
                      )}
                      {test.response && (
                        <div className="detail-section">
                          <h4>📥 응답</h4>
                          <pre>{JSON.stringify(test.response, null, 2)}</pre>
                        </div>
                      )}
                      {test.error && (
                        <div className="detail-section error">
                          <h4>❌ 에러</h4>
                          <pre>{test.error}</pre>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
};

export default ApiHealthCheck;

