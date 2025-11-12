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

describe('DICOM Assignment Status - check_assignment_for_project', () => {
  beforeEach(() => {
    (global.fetch as jest.Mock).mockClear();
  });

  const mockStudiesWithAssignment = [
    {
      "0020000D": { "Value": ["1.2.826.0.1.3680043.8.498.11111111111111111111111111111111"] },
      "00100020": { "Value": ["PATIENT001"] },
      "00080060": { "Value": ["CT"] },
      "is_assigned": true,
      "checked_project_id": 150
    },
    {
      "0020000D": { "Value": ["1.2.826.0.1.3680043.8.498.22222222222222222222222222222222"] },
      "00100020": { "Value": ["PATIENT002"] },
      "00080060": { "Value": ["MR"] },
      "is_assigned": false,
      "checked_project_id": 150
    },
    {
      "0020000D": { "Value": ["1.2.826.0.1.3680043.8.498.33333333333333333333333333333333"] },
      "00100020": { "Value": ["PATIENT003"] },
      "00080060": { "Value": ["CT"] },
      "is_assigned": true,
      "checked_project_id": 150
    }
  ];

  test('Check assignment status with check_assignment_for_project parameter', async () => {
    // Given: check_assignment_for_project 파라미터 사용
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockStudiesWithAssignment
    });

    // When: check_assignment_for_project=150으로 조회
    const response = await fetch('/api/dicom/studies?check_assignment_for_project=150');
    const data = await response.json();

    // Then: 할당 여부 필드 포함
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(3);
    expect(data[0].is_assigned).toBe(true);
    expect(data[0].checked_project_id).toBe(150);
    expect(data[1].is_assigned).toBe(false);
    expect(data[1].checked_project_id).toBe(150);
  });

  test('Check assignment with both project_id and check_assignment_for_project', async () => {
    // Given: project_id와 check_assignment_for_project 모두 사용
    const filteredWithAssignment = [mockStudiesWithAssignment[0]];
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => filteredWithAssignment
    });

    // When: project_id=150&check_assignment_for_project=150으로 조회
    const response = await fetch('/api/dicom/studies?project_id=150&check_assignment_for_project=150');
    const data = await response.json();

    // Then: 필터링 + 할당 여부 확인
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(1);
    expect(data[0].is_assigned).toBe(true);
    expect(data[0].checked_project_id).toBe(150);
  });

  test('Check assignment without filtering (SUPER_ADMIN)', async () => {
    // Given: SUPER_ADMIN이 전체 데이터 + 할당 여부 확인
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockStudiesWithAssignment
    });

    // When: check_assignment_for_project만 사용 (project_id 없음)
    const response = await fetch('/api/dicom/studies?check_assignment_for_project=150');
    const data = await response.json();

    // Then: 전체 데이터 + 할당 여부
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(3);
    expect(data.every((study: any) => study.hasOwnProperty('is_assigned'))).toBe(true);
    expect(data.every((study: any) => study.checked_project_id === 150)).toBe(true);
  });

  test('Series endpoint with assignment status', async () => {
    // Given: Series 조회 + 할당 여부 확인
    const mockSeriesWithAssignment = [
      {
        "0020000E": { "Value": ["1.2.826.0.1.3680043.8.498.44444444444444444444444444444444"] },
        "00080060": { "Value": ["CT"] },
        "is_assigned": true,
        "checked_project_id": 150
      },
      {
        "0020000E": { "Value": ["1.2.826.0.1.3680043.8.498.55555555555555555555555555555555"] },
        "00080060": { "Value": ["MR"] },
        "is_assigned": false,
        "checked_project_id": 150
      }
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockSeriesWithAssignment
    });

    const studyUid = "1.2.826.0.1.3680043.8.498.11111111111111111111111111111111";

    // When: Series 조회 + check_assignment_for_project
    const response = await fetch(`/api/dicom/series/${studyUid}?check_assignment_for_project=150`);
    const data = await response.json();

    // Then: Series + 할당 여부
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(2);
    expect(data[0].is_assigned).toBe(true);
    expect(data[1].is_assigned).toBe(false);
  });
});

