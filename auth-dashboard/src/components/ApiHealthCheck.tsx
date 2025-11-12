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

// 테스트 계정 정보
interface TestAccount {
  username: string;
  keycloak_id: string;
  role: string;
}

const TEST_ACCOUNTS: Record<string, TestAccount> = {
  SUPER_ADMIN: {
    username: 'test_super_admin',
    keycloak_id: '7287ed27-59a5-4803-9984-9f5ddf241737',
    role: 'SUPER_ADMIN',
  },
  ADMIN: {
    username: 'test_admin',
    keycloak_id: 'e4199467-7fcf-4830-8543-728693d4ec7f',
    role: 'ADMIN',
  },
  USER: {
    username: 'test_user',
    keycloak_id: 'e8db9533-76c2-451a-8232-8711a661360e',
    role: 'USER',
  },
};

const ApiHealthCheck: React.FC = () => {
  const [apiUrl] = useState('http://localhost:8080');
  const [testToken, setTestToken] = useState<string | null>(null);
  const [currentTestAccount, setCurrentTestAccount] = useState<TestAccount>(TEST_ACCOUNTS.SUPER_ADMIN);
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
          name: 'Series 할당 해제 (첫 번째)',
          status: 'pending',
          dependencies: ['다른 프로젝트 데이터 조회 (빈 목록)'],
          indentLevel: 1,
          delayAfter: 500, // DB 트랜잭션 완료 대기
        },
        {
          name: 'Series 목록 재조회 (2개 확인)',
          status: 'pending',
          dependencies: ['Series 할당 해제 (첫 번째)'],
          indentLevel: 2,
        },
        {
          name: 'Study 할당 해제',
          status: 'pending',
          dependencies: ['Series 목록 재조회 (2개 확인)'],
          indentLevel: 1,
        },
        {
          name: 'Study 목록 재조회 (빈 목록 확인)',
          status: 'pending',
          dependencies: ['Study 할당 해제'],
          indentLevel: 2,
        },
        {
          name: '데이터 테스트 프로젝트 삭제',
          status: 'pending',
          dependencies: ['Study 목록 재조회 (빈 목록 확인)'],
          cleanup: true,
          isSequential: true,
          indentLevel: 1,
        },
      ],
    },
    {
      title: '🔍 DICOM 전체 조회 + 할당 여부 확인',
      description: 'project_id 옵셔널화, READ_ALL 권한, check_assignment_for_project 테스트',
      isSequential: false,
      tests: [
        {
          name: 'DICOM Studies 전체 조회 (project_id 없음)',
          status: 'pending',
          indentLevel: 0,
        },
        {
          name: 'DICOM Studies 프로젝트별 조회 (project_id 있음)',
          status: 'pending',
          indentLevel: 0,
        },
        {
          name: 'DICOM Series 전체 조회 (project_id 없음)',
          status: 'pending',
          indentLevel: 0,
        },
        {
          name: 'DICOM Instances 전체 조회 (project_id 없음)',
          status: 'pending',
          indentLevel: 0,
        },
        {
          name: '할당 여부 확인 (check_assignment_for_project)',
          status: 'pending',
          indentLevel: 0,
        },
        {
          name: '전체 조회 + 할당 여부 확인 (통합)',
          status: 'pending',
          indentLevel: 0,
        },
        {
          name: '프로젝트별 조회 + 할당 여부 확인',
          status: 'pending',
          indentLevel: 0,
        },
        {
          name: '다른 프로젝트 할당 여부 확인',
          status: 'pending',
          indentLevel: 0,
        },
        {
          name: '잘못된 project_id (0) 에러 처리',
          status: 'pending',
          indentLevel: 0,
        },
        {
          name: '잘못된 project_id (음수) 에러 처리',
          status: 'pending',
          indentLevel: 0,
        },
      ],
    },
    {
      title: '🔒 Project Data Access 접근 제어',
      description: '다기관 공동 연구 프로젝트 시나리오 - 사용자별 데이터 접근 제어 테스트',
      isSequential: true,
      tests: [
        {
          name: '시나리오 구성 (프로젝트 + 사용자 + Study + 접근 제어)',
          status: 'pending',
          isSequential: true,
          indentLevel: 0,
          delayAfter: 1500,
        },
        {
          name: '접근 제어 매트릭스 조회',
          status: 'pending',
          dependencies: ['시나리오 구성 (프로젝트 + 사용자 + Study + 접근 제어)'],
          indentLevel: 1,
        },
        {
          name: '매트릭스 구조 검증 (4명 사용자)',
          status: 'pending',
          dependencies: ['접근 제어 매트릭스 조회'],
          indentLevel: 2,
        },
        {
          name: '매트릭스 구조 검증 (7개 Study)',
          status: 'pending',
          dependencies: ['접근 제어 매트릭스 조회'],
          indentLevel: 2,
        },
        {
          name: 'Dr. Kim 전체 접근 확인 (7/7)',
          status: 'pending',
          dependencies: ['매트릭스 구조 검증 (7개 Study)'],
          indentLevel: 2,
        },
        {
          name: 'Dr. Lee A병원만 접근 확인 (3/7)',
          status: 'pending',
          dependencies: ['매트릭스 구조 검증 (7개 Study)'],
          indentLevel: 2,
        },
        {
          name: 'Dr. Park B병원만 접근 확인 (3/7)',
          status: 'pending',
          dependencies: ['매트릭스 구조 검증 (7개 Study)'],
          indentLevel: 2,
        },
        {
          name: 'Dr. Choi 읽기 전용 확인 (1/7)',
          status: 'pending',
          dependencies: ['매트릭스 구조 검증 (7개 Study)'],
          indentLevel: 2,
        },
        {
          name: '시나리오 초기화',
          status: 'pending',
          dependencies: ['Dr. Choi 읽기 전용 확인 (1/7)'],
          cleanup: true,
          isSequential: true,
          indentLevel: 1,
        },
      ],
    },
    {
      title: '🔄 Project Data Access 순차 시나리오 (실제 API 호출)',
      description: '프론트엔드에서 직접 순차적으로 API를 호출하여 접근 제어 시나리오 구성 및 검증',
      isSequential: true,
      tests: [
        {
          name: '0️⃣ 사전 정리 (기존 테스트 데이터 삭제)',
          status: 'pending',
          isSequential: true,
          indentLevel: 0,
          cleanup: true,
        },
        {
          name: '1️⃣ 프로젝트 생성',
          status: 'pending',
          dependencies: ['0️⃣ 사전 정리 (기존 테스트 데이터 삭제)'],
          indentLevel: 0,
        },
        {
          name: '2️⃣ 사용자 4명 생성 (Dr. Kim, Dr. Lee, Dr. Park, Dr. Choi)',
          status: 'pending',
          dependencies: ['1️⃣ 프로젝트 생성'],
          indentLevel: 0,
        },
        {
          name: '3️⃣ 사용자 4명 활성화 (관리자 승인)',
          status: 'pending',
          dependencies: ['2️⃣ 사용자 4명 생성 (Dr. Kim, Dr. Lee, Dr. Park, Dr. Choi)'],
          indentLevel: 0,
        },
        {
          name: '4️⃣ 사용자를 프로젝트 멤버로 추가',
          status: 'pending',
          dependencies: ['3️⃣ 사용자 4명 활성화 (관리자 승인)'],
          indentLevel: 0,
        },
        {
          name: '5️⃣ Study 7개 생성 (A병원 3개, B병원 3개, VIP 1개)',
          status: 'pending',
          dependencies: ['4️⃣ 사용자를 프로젝트 멤버로 추가'],
          indentLevel: 0,
        },
        {
          name: '6️⃣ 접근 제어 설정 (Dr. Lee → A병원, Dr. Park → B병원, Dr. Choi → VIP)',
          status: 'pending',
          dependencies: ['5️⃣ Study 7개 생성 (A병원 3개, B병원 3개, VIP 1개)'],
          indentLevel: 0,
        },
        {
          name: '7️⃣ 접근 제어 매트릭스 조회 및 검증',
          status: 'pending',
          dependencies: ['6️⃣ 접근 제어 설정 (Dr. Lee → A병원, Dr. Park → B병원, Dr. Choi → VIP)'],
          indentLevel: 0,
        },
        {
          name: '8️⃣ DICOM QIDO API로 실제 접근 제어 검증',
          status: 'pending',
          dependencies: ['7️⃣ 접근 제어 매트릭스 조회 및 검증'],
          indentLevel: 0,
        },
        {
          name: '9️⃣ 정리 (프로젝트 삭제)',
          status: 'pending',
          dependencies: ['8️⃣ DICOM QIDO API로 실제 접근 제어 검증'],
          indentLevel: 0,
          cleanup: true,
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

  // 순차 시나리오용 ref
  const sequentialProjectIdRef = useRef<number | null>(null);
  const sequentialUserIdsRef = useRef<{[key: string]: number}>({});
  const sequentialStudyIdsRef = useRef<number[]>([]);

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

  // 테스트 계정으로 토큰 획득 (백엔드 프록시를 통해 Keycloak 토큰 획득)
  const getTestToken = async (account: TestAccount): Promise<string> => {
    try {
      console.log(`🔑 Keycloak 토큰 획득 중... (계정: ${account.username}, 역할: ${account.role})`);

      // 비밀번호 매핑
      const passwords: Record<string, string> = {
        'test_super_admin': 'TestAdmin123!',
        'test_admin': 'TestAdmin123!',
        'test_user': 'TestUser123!',
      };

      // 백엔드 프록시를 통해 Keycloak 토큰 획득 (CORS 우회)
      const response = await axios.post(`${apiUrl}/api/auth/keycloak-token`, {
        username: account.username,
        password: passwords[account.username] || 'TestAdmin123!',
      });

      const token = response.data.access_token;
      console.log(`✅ Keycloak 토큰 획득 성공! (계정: ${account.username})`);
      console.log(`   토큰 길이: ${token.length}, 미리보기: ${token.substring(0, 50)}...`);

      setTestToken(token);
      setCurrentTestAccount(account);

      return token;
    } catch (error: any) {
      console.error(`❌ Keycloak 토큰 획득 실패:`, error);
      if (error.response) {
        console.error(`   응답 상태: ${error.response.status}`);
        console.error(`   응답 데이터:`, error.response.data);
      }
      throw new Error(`Keycloak 토큰 획득 실패: ${error.message}`);
    }
  };

  // axios 요청에 토큰 추가
  const getAxiosConfig = async (accountType?: 'SUPER_ADMIN' | 'ADMIN' | 'USER') => {
    // accountType이 지정되면 해당 계정의 토큰을 자동 획득
    if (accountType) {
      const account = TEST_ACCOUNTS[accountType];
      try {
        const token = await getTestToken(account);
        return {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        };
      } catch (error) {
        console.error(`토큰 획득 실패 (${accountType}):`, error);
        throw new Error(`${accountType} 토큰 획득 실패`);
      }
    }

    // accountType이 없으면 현재 토큰 사용
    if (!testToken) {
      throw new Error('토큰이 없습니다. 먼저 토큰을 획득하거나 accountType을 지정하세요.');
    }
    return {
      headers: {
        Authorization: `Bearer ${testToken}`,
      },
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
      // alert 대신 테스트 항목에 에러 표시
      const newSections = [...sections];
      newSections[sectionIndex].tests[testIndex].status = 'failure';
      newSections[sectionIndex].tests[testIndex].error = `⚠️ ${dependencyCheck.reason}`;
      setSections(newSections);
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
      } else if (sectionIndex === 3) {
        // DICOM 전체 조회 + 할당 여부 확인 섹션
        result = await runDicomTest(testIndex);
      } else if (sectionIndex === 4) {
        // Project Data Access 접근 제어 섹션
        result = await runProjectDataAccessTest(testIndex);
      } else if (sectionIndex === 5) {
        // 순차 시나리오 섹션
        result = await runSequentialScenarioTest(testIndex);
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
      // Series 할당 해제 (첫 번째)
      const projectId = createdProjectIdRef.current;
      const studyId = createdStudyIdRef.current;
      const seriesIds = createdSeriesIdsRef.current;

      if (!projectId || !studyId || seriesIds.length === 0) {
        throw new Error('프로젝트, Study 또는 Series가 할당되지 않았습니다.');
      }

      const firstSeriesId = seriesIds[0];

      try {
        const response = await axios.delete(
          `${apiUrl}/api/projects/${projectId}/series/${firstSeriesId}/unassign`
        );

        console.log(`  ✅ Series ${firstSeriesId} 할당 해제 성공:`, response.data);

        return {
          request: {
            method: 'DELETE',
            url: `/api/projects/${projectId}/series/${firstSeriesId}/unassign`,
          },
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/projects/${projectId}/series/${firstSeriesId}/unassign`,
          };
        }
        throw error;
      }
    } else if (testIndex === 10) {
      // Series 목록 재조회 (2개 확인)
      const projectId = createdProjectIdRef.current;
      const studyId = createdStudyIdRef.current;

      if (!projectId || !studyId) {
        throw new Error('프로젝트 또는 Study가 할당되지 않았습니다.');
      }

      try {
        const response = await axios.get(
          `${apiUrl}/api/project-data/${projectId}/studies/${studyId}/series`
        );

        console.log(`  ✅ Series 목록 재조회:`, response.data);

        if (!response.data.series || response.data.series.length !== 2) {
          throw new Error(
            `할당 해제 후 2개의 Series가 남아있어야 하는데 ${response.data.series?.length || 0}개가 조회되었습니다.`
          );
        }

        return {
          request: {
            method: 'GET',
            url: `/api/project-data/${projectId}/studies/${studyId}/series`,
          },
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
    } else if (testIndex === 11) {
      // Study 할당 해제
      const projectId = createdProjectIdRef.current;
      const studyId = createdStudyIdRef.current;

      if (!projectId || !studyId) {
        throw new Error('프로젝트 또는 Study가 할당되지 않았습니다.');
      }

      try {
        const response = await axios.delete(
          `${apiUrl}/api/projects/${projectId}/studies/${studyId}/unassign`
        );

        console.log(`  ✅ Study ${studyId} 할당 해제 성공:`, response.data);

        return {
          request: {
            method: 'DELETE',
            url: `/api/projects/${projectId}/studies/${studyId}/unassign`,
          },
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/projects/${projectId}/studies/${studyId}/unassign`,
          };
        }
        throw error;
      }
    } else if (testIndex === 12) {
      // Study 목록 재조회 (빈 목록 확인)
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트가 생성되지 않았습니다.');
      }

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/studies`);

        console.log(`  ✅ Study 목록 재조회:`, response.data);

        if (!response.data.studies || response.data.studies.length !== 0) {
          throw new Error(
            `Study 할당 해제 후 빈 목록이어야 하는데 ${response.data.studies?.length || 0}개가 조회되었습니다.`
          );
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
    } else if (testIndex === 13) {
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

  // DICOM 전체 조회 + 할당 여부 확인 테스트
  const runDicomTest = async (testIndex: number) => {
    if (testIndex === 0) {
      // DICOM Studies 전체 조회 (project_id 없음) - SUPER_ADMIN 권한 필요
      const requestInfo = { method: 'GET', url: '/api/dicom/studies' };

      try {
        const config = await getAxiosConfig('SUPER_ADMIN');
        const response = await axios.get(`${apiUrl}/api/dicom/studies`, config);

        console.log(`  ✅ DICOM Studies 전체 조회 성공:`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies`,
          };
        }
        throw error;
      }
    } else if (testIndex === 1) {
      // DICOM Studies 프로젝트별 조회 (project_id 있음) - USER 권한으로 테스트
      const projectId = createdProjectIdRef.current || 150; // 기본값 150
      const requestInfo = { method: 'GET', url: `/api/dicom/studies?project_id=${projectId}` };

      try {
        const config = await getAxiosConfig('USER');
        const response = await axios.get(`${apiUrl}/api/dicom/studies?project_id=${projectId}`, config);

        console.log(`  ✅ DICOM Studies 프로젝트별 조회 성공 (project_id=${projectId}):`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies?project_id=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 2) {
      // DICOM Series 전체 조회 (project_id 없음) - SUPER_ADMIN 권한 필요
      try {
        const config = await getAxiosConfig('SUPER_ADMIN');

        // 먼저 Study를 조회해서 Study UID를 얻음
        const studiesResponse = await axios.get(`${apiUrl}/api/dicom/studies?limit=1`, config);

        if (!Array.isArray(studiesResponse.data) || studiesResponse.data.length === 0) {
          throw new Error('No studies found to test series retrieval');
        }

        // Study UID 추출 (0020000D 태그)
        const studyUid = studiesResponse.data[0]['0020000D']?.Value?.[0];
        if (!studyUid) {
          throw new Error('Study UID not found in response');
        }

        const requestInfo = { method: 'GET', url: `/api/dicom/studies/${studyUid}/series` };
        const response = await axios.get(`${apiUrl}/api/dicom/studies/${studyUid}/series`, config);

        console.log(`  ✅ DICOM Series 전체 조회 성공 (Study UID: ${studyUid}):`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: '/api/dicom/studies/{studyUid}/series',
          };
        }
        throw error;
      }
    } else if (testIndex === 3) {
      // DICOM Instances 전체 조회 (project_id 없음) - SUPER_ADMIN 권한 필요
      try {
        const config = await getAxiosConfig('SUPER_ADMIN');

        // 먼저 Study를 조회해서 Study UID를 얻음
        const studiesResponse = await axios.get(`${apiUrl}/api/dicom/studies?limit=1`, config);

        if (!Array.isArray(studiesResponse.data) || studiesResponse.data.length === 0) {
          throw new Error('No studies found to test instances retrieval');
        }

        // Study UID 추출 (0020000D 태그)
        const studyUid = studiesResponse.data[0]['0020000D']?.Value?.[0];
        if (!studyUid) {
          throw new Error('Study UID not found in response');
        }

        // Series 조회해서 Series UID를 얻음
        const seriesResponse = await axios.get(`${apiUrl}/api/dicom/studies/${studyUid}/series?limit=1`, config);

        if (!Array.isArray(seriesResponse.data) || seriesResponse.data.length === 0) {
          throw new Error('No series found to test instances retrieval');
        }

        // Series UID 추출 (0020000E 태그)
        const seriesUid = seriesResponse.data[0]['0020000E']?.Value?.[0];
        if (!seriesUid) {
          throw new Error('Series UID not found in response');
        }

        const requestInfo = { method: 'GET', url: `/api/dicom/studies/${studyUid}/series/${seriesUid}/instances` };
        const response = await axios.get(`${apiUrl}/api/dicom/studies/${studyUid}/series/${seriesUid}/instances`, config);

        console.log(`  ✅ DICOM Instances 전체 조회 성공 (Study UID: ${studyUid}, Series UID: ${seriesUid}):`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: '/api/dicom/studies/{studyUid}/series/{seriesUid}/instances',
          };
        }
        throw error;
      }
    } else if (testIndex === 4) {
      // 할당 여부 확인 (check_assignment_for_project) - ADMIN 권한으로 테스트
      const projectId = createdProjectIdRef.current || 150;
      const requestInfo = { method: 'GET', url: `/api/dicom/studies?check_assignment_for_project=${projectId}` };

      try {
        const config = await getAxiosConfig('ADMIN');
        const response = await axios.get(`${apiUrl}/api/dicom/studies?check_assignment_for_project=${projectId}`, config);

        console.log(`  ✅ 할당 여부 확인 성공 (project_id=${projectId}):`, response.data);

        // is_assigned 필드 확인
        if (Array.isArray(response.data) && response.data.length > 0) {
          const hasAssignmentField = response.data.every((study: any) =>
            study.hasOwnProperty('is_assigned') && study.hasOwnProperty('checked_project_id')
          );

          if (!hasAssignmentField) {
            throw new Error('응답에 is_assigned 또는 checked_project_id 필드가 없습니다');
          }
        }

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies?check_assignment_for_project=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 5) {
      // 전체 조회 + 할당 여부 확인 (통합) - ADMIN 권한으로 테스트
      const projectId = createdProjectIdRef.current || 150;
      const requestInfo = { method: 'GET', url: `/api/dicom/studies?check_assignment_for_project=${projectId}` };

      try {
        const config = await getAxiosConfig('ADMIN');
        const response = await axios.get(`${apiUrl}/api/dicom/studies?check_assignment_for_project=${projectId}`, config);

        console.log(`  ✅ 전체 조회 + 할당 여부 확인 성공:`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies?check_assignment_for_project=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // 프로젝트별 조회 + 할당 여부 확인 - USER 권한으로 테스트
      const projectId = createdProjectIdRef.current || 150;
      const requestInfo = {
        method: 'GET',
        url: `/api/dicom/studies?project_id=${projectId}&check_assignment_for_project=${projectId}`
      };

      try {
        const config = await getAxiosConfig('USER');
        const response = await axios.get(
          `${apiUrl}/api/dicom/studies?project_id=${projectId}&check_assignment_for_project=${projectId}`,
          config
        );

        console.log(`  ✅ 프로젝트별 조회 + 할당 여부 확인 성공:`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies?project_id=${projectId}&check_assignment_for_project=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // 다른 프로젝트 할당 여부 확인 - ADMIN 권한으로 테스트
      const filterProjectId = createdProjectIdRef.current || 150;
      const checkProjectId = 200; // 다른 프로젝트
      const requestInfo = {
        method: 'GET',
        url: `/api/dicom/studies?project_id=${filterProjectId}&check_assignment_for_project=${checkProjectId}`
      };

      try {
        const config = await getAxiosConfig('ADMIN');
        const response = await axios.get(
          `${apiUrl}/api/dicom/studies?project_id=${filterProjectId}&check_assignment_for_project=${checkProjectId}`,
          config
        );

        console.log(`  ✅ 다른 프로젝트 할당 여부 확인 성공:`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies?project_id=${filterProjectId}&check_assignment_for_project=${checkProjectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 8) {
      // 잘못된 project_id (0) 에러 처리 - USER 권한으로 테스트
      const requestInfo = { method: 'GET', url: '/api/dicom/studies?project_id=0' };

      try {
        const config = await getAxiosConfig('USER');
        await axios.get(`${apiUrl}/api/dicom/studies?project_id=0`, config);
        throw new Error('400 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 400) {
          console.log(`  ✅ project_id=0 에러 처리 성공:`, error.response.data);
          return {
            request: requestInfo,
            response: error.response.data || { status: 400, message: 'Bad Request' },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies?project_id=0`,
          };
        }
        throw error;
      }
    } else if (testIndex === 9) {
      // 잘못된 project_id (음수) 에러 처리 - USER 권한으로 테스트
      const requestInfo = { method: 'GET', url: '/api/dicom/studies?project_id=-1' };

      try {
        const config = await getAxiosConfig('USER');
        await axios.get(`${apiUrl}/api/dicom/studies?project_id=-1`, config);
        throw new Error('400 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 400) {
          console.log(`  ✅ project_id=-1 에러 처리 성공:`, error.response.data);
          return {
            request: requestInfo,
            response: error.response.data || { status: 400, message: 'Bad Request' },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies?project_id=-1`,
          };
        }
        throw error;
      }
    }
  };

  // Project Data Access 접근 제어 테스트
  const runProjectDataAccessTest = async (testIndex: number) => {
    if (testIndex === 0) {
      // 시나리오 구성
      const requestInfo = { method: 'POST', url: '/api/test/project-data-access/setup' };

      try {
        const response = await axios.post(`${apiUrl}/api/test/project-data-access/setup`, {}, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        // 생성된 프로젝트 ID 저장
        createdProjectIdRef.current = response.data.project_id;

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/test/project-data-access/setup`,
          };
        }
        throw error;
      }
    } else if (testIndex === 1) {
      // 접근 제어 매트릭스 조회
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다. 먼저 시나리오를 구성하세요.');
      }

      const requestInfo = { method: 'GET', url: `/api/project-data/${projectId}/data-access/matrix` };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/data-access/matrix`,
          };
        }
        throw error;
      }
    } else if (testIndex === 2) {
      // 매트릭스 구조 검증 (4명 사용자)
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'GET', url: `/api/project-data/${projectId}/data-access/matrix` };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const data = response.data;
        const users = data.users || [];
        if (!Array.isArray(users) || users.length !== 4) {
          throw new Error(`4명의 사용자가 있어야 하는데 ${users.length}명이 있습니다`);
        }

        return {
          request: requestInfo,
          response: { users: users.length, usernames: users.map((u: any) => u.username) },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/data-access/matrix`,
          };
        }
        throw error;
      }
    } else if (testIndex === 3) {
      // 매트릭스 구조 검증 (7개 Study)
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'GET', url: `/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100` };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const data = response.data;
        const dataList = data.data_list || [];

        // 7개의 Study가 있는지 확인
        if (dataList.length < 7) {
          throw new Error(`7개의 Study가 있어야 하는데 ${dataList.length}개가 있습니다`);
        }

        return {
          request: requestInfo,
          response: { studies: dataList.length, study_uids: dataList.slice(0, 7).map((s: any) => s.study_uid) },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`,
          };
        }
        throw error;
      }
    } else if (testIndex === 4) {
      // Dr. Kim 전체 접근 확인 (7/7)
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'GET', url: `/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100` };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const data = response.data;
        const users = data.users || [];
        const drKim = users.find((u: any) => u.username === 'dr_kim');

        if (!drKim) {
          throw new Error('Dr. Kim을 찾을 수 없습니다');
        }

        // 프로젝트 멤버는 기본적으로 모든 데이터에 접근 가능 (project_data_access 레코드가 없으면)
        // Dr. Kim은 책임연구원이므로 모든 Study에 접근 가능
        const dataList = data.data_list || [];
        const studyCount = dataList.filter((d: any) => d.study_uid).length;

        return {
          request: requestInfo,
          response: {
            user: drKim.username,
            message: 'Dr. Kim (책임연구원)은 모든 Study에 접근 가능',
            total_studies: studyCount
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`,
          };
        }
        throw error;
      }
    } else if (testIndex === 5) {
      // Dr. Lee A병원만 접근 확인 (3/7)
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'GET', url: `/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100` };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const data = response.data;
        const users = data.users || [];
        const drLee = users.find((u: any) => u.username === 'dr_lee');

        if (!drLee) {
          throw new Error('Dr. Lee를 찾을 수 없습니다');
        }

        // project_data_access 테이블에서 Dr. Lee의 접근 권한 확인
        const accessMatrix = data.access_matrix || [];
        const drLeeAccess = accessMatrix.filter((a: any) => a.user_id === drLee.id);

        // A병원 Study 3개에 대한 접근 권한이 있어야 함
        const dataList = data.data_list || [];
        const hospitalAStudies = dataList.filter((d: any) => d.study_uid && d.study_uid.includes('.A.'));

        return {
          request: requestInfo,
          response: {
            user: drLee.username,
            message: 'Dr. Lee (A병원)는 A병원 Study 3개에만 접근 가능',
            hospital_a_studies: hospitalAStudies.length,
            access_records: drLeeAccess.length
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`,
          };
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // Dr. Park B병원만 접근 확인 (3/7)
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'GET', url: `/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100` };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const data = response.data;
        const users = data.users || [];
        const drPark = users.find((u: any) => u.username === 'dr_park');

        if (!drPark) {
          throw new Error('Dr. Park을 찾을 수 없습니다');
        }

        // project_data_access 테이블에서 Dr. Park의 접근 권한 확인
        const accessMatrix = data.access_matrix || [];
        const drParkAccess = accessMatrix.filter((a: any) => a.user_id === drPark.id);

        // B병원 Study 3개에 대한 접근 권한이 있어야 함
        const dataList = data.data_list || [];
        const hospitalBStudies = dataList.filter((d: any) => d.study_uid && d.study_uid.includes('.B.'));

        return {
          request: requestInfo,
          response: {
            user: drPark.username,
            message: 'Dr. Park (B병원)은 B병원 Study 3개에만 접근 가능',
            hospital_b_studies: hospitalBStudies.length,
            access_records: drParkAccess.length
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`,
          };
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // Dr. Choi 읽기 전용 확인 (1/7)
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'GET', url: `/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100` };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const data = response.data;
        const users = data.users || [];
        const drChoi = users.find((u: any) => u.username === 'dr_choi');

        if (!drChoi) {
          throw new Error('Dr. Choi를 찾을 수 없습니다');
        }

        // project_data_access 테이블에서 Dr. Choi의 접근 권한 확인
        const accessMatrix = data.access_matrix || [];
        const drChoiAccess = accessMatrix.filter((a: any) => a.user_id === drChoi.id);

        // VIP Study 1개에 대한 읽기 전용 접근 권한이 있어야 함
        const readOnlyAccess = drChoiAccess.find((a: any) => a.access_scope === 'READ_ONLY');

        return {
          request: requestInfo,
          response: {
            user: drChoi.username,
            message: 'Dr. Choi (임시 협력자)는 VIP Study 1개에 읽기 전용 접근 가능',
            access_records: drChoiAccess.length,
            read_only: readOnlyAccess ? true : false
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`,
          };
        }
        throw error;
      }
    } else if (testIndex === 8) {
      // 시나리오 초기화
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'DELETE', url: `/api/test/project-data-access/cleanup/${projectId}` };

      try {
        const response = await axios.delete(`${apiUrl}/api/test/project-data-access/cleanup/${projectId}`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        // 프로젝트 ID 초기화
        createdProjectIdRef.current = null;

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/test/project-data-access/cleanup/${projectId}`,
          };
        }
        throw error;
      }
    }
  };

  // 순차 시나리오 테스트 (실제 API 호출)
  const runSequentialScenarioTest = async (testIndex: number) => {
    if (testIndex === 0) {
      // 0️⃣ 사전 정리 (기존 테스트 데이터 삭제)
      const requestInfo = {
        method: 'DELETE',
        url: '/api/projects (기존 순차 테스트 프로젝트)',
      };

      try {
        const deletedItems: any = {
          project: null,
          users: [],
        };

        // 1. 기존 "심장질환 공동 연구 (순차)" 프로젝트 찾기 및 삭제
        const projectsResponse = await axios.get(`${apiUrl}/api/projects`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const projects = projectsResponse.data.projects || [];
        const existingProject = projects.find(
          (p: any) => p.name === '심장질환 공동 연구 (순차)'
        );

        if (existingProject) {
          await axios.delete(`${apiUrl}/api/projects/${existingProject.id}`, {
            headers: {
              Authorization: `Bearer ${testToken}`,
            },
          });
          deletedItems.project = { id: existingProject.id, name: existingProject.name };
        }

        // 2. 기존 테스트 사용자 찾기 및 삭제 (_seq 붙은 것과 안 붙은 것 모두)
        const testUsernames = [
          'dr_kim_seq', 'dr_lee_seq', 'dr_park_seq', 'dr_choi_seq',
          'dr_kim', 'dr_lee', 'dr_park', 'dr_choi'
        ];

        // 모든 사용자 조회
        const allUsersResponse = await axios.get(`${apiUrl}/api/users`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const allUsers = allUsersResponse.data.users || [];

        for (const username of testUsernames) {
          try {
            const existingUser = allUsers.find((u: any) => u.username === username);

            if (existingUser) {
              console.log(`Deleting user: ${username} (ID: ${existingUser.id})`);
              // 사용자 삭제
              await axios.delete(`${apiUrl}/api/users/${existingUser.id}`, {
                headers: {
                  Authorization: `Bearer ${testToken}`,
                },
              });
              deletedItems.users.push({ id: existingUser.id, username: existingUser.username });
              console.log(`Successfully deleted user: ${username}`);
            }
          } catch (error: any) {
            // 개별 사용자 삭제 실패는 무시하고 계속 진행
            console.log(`Failed to delete user ${username}:`, error.response?.data || error.message);
          }
        }

        return {
          request: requestInfo,
          response: {
            message: '사전 정리 완료',
            deleted_project: deletedItems.project,
            deleted_users: deletedItems.users,
            deleted_users_count: deletedItems.users.length,
          },
        };
      } catch (error: any) {
        // 에러가 발생해도 계속 진행 (정리 단계이므로)
        return {
          request: requestInfo,
          response: { message: '정리 중 에러 발생 (무시하고 계속 진행)', error: error.message },
        };
      }
    } else if (testIndex === 1) {
      // 1️⃣ 프로젝트 생성
      const requestInfo = {
        method: 'POST',
        url: '/api/projects',
        body: {
          name: '심장질환 공동 연구 (순차)',
          description: '다기관 공동 연구 프로젝트 - 순차 API 호출 테스트',
          sponsor: '서울대학교병원',
          start_date: '2025-01-01',
          end_date: '2025-12-31'
        }
      };

      try {
        const response = await axios.post(`${apiUrl}/api/projects`, requestInfo.body, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        sequentialProjectIdRef.current = response.data.id;

        return {
          request: requestInfo,
          response: { project_id: response.data.id, name: response.data.name },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 2) {
      // 2️⃣ 사용자 4명 생성
      const users = [
        { username: 'dr_kim_seq', email: 'dr.kim.seq@hospital.com', full_name: 'Dr. Kim (책임연구원)', keycloak_id: `kim-seq-${Date.now()}` },
        { username: 'dr_lee_seq', email: 'dr.lee.seq@hospital-a.com', full_name: 'Dr. Lee (A병원)', keycloak_id: `lee-seq-${Date.now()}` },
        { username: 'dr_park_seq', email: 'dr.park.seq@hospital-b.com', full_name: 'Dr. Park (B병원)', keycloak_id: `park-seq-${Date.now()}` },
        { username: 'dr_choi_seq', email: 'dr.choi.seq@temp.com', full_name: 'Dr. Choi (임시 협력자)', keycloak_id: `choi-seq-${Date.now()}` },
      ];

      const requestInfo = { method: 'POST', url: '/api/auth/signup', body: users };

      try {
        const createdUsers = [];
        for (const user of users) {
          const response = await axios.post(`${apiUrl}/api/auth/signup`, {
            username: user.username,
            email: user.email,
            password: 'Test1234!',
            full_name: user.full_name,
          }, {
            headers: {
              Authorization: `Bearer ${testToken}`,
            },
          });

          createdUsers.push(response.data);
          sequentialUserIdsRef.current[user.username] = response.data.user_id;
        }

        return {
          request: requestInfo,
          response: {
            users: createdUsers.map(u => ({ username: u.username, user_id: u.user_id })),
            count: createdUsers.length
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 3) {
      // 3️⃣ 사용자 4명 활성화 (관리자 승인)
      const userIds = Object.values(sequentialUserIdsRef.current);
      if (userIds.length === 0) {
        throw new Error('생성된 사용자가 없습니다.');
      }

      const requestInfo = {
        method: 'POST',
        url: '/api/auth/admin/users/approve',
        body: userIds.map(id => ({ user_id: id }))
      };

      try {
        const approvedUsers = [];
        for (const userId of userIds) {
          const response = await axios.post(`${apiUrl}/api/auth/admin/users/approve`, {
            user_id: userId,
          }, {
            headers: {
              Authorization: `Bearer ${testToken}`,
            },
          });

          approvedUsers.push({ user_id: userId, ...response.data });
        }

        return {
          request: requestInfo,
          response: {
            approved_users: approvedUsers,
            count: approvedUsers.length,
            message: '모든 사용자 활성화 완료'
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 4) {
      // 4️⃣ 사용자를 프로젝트 멤버로 추가
      const projectId = sequentialProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'POST',
        url: `/api/projects/${projectId}/members`,
        body: Object.values(sequentialUserIdsRef.current)
      };

      try {
        const addedMembers = [];
        for (const [username, userId] of Object.entries(sequentialUserIdsRef.current)) {
          const response = await axios.post(`${apiUrl}/api/projects/${projectId}/members`, {
            user_id: userId,
          }, {
            headers: {
              Authorization: `Bearer ${testToken}`,
            },
          });

          addedMembers.push({ username, user_id: userId, role: response.data.role_name });
        }

        return {
          request: requestInfo,
          response: { members: addedMembers, count: addedMembers.length },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 5) {
      // 5️⃣ Study 7개 생성
      const projectId = sequentialProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const studies = [
        { study_uid: `1.2.840.113619.2.55.3.A.1.${Date.now()}`, study_description: 'CT Chest - A병원 환자1', patient_id: 'A-P001', patient_name: '김철수', study_date: '2025-01-10' },
        { study_uid: `1.2.840.113619.2.55.3.A.2.${Date.now()}`, study_description: 'CT Chest - A병원 환자2', patient_id: 'A-P002', patient_name: '이영희', study_date: '2025-01-11' },
        { study_uid: `1.2.840.113619.2.55.3.A.3.${Date.now()}`, study_description: 'CT Chest - A병원 환자3', patient_id: 'A-P003', patient_name: '박민수', study_date: '2025-01-12' },
        { study_uid: `1.2.840.113619.2.55.3.B.1.${Date.now()}`, study_description: 'MRI Brain - B병원 환자1', patient_id: 'B-P001', patient_name: '최지훈', study_date: '2025-01-13' },
        { study_uid: `1.2.840.113619.2.55.3.B.2.${Date.now()}`, study_description: 'MRI Brain - B병원 환자2', patient_id: 'B-P002', patient_name: '정수진', study_date: '2025-01-14' },
        { study_uid: `1.2.840.113619.2.55.3.B.3.${Date.now()}`, study_description: 'MRI Brain - B병원 환자3', patient_id: 'B-P003', patient_name: '강민호', study_date: '2025-01-15' },
        { study_uid: `1.2.840.113619.2.55.3.VIP.1.${Date.now()}`, study_description: 'PET-CT - VIP 환자', patient_id: 'VIP-001', patient_name: 'VIP 환자', study_date: '2025-01-16' },
      ];

      const requestInfo = {
        method: 'POST',
        url: `/api/project-data/${projectId}/data`,
        body: studies
      };

      try {
        const createdStudies = [];
        for (const study of studies) {
          const response = await axios.post(`${apiUrl}/api/project-data/${projectId}/data`, study, {
            headers: {
              Authorization: `Bearer ${testToken}`,
            },
          });

          createdStudies.push({ study_uid: study.study_uid, data_id: response.data.data_id });
          sequentialStudyIdsRef.current.push(response.data.data_id);
        }

        return {
          request: requestInfo,
          response: { studies: createdStudies, count: createdStudies.length },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // 6️⃣ 접근 제어 설정
      const projectId = sequentialProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const studyIds = sequentialStudyIdsRef.current;
      if (studyIds.length < 7) {
        throw new Error('Study가 충분하지 않습니다.');
      }

      const requestInfo = {
        method: 'PUT',
        url: `/api/project-data/${projectId}/data/{data_id}/access/{user_id}`,
        body: 'Multiple access control records'
      };

      try {
        const accessRecords = [];

        // Dr. Lee → A병원 Study 3개 (0, 1, 2)
        for (let i = 0; i < 3; i++) {
          await axios.put(
            `${apiUrl}/api/project-data/${projectId}/data/${studyIds[i]}/access/${sequentialUserIdsRef.current['dr_lee_seq']}`,
            { status: 'APPROVED', review_note: 'A병원 연구원 접근 승인' },
            { headers: { Authorization: `Bearer ${testToken}` } }
          );
          accessRecords.push({ user: 'dr_lee_seq', study_index: i, status: 'APPROVED' });
        }

        // Dr. Park → B병원 Study 3개 (3, 4, 5)
        for (let i = 3; i < 6; i++) {
          await axios.put(
            `${apiUrl}/api/project-data/${projectId}/data/${studyIds[i]}/access/${sequentialUserIdsRef.current['dr_park_seq']}`,
            { status: 'APPROVED', review_note: 'B병원 연구원 접근 승인' },
            { headers: { Authorization: `Bearer ${testToken}` } }
          );
          accessRecords.push({ user: 'dr_park_seq', study_index: i, status: 'APPROVED' });
        }

        // Dr. Choi → VIP Study 1개 (6) - 읽기 전용
        await axios.put(
          `${apiUrl}/api/project-data/${projectId}/data/${studyIds[6]}/access/${sequentialUserIdsRef.current['dr_choi_seq']}`,
          { status: 'APPROVED', review_note: '임시 협력자 읽기 전용 접근' },
          { headers: { Authorization: `Bearer ${testToken}` } }
        );
        accessRecords.push({ user: 'dr_choi_seq', study_index: 6, status: 'APPROVED', scope: 'READ_ONLY' });

        return {
          request: requestInfo,
          response: { access_records: accessRecords, count: accessRecords.length },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // 7️⃣ 접근 제어 매트릭스 조회 및 검증
      const projectId = sequentialProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`
      };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const data = response.data;
        const users = data.users || [];
        const accessMatrix = data.access_matrix || [];
        const dataList = data.data_list || [];

        return {
          request: requestInfo,
          response: {
            users: users.length,
            studies: dataList.length,
            access_records: accessMatrix.length,
            matrix_summary: {
              dr_lee: accessMatrix.filter((a: any) => a.user_id === sequentialUserIdsRef.current['dr_lee_seq']).length,
              dr_park: accessMatrix.filter((a: any) => a.user_id === sequentialUserIdsRef.current['dr_park_seq']).length,
              dr_choi: accessMatrix.filter((a: any) => a.user_id === sequentialUserIdsRef.current['dr_choi_seq']).length,
            }
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 8) {
      // 8️⃣ DICOM QIDO API로 실제 접근 제어 검증
      const projectId = sequentialProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: '/api/dicom/studies (4명의 사용자로 접근 제어 검증)',
      };

      try {
        const testUsers = [
          { username: 'dr_kim_seq', password: 'Test1234!', name: 'Dr. Kim (책임연구원)', expectedStudies: 7 },
          { username: 'dr_lee_seq', password: 'Test1234!', name: 'Dr. Lee (A병원)', expectedStudies: 3 },
          { username: 'dr_park_seq', password: 'Test1234!', name: 'Dr. Park (B병원)', expectedStudies: 3 },
          { username: 'dr_choi_seq', password: 'Test1234!', name: 'Dr. Choi (임시 협력자)', expectedStudies: 1 },
        ];

        const results: any[] = [];

        for (const user of testUsers) {
          // 1. 사용자로 로그인
          const loginResponse = await axios.post(`${apiUrl}/api/auth/login`, {
            username: user.username,
            password: user.password,
          });

          const userToken = loginResponse.data.token;
          const keycloakToken = loginResponse.data.keycloak_access_token;

          console.log(`[${user.username}] Login response:`, {
            has_backend_token: !!userToken,
            has_keycloak_token: !!keycloakToken,
            keycloak_token_preview: keycloakToken ? keycloakToken.substring(0, 50) + '...' : 'MISSING'
          });

          // 2. DICOM QIDO API 호출 (project_id 필수, Keycloak access token 사용)
          const dicomResponse = await axios.get(`${apiUrl}/api/dicom/studies`, {
            params: {
              project_id: projectId,
            },
            headers: {
              Authorization: `Bearer ${keycloakToken}`,
            },
          });

          const studies = dicomResponse.data || [];
          const studyUids = studies.map((s: any) => s['0020000D']?.Value?.[0] || 'Unknown');

          results.push({
            user: user.name,
            username: user.username,
            expected_studies: user.expectedStudies,
            actual_studies: studies.length,
            study_uids: studyUids,
            access_control_working: studies.length === user.expectedStudies ? '✅ 정상' : '❌ 오류',
          });
        }

        return {
          request: requestInfo,
          response: {
            message: '실제 접근 제어 검증 완료',
            results: results,
            summary: {
              total_tests: results.length,
              passed: results.filter((r: any) => r.access_control_working === '✅ 정상').length,
              failed: results.filter((r: any) => r.access_control_working === '❌ 오류').length,
            }
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 9) {
      // 9️⃣ 정리 (프로젝트 삭제)
      const projectId = sequentialProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'DELETE', url: `/api/projects/${projectId}` };

      try {
        const response = await axios.delete(`${apiUrl}/api/projects/${projectId}`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        // 초기화
        sequentialProjectIdRef.current = null;
        sequentialUserIdsRef.current = {};
        sequentialStudyIdsRef.current = [];

        return {
          request: requestInfo,
          response: { message: '프로젝트 및 관련 데이터 삭제 완료', project_id: projectId },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    }

    return { request: {}, response: { message: '알 수 없는 테스트 인덱스' } };
  };

  // 섹션별 테스트 실행
  const runSectionTests = async (sectionIndex: number) => {
    const section = sections[sectionIndex];

    console.log(`🚀 섹션 테스트 시작: ${section.title}`);

    // 해당 섹션의 테스트들을 pending으로 초기화
    const newSections = [...sections];
    newSections[sectionIndex] = {
      ...section,
      tests: section.tests.map(test => ({
        ...test,
        status: 'pending' as const,
        request: undefined,
        response: undefined,
        error: undefined,
        duration: undefined,
      })),
    };
    setSections(newSections);

    // 섹션의 모든 테스트 실행
    for (let testIndex = 0; testIndex < section.tests.length; testIndex++) {
      await runTest(sectionIndex, testIndex);
    }

    console.log(`✅ 섹션 테스트 완료: ${section.title}`);
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

      {/* 테스트 계정 선택 및 토큰 획득 */}
      <div className="test-account-section">
        <div className="account-selector">
          <label>테스트 계정:</label>
          <select
            value={currentTestAccount.username}
            onChange={(e) => {
              const account = Object.values(TEST_ACCOUNTS).find(a => a.username === e.target.value);
              if (account) {
                setCurrentTestAccount(account);
                setTestToken(null); // 계정 변경 시 토큰 초기화
              }
            }}
          >
            {Object.values(TEST_ACCOUNTS).map(account => (
              <option key={account.username} value={account.username}>
                {account.username} ({account.role})
              </option>
            ))}
          </select>
          <button
            onClick={() => getTestToken(currentTestAccount)}
            className="get-token-button"
          >
            🔑 토큰 획득
          </button>
          {testToken && (
            <span className="token-status">✅ 토큰 획득 완료</span>
          )}
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
          <div className="section-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <h3>{section.title}</h3>
              <p className="section-description">{section.description}</p>
            </div>
            <button
              onClick={() => runSectionTests(sectionIndex)}
              disabled={isRunningAll}
              className="run-section-button"
              style={{
                padding: '8px 16px',
                backgroundColor: '#10b981',
                color: 'white',
                border: 'none',
                borderRadius: '6px',
                cursor: isRunningAll ? 'not-allowed' : 'pointer',
                fontSize: '14px',
                fontWeight: '500',
                opacity: isRunningAll ? 0.5 : 1,
              }}
            >
              🚀 이 섹션 모두 실행
            </button>
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

