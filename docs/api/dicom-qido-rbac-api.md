# DICOM QIDO-RS + RBAC API

QIDO-RS 표준을 따르는 DICOM 데이터 조회 API입니다. **프로젝트 기반 RBAC (Role-Based Access Control)**가 적용되어 사용자 권한에 따라 필터링된 결과를 반환합니다.

## 목차

- [1. Study 목록 조회](#1-study-목록-조회)
- [2. Series 목록 조회](#2-series-목록-조회)
- [3. Instance 목록 조회](#3-instance-목록-조회)
- [4. RBAC 필터링 규칙](#4-rbac-필터링-규칙)
- [5. 에러 응답](#5-에러-응답)

---

## 1. Study 목록 조회

프로젝트에 할당된 Study 목록을 QIDO-RS를 통해 조회합니다. **직접 할당**과 **규칙 기반 할당**을 모두 포함하며, 사용자 권한에 따라 필터링됩니다.

### 엔드포인트

```
GET /api/dicom/studies
```

### 쿼리 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID (필수, > 0) |
| `limit` | integer | ❌ | 반환할 최대 항목 수 (기본값: 서버 설정) |
| `offset` | integer | ❌ | 건너뛸 항목 수 (페이지네이션) |
| `modality` | string | ❌ | 모달리티 필터 (예: CT, MR, US) |
| `patient_id` | string | ❌ | 환자 ID 필터 |
| `patient_name` | string | ❌ | 환자 이름 필터 |
| `accession_number` | string | ❌ | Accession Number 필터 |
| `study_date` | string | ❌ | 검사 날짜 필터 (YYYYMMDD 또는 YYYYMMDD-YYYYMMDD) |

### 헤더

| 헤더 | 필수 | 설명 |
|------|------|------|
| `Authorization` | ✅ | Bearer 토큰 (JWT) |

### 요청 예시

```bash
# 기본 조회
curl -X GET "http://localhost:8080/api/dicom/studies?project_id=2&limit=50&offset=0" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# 필터 적용
curl -X GET "http://localhost:8080/api/dicom/studies?project_id=2&modality=CT&patient_id=P001" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# 날짜 범위 필터
curl -X GET "http://localhost:8080/api/dicom/studies?project_id=2&study_date=20230101-20231231" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 응답 예시

QIDO-RS 표준 JSON 형식으로 반환됩니다:

```json
[
  {
    "0020000D": {
      "vr": "UI",
      "Value": ["1.3.12.2.1107.5.1.4.51698.30000006122005083573400000008"]
    },
    "00100020": {
      "vr": "LO",
      "Value": ["P001"]
    },
    "00100010": {
      "vr": "PN",
      "Value": [{"Alphabetic": "Patient^Name"}]
    },
    "00080020": {
      "vr": "DA",
      "Value": ["20061220"]
    },
    "00081030": {
      "vr": "LO",
      "Value": ["CT ABDOMEN"]
    },
    "00080060": {
      "vr": "CS",
      "Value": ["CT"]
    }
  }
]
```

### 주요 DICOM 태그

| 태그 | 이름 | 설명 |
|------|------|------|
| `0020000D` | StudyInstanceUID | Study 고유 식별자 |
| `00100020` | PatientID | 환자 ID |
| `00100010` | PatientName | 환자 이름 |
| `00080020` | StudyDate | 검사 날짜 (YYYYMMDD) |
| `00081030` | StudyDescription | Study 설명 |
| `00080060` | Modality | 모달리티 |

---

## 2. Series 목록 조회

특정 Study에 속한 Series 목록을 조회합니다.

### 엔드포인트

```
GET /api/dicom/studies/{studyInstanceUID}/series
```

### 경로 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `studyInstanceUID` | string | ✅ | Study Instance UID |

### 쿼리 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID (필수, > 0) |
| `limit` | integer | ❌ | 반환할 최대 항목 수 |
| `offset` | integer | ❌ | 건너뛸 항목 수 |
| `modality` | string | ❌ | 모달리티 필터 |

### 헤더

| 헤더 | 필수 | 설명 |
|------|------|------|
| `Authorization` | ✅ | Bearer 토큰 (JWT) |

### 요청 예시

```bash
curl -X GET "http://localhost:8080/api/dicom/studies/1.3.12.2.1107.5.1.4.51698.30000006122005083573400000008/series?project_id=2" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 응답 예시

```json
[
  {
    "0020000E": {
      "vr": "UI",
      "Value": ["1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771"]
    },
    "0020000D": {
      "vr": "UI",
      "Value": ["1.3.12.2.1107.5.1.4.51698.30000006122005083573400000008"]
    },
    "00080060": {
      "vr": "CS",
      "Value": ["CT"]
    },
    "0008103E": {
      "vr": "LO",
      "Value": ["Portal(Supine)  5.0  B30f"]
    },
    "00200011": {
      "vr": "IS",
      "Value": ["1"]
    }
  }
]
```

### 주요 DICOM 태그

| 태그 | 이름 | 설명 |
|------|------|------|
| `0020000E` | SeriesInstanceUID | Series 고유 식별자 |
| `0020000D` | StudyInstanceUID | 상위 Study UID |
| `00080060` | Modality | 모달리티 |
| `0008103E` | SeriesDescription | Series 설명 |
| `00200011` | SeriesNumber | Series 번호 |

---

## 3. Instance 목록 조회

특정 Series에 속한 Instance (DICOM 이미지) 목록을 조회합니다.

### 엔드포인트

```
GET /api/dicom/studies/{studyInstanceUID}/series/{seriesInstanceUID}/instances
```

### 경로 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `studyInstanceUID` | string | ✅ | Study Instance UID |
| `seriesInstanceUID` | string | ✅ | Series Instance UID |

### 쿼리 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID (필수, > 0) |
| `limit` | integer | ❌ | 반환할 최대 항목 수 |
| `offset` | integer | ❌ | 건너뛸 항목 수 |

### 헤더

| 헤더 | 필수 | 설명 |
|------|------|------|
| `Authorization` | ✅ | Bearer 토큰 (JWT) |

### 요청 예시

```bash
curl -X GET "http://localhost:8080/api/dicom/studies/1.3.12.2.1107.5.1.4.51698.30000006122005083573400000008/series/1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771/instances?project_id=2" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 응답 예시

```json
[
  {
    "00080018": {
      "vr": "UI",
      "Value": ["1.3.12.2.1107.5.1.4.51698.30000006122005083573400013812"]
    },
    "0020000E": {
      "vr": "UI",
      "Value": ["1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771"]
    },
    "0020000D": {
      "vr": "UI",
      "Value": ["1.3.12.2.1107.5.1.4.51698.30000006122005083573400000008"]
    },
    "00080016": {
      "vr": "UI",
      "Value": ["1.2.840.10008.5.1.4.1.1.2"]
    },
    "00200013": {
      "vr": "IS",
      "Value": ["1"]
    }
  }
]
```

### 주요 DICOM 태그

| 태그 | 이름 | 설명 |
|------|------|------|
| `00080018` | SOPInstanceUID | Instance 고유 식별자 |
| `0020000E` | SeriesInstanceUID | 상위 Series UID |
| `0020000D` | StudyInstanceUID | 상위 Study UID |
| `00080016` | SOPClassUID | SOP Class UID |
| `00200013` | InstanceNumber | Instance 번호 |

---

## 4. RBAC 필터링 규칙

모든 API는 다음 순서로 RBAC 필터링을 적용합니다:

### 4.1 프로젝트 멤버십 확인

- 사용자가 프로젝트 멤버가 아니면 **모든 데이터 접근 거부**
- `security_user_project` 테이블에서 확인

### 4.2 규칙 기반 조건 적용

프로젝트에 정의된 DICOM 조건을 QIDO-RS 쿼리 파라미터로 변환:

| 조건 타입 | DICOM 태그 | QIDO 파라미터 | 예시 |
|----------|-----------|--------------|------|
| `EQ` (같음) | Modality (00080060) | `Modality` | `Modality=CT` |
| `EQ` | PatientID (00100020) | `PatientID` | `PatientID=P001` |
| `EQ` | AccessionNumber (00080050) | `AccessionNumber` | `AccessionNumber=A123` |
| `CONTAINS` | PatientName (00100010) | `PatientName` | `PatientName=John` |
| `RANGE` | StudyDate (00080020) | `StudyDate` | `StudyDate=20230101-20231231` |

### 4.3 사용자 입력 우선

- 사용자가 쿼리 파라미터로 전달한 값이 규칙 기반 조건보다 **우선**
- 예: 규칙에 `Modality=CT`가 있어도 사용자가 `Modality=MR`을 전달하면 `MR`로 조회

### 4.4 사후 RBAC 필터링

QIDO-RS 응답을 받은 후, 각 항목에 대해 RBAC 평가:

1. **명시적 거부 확인** (최우선)
   - `project_data_access` 테이블에서 `status='DENIED'` 확인
   - 거부된 항목은 제외

2. **명시적 허용 확인**
   - `project_data_access` 테이블에서 `status='APPROVED'` 확인
   - 허용된 항목은 포함

3. **기관 기반 접근 확인**
   - 사용자와 데이터가 같은 기관에 속하면 허용

4. **규칙 기반 조건 평가**
   - `security_project_dicom_condition` 테이블의 조건 평가
   - Modality, PatientID, StudyDate 등 매칭 확인

5. **상속 규칙**
   - Study가 허용되면 하위 Series/Instance도 허용
   - Series가 허용되면 하위 Instance도 허용
   - 상위가 거부되면 하위도 거부

### 4.5 필터링 결과

- 허용된 항목만 응답에 포함
- 거부된 항목은 응답에서 제외 (404가 아님)

---

## 5. 에러 응답

### 5.1 인증 오류

```json
{
  "error": "Invalid or missing authorization token"
}
```

**HTTP 상태 코드**: `401 Unauthorized`

### 5.2 필수 파라미터 누락

```json
{
  "error": "project_id is required and must be greater than 0"
}
```

**HTTP 상태 코드**: `400 Bad Request`

### 5.3 QIDO-RS 서버 오류

```json
{
  "error": "QIDO /studies failed (500): Internal Server Error"
}
```

**HTTP 상태 코드**: `502 Bad Gateway`

### 5.4 잘못된 파라미터

```json
{
  "error": "Invalid study_date format. Expected YYYYMMDD or YYYYMMDD-YYYYMMDD"
}
```

**HTTP 상태 코드**: `400 Bad Request`

---

## 6. 사용 예시

### 6.1 프로젝트의 모든 CT Study 조회

```bash
curl -X GET "http://localhost:8080/api/dicom/studies?project_id=2&modality=CT&limit=100" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 6.2 특정 환자의 Study 조회

```bash
curl -X GET "http://localhost:8080/api/dicom/studies?project_id=2&patient_id=P001" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 6.3 날짜 범위로 Study 조회

```bash
curl -X GET "http://localhost:8080/api/dicom/studies?project_id=2&study_date=20230101-20231231" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 6.4 Study → Series → Instance 계층 조회

```bash
# 1. Study 조회
STUDY_UID=$(curl -s "http://localhost:8080/api/dicom/studies?project_id=2&limit=1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" | jq -r '.[0]["0020000D"].Value[0]')

# 2. Series 조회
SERIES_UID=$(curl -s "http://localhost:8080/api/dicom/studies/${STUDY_UID}/series?project_id=2" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" | jq -r '.[0]["0020000E"].Value[0]')

# 3. Instance 조회
curl -s "http://localhost:8080/api/dicom/studies/${STUDY_UID}/series/${SERIES_UID}/instances?project_id=2" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 7. 참고사항

### 7.1 QIDO-RS 표준

이 API는 [DICOM QIDO-RS](https://www.dicomstandard.org/using/dicomweb/query-qido-rs) 표준을 따릅니다.

### 7.2 RBAC 평가 로직

자세한 RBAC 평가 로직은 `DicomRbacEvaluatorImpl`에 구현되어 있습니다:
- `evaluate_study_uid()` - Study 접근 권한 평가
- `evaluate_series_uid()` - Series 접근 권한 평가
- `evaluate_instance_uid()` - Instance 접근 권한 평가

### 7.3 성능 고려사항

- QIDO-RS 호출 후 RBAC 필터링이 적용되므로, 대량 데이터 조회 시 성능 영향 가능
- `limit`/`offset` 파라미터를 사용하여 페이지네이션 권장
- 규칙 기반 조건을 QIDO 파라미터로 변환하여 서버 측 필터링 최대화

### 7.4 할당 방식

1. **직접 할당**: `project_data` 테이블에 명시적으로 할당
2. **규칙 기반 할당**: `security_project_dicom_condition`에 정의된 조건에 따라 자동 매칭

두 방식 모두 RBAC 필터링을 거쳐 최종 결과가 반환됩니다.

---

## 버전 정보

- **API 버전**: v1
- **QIDO-RS 표준**: DICOM PS3.18
- **최종 업데이트**: 2025-11-06

