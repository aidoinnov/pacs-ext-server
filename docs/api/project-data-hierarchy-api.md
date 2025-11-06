# 프로젝트 데이터 계층 조회 API

프로젝트에 할당된 DICOM 데이터를 Study → Series → Instance 계층 구조로 조회하는 API입니다.

## 목차

- [1. Study 목록 조회](#1-study-목록-조회)
- [2. Series 목록 조회](#2-series-목록-조회)
- [3. Instance 목록 조회](#3-instance-목록-조회)

---

## 1. Study 목록 조회

프로젝트에 할당된 Study 목록을 조회합니다. **직접 할당**과 **규칙 기반 할당**을 모두 포함합니다.

### 엔드포인트

```
GET /api/project-data/{project_id}/studies
```

### 경로 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |

### 쿼리 파라미터

| 파라미터 | 타입 | 필수 | 기본값 | 설명 |
|---------|------|------|--------|------|
| `page` | integer | ❌ | 1 | 페이지 번호 |
| `page_size` | integer | ❌ | 20 | 페이지당 항목 수 |

### 요청 예시

```bash
curl -X GET "http://localhost:8080/api/project-data/2/studies?page=1&page_size=20"
```

### 응답 예시

```json
{
  "success": true,
  "studies": [
    {
      "study": {
        "id": 242,
        "study_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400000008",
        "study_description": "CT ABDOMEN",
        "patient_id": "1",
        "patient_name": null,
        "patient_birth_date": null,
        "study_date": "2006-12-20",
        "created_at": "2025-10-31T02:46:41.402455+00:00"
      },
      "assigned_at": "2025-10-31T02:46:41.402455+00:00"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total_items": 2,
    "total_pages": 1
  }
}
```

### 응답 필드 설명

#### `studies` 배열

| 필드 | 타입 | 설명 |
|------|------|------|
| `study.id` | integer | Study 내부 ID |
| `study.study_uid` | string | DICOM Study Instance UID |
| `study.study_description` | string | Study 설명 |
| `study.patient_id` | string | 환자 ID |
| `study.patient_name` | string \| null | 환자 이름 |
| `study.patient_birth_date` | string \| null | 환자 생년월일 (YYYY-MM-DD) |
| `study.study_date` | string \| null | 검사 날짜 (YYYY-MM-DD) |
| `study.created_at` | string | 생성 시간 (ISO 8601) |
| `assigned_at` | string | 프로젝트 할당 시간 (ISO 8601) |

#### `pagination` 객체

| 필드 | 타입 | 설명 |
|------|------|------|
| `page` | integer | 현재 페이지 번호 |
| `page_size` | integer | 페이지당 항목 수 |
| `total_items` | integer | 전체 항목 수 |
| `total_pages` | integer | 전체 페이지 수 |

### 특징

- ✅ **직접 할당**: `project_data` 테이블에 명시적으로 할당된 Study
- ✅ **규칙 기반 할당**: `security_project_dicom_condition`에 정의된 조건에 매칭되는 Study
- ✅ 중복 제거: 직접 할당과 규칙 기반 모두에 해당하는 경우 한 번만 표시
- ✅ 정렬: `study_date` 내림차순 → `created_at` 내림차순

---

## 2. Series 목록 조회

특정 Study에 속한 Series 목록을 조회합니다.

### 엔드포인트

```
GET /api/project-data/{project_id}/studies/{study_id}/series
```

### 경로 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |
| `study_id` | integer | ✅ | Study ID |

### 쿼리 파라미터

| 파라미터 | 타입 | 필수 | 기본값 | 설명 |
|---------|------|------|--------|------|
| `page` | integer | ❌ | 1 | 페이지 번호 |
| `page_size` | integer | ❌ | 20 | 페이지당 항목 수 |

### 요청 예시

```bash
curl -X GET "http://localhost:8080/api/project-data/2/studies/242/series?page=1&page_size=20"
```

### 응답 예시

```json
{
  "success": true,
  "study": {
    "id": 242,
    "study_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400000008",
    "study_description": "CT ABDOMEN",
    "patient_id": "1",
    "patient_name": null,
    "patient_birth_date": null,
    "study_date": "2006-12-20",
    "created_at": "2025-10-31T02:46:41.402455+00:00"
  },
  "series": [
    {
      "series": {
        "id": 216,
        "series_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771",
        "series_description": "Portal(Supine)  5.0  B30f",
        "modality": "CT",
        "series_number": null,
        "created_at": "2025-10-31T02:46:41.402455+00:00"
      },
      "assigned_at": "2025-10-31T02:46:41.402455+00:00"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total_items": 1,
    "total_pages": 1
  }
}
```

### 응답 필드 설명

#### `study` 객체

Study 정보 (1번 API와 동일)

#### `series` 배열

| 필드 | 타입 | 설명 |
|------|------|------|
| `series.id` | integer | Series 내부 ID |
| `series.series_uid` | string | DICOM Series Instance UID |
| `series.series_description` | string \| null | Series 설명 |
| `series.modality` | string | 모달리티 (CT, MR, US 등) |
| `series.series_number` | integer \| null | Series 번호 |
| `series.created_at` | string | 생성 시간 (ISO 8601) |
| `assigned_at` | string | 프로젝트 할당 시간 (ISO 8601) |

#### `pagination` 객체

1번 API와 동일

### 특징

- ✅ Study 정보와 Series 목록을 함께 반환
- ✅ 정렬: `series_number` 오름차순 → `created_at` 오름차순

---

## 3. Instance 목록 조회

특정 Series에 속한 Instance 목록을 조회합니다.

### 엔드포인트

```
GET /api/project-data/{project_id}/series/{series_id}/instances
```

### 경로 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |
| `series_id` | integer | ✅ | Series ID |

### 쿼리 파라미터

| 파라미터 | 타입 | 필수 | 기본값 | 설명 |
|---------|------|------|--------|------|
| `page` | integer | ❌ | 1 | 페이지 번호 |
| `page_size` | integer | ❌ | 20 | 페이지당 항목 수 |

### 요청 예시

```bash
curl -X GET "http://localhost:8080/api/project-data/2/series/216/instances?page=1&page_size=10"
```

### 응답 예시

```json
{
  "success": true,
  "instances": [
    {
      "series": {
        "id": 216,
        "series_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771",
        "series_description": "Portal(Supine)  5.0  B30f",
        "modality": "CT",
        "series_number": null,
        "created_at": "2025-10-31T02:46:41.402455+00:00"
      },
      "instance": {
        "id": 45,
        "instance_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771.1",
        "sop_class_uid": "1.2.840.10008.5.1.4.1.1.2",
        "instance_number": 1,
        "created_at": "2025-11-06T05:37:15.666196+00:00"
      },
      "assigned_at": "2025-11-06T05:37:15.666196+00:00"
    },
    {
      "series": {
        "id": 216,
        "series_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771",
        "series_description": "Portal(Supine)  5.0  B30f",
        "modality": "CT",
        "series_number": null,
        "created_at": "2025-10-31T02:46:41.402455+00:00"
      },
      "instance": {
        "id": 46,
        "instance_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771.2",
        "sop_class_uid": "1.2.840.10008.5.1.4.1.1.2",
        "instance_number": 2,
        "created_at": "2025-11-06T05:37:15.666196+00:00"
      },
      "assigned_at": "2025-11-06T05:37:15.666196+00:00"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 10,
    "total_items": 15,
    "total_pages": 2
  }
}
```

### 응답 필드 설명

#### `instances` 배열

| 필드 | 타입 | 설명 |
|------|------|------|
| `series.id` | integer | Series 내부 ID |
| `series.series_uid` | string | DICOM Series Instance UID |
| `series.series_description` | string \| null | Series 설명 |
| `series.modality` | string | 모달리티 (CT, MR, US 등) |
| `series.series_number` | integer \| null | Series 번호 |
| `series.created_at` | string | Series 생성 시간 (ISO 8601) |
| `instance.id` | integer | Instance 내부 ID |
| `instance.instance_uid` | string | DICOM SOP Instance UID |
| `instance.sop_class_uid` | string | DICOM SOP Class UID |
| `instance.instance_number` | integer \| null | Instance 번호 |
| `instance.created_at` | string | Instance 생성 시간 (ISO 8601) |
| `assigned_at` | string | 프로젝트 할당 시간 (ISO 8601) |

#### `pagination` 객체

1번 API와 동일

### 특징

- ✅ Series 정보와 Instance 목록을 함께 반환
- ✅ 정렬: `instance_number` 오름차순 → `created_at` 오름차순
- ✅ DICOM 이미지 파일 단위 조회 가능

---

## 에러 응답

모든 API는 에러 발생 시 다음 형식으로 응답합니다:

```json
{
  "success": false,
  "error": "에러 메시지"
}
```

### HTTP 상태 코드

| 상태 코드 | 설명 |
|----------|------|
| `200 OK` | 성공 |
| `404 Not Found` | 리소스를 찾을 수 없음 (Study, Series 등) |
| `500 Internal Server Error` | 서버 내부 오류 |

---

## 사용 예시

### 1. 프로젝트의 모든 Study 조회

```bash
curl -X GET "http://localhost:8080/api/project-data/2/studies?page=1&page_size=20"
```

### 2. 특정 Study의 Series 조회

```bash
# 1번 응답에서 study.id = 242를 얻었다면
curl -X GET "http://localhost:8080/api/project-data/2/studies/242/series?page=1&page_size=20"
```

### 3. 특정 Series의 Instance 조회

```bash
# 2번 응답에서 series.id = 216을 얻었다면
curl -X GET "http://localhost:8080/api/project-data/2/series/216/instances?page=1&page_size=10"
```

---

## 참고사항

### DICOM 계층 구조

```
Study (검사)
  └── Series (시리즈)
        └── Instance (이미지)
```

- **Study**: 한 번의 검사 (예: CT 복부 검사)
- **Series**: 검사 내의 시리즈 (예: 조영 전, 조영 후)
- **Instance**: 개별 DICOM 이미지 파일

### 할당 방식

1. **직접 할당**: 관리자가 수동으로 프로젝트에 할당
2. **규칙 기반 할당**: DICOM 조건 (환자 ID, 모달리티, 검사 날짜 등)에 따라 자동 할당

Study 목록 조회 API는 두 방식을 모두 포함하여 반환합니다.

---

## 버전 정보

- **API 버전**: v1
- **최종 업데이트**: 2025-11-06

