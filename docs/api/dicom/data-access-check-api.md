# DICOM Data Access Check API

**버전**: 1.0.0  
**작성일**: 2026-01-22  
**대상**: 프론트엔드 개발자, 백엔드 개발자

---

## 📋 목차

1. [개요](#1-개요)
2. [API 엔드포인트](#2-api-엔드포인트)
3. [요청/응답 형식](#3-요청응답-형식)
4. [사용 예시](#4-사용-예시)
5. [에러 처리](#5-에러-처리)
6. [권한 확인 로직](#6-권한-확인-로직)

---

## 1. 개요

DICOM Data Access Check API는 사용자가 특정 Study 또는 Series 데이터에 접근 가능한지 확인하는 API입니다.

### 주요 기능

- ✅ **Study 레벨 접근 권한 확인**: 사용자가 특정 Study에 접근 가능한지 확인
- ✅ **Series 레벨 접근 권한 확인**: 사용자가 특정 Series에 접근 가능한지 확인
- ✅ **프로젝트별 접근 권한 확인**: 특정 프로젝트에 대한 접근 권한만 확인 가능
- ✅ **다중 프로젝트 지원**: 사용자가 속한 모든 프로젝트에서 접근 권한 확인
- ✅ **RBAC 기반 권한 평가**: 역할 기반 접근 제어 적용

### 사용 사례

1. **뷰어 접근 전 권한 확인**: DICOM 뷰어에서 Study/Series를 열기 전에 접근 권한 확인
2. **데이터 다운로드 권한 확인**: DICOM 데이터 다운로드 전에 권한 확인
3. **프로젝트별 데이터 필터링**: 특정 프로젝트에서 접근 가능한 데이터만 표시
4. **권한 기반 UI 렌더링**: 접근 가능한 데이터에 대해서만 버튼/링크 활성화

---

## 2. API 엔드포인트

### POST /api/v1/dicom/access/check

사용자가 특정 Study/Series에 접근 가능한지 확인합니다.

**Base URL**: `http://localhost:8080` (개발), `https://api.pacs.ai-do.co.kr` (프로덕션)

**인증**: Bearer Token (JWT) 필수

**Content-Type**: `application/json`

---

## 3. 요청/응답 형식

### 요청 (Request)

#### Headers

```http
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json
```

#### Request Body

```json
{
  "study_uid": "1.2.410.200022.500.12252244129",
  "series_uid": "1.2.410.200022.500.12252244130",  // 선택 (Optional)
  "project_id": 2                                   // 선택 (Optional)
}
```

#### 파라미터 설명

| 필드 | 타입 | 필수 | 설명 |
|------|------|------|------|
| `study_uid` | string | ✅ 필수 | DICOM Study Instance UID |
| `series_uid` | string | ⬜ 선택 | DICOM Series Instance UID (Series 레벨 권한 확인 시) |
| `project_id` | integer | ⬜ 선택 | 프로젝트 ID (특정 프로젝트에 대한 권한만 확인) |

### 응답 (Response)

#### 성공 응답 (200 OK)

```json
{
  "accessible": true,
  "projects": [
    {
      "project_id": 2,
      "project_name": "AI Image Analysis Project",
      "access_level": "STUDY",
      "reason": "approved"
    }
  ]
}
```

#### 응답 필드 설명

| 필드 | 타입 | 설명 |
|------|------|------|
| `accessible` | boolean | 접근 가능 여부 (프로젝트 목록이 비어있으면 `false`) |
| `projects` | array | 접근 가능한 프로젝트 목록 |
| `projects[].project_id` | integer | 프로젝트 ID |
| `projects[].project_name` | string | 프로젝트 이름 |
| `projects[].access_level` | string | 접근 레벨 (`STUDY` 또는 `SERIES`) |
| `projects[].reason` | string | 접근 승인 사유 (`approved`, `member`, `denied`) |

#### 접근 레벨 (access_level)

- `STUDY`: Study 전체에 대한 접근 권한
- `SERIES`: 특정 Series에 대한 접근 권한

#### 접근 사유 (reason)

- `approved`: `project_data_access` 테이블에서 명시적으로 승인됨
- `member`: 프로젝트 멤버이며 RBAC 규칙에 의해 접근 가능
- `denied`: 접근 거부 (응답에 포함되지 않음)

---

## 4. 사용 예시

### 예시 1: Study 접근 권한 확인 (모든 프로젝트)

사용자가 속한 모든 프로젝트에서 특정 Study에 접근 가능한지 확인합니다.

**Request**:
```bash
curl -X POST http://localhost:8080/api/v1/dicom/access/check \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "1.2.410.200022.500.12252244129"
  }'
```

**Response**:
```json
{
  "accessible": true,
  "projects": [
    {
      "project_id": 2,
      "project_name": "AI Image Analysis Project",
      "access_level": "STUDY",
      "reason": "approved"
    },
    {
      "project_id": 5,
      "project_name": "Clinical Trial Project",
      "access_level": "STUDY",
      "reason": "member"
    }
  ]
}
```

### 예시 2: 특정 프로젝트에서 Study 접근 권한 확인

특정 프로젝트에서만 Study 접근 권한을 확인합니다.

**Request**:
```bash
curl -X POST http://localhost:8080/api/v1/dicom/access/check \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "1.2.410.200022.500.12252244129",
    "project_id": 2
  }'
```

**Response**:
```json
{
  "accessible": true,
  "projects": [
    {
      "project_id": 2,
      "project_name": "AI Image Analysis Project",
      "access_level": "STUDY",
      "reason": "approved"
    }
  ]
}
```

### 예시 3: Series 접근 권한 확인

특정 Series에 대한 접근 권한을 확인합니다.

**Request**:
```bash
curl -X POST http://localhost:8080/api/v1/dicom/access/check \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "1.2.410.200022.500.12252244129",
    "series_uid": "1.2.410.200022.500.12252244130"
  }'
```

**Response**:
```json
{
  "accessible": true,
  "projects": [
    {
      "project_id": 2,
      "project_name": "AI Image Analysis Project",
      "access_level": "SERIES",
      "reason": "approved"
    }
  ]
}
```

### 예시 4: 접근 불가능한 Study

사용자가 접근할 수 없는 Study를 확인하면 빈 프로젝트 목록이 반환됩니다.

**Request**:
```bash
curl -X POST http://localhost:8080/api/v1/dicom/access/check \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9"
  }'
```

**Response**:
```json
{
  "accessible": false,
  "projects": []
}
```

### 예시 5: JavaScript/TypeScript 사용 예시

```typescript
interface DataAccessCheckRequest {
  study_uid: string;
  series_uid?: string;
  project_id?: number;
}

interface ProjectAccessInfo {
  project_id: number;
  project_name: string;
  access_level: 'STUDY' | 'SERIES';
  reason: 'approved' | 'member' | 'denied';
}

interface DataAccessCheckResponse {
  accessible: boolean;
  projects: ProjectAccessInfo[];
}

async function checkDataAccess(
  token: string,
  request: DataAccessCheckRequest
): Promise<DataAccessCheckResponse> {
  const response = await fetch('http://localhost:8080/api/v1/dicom/access/check', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  return await response.json();
}

// 사용 예시
const result = await checkDataAccess(token, {
  study_uid: '1.2.410.200022.500.12252244129',
  project_id: 2,
});

if (result.accessible) {
  console.log('접근 가능한 프로젝트:', result.projects);
  // 뷰어 열기 또는 데이터 로드
} else {
  console.log('접근 불가능');
  // 에러 메시지 표시
}
```

---

## 5. 에러 처리

### 400 Bad Request

**원인**: 잘못된 요청 형식 또는 필수 파라미터 누락

**Response**:
```json
{
  "error": "Bad Request",
  "message": "study_uid is required and cannot be empty"
}
```

### 401 Unauthorized

**원인**: 인증 토큰이 없거나 유효하지 않음

**Response**:
```json
{
  "error": "Unauthorized",
  "message": "Invalid or missing authentication token"
}
```

**해결 방법**:
- Authorization 헤더에 유효한 JWT 토큰을 포함하세요
- 토큰이 만료된 경우 `/api/auth/refresh`로 토큰을 갱신하세요

### 500 Internal Server Error

**원인**: 서버 내부 오류

**Response**:
```json
{
  "error": "Internal Server Error",
  "message": "Failed to get user projects: database connection error"
}
```

**해결 방법**:
- 서버 로그를 확인하세요
- 데이터베이스 연결 상태를 확인하세요
- 관리자에게 문의하세요

---

## 6. 권한 확인 로직

### 접근 권한 확인 프로세스

```
1. 사용자 인증
   ↓
2. 프로젝트 멤버십 확인
   - project_id가 있으면: 해당 프로젝트의 멤버인지 확인
   - project_id가 없으면: 사용자가 속한 모든 프로젝트 조회
   ↓
3. 각 프로젝트에서 접근 권한 확인
   ↓
4. RBAC 평가 (DicomRbacEvaluator)
   - 사용자의 역할(Role) 확인
   - 역할에 할당된 권한(Permission) 확인
   - 접근 조건(AccessCondition) 평가
   ↓
5. project_data_access 테이블 확인
   - Study/Series가 프로젝트에 할당되어 있는지 확인
   - 접근 상태(APPROVED/DENIED/PENDING) 확인
   ↓
6. 접근 가능한 프로젝트 목록 반환
```

### 데이터베이스 테이블

#### security_user_project
사용자-프로젝트 멤버십 관리

```sql
SELECT project_id
FROM security_user_project
WHERE user_id = $1 AND project_id = $2
```

#### project_data_access
Study/Series 레벨 접근 권한 관리

```sql
SELECT access_level, access_status
FROM project_data_access
WHERE project_id = $1
  AND study_uid = $2
  AND (series_uid IS NULL OR series_uid = $3)
```

### 접근 레벨 우선순위

1. **SERIES 레벨**: 가장 세밀한 권한 (특정 Series만 접근 가능)
2. **STUDY 레벨**: Study 전체 접근 가능 (모든 Series 포함)

Series UID가 요청에 포함된 경우:
- Series 레벨 권한이 있으면 → `access_level: "SERIES"` 반환
- Series 레벨 권한이 없으면 → 접근 불가 (빈 목록 반환)

---

## 7. 참고 사항

### Study Instance UID

- DICOM 표준 태그 (0020,000D)
- 전역적으로 고유한 식별자
- 형식: `1.2.840.113619.2.55.3.604688119.868.1234567890.1`

### Series Instance UID

- DICOM 표준 태그 (0020,000E)
- Study 내에서 Series를 식별
- 형식: `1.2.840.113619.2.55.3.604688119.868.1234567890.2`

### 프로젝트 기반 접근 제어

- 모든 DICOM 데이터는 프로젝트에 할당됨
- 사용자는 프로젝트 멤버로 등록되어야 함
- 프로젝트 내에서 역할(Role)에 따라 권한이 부여됨

### RBAC (Role-Based Access Control)

- **Role**: 사용자에게 할당되는 역할 (예: Admin, Researcher, Viewer)
- **Permission**: 역할에 할당되는 권한 (예: READ_STUDY, WRITE_ANNOTATION)
- **AccessCondition**: 권한에 대한 조건 (예: Modality=CT, StudyDate>2024-01-01)

---

## 8. 변경 이력

| 날짜 | 버전 | 변경 내용 |
|------|------|----------|
| 2026-01-22 | 1.0.0 | 초기 문서 작성 |
| 2026-01-22 | 1.0.0 | `project_id` 파라미터 추가 |

---

## 9. 관련 API

- **[DICOM Gateway API](./dicom-gateway-api.md)**: DICOM 데이터 조회 API
- **[시리즈/인스턴스 조회 API](./시리즈-인스턴스-조회-API.md)**: Series 및 Instance 조회 API
- **[Project Data Access Matrix API](../project-data-access-matrix-api.md)**: 프로젝트 데이터 접근 관리 API

---

## 10. 문의 및 지원

- **문서**: [https://docs.pacs.ai-do.co.kr](https://docs.pacs.ai-do.co.kr)
- **이슈 트래커**: [GitHub Issues](https://github.com/pacs-server/issues)
- **이메일**: support@ai-do.co.kr