describe('DICOM Complete Scenarios - project_id Optional + READ_ALL + Assignment Check', () => {
  beforeEach(() => {
    (global.fetch as jest.Mock).mockClear();
  });

  test('Scenario 1: SUPER_ADMIN views all data with assignment status', async () => {
    // Given: SUPER_ADMIN 사용자
    const mockResponse = [
      {
        "0020000D": { "Value": ["1.2.826.0.1.3680043.8.498.11111111111111111111111111111111"] },
        "00100020": { "Value": ["PATIENT001"] },
        "00080060": { "Value": ["CT"] },
        "is_assigned": true,
        "checked_project_id": 150
      },
      {
        "0020000D": { "Value": ["1.2.826.0.1.3680043.8.498.22222222222222222222222222222222"] },
        "00100020": { "Value": ["PATIENT002"] },
        "00080060": { "Value": ["MR"] },
        "is_assigned": false,
        "checked_project_id": 150
      }
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    });

    // When: project_id 없이 전체 데이터 + 할당 여부 확인
    const response = await fetch('/api/dicom/studies?check_assignment_for_project=150');
    const data = await response.json();

    // Then: 전체 데이터 + 할당 여부 표시
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(2);
    expect(data[0].is_assigned).toBe(true);
    expect(data[1].is_assigned).toBe(false);
  });

  test('Scenario 2: ADMIN filters by project and checks assignment', async () => {
    // Given: ADMIN 사용자
    const mockResponse = [
      {
        "0020000D": { "Value": ["1.2.826.0.1.3680043.8.498.11111111111111111111111111111111"] },
        "00100020": { "Value": ["PATIENT001"] },
        "00080060": { "Value": ["CT"] },
        "is_assigned": true,
        "checked_project_id": 150
      }
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    });

    // When: project_id로 필터링 + 할당 여부 확인
    const response = await fetch('/api/dicom/studies?project_id=150&check_assignment_for_project=150');
    const data = await response.json();

    // Then: 필터링된 데이터 + 할당 여부
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(1);
    expect(data[0].is_assigned).toBe(true);
  });

  test('Scenario 3: Regular user must provide project_id', async () => {
    // Given: 일반 사용자 (전체 권한 없음)
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: false,
      status: 400,
      json: async () => ({
        error: "project_id is required (no global access permission)"
      })
    });

    // When: project_id 없이 조회 시도
    const response = await fetch('/api/dicom/studies?check_assignment_for_project=150');
    const data = await response.json();

    // Then: 400 Bad Request
    expect(response.ok).toBe(false);
    expect(response.status).toBe(400);
    expect(data.error).toBe("project_id is required (no global access permission)");
  });

  test('Scenario 4: Regular user can check assignment with project_id', async () => {
    // Given: 일반 사용자
    const mockResponse = [
      {
        "0020000D": { "Value": ["1.2.826.0.1.3680043.8.498.11111111111111111111111111111111"] },
        "00100020": { "Value": ["PATIENT001"] },
        "00080060": { "Value": ["CT"] },
        "is_assigned": true,
        "checked_project_id": 150
      }
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    });

    // When: project_id + check_assignment_for_project 함께 사용
    const response = await fetch('/api/dicom/studies?project_id=150&check_assignment_for_project=150');
    const data = await response.json();

    // Then: 정상 동작 (필터링 + 할당 여부)
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(1);
    expect(data[0].is_assigned).toBe(true);
  });

  test('Scenario 5: SUPER_ADMIN views all data without any parameters', async () => {
    // Given: SUPER_ADMIN 사용자
    const mockResponse = [
      {
        "0020000D": { "Value": ["1.2.826.0.1.3680043.8.498.11111111111111111111111111111111"] },
        "00100020": { "Value": ["PATIENT001"] },
        "00080060": { "Value": ["CT"] }
      },
      {
        "0020000D": { "Value": ["1.2.826.0.1.3680043.8.498.22222222222222222222222222222222"] },
        "00100020": { "Value": ["PATIENT002"] },
        "00080060": { "Value": ["MR"] }
      }
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    });

    // When: 파라미터 없이 전체 데이터 조회
    const response = await fetch('/api/dicom/studies');
    const data = await response.json();

    // Then: 전체 데이터 반환 (할당 여부 필드 없음)
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(2);
    expect(data[0].hasOwnProperty('is_assigned')).toBe(false);
  });

  test('Scenario 6: Check assignment for different project than filter', async () => {
    // Given: project_id와 check_assignment_for_project가 다른 경우
    const mockResponse = [
      {
        "0020000D": { "Value": ["1.2.826.0.1.3680043.8.498.11111111111111111111111111111111"] },
        "00100020": { "Value": ["PATIENT001"] },
        "00080060": { "Value": ["CT"] },
        "is_assigned": false,  // project 200에는 할당 안 됨
        "checked_project_id": 200
      }
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    });

    // When: project_id=150으로 필터링, project 200 할당 여부 확인
    const response = await fetch('/api/dicom/studies?project_id=150&check_assignment_for_project=200');
    const data = await response.json();

    // Then: project 150 데이터 중 project 200에 할당 안 된 데이터
    expect(response.ok).toBe(true);
    expect(data).toHaveLength(1);
    expect(data[0].is_assigned).toBe(false);
    expect(data[0].checked_project_id).toBe(200);
  });
});

