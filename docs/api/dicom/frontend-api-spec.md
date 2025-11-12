# DICOM API 명세 (프론트엔드용)

**Base URL**: `http://localhost:8080/api`

**작성일**: 2025-11-11  
**대상**: 프론트엔드 개발자

---

## 📋 목차

1. [인증](#1-인증)
2. [DICOM 데이터 조회](#2-dicom-데이터-조회)
3. [프로젝트 데이터 할당](#3-프로젝트-데이터-할당)
4. [에러 처리](#4-에러-처리)

---

## 1. 인증

### 1.1 Keycloak 토큰 획득 (CORS 우회용)

브라우저에서 Keycloak으로 직접 요청하면 CORS 에러가 발생하므로, 백엔드 프록시를 통해 토큰을 획득합니다.

#### Endpoint

```http
POST /api/auth/keycloak-token
```

#### Request Body

```json
{
  "username": "test_super_admin",
  "password": "TestAdmin123!"
}
```

#### Response (200 OK)

```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Ii...",
  "expires_in": 300,
  "refresh_expires_in": 1800,
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6IC...",
  "token_type": "Bearer"
}
```

#### JavaScript 예제

```javascript
const response = await axios.post(`${apiUrl}/auth/keycloak-token`, {
  username: 'test_super_admin',
  password: 'TestAdmin123!'
});

const token = response.data.access_token;

// 이후 모든 DICOM API 요청에 사용
const config = {
  headers: {
    'Authorization': `Bearer ${token}`
  }
};
```

---

## 2. DICOM 데이터 조회

모든 DICOM 조회 API는 **Keycloak Bearer Token**이 필요합니다.

### 2.1 Studies 조회

#### Endpoint

```http
GET /api/dicom/studies
```

#### Query Parameters

| 파라미터 | 타입 | 필수 | 설명 | 예시 |
|---------|------|------|------|------|
| `project_id` | integer | ❌ | 프로젝트 ID (없으면 전체 조회, SUPER_ADMIN/ADMIN만 가능) | `150` |
| `check_assignment_for_project` | integer | ❌ | 할당 여부 확인할 프로젝트 ID | `150` |
| `modality` | string | ❌ | 모달리티 필터 | `CT`, `MR` |
| `patient_id` | string | ❌ | 환자 ID 필터 | `P12345` |
| `study_date` | string | ❌ | Study 날짜 필터 (YYYYMMDD 또는 YYYYMMDD-YYYYMMDD) | `20240101-20241231` |
| `accession_number` | string | ❌ | Accession Number 필터 | `ACC-123` |
| `patient_name` | string | ❌ | 환자 이름 필터 (부분 일치) | `홍길동` |
| `page` | integer | ❌ | 페이지 번호 (1부터 시작, 기본값: 1) | `1` |
| `page_size` | integer | ❌ | 페이지 크기 (기본값: 50, 최대: 200) | `50` |

#### 권한 요구사항

- **전체 조회 (project_id 없음)**: `DICOM_GLOBAL_ACCESS` 권한 필요 (SUPER_ADMIN, ADMIN)
- **프로젝트별 조회 (project_id 있음)**: 해당 프로젝트 멤버여야 함

#### Response (200 OK)

DICOM QIDO-RS 표준 JSON 배열을 반환합니다.

**기본 응답 (check_assignment_for_project 없음)**:

```json
[
  {
    "0020000D": {
      "vr": "UI",
      "Value": ["1.2.840.113619.2.55.3.604688119.868.1234567890.1"]
    },
    "00100020": {
      "vr": "LO",
      "Value": ["P12345"]
    },
    "00100010": {
      "vr": "PN",
      "Value": [{"Alphabetic": "홍길동"}]
    },
    "00080020": {
      "vr": "DA",
      "Value": ["20241101"]
    },
    "00080060": {
      "vr": "CS",
      "Value": ["CT"]
    }
  }
]
```

**할당 여부 확인 응답 (check_assignment_for_project 있음)**:

```json
[
  {
    "0020000D": {
      "vr": "UI",
      "Value": ["1.2.840.113619.2.55.3.604688119.868.1234567890.1"]
    },
    "00100020": {
      "vr": "LO",
      "Value": ["P12345"]
    },
    "is_assigned": false,
    "checked_project_id": 150
  }
]
```

#### JavaScript 예제

```javascript
// 1. 전체 조회 (SUPER_ADMIN/ADMIN만 가능)
const response = await axios.get(`${apiUrl}/dicom/studies`, {
  headers: {
    'Authorization': `Bearer ${keycloakToken}`
  }
});

// 2. 프로젝트별 조회
const response = await axios.get(`${apiUrl}/dicom/studies?project_id=150`, {
  headers: {
    'Authorization': `Bearer ${keycloakToken}`
  }
});

// 3. 할당 여부 확인
const response = await axios.get(
  `${apiUrl}/dicom/studies?check_assignment_for_project=150`,
  {
    headers: {
      'Authorization': `Bearer ${keycloakToken}`
    }
  }
);

// 4. 필터링 + 페이지네이션
const response = await axios.get(
  `${apiUrl}/dicom/studies?project_id=150&modality=CT&page=1&page_size=20`,
  {
    headers: {
      'Authorization': `Bearer ${keycloakToken}`
    }
  }
);
```

### 2.2 Series 조회

#### Endpoint

```http
GET /api/dicom/studies/{study_uid}/series
```

#### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `study_uid` | string | ✅ | Study Instance UID |

#### Query Parameters

Studies 조회와 동일한 파라미터를 지원합니다 (`project_id`, `modality`, `page`, `page_size` 등).

#### Response (200 OK)

```json
[
  {
    "0020000E": {
      "vr": "UI",
      "Value": ["1.2.840.113619.2.55.3.604688119.868.1234567890.2"]
    },
    "00080060": {
      "vr": "CS",
      "Value": ["CT"]
    },
    "0008103E": {
      "vr": "LO",
      "Value": ["Chest CT"]
    },
    "00200011": {
      "vr": "IS",
      "Value": ["1"]
    }
  }
]
```

#### JavaScript 예제

```javascript
// Study UID를 먼저 조회한 후 Series 조회
const studiesResponse = await axios.get(
  `${apiUrl}/dicom/studies?project_id=150&limit=1`,
  { headers: { 'Authorization': `Bearer ${token}` } }
);

const studyUid = studiesResponse.data[0]['0020000D'].Value[0];

const seriesResponse = await axios.get(
  `${apiUrl}/dicom/studies/${studyUid}/series?project_id=150`,
  { headers: { 'Authorization': `Bearer ${token}` } }
);
```

### 2.3 Instances 조회

#### Endpoint

```http
GET /api/dicom/studies/{study_uid}/series/{series_uid}/instances
```

#### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `study_uid` | string | ✅ | Study Instance UID |
| `series_uid` | string | ✅ | Series Instance UID |

#### Query Parameters

Studies 조회와 동일한 파라미터를 지원합니다.

#### Response (200 OK)

```json
[
  {
    "00080018": {
      "vr": "UI",
      "Value": ["1.2.840.113619.2.55.3.604688119.868.1234567890.3"]
    },
    "00200013": {
      "vr": "IS",
      "Value": ["1"]
    }
  }
]
```

#### JavaScript 예제

```javascript
const instancesResponse = await axios.get(
  `${apiUrl}/dicom/studies/${studyUid}/series/${seriesUid}/instances?project_id=150`,
  { headers: { 'Authorization': `Bearer ${token}` } }
);
```

---

## 3. 프로젝트 데이터 할당

### 3.1 Study 할당

프로젝트에 DICOM Study를 할당합니다.

#### Endpoint

```http
POST /api/projects/{project_id}/studies/assign
```

#### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |

#### Request Body

```json
{
  "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
  "study_description": "Brain MRI Study",
  "patient_id": "P12345",
  "patient_name": "홍길동",
  "patient_birth_date": "1990-01-01",
  "study_date": "2024-12-01",
  "modality": "MR"
}
```

| 필드 | 타입 | 필수 | 설명 |
|------|------|------|------|
| `study_uid` | string | ✅ | Study Instance UID |
| `study_description` | string | ❌ | Study 설명 |
| `patient_id` | string | ❌ | 환자 ID |
| `patient_name` | string | ❌ | 환자 이름 |
| `patient_birth_date` | string | ❌ | 환자 생년월일 (YYYY-MM-DD) |
| `study_date` | string | ❌ | Study 날짜 (YYYY-MM-DD) |
| `modality` | string | ❌ | 모달리티 |

#### Response (200 OK)

```json
{
  "success": true,
  "message": "Study 1.2.840.113619.2.55.3.604688119.868.1234567890.1 assigned to project successfully",
  "study_id": 123
}
```

#### JavaScript 예제

```javascript
const response = await axios.post(
  `${apiUrl}/projects/150/studies/assign`,
  {
    study_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
    study_description: "Brain MRI Study",
    patient_id: "P12345",
    patient_name: "홍길동",
    study_date: "2024-12-01"
  },
  {
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    }
  }
);
```

### 3.2 Series 할당

프로젝트에 DICOM Series를 할당합니다. 부모 Study가 없으면 자동으로 생성됩니다.

#### Endpoint

```http
POST /api/projects/{project_id}/series/assign
```

#### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |

#### Request Body

```json
{
  "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
  "series_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
  "series_description": "T1 Axial",
  "modality": "MR",
  "series_number": 1
}
```

| 필드 | 타입 | 필수 | 설명 |
|------|------|------|------|
| `study_uid` | string | ✅ | 부모 Study Instance UID |
| `series_uid` | string | ✅ | Series Instance UID |
| `series_description` | string | ❌ | Series 설명 |
| `modality` | string | ❌ | 모달리티 |
| `series_number` | integer | ❌ | Series 번호 |

#### Response (200 OK)

```json
{
  "success": true,
  "message": "Series 1.2.840.113619.2.55.3.604688119.868.1234567890.2 assigned to project successfully",
  "series_id": 456
}
```

#### JavaScript 예제

```javascript
const response = await axios.post(
  `${apiUrl}/projects/150/series/assign`,
  {
    study_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
    series_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
    series_description: "T1 Axial",
    modality: "MR",
    series_number: 1
  },
  {
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    }
  }
);
```

---

## 4. 에러 처리

### 4.1 HTTP 상태 코드

| 상태 코드 | 설명 |
|----------|------|
| `200 OK` | 요청 성공 |
| `400 Bad Request` | 잘못된 요청 (파라미터 오류, 검증 실패) |
| `401 Unauthorized` | 인증 실패 (토큰 없음, 만료, 유효하지 않음) |
| `403 Forbidden` | 권한 없음 (DICOM_GLOBAL_ACCESS 권한 필요) |
| `404 Not Found` | 리소스를 찾을 수 없음 (프로젝트, Study 등) |
| `409 Conflict` | 중복 (이미 할당된 Study/Series) |
| `500 Internal Server Error` | 서버 내부 오류 |
| `502 Bad Gateway` | DCM4CHEE QIDO 요청 실패 |

### 4.2 에러 응답 형식

#### 일반 에러

```json
{
  "error": "Invalid or missing authorization token"
}
```

#### 권한 에러

```json
{
  "error": "Forbidden: DICOM_GLOBAL_ACCESS permission required for global access"
}
```

#### DCM4CHEE 에러

```json
{
  "error": "External service error: QIDO /studies failed (400 Bad Request): {...}"
}
```

#### 할당 에러

```json
{
  "success": false,
  "error": "STUDY_ALREADY_ASSIGNED",
  "message": "Study already assigned to this project"
}
```

### 4.3 JavaScript 에러 처리 예제

```javascript
try {
  const response = await axios.get(`${apiUrl}/dicom/studies`, {
    headers: {
      'Authorization': `Bearer ${token}`
    }
  });

  console.log('Studies:', response.data);

} catch (error) {
  if (error.response) {
    // 서버가 응답을 반환한 경우
    const status = error.response.status;
    const data = error.response.data;

    switch (status) {
      case 401:
        console.error('인증 실패:', data.error);
        // 토큰 재발급 또는 로그인 페이지로 이동
        break;

      case 403:
        console.error('권한 없음:', data.error);
        // 권한 없음 메시지 표시
        break;

      case 502:
        console.error('DCM4CHEE 연결 실패:', data.error);
        // 서버 오류 메시지 표시
        break;

      default:
        console.error('에러:', data.error || data);
    }
  } else if (error.request) {
    // 요청은 보냈지만 응답을 받지 못한 경우
    console.error('네트워크 오류: 서버 응답 없음');
  } else {
    // 요청 설정 중 오류 발생
    console.error('요청 오류:', error.message);
  }
}
```

---

## 5. 주요 사용 시나리오

### 5.1 전체 워크플로우

```javascript
// 1. Keycloak 토큰 획득
const authResponse = await axios.post(`${apiUrl}/auth/keycloak-token`, {
  username: 'test_super_admin',
  password: 'TestAdmin123!'
});

const token = authResponse.data.access_token;

// 2. Studies 조회 (전체)
const studiesResponse = await axios.get(`${apiUrl}/dicom/studies`, {
  headers: { 'Authorization': `Bearer ${token}` }
});

// 3. 첫 번째 Study의 Series 조회
const studyUid = studiesResponse.data[0]['0020000D'].Value[0];
const seriesResponse = await axios.get(
  `${apiUrl}/dicom/studies/${studyUid}/series`,
  { headers: { 'Authorization': `Bearer ${token}` } }
);

// 4. Study를 프로젝트에 할당
const assignResponse = await axios.post(
  `${apiUrl}/projects/150/studies/assign`,
  {
    study_uid: studyUid,
    study_description: "CT Chest",
    patient_id: "P12345"
  },
  { headers: { 'Authorization': `Bearer ${token}` } }
);

// 5. 할당 여부 확인
const checkResponse = await axios.get(
  `${apiUrl}/dicom/studies?check_assignment_for_project=150`,
  { headers: { 'Authorization': `Bearer ${token}` } }
);

console.log('Is assigned:', checkResponse.data[0].is_assigned);
```

### 5.2 프로젝트별 조회

```javascript
// 특정 프로젝트의 Studies만 조회
const response = await axios.get(
  `${apiUrl}/dicom/studies?project_id=150&modality=CT&page=1&page_size=20`,
  { headers: { 'Authorization': `Bearer ${token}` } }
);
```

### 5.3 할당 여부 확인

```javascript
// 전체 Studies를 조회하면서 특정 프로젝트에 할당되었는지 확인
const response = await axios.get(
  `${apiUrl}/dicom/studies?check_assignment_for_project=150`,
  { headers: { 'Authorization': `Bearer ${token}` } }
);

// 각 Study에 is_assigned, checked_project_id 필드가 추가됨
response.data.forEach(study => {
  const studyUid = study['0020000D'].Value[0];
  console.log(`Study ${studyUid}: ${study.is_assigned ? '할당됨' : '미할당'}`);
});
```

---

## 6. DICOM 태그 참조

QIDO-RS 응답에서 자주 사용되는 DICOM 태그:

| 태그 | 이름 | 설명 | 예시 |
|------|------|------|------|
| `0020000D` | StudyInstanceUID | Study 고유 식별자 | `1.2.840.113619...` |
| `0020000E` | SeriesInstanceUID | Series 고유 식별자 | `1.2.840.113619...` |
| `00080018` | SOPInstanceUID | Instance 고유 식별자 | `1.2.840.113619...` |
| `00100020` | PatientID | 환자 ID | `P12345` |
| `00100010` | PatientName | 환자 이름 | `{"Alphabetic": "홍길동"}` |
| `00080020` | StudyDate | Study 날짜 | `20241101` |
| `00080060` | Modality | 모달리티 | `CT`, `MR`, `CR` |
| `0008103E` | SeriesDescription | Series 설명 | `Chest CT` |
| `00200011` | SeriesNumber | Series 번호 | `1` |
| `00200013` | InstanceNumber | Instance 번호 | `1` |

### DICOM 태그 값 추출 예제

```javascript
// Study UID 추출
const studyUid = study['0020000D'].Value[0];

// Patient ID 추출
const patientId = study['00100020'].Value[0];

// Patient Name 추출 (PN 타입은 객체)
const patientName = study['00100010'].Value[0].Alphabetic;

// Study Date 추출
const studyDate = study['00080020'].Value[0];

// Modality 추출
const modality = study['00080060'].Value[0];
```

---

## 7. 테스트 계정

개발/테스트용 계정:

| Username | Password | Role | 설명 |
|----------|----------|------|------|
| `test_super_admin` | `TestAdmin123!` | SUPER_ADMIN | 전체 데이터 조회 가능 |
| `test_admin` | `TestAdmin123!` | ADMIN | 전체 데이터 조회 가능 |
| `test_user` | `TestUser123!` | USER | 프로젝트별 조회만 가능 |

---

## 8. 참고 사항

### 8.1 권한 시스템

- **DICOM_GLOBAL_ACCESS**: SUPER_ADMIN, ADMIN 역할에 부여됨
  - `project_id` 없이 전체 DICOM 데이터 조회 가능
  - 모든 프로젝트의 데이터 할당 가능

- **일반 사용자**: 프로젝트 멤버십 필요
  - `project_id` 파라미터 필수
  - 자신이 속한 프로젝트의 데이터만 조회 가능

### 8.2 페이지네이션

- `page`: 1부터 시작 (기본값: 1)
- `page_size`: 1~200 (기본값: 50)
- 내부적으로 `offset = (page - 1) * page_size`, `limit = page_size`로 변환됨

### 8.3 필터링

- 모든 필터는 선택적(optional)
- 여러 필터를 동시에 사용 가능
- `study_date` 형식:
  - 단일 날짜: `20240101`
  - 범위: `20240101-20241231`

### 8.4 할당 여부 확인

- `check_assignment_for_project` 파라미터는 `project_id`와 독립적
- 전체 조회 + 할당 여부 확인 가능: `?check_assignment_for_project=150`
- 프로젝트별 조회 + 할당 여부 확인 가능: `?project_id=100&check_assignment_for_project=150`
- 응답에 `is_assigned` (boolean), `checked_project_id` (integer) 필드 추가됨

---

## 9. 문의

API 관련 문의사항은 백엔드 팀에 문의해주세요.

**문서 버전**: 1.0
**최종 수정일**: 2025-11-11


