/**
 * 애플리케이션 전역 상수
 */

// 애플리케이션 정보
export const APP_NAME = 'PACS 관리 대시보드';
export const APP_SHORT_NAME = 'PACS Dashboard';
export const APP_VERSION = '1.0.0';

// API 설정
export const DEFAULT_API_URL = 'http://localhost:8080';

// 로그인 페이지 텍스트
export const LOGIN_PAGE = {
  TITLE: '🔐 PACS 관리 시스템',
  SUBTITLE: '인증 및 API 관리 대시보드',
  INFO_TITLE: 'ℹ️ 로그인 안내',
  INFO_DESCRIPTION: 'Keycloak 계정으로 로그인하여 시스템을 관리하세요.',
  INFO_NOTE: '이 대시보드는 인증 및 프로젝트 관리 API를 테스트합니다.',
  BUTTON_LOGIN: '로그인',
  BUTTON_LOGGING_IN: '로그인 중...',
  LABEL_API_URL: 'API URL',
  LABEL_USERNAME: '사용자명',
  LABEL_PASSWORD: '비밀번호',
  PLACEHOLDER_API_URL: 'http://localhost:8080',
  PLACEHOLDER_USERNAME: '사용자명을 입력하세요',
  PLACEHOLDER_PASSWORD: '비밀번호를 입력하세요',
};

// 대시보드 페이지 텍스트
export const DASHBOARD_PAGE = {
  TITLE: '🎛️ PACS 관리 대시보드',
  BUTTON_LOGOUT: '로그아웃',
};

// 사이드바 메뉴
export const SIDEBAR_MENU = {
  AUTH_TEST: {
    id: 'auth',
    icon: '🔐',
    label: '인증 테스트',
    description: '토큰 검증 및 갱신',
  },
  API_HEALTH: {
    id: 'api-health',
    icon: '🔍',
    label: 'API 점검',
    description: 'E2E 테스트 실행',
    subMenus: [
      { id: 'api-health-scenario', label: '시나리오 테스트', icon: '📊' },
      { id: 'api-health-study-list-view', label: '컬럼 설정 (View)', icon: '📋' },
      { id: 'api-health-view-selection', label: 'View Selection', icon: '🎬' },
      { id: 'api-health-qido-enhanced', label: 'QIDO Enhanced', icon: '🚀' },
    ],
  },
  PROJECT_MANAGEMENT: {
    id: 'projects',
    icon: '📁',
    label: '프로젝트 관리',
    description: '프로젝트 CRUD',
  },
  USER_MANAGEMENT: {
    id: 'users',
    icon: '👥',
    label: '사용자 관리',
    description: '사용자 및 권한',
  },
} as const;

// 사이드바 메뉴 순서
export const SIDEBAR_MENU_ORDER = [
  SIDEBAR_MENU.AUTH_TEST,
  SIDEBAR_MENU.API_HEALTH,
  // SIDEBAR_MENU.PROJECT_MANAGEMENT,  // 향후 추가
  // SIDEBAR_MENU.USER_MANAGEMENT,     // 향후 추가
] as const;

// 인증 테스트 섹션
export const AUTH_TEST_SECTION = {
  USER_INFO: {
    TITLE: '👤 사용자 정보',
    LABEL_USER_ID: 'User ID',
    LABEL_USERNAME: '사용자명',
    LABEL_EMAIL: '이메일',
    LABEL_KEYCLOAK_ID: 'Keycloak ID',
  },
  TOKEN_INFO: {
    TITLE: '🔑 토큰 정보',
    JWT_TOKEN: 'JWT Access Token',
    REFRESH_TOKEN: 'Refresh Token',
    BUTTON_COPY: '📋 복사',
    LABEL_EXPIRES: '만료 시간',
    LABEL_TIME_REMAINING: '⏱️',
    MESSAGE_COPIED: '클립보드에 복사되었습니다!',
  },
  TEST_ACTIONS: {
    TITLE: '🧪 테스트 작업',
    BUTTON_VERIFY: '🔍 토큰 검증',
    BUTTON_REFRESH: '🔄 토큰 갱신',
    RESULT_VALID: '✅ 토큰 유효',
    RESULT_INVALID: '❌ 토큰 무효',
    RESULT_REFRESHED: '✅ 토큰이 성공적으로 갱신되었습니다!',
    RESULT_REFRESH_FAILED: '❌ 토큰 갱신 실패',
  },
  API_ENDPOINTS: {
    TITLE: '📡 API 엔드포인트',
    LOGIN: {
      METHOD: 'POST',
      PATH: '/api/auth/login',
      DESCRIPTION: '사용자명/비밀번호로 로그인',
    },
    VERIFY: {
      METHOD: 'GET',
      PATH: '/api/auth/verify/:token',
      DESCRIPTION: 'JWT 토큰 검증',
    },
    REFRESH: {
      METHOD: 'POST',
      PATH: '/api/auth/refresh',
      DESCRIPTION: '토큰 갱신',
    },
  },
};

// 시간 포맷
export const TIME_FORMAT = {
  EXPIRED: '만료됨',
  NOT_AVAILABLE: 'N/A',
};

// HTTP 메서드 색상
export const HTTP_METHOD_COLORS = {
  GET: 'get',
  POST: 'post',
  PUT: 'put',
  DELETE: 'delete',
} as const;

