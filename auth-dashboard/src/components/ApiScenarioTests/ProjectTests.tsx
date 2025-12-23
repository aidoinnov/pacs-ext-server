import axios from 'axios';
import { TestSection } from './types';

export interface ProjectTestRefs {
  createdProjectIdRef: React.MutableRefObject<number | null>;
  createdStudyIdRef: React.MutableRefObject<number | null>;
  createdStudyUidRef: React.MutableRefObject<string | null>;
  createdSeriesIdsRef: React.MutableRefObject<number[]>;
  createdSeriesUidsRef: React.MutableRefObject<string[]>;
  setCreatedProjectId: (id: number | null) => void;
}

export interface ProjectTestContext {
  apiUrl: string;
  handleGetAxiosConfig: (accountType?: 'SUPER_ADMIN' | 'ADMIN' | 'USER') => Promise<any>;
  refs: ProjectTestRefs;
  sections: TestSection[];
}

export const getProjectSections = (): TestSection[] => [
  {
    title: '📊 프로젝트 메타데이터',
    description: '프로젝트 상태 메타데이터 조회 테스트 (순서 무관)',
    isSequential: false,
    tests: [
      { name: '메타데이터 조회', status: 'pending' },
      { name: '메타데이터 구조 검증', status: 'pending' },
      { name: '5개 상태 존재 확인', status: 'pending' },
    ],
  },
  {
    title: '🔄 프로젝트 생명주기',
    description: '프로젝트 생성 및 상태 변경 테스트 (순차 실행)',
    isSequential: true,
    tests: [
      {
        name: '프로젝트 생성 (PREPARING)',
        status: 'pending',
        isSequential: true,
        indentLevel: 0,
        delayAfter: 1500,
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
        delayAfter: 500,
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
];

// Note: 실제 테스트 함수들은 index.tsx에 있으므로, 여기서는 섹션 정의만 export합니다.
// 테스트 함수는 index.tsx에서 각 섹션별로 호출하도록 유지합니다.


