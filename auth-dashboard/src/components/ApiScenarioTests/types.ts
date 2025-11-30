export interface TestResult {
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

export interface TestSection {
  title: string;
  description: string;
  tests: TestResult[];
  isSequential?: boolean; // 섹션 전체가 순차 실행되어야 하는지
}

export interface TestAccount {
  username: string;
  keycloak_id: string;
  role: string;
}

export interface TestContext {
  apiUrl: string;
  testToken: string | null;
  currentTestAccount: TestAccount;
  setTestToken: (token: string | null) => void;
  setCurrentTestAccount: (account: TestAccount) => void;
}

