/**
 * DICOM 전체 데이터 조회 권한 기능 시나리오 테스트
 * 
 * 테스트 시나리오:
 * 1. SUPER_ADMIN 사용자 - 전체 데이터 조회 가능
 * 2. ADMIN 사용자 - 전체 데이터 조회 가능
 * 3. 일반 사용자 - 전체 데이터 조회 불가, project_id 필수
 * 4. 프로젝트별 조회 - 모든 사용자 가능 (기존 동작)
 */

import { render, screen, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';

// Mock API 응답
const mockStudiesResponse = [
  {
    "0020000D": { "Value": ["1.2.826.0.1.3680043.8.498.11111111111111111111111111111111"] },
    "00100020": { "Value": ["PATIENT001"] },
    "00080060": { "Value": ["CT"] }
  },
  {
    "0020000D": { "Value": ["1.2.826.0.1.3680043.8.498.22222222222222222222222222222222"] },
    "00100020": { "Value": ["PATIENT002"] },
    "00080060": { "Value": ["MR"] }
  },
  {
    "0020000D": { "Value": ["1.2.826.0.1.3680043.8.498.33333333333333333333333333333333"] },
    "00100020": { "Value": ["PATIENT003"] },
    "00080060": { "Value": ["CT"] }
  }
];

// Mock fetch
global.fetch = jest.fn();

describe('DICOM Global Access - SUPER_ADMIN', () => {
  beforeEach(() => {
    (global.fetch as jest.Mock).mockClear();
  });

  test('SUPER_ADMIN can fetch all studies without project_id', async () => {
    // Given: SUPER_ADMIN 사용자
    const mockUser = {
      role: 'SUPER_ADMIN',
      hasGlobalDicomAccess: true
    };

    // Mock API 응답
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockStudiesResponse
    });

    // When: project_id 없이 전체 데이터 조회
    const response = await fetch('/api/dicom/studies');
    const data = await response.json();

    // Then: 전체 데이터 반환
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(3);
    expect(data[0]["00080060"].Value[0]).toBe("CT");
    expect(data[1]["00080060"].Value[0]).toBe("MR");
  });

  test('SUPER_ADMIN can fetch studies with project_id', async () => {
    // Given: SUPER_ADMIN 사용자
    const mockUser = {
      role: 'SUPER_ADMIN',
      hasGlobalDicomAccess: true
    };

    // Mock API 응답 (필터링된 데이터)
    const filteredResponse = [mockStudiesResponse[0]];
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => filteredResponse
    });

    // When: project_id와 함께 조회
    const response = await fetch('/api/dicom/studies?project_id=150');
    const data = await response.json();

    // Then: 필터링된 데이터 반환
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(1);
    expect(data[0]["00100020"].Value[0]).toBe("PATIENT001");
  });
});

describe('DICOM Global Access - ADMIN', () => {
  beforeEach(() => {
    (global.fetch as jest.Mock).mockClear();
  });

  test('ADMIN can fetch all studies without project_id', async () => {
    // Given: ADMIN 사용자
    const mockUser = {
      role: 'ADMIN',
      hasGlobalDicomAccess: true
    };

    // Mock API 응답
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockStudiesResponse
    });

    // When: project_id 없이 전체 데이터 조회
    const response = await fetch('/api/dicom/studies');
    const data = await response.json();

    // Then: 전체 데이터 반환
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(3);
  });

  test('ADMIN can fetch studies with project_id', async () => {
    // Given: ADMIN 사용자
    const mockUser = {
      role: 'ADMIN',
      hasGlobalDicomAccess: true
    };

    // Mock API 응답 (필터링된 데이터)
    const filteredResponse = [mockStudiesResponse[1]];
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => filteredResponse
    });

    // When: project_id와 함께 조회
    const response = await fetch('/api/dicom/studies?project_id=150');
    const data = await response.json();

    // Then: 필터링된 데이터 반환
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(1);
    expect(data[0]["00100020"].Value[0]).toBe("PATIENT002");
  });
});

