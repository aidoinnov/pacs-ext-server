# DICOM API 문서

> **대상**: 클라이언트 개발팀  
> **버전**: 1.0  
> **최종 업데이트**: 2025-12-02

---

## 📋 목차

1. [개요](#1-개요)
2. [인증](#2-인증)
3. [환자 목록 조회 API](#3-환자-목록-조회-api)
4. [시리즈 목록 조회 API](#4-시리즈-목록-조회-api)
5. [에러 처리](#5-에러-처리)
6. [코드 예시](#6-코드-예시)

---

## 1. 개요

PACS Extension Server는 DICOM 이미지 메타데이터를 조회할 수 있는 RESTful API를 제공합니다.

### 기본 정보
- **Base URL**: `http://localhost:8080` (개발), `https://api.pacs.ai-do.kr` (프로덕션)
- **프로토콜**: HTTPS (프로덕션), HTTP (개발)
- **데이터 형식**: JSON (DICOM JSON 표준)
- **인증 방식**: Bearer Token (JWT)

### 주요 특징
- ✅ **RBAC 기반 권한 관리**: 프로젝트별 데이터 접근 제어
- ✅ **DICOM 표준 준수**: DICOMweb QIDO-RS 프록시
- ✅ **썸네일 URL 자동 생성**: Series 조회 시 WADO-RS 썸네일 URL 포함
- ✅ **페이지네이션 지원**: 대용량 데이터 효율적 조회

---

## 2. 인증

모든 API 요청은 JWT Bearer Token을 사용한 인증이 필요합니다.

### 로그인 (테스트용)

```http
POST /api/test/login
Content-Type: application/json

{
  "username": "test_super_admin",
  "password": "TestAdmin123!"
}
```

**응답:**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### 인증 헤더 사용

```http
Authorization: Bearer {access_token}
```

---

## 3. 환자 목록 조회 API

### 엔드포인트

```
GET /api/dicom/patients
```

### 요청 파라미터

| 파라미터 | 타입 | 필수 | 기본값 | 설명 |
|---------|------|------|--------|------|
| `project_id` | integer | 조건부* | - | 프로젝트 ID |
| `limit` | integer | 선택 | 100 | 결과 개수 제한 (최대 1000) |
| `offset` | integer | 선택 | 0 | 페이지네이션 오프셋 |

> **\* 조건부 필수**: 일반 사용자는 필수, SUPER_ADMIN은 선택

### 요청 예시

```http
GET /api/dicom/patients?project_id=2&limit=50&offset=0
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

### 응답 형식

**성공 (200 OK):**
```json
[
  {
    "00100020": {
      "Value": ["PATIENT_001"],
      "vr": "LO"
    },
    "00100010": {
      "Value": [
        {
          "Alphabetic": "홍길동"
        }
      ],
      "vr": "PN"
    },
    "00100030": {
      "Value": ["19900101"],
      "vr": "DA"
    },
    "00100040": {
      "Value": ["M"],
      "vr": "CS"
    },
    "00201200": {
      "Value": ["3"],
      "vr": "IS"
    }
  }
]
```

### 응답 필드 설명

| DICOM 태그 | 이름 | 타입 | 설명 |
|-----------|------|------|------|
| `00100020` | PatientID | string | 환자 고유 식별자 |
| `00100010` | PatientName | object | 환자 이름 (Alphabetic) |
| `00100030` | PatientBirthDate | string | 생년월일 (YYYYMMDD) |
| `00100040` | PatientSex | string | 성별 (M/F/O) |
| `00201200` | NumberOfPatientRelatedStudies | string | 환자의 Study 개수 |

### 참고 사항

- 현재 대부분의 PACS 환경에서 Patient 레벨 메타데이터가 없어 빈 배열 `[]`을 반환할 수 있습니다
- **권장**: Series API를 통해 환자 정보를 조회하는 것을 권장합니다

---

## 4. 시리즈 목록 조회 API

### 엔드포인트

```
GET /api/dicom/series
```

### 요청 파라미터

| 파라미터 | 타입 | 필수 | 기본값 | 설명 |
|---------|------|------|--------|------|
| `project_id` | integer | 조건부* | - | 프로젝트 ID |
| `PatientID` | string | 선택 | - | 환자 ID 필터 |
| `StudyInstanceUID` | string | 선택 | - | Study UID 필터 |
| `SeriesInstanceUID` | string | 선택 | - | Series UID 필터 |
| `Modality` | string | 선택 | - | 모달리티 필터 (CT, MR, SM 등) |
| `limit` | integer | 선택 | 100 | 결과 개수 제한 (최대 1000) |
| `offset` | integer | 선택 | 0 | 페이지네이션 오프셋 |

> **\* 조건부 필수**: 일반 사용자는 필수, SUPER_ADMIN은 선택

### 요청 예시

```http
GET /api/dicom/series?project_id=2&PatientID=SarcopeniaCase1&Modality=CT
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

### 응답 형식

**성공 (200 OK):**
```json
[
  {
    "00080020": {
      "Value": ["20061220"],
      "vr": "DA"
    },
    "00080030": {
      "Value": ["194000.0"],
      "vr": "TM"
    },
    "00080060": {
      "Value": ["CT"],
      "vr": "CS"
    },
    "0008103E": {
      "Value": ["Portal(Supine)  5.0  B30f"],
      "vr": "LO"
    },
    "00100010": {
      "Value": [
        {
          "Alphabetic": "SarcopeniaCase1"
        }
      ],
      "vr": "PN"
    },
    "00100020": {
      "Value": ["SarcopeniaCase1"],
      "vr": "LO"
    },
    "00100030": {
      "Value": ["Anonymization"],
      "vr": "DA"
    },
    "00100040": {
      "Value": ["F"],
      "vr": "CS"
    },
    "0020000D": {
      "Value": ["1.2.410.200022.500.200612201921171.113378644"],
      "vr": "UI"
    },
    "0020000E": {
      "Value": ["1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771"],
      "vr": "UI"
    },
    "00200011": {
      "Value": ["4"],
      "vr": "IS"
    },
    "00201208": {
      "Value": ["8"],
      "vr": "IS"
    },
    "thumbnail_url": "https://archive.pacs.ai-do.kr/rs/studies/1.2.410.200022.500.200612201921171.113378644/series/1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771/thumbnail"
  }
]
```

### 응답 필드 설명

#### 기본 DICOM 태그

| DICOM 태그 | 이름 | 타입 | 설명 |
|-----------|------|------|------|
| `00080020` | StudyDate | string | 검사 날짜 (YYYYMMDD) |
| `00080030` | StudyTime | string | 검사 시간 (HHMMSS.ffffff) |
| `00080060` | Modality | string | 모달리티 (CT, MR, SM, CR 등) |
| `0008103E` | SeriesDescription | string | Series 설명 |
| `00100010` | PatientName | object | 환자 이름 (Alphabetic) |
| `00100020` | PatientID | string | 환자 ID |
| `00100030` | PatientBirthDate | string | 생년월일 (YYYYMMDD) |
| `00100040` | PatientSex | string | 성별 (M/F/O) |
| `0020000D` | StudyInstanceUID | string | Study 고유 식별자 (UID) |
| `0020000E` | SeriesInstanceUID | string | Series 고유 식별자 (UID) |
| `00200011` | SeriesNumber | string | Series 번호 |
| `00201208` | NumberOfSeriesRelatedInstances | string | Series의 Instance(이미지) 개수 |

#### 추가 필드

| 필드 | 타입 | 설명 |
|------|------|------|
| `thumbnail_url` | string | WADO-RS 썸네일 URL (자동 생성) |

### 썸네일 URL 형식

```
https://archive.pacs.ai-do.kr/rs/studies/{StudyInstanceUID}/series/{SeriesInstanceUID}/thumbnail
```

- WADO-RS 표준 준수
- 인증 없이 직접 접근 가능 (PACS 설정에 따라 다름)
- 이미지 형식: JPEG (기본)

---

## 5. 에러 처리

### 에러 응답 형식

```json
{
  "error": "에러 메시지"
}
```

### 주요 에러 코드

| HTTP 상태 | 에러 메시지 | 설명 | 해결 방법 |
|----------|-----------|------|----------|
| `400` | `project_id is required (no global access permission)` | 일반 사용자가 project_id 없이 요청 | project_id 파라미터 추가 |
| `400` | `Invalid project_id` | 잘못된 project_id (0, 음수 등) | 유효한 project_id 사용 |
| `401` | `Unauthorized` | 인증 토큰 없음 또는 만료 | 로그인 후 새 토큰 발급 |
| `403` | `Forbidden` | 프로젝트 접근 권한 없음 | 권한 확인 또는 관리자 문의 |
| `500` | `Internal Server Error` | 서버 내부 오류 | 관리자 문의 |
| `502` | `Bad Gateway` | PACS 서버 연결 실패 | PACS 서버 상태 확인 |

### 빈 결과 vs 에러

**빈 결과 (200 OK):**
```json
[]
```
- 조회 조건에 맞는 데이터가 없음
- 정상적인 응답

**에러 (4xx/5xx):**
```json
{
  "error": "에러 메시지"
}
```
- 요청이 잘못되었거나 서버 오류
- 클라이언트 측 처리 필요

---

## 6. 코드 예시

### JavaScript (Fetch API)

```javascript
// 1. 로그인
async function login(username, password) {
  const response = await fetch('http://localhost:8080/api/test/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password })
  });
  const data = await response.json();
  return data.access_token;
}

// 2. 환자 목록 조회
async function getPatients(token, projectId) {
  const response = await fetch(
    `http://localhost:8080/api/dicom/patients?project_id=${projectId}`,
    {
      headers: { 'Authorization': `Bearer ${token}` }
    }
  );
  return await response.json();
}

// 3. 시리즈 목록 조회
async function getSeries(token, projectId, patientId) {
  const params = new URLSearchParams({
    project_id: projectId,
    PatientID: patientId
  });

  const response = await fetch(
    `http://localhost:8080/api/dicom/series?${params}`,
    {
      headers: { 'Authorization': `Bearer ${token}` }
    }
  );
  return await response.json();
}

// 4. 사용 예시
async function main() {
  try {
    // 로그인
    const token = await login('test_super_admin', 'TestAdmin123!');

    // 환자 목록 조회
    const patients = await getPatients(token, 2);
    console.log('환자 수:', patients.length);

    // 시리즈 목록 조회
    const series = await getSeries(token, 2, 'SarcopeniaCase1');
    console.log('시리즈 수:', series.length);

    // 썸네일 URL 추출
    series.forEach(s => {
      console.log('썸네일:', s.thumbnail_url);
    });
  } catch (error) {
    console.error('에러:', error);
  }
}
```

### React 예시

```typescript
import { useState, useEffect } from 'react';

interface DicomSeries {
  '00100020': { Value: string[] };  // PatientID
  '0020000E': { Value: string[] };  // SeriesInstanceUID
  '00080060': { Value: string[] };  // Modality
  '00201208': { Value: string[] };  // NumberOfSeriesRelatedInstances
  thumbnail_url: string;
}

function SeriesList() {
  const [series, setSeries] = useState<DicomSeries[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchSeries() {
      try {
        // 1. 로그인
        const loginRes = await fetch('/api/test/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            username: 'test_super_admin',
            password: 'TestAdmin123!'
          })
        });
        const { access_token } = await loginRes.json();

        // 2. 시리즈 조회
        const seriesRes = await fetch(
          '/api/dicom/series?project_id=2&PatientID=SarcopeniaCase1',
          {
            headers: { 'Authorization': `Bearer ${access_token}` }
          }
        );

        if (!seriesRes.ok) {
          throw new Error(`HTTP ${seriesRes.status}`);
        }

        const data = await seriesRes.json();
        setSeries(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    }

    fetchSeries();
  }, []);

  if (loading) return <div>로딩 중...</div>;
  if (error) return <div>에러: {error}</div>;

  return (
    <div>
      <h2>시리즈 목록 ({series.length}개)</h2>
      {series.map((s, idx) => (
        <div key={idx}>
          <p>환자 ID: {s['00100020'].Value[0]}</p>
          <p>모달리티: {s['00080060'].Value[0]}</p>
          <p>이미지 수: {s['00201208'].Value[0]}</p>
          <img src={s.thumbnail_url} alt="썸네일" />
        </div>
      ))}
    </div>
  );
}
```

### Python 예시

```python
import requests
from typing import List, Dict, Any

class DicomApiClient:
    def __init__(self, base_url: str):
        self.base_url = base_url
        self.token = None

    def login(self, username: str, password: str) -> str:
        """로그인하여 토큰 발급"""
        response = requests.post(
            f"{self.base_url}/api/test/login",
            json={"username": username, "password": password}
        )
        response.raise_for_status()
        self.token = response.json()["access_token"]
        return self.token

    def _get_headers(self) -> Dict[str, str]:
        """인증 헤더 생성"""
        if not self.token:
            raise ValueError("로그인이 필요합니다")
        return {"Authorization": f"Bearer {self.token}"}

    def get_patients(self, project_id: int, limit: int = 100, offset: int = 0) -> List[Dict[str, Any]]:
        """환자 목록 조회"""
        response = requests.get(
            f"{self.base_url}/api/dicom/patients",
            headers=self._get_headers(),
            params={"project_id": project_id, "limit": limit, "offset": offset}
        )
        response.raise_for_status()
        return response.json()

    def get_series(
        self,
        project_id: int,
        patient_id: str = None,
        modality: str = None,
        limit: int = 100,
        offset: int = 0
    ) -> List[Dict[str, Any]]:
        """시리즈 목록 조회"""
        params = {"project_id": project_id, "limit": limit, "offset": offset}
        if patient_id:
            params["PatientID"] = patient_id
        if modality:
            params["Modality"] = modality

        response = requests.get(
            f"{self.base_url}/api/dicom/series",
            headers=self._get_headers(),
            params=params
        )
        response.raise_for_status()
        return response.json()

    def extract_patient_id(self, series: Dict[str, Any]) -> str:
        """시리즈에서 환자 ID 추출"""
        return series.get("00100020", {}).get("Value", [""])[0]

    def extract_thumbnail_url(self, series: Dict[str, Any]) -> str:
        """시리즈에서 썸네일 URL 추출"""
        return series.get("thumbnail_url", "")

# 사용 예시
if __name__ == "__main__":
    client = DicomApiClient("http://localhost:8080")

    # 로그인
    client.login("test_super_admin", "TestAdmin123!")

    # 환자 목록 조회
    patients = client.get_patients(project_id=2)
    print(f"환자 수: {len(patients)}")

    # 시리즈 목록 조회
    series_list = client.get_series(
        project_id=2,
        patient_id="SarcopeniaCase1",
        modality="CT"
    )
    print(f"시리즈 수: {len(series_list)}")

    # 썸네일 URL 출력
    for series in series_list:
        patient_id = client.extract_patient_id(series)
        thumbnail = client.extract_thumbnail_url(series)
        print(f"환자: {patient_id}, 썸네일: {thumbnail}")
```

---

## 7. 자주 묻는 질문 (FAQ)

### Q1. Patient API가 빈 배열을 반환하는데 정상인가요?

**A**: 네, 정상입니다. 대부분의 PACS 환경에서는 Patient 레벨 메타데이터를 별도로 저장하지 않습니다.
**Series API를 사용하여 환자 정보를 조회하는 것을 권장**합니다. Series 응답에 환자 정보(`00100020`, `00100010` 등)가 포함되어 있습니다.

### Q2. 썸네일 URL에 인증이 필요한가요?

**A**: PACS 서버 설정에 따라 다릅니다. 현재 개발 환경에서는 인증 없이 접근 가능하지만,
프로덕션 환경에서는 인증이 필요할 수 있습니다. 403 에러가 발생하면 관리자에게 문의하세요.

### Q3. DICOM 태그 번호를 어떻게 해석하나요?

**A**: DICOM 태그는 8자리 16진수로 표현됩니다 (예: `00100020`).
- 앞 4자리: Group Number
- 뒤 4자리: Element Number

주요 태그는 위 문서의 "응답 필드 설명" 섹션을 참고하세요.

### Q4. 페이지네이션은 어떻게 사용하나요?

**A**: `limit`과 `offset` 파라미터를 사용합니다.

```javascript
// 1페이지 (1-100)
getSeries(token, 2, 'PATIENT_001', { limit: 100, offset: 0 });

// 2페이지 (101-200)
getSeries(token, 2, 'PATIENT_001', { limit: 100, offset: 100 });

// 3페이지 (201-300)
getSeries(token, 2, 'PATIENT_001', { limit: 100, offset: 200 });
```

### Q5. 여러 환자의 시리즈를 한 번에 조회할 수 있나요?

**A**: 현재는 단일 `PatientID` 필터만 지원합니다. 여러 환자의 데이터가 필요한 경우:
1. `PatientID` 파라미터 없이 전체 조회 (SUPER_ADMIN만 가능)
2. 각 환자별로 개별 요청

### Q6. Modality 값은 어떤 것들이 있나요?

**A**: 주요 Modality 값:
- `CT`: Computed Tomography (컴퓨터 단층촬영)
- `MR`: Magnetic Resonance (자기공명영상)
- `CR`: Computed Radiography (컴퓨터 방사선촬영)
- `DX`: Digital Radiography (디지털 방사선촬영)
- `SM`: Slide Microscopy (슬라이드 현미경)
- `US`: Ultrasound (초음파)
- `XA`: X-Ray Angiography (혈관조영술)

전체 목록은 [DICOM 표준](https://dicom.nema.org/medical/dicom/current/output/chtml/part03/sect_C.7.3.html#sect_C.7.3.1.1.1)을 참고하세요.

---

## 8. 변경 이력

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0 | 2025-12-02 | 초기 문서 작성 |

---

## 9. 문의

- **기술 지원**: dev@ai-do.kr
- **버그 리포트**: GitHub Issues
- **API 문서**: http://localhost:8080/swagger-ui/ (개발 환경)

