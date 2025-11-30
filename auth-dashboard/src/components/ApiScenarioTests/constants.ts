import { TestAccount } from './types';

export const TEST_ACCOUNTS: Record<string, TestAccount> = {
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

export const DEFAULT_API_URL = 'http://localhost:8080';