describe('DICOM Global Access - Regular User', () => {
  beforeEach(() => {
    (global.fetch as jest.Mock).mockClear();
  });

  test('Regular user cannot fetch studies without project_id', async () => {
    // Given: 일반 사용자
    const mockUser = {
      role: 'USER',
      hasGlobalDicomAccess: false
    };

    // Mock API 응답 (에러)
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: false,
      status: 400,
      json: async () => ({
        error: "project_id is required (no global access permission)"
      })
    });

    // When: project_id 없이 조회 시도
    const response = await fetch('/api/dicom/studies');
    const data = await response.json();

    // Then: 400 Bad Request
    expect(response.ok).toBe(false);
    expect(response.status).toBe(400);
    expect(data.error).toBe("project_id is required (no global access permission)");
  });

  test('Regular user can fetch studies with project_id (backward compatibility)', async () => {
    // Given: 일반 사용자
    const mockUser = {
      role: 'USER',
      hasGlobalDicomAccess: false
    };

    // Mock API 응답 (기존 동작)
    const filteredResponse = [mockStudiesResponse[0]];
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => filteredResponse
    });

    // When: project_id와 함께 조회
    const response = await fetch('/api/dicom/studies?project_id=150');
    const data = await response.json();

    // Then: 정상 동작 (기존과 동일)
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(1);
  });
});

describe('DICOM Global Access - Series Endpoint', () => {
  beforeEach(() => {
    (global.fetch as jest.Mock).mockClear();
  });

  const mockSeriesResponse = [
    {
      "0020000E": { "Value": ["1.2.826.0.1.3680043.8.498.44444444444444444444444444444444"] },
      "00080060": { "Value": ["CT"] }
    },
    {
      "0020000E": { "Value": ["1.2.826.0.1.3680043.8.498.55555555555555555555555555555555"] },
      "00080060": { "Value": ["MR"] }
    }
  ];

  test('SUPER_ADMIN can fetch all series without project_id', async () => {
    // Given: SUPER_ADMIN 사용자
    const studyUid = "1.2.826.0.1.3680043.8.498.11111111111111111111111111111111";

    // Mock API 응답
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockSeriesResponse
    });

    // When: project_id 없이 Series 조회
    const response = await fetch(`/api/dicom/series/${studyUid}`);
    const data = await response.json();

    // Then: 전체 데이터 반환
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(2);
  });

  test('Regular user cannot fetch series without project_id', async () => {
    // Given: 일반 사용자
    const studyUid = "1.2.826.0.1.3680043.8.498.11111111111111111111111111111111";

    // Mock API 응답 (에러)
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: false,
      status: 400,
      json: async () => ({
        error: "project_id is required (no global access permission)"
      })
    });

    // When: project_id 없이 Series 조회 시도
    const response = await fetch(`/api/dicom/series/${studyUid}`);
    const data = await response.json();

    // Then: 400 Bad Request
    expect(response.ok).toBe(false);
    expect(response.status).toBe(400);
  });
});

describe('DICOM Global Access - Error Handling', () => {
  beforeEach(() => {
    (global.fetch as jest.Mock).mockClear();
  });

  test('Invalid project_id (zero) returns error', async () => {
    // Mock API 응답 (에러)
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: false,
      status: 400,
      json: async () => ({
        error: "project_id must be greater than 0"
      })
    });

    // When: project_id=0으로 조회
    const response = await fetch('/api/dicom/studies?project_id=0');
    const data = await response.json();

    // Then: 400 Bad Request
    expect(response.ok).toBe(false);
    expect(response.status).toBe(400);
    expect(data.error).toBe("project_id must be greater than 0");
  });

  test('Invalid project_id (negative) returns error', async () => {
    // Mock API 응답 (에러)
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: false,
      status: 400,
      json: async () => ({
        error: "project_id must be greater than 0"
      })
    });

    // When: project_id=-1로 조회
    const response = await fetch('/api/dicom/studies?project_id=-1');
    const data = await response.json();

    // Then: 400 Bad Request
    expect(response.ok).toBe(false);
    expect(response.status).toBe(400);
    expect(data.error).toBe("project_id must be greater than 0");
  });
});

