import axios from 'axios';
import { TestAccount } from './types';
import { TEST_ACCOUNTS } from './constants';

export const getTestToken = async (
  account: TestAccount,
  apiUrl: string,
  setTestToken: (token: string | null) => void,
  setCurrentTestAccount: (account: TestAccount) => void
): Promise<string> => {
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

export const getAxiosConfig = async (
  accountType: 'SUPER_ADMIN' | 'ADMIN' | 'USER' | undefined,
  testToken: string | null,
  apiUrl: string,
  setTestToken: (token: string | null) => void,
  setCurrentTestAccount: (account: TestAccount) => void
) => {
  // accountType이 지정되면 해당 계정의 토큰을 자동 획득
  if (accountType) {
    const account = TEST_ACCOUNTS[accountType];
    try {
      const token = await getTestToken(account, apiUrl, setTestToken, setCurrentTestAccount);
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

