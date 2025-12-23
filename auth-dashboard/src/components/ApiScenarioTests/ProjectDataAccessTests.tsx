import { TestSection } from './types';

export const getProjectDataAccessSections = (): TestSection[] => [
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
];


