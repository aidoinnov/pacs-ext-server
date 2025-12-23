# DICOM API 응답 예시

## 📋 목차
1. [Patient 목록 조회 API](#1-patient-목록-조회-api)
2. [Series 목록 조회 API](#2-series-목록-조회-api)

---

## 1. Patient 목록 조회 API

### 요청
```http
GET /api/dicom/patients?project_id=2
Authorization: Bearer {access_token}
```

### 쿼리 파라미터
| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | 선택* | 프로젝트 ID (일반 사용자는 필수, SUPER_ADMIN은 선택) |
| `limit` | integer | 선택 | 결과 개수 제한 (기본값: 100) |
| `offset` | integer | 선택 | 페이지네이션 오프셋 (기본값: 0) |

> **참고**: 현재 Patient API는 Dcm4chee PACS의 `/rs/patients` 엔드포인트를 사용하며,
> 프로젝트에 할당된 환자 데이터만 필터링하여 반환합니다.

### 응답 예시
```json
[]
```

> **참고**: Patient 목록은 PACS에 환자 레벨 메타데이터가 있을 때만 반환됩니다.
> 대부분의 경우 Series API를 통해 환자 정보를 조회하는 것을 권장합니다.

### 주요 DICOM 태그 설명
| 태그 | 이름 | 설명 |
|------|------|------|
| `00100020` | PatientID | 환자 ID (고유 식별자) |
| `00100010` | PatientName | 환자 이름 |
| `00100030` | PatientBirthDate | 생년월일 (YYYYMMDD) |
| `00100040` | PatientSex | 성별 (M/F/O) |
| `00201200` | NumberOfPatientRelatedStudies | 환자의 Study 개수 |

---

## 2. Series 목록 조회 API

### 요청
```http
GET /api/dicom/series?project_id=2&PatientID=SarcopeniaCase1
Authorization: Bearer {access_token}
```

### 쿼리 파라미터
| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | 선택* | 프로젝트 ID (일반 사용자는 필수, SUPER_ADMIN은 선택) |
| `PatientID` | string | 선택 | 환자 ID 필터 |
| `StudyInstanceUID` | string | 선택 | Study UID 필터 |
| `SeriesInstanceUID` | string | 선택 | Series UID 필터 |
| `Modality` | string | 선택 | 모달리티 필터 (CT, MR, SM 등) |
| `limit` | integer | 선택 | 결과 개수 제한 (기본값: 100) |
| `offset` | integer | 선택 | 페이지네이션 오프셋 (기본값: 0) |

### 응답 예시 1: CT 모달리티 (SarcopeniaCase1)
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
    "00080050": {
      "Value": ["Anonymization"],
      "vr": "SH"
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

### 응답 예시 2: SM 모달리티 (C3L-00165-26)
```json
[
  {
    "00080020": {
      "Value": ["20250911"],
      "vr": "DA"
    },
    "00080030": {
      "Value": ["155325"],
      "vr": "TM"
    },
    "00080050": {
      "Value": ["C3L-00165-26"],
      "vr": "SH"
    },
    "00080060": {
      "Value": ["SM"],
      "vr": "CS"
    },
    "00100010": {
      "Value": [
        {
          "Alphabetic": "C3L-00165-26"
        }
      ],
      "vr": "PN"
    },
    "00100020": {
      "Value": ["C3L-00165-26"],
      "vr": "LO"
    },
    "0020000D": {
      "Value": ["1.2.826.0.1.3680043.8.498.12345678901234567890"],
      "vr": "UI"
    },
    "0020000E": {
      "Value": ["1.2.826.0.1.3680043.8.498.98765432109876543210"],
      "vr": "UI"
    },
    "00201208": {
      "Value": ["6"],
      "vr": "IS"
    },
    "thumbnail_url": "https://archive.pacs.ai-do.kr/rs/studies/1.2.826.0.1.3680043.8.498.12345678901234567890/series/1.2.826.0.1.3680043.8.498.98765432109876543210/thumbnail"
  }
]
```

### 주요 DICOM 태그 설명
| 태그 | 이름 | 설명 |
|------|------|------|
| `00080020` | StudyDate | 검사 날짜 (YYYYMMDD) |
| `00080030` | StudyTime | 검사 시간 (HHMMSS) |
| `00080050` | AccessionNumber | 접수 번호 |
| `00080060` | Modality | 모달리티 (CT, MR, SM 등) |
| `0008103E` | SeriesDescription | Series 설명 |
| `00100010` | PatientName | 환자 이름 |
| `00100020` | PatientID | 환자 ID |
| `00100030` | PatientBirthDate | 생년월일 |
| `00100040` | PatientSex | 성별 (M/F/O) |
| `0020000D` | StudyInstanceUID | Study 고유 식별자 |
| `0020000E` | SeriesInstanceUID | Series 고유 식별자 |
| `00200011` | SeriesNumber | Series 번호 |
| `00201208` | NumberOfSeriesRelatedInstances | Series의 Instance 개수 |
| `thumbnail_url` | (추가 필드) | WADO-RS 썸네일 URL |

---

## 3. 빈 응답 예시

### 존재하지 않는 PatientID
```http
GET /api/dicom/series?project_id=2&PatientID=NONEXISTENT_12345
```

**응답:**
```json
[]
```

### 존재하지 않는 Modality
```http
GET /api/dicom/series?project_id=2&PatientID=SarcopeniaCase1&Modality=MR
```

**응답:**
```json
[]
```

---

## 4. 에러 응답 예시

### 권한 없음 (일반 사용자가 project_id 없이 조회)
```http
GET /api/dicom/series?PatientID=SarcopeniaCase1
Authorization: Bearer {user_token}
```

**응답 (400 Bad Request):**
```json
{
  "error": "project_id is required (no global access permission)"
}
```

### 잘못된 project_id
```http
GET /api/dicom/series?project_id=0&PatientID=TEST
```

**응답 (400 Bad Request):**
```json
{
  "error": "Invalid project_id"
}
```

---

## 5. API 사용 예시

### Python
```python
import requests

# 1. 로그인
response = requests.post(
    "http://localhost:8080/api/test/login",
    json={"username": "test_super_admin", "password": "TestAdmin123!"}
)
token = response.json()["access_token"]
headers = {"Authorization": f"Bearer {token}"}

# 2. Patient 목록 조회
patients = requests.get(
    "http://localhost:8080/api/dicom/patients?project_id=2",
    headers=headers
).json()

# 3. Series 목록 조회
series = requests.get(
    "http://localhost:8080/api/dicom/series",
    headers=headers,
    params={"project_id": 2, "PatientID": "SarcopeniaCase1"}
).json()

# 4. 썸네일 URL 추출
for s in series:
    print(s["thumbnail_url"])
```

### JavaScript
```javascript
// 1. 로그인
const loginResponse = await fetch('http://localhost:8080/api/test/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    username: 'test_super_admin',
    password: 'TestAdmin123!'
  })
});
const { access_token } = await loginResponse.json();

// 2. Patient 목록 조회
const patientsResponse = await fetch(
  'http://localhost:8080/api/dicom/patients?project_id=2',
  { headers: { 'Authorization': `Bearer ${access_token}` } }
);
const patients = await patientsResponse.json();

// 3. Series 목록 조회
const seriesResponse = await fetch(
  'http://localhost:8080/api/dicom/series?project_id=2&PatientID=SarcopeniaCase1',
  { headers: { 'Authorization': `Bearer ${access_token}` } }
);
const series = await seriesResponse.json();

// 4. 썸네일 URL 추출
series.forEach(s => console.log(s.thumbnail_url));
```

---

## 6. 참고 사항

### DICOM JSON 형식
- 모든 응답은 **DICOM JSON** 형식을 따릅니다
- 태그는 8자리 16진수 문자열로 표현됩니다 (예: `"00100020"`)
- 각 태그는 `Value`와 `vr` (Value Representation) 속성을 가집니다

### 권한 관리
- **SUPER_ADMIN**: `project_id` 없이 전체 데이터 조회 가능
- **일반 사용자**: `project_id` 필수, 할당된 프로젝트 데이터만 조회 가능

### 썸네일 URL
- Series API는 각 Series에 `thumbnail_url` 필드를 자동으로 추가합니다
- WADO-RS 표준 형식: `/rs/studies/{study_uid}/series/{series_uid}/thumbnail`
- 썸네일은 Dcm4chee PACS에서 직접 제공됩니다

### 페이지네이션
- `limit`: 한 번에 가져올 결과 개수 (기본값: 100)
- `offset`: 건너뛸 결과 개수 (기본값: 0)
- 예: `?limit=10&offset=20` → 21번째부터 30번째까지 10개 조회


