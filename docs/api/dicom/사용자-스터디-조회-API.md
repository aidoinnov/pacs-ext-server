# 사용자 Study 조회 API

## 개요

`GET /api/me/dicom/studies` 엔드포인트는 현재 로그인한 사용자가 접근 가능한 모든 Study를 조회하는 API입니다.

### 주요 특징

- **다중 프로젝트 통합 조회**: 사용자가 속한 모든 프로젝트의 Study를 한 번에 조회
- **RBAC 기반 필터링**: 사용자의 역할과 권한에 따라 자동으로 필터링
- **Access Condition 적용**: 프로젝트별 접근 조건(Access Condition)을 자동으로 적용
- **Extension Fields 지원**: DICOM 표준 필드 외에 확장 필드(report_status, review 등) 제공
- **Study List View 지원**: 사전 정의된 View를 통한 필터링 및 정렬
- **병렬 처리**: 여러 프로젝트의 QIDO 요청을 병렬로 처리하여 성능 최적화

---

## 엔드포인트

```
GET /api/me/dicom/studies
```

---

## 인증

### 헤더

| 헤더 | 필수 | 설명 |
|------|------|------|
| `Authorization` | ✅ | Bearer 토큰 (JWT) |

### 예시

```http
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

---

## Query Parameters

### 기본 파라미터

| 파라미터 | 타입 | 필수 | 기본값 | 설명 |
|---------|------|------|--------|------|
| `project_id` | integer | ❌ | - | 특정 프로젝트만 필터링 (없으면 모든 프로젝트) |
| `view` | string | ❌ | - | Study List View ID (예: "default", "research") |
| `report_status` | string | ❌ | - | 리포트 상태 필터 (쉼표로 구분, 예: "approved,unread") |

### DICOM 필터 파라미터

| 파라미터 | 타입 | 필수 | 설명 | 예시 |
|---------|------|------|------|------|
| `modality` | string | ❌ | Modality 필터 | `CT`, `MR`, `US` |
| `patient_id` | string | ❌ | Patient ID 필터 | `P12345` |
| `patient_name` | string | ❌ | Patient Name 필터 (부분 일치) | `홍길동` |
| `study_date` | string | ❌ | Study Date 범위 (YYYYMMDD 또는 YYYYMMDD-YYYYMMDD) | `20240101-20241231` |
| `accession_number` | string | ❌ | Accession Number 필터 | `ACC-123` |

### 페이지네이션 파라미터

| 파라미터 | 타입 | 필수 | 기본값 | 설명 |
|---------|------|------|--------|------|
| `page` | integer | ❌ | 1 | 페이지 번호 (1부터 시작) |
| `page_size` | integer | ❌ | 50 | 페이지당 항목 수 (1~200) |
| `limit` | integer | ❌ | - | DICOMweb 표준 limit (page_size보다 우선) |
| `offset` | integer | ❌ | - | DICOMweb 표준 offset (page보다 우선) |

---

## 요청 예시

### 1. 기본 조회 (모든 프로젝트)

```bash
curl -X GET "http://localhost:8080/api/me/dicom/studies?page=1&page_size=10" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 2. 특정 프로젝트 조회

```bash
curl -X GET "http://localhost:8080/api/me/dicom/studies?project_id=2&page=1&page_size=10" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 3. Modality 필터링

```bash
curl -X GET "http://localhost:8080/api/me/dicom/studies?modality=CT&page=1&page_size=10" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 4. 날짜 범위 필터링

```bash
curl -X GET "http://localhost:8080/api/me/dicom/studies?study_date=20240101-20241231&page=1&page_size=10" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 5. Report Status 필터링

```bash
curl -X GET "http://localhost:8080/api/me/dicom/studies?report_status=approved,unread&page=1&page_size=10" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 6. Study List View 사용

```bash
curl -X GET "http://localhost:8080/api/me/dicom/studies?view=research&page=1&page_size=10" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 7. 복합 필터링

```bash
curl -X GET "http://localhost:8080/api/me/dicom/studies?project_id=2&modality=CT&study_date=20240101-20241231&report_status=approved&page=1&page_size=10" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 응답 형식

### 성공 응답 (200 OK)

응답은 DICOM JSON 배열 형식으로 반환되며, 각 Study는 DICOM 표준 필드와 확장 필드를 포함합니다.

```json
[
  {
    "0020000D": {
      "vr": "UI",
      "Value": ["1.2.840.113619.2.55.3.604688433.1234"]
    },
    "00080020": {
      "vr": "DA",
      "Value": ["20240715"]
    },
    "00080030": {
      "vr": "TM",
      "Value": ["143000"]
    },
    "00080060": {
      "vr": "CS",
      "Value": ["CT"]
    },
    "00100010": {
      "vr": "PN",
      "Value": [{"Alphabetic": "홍길동"}]
    },
    "00100020": {
      "vr": "LO",
      "Value": ["P12345"]
    },
    "00081030": {
      "vr": "LO",
      "Value": ["Chest CT"]
    },
    "00080050": {
      "vr": "SH",
      "Value": ["ACC-123"]
    },
    "extensions": {
      "project_ids": [2, 5],
      "report_status": "approved",
      "review": {
        "status": "completed",
        "reviewer_id": 10,
        "reviewer_name": "김의사",
        "reviewed_at": "2024-07-20T10:30:00Z"
      }
    }
  }
]
```

### DICOM 필드 설명

| DICOM 태그 | 이름 | VR | 설명 |
|-----------|------|----|----|
| `0020000D` | StudyInstanceUID | UI | Study 고유 식별자 |
| `00080020` | StudyDate | DA | Study 날짜 (YYYYMMDD) |
| `00080030` | StudyTime | TM | Study 시간 (HHMMSS) |
| `00080060` | Modality | CS | Modality (CT, MR, US 등) |
| `00100010` | PatientName | PN | 환자 이름 |
| `00100020` | PatientID | LO | 환자 ID |
| `00081030` | StudyDescription | LO | Study 설명 |
| `00080050` | AccessionNumber | SH | Accession Number |

### Extension Fields 설명

응답의 `extensions` 객체는 DICOM 표준 외의 추가 정보를 포함합니다.

| 필드 | 타입 | 설명 |
|------|------|------|
| `project_ids` | array[integer] | 이 Study가 속한 프로젝트 ID 목록 |
| `report_status` | string | 리포트 상태 (`approved`, `unread`, `pending` 등) |
| `review` | object | 리뷰 정보 (상태, 리뷰어, 리뷰 시각 등) |

#### Review 객체 구조

```json
{
  "status": "completed",
  "reviewer_id": 10,
  "reviewer_name": "김의사",
  "reviewed_at": "2024-07-20T10:30:00Z"
}
```

---

## 에러 응답

### 401 Unauthorized

인증 토큰이 없거나 유효하지 않은 경우

```json
{
  "error": "Invalid or missing authorization token"
}
```

### 400 Bad Request

잘못된 쿼리 파라미터

```json
{
  "error": "Invalid page_size: must be between 1 and 200"
}
```

### 502 Bad Gateway

QIDO 서버 연결 실패

```json
{
  "error": "Failed to connect to QIDO server: connection timeout"
}
```

---

## 동작 방식

### 1. 사용자 프로젝트 조회

사용자가 속한 모든 프로젝트를 조회합니다. `project_id` 파라미터가 제공된 경우 해당 프로젝트만 조회합니다.

### 2. 병렬 QIDO 요청

각 프로젝트에 대해 QIDO 요청을 **병렬로** 실행하여 성능을 최적화합니다.

```rust
// 병렬 처리 예시
let qido_futures: Vec<_> = user_projects.iter().map(|project_id| {
    async move {
        // 프로젝트별 Access Condition 적용
        let qido_params = apply_access_conditions(project_id, user_params);

        // QIDO 호출
        qido.qido_studies_with_bearer(bearer_token, qido_params).await
    }
}).collect();

let results = join_all(qido_futures).await;
```

### 3. Access Condition 적용

각 프로젝트의 Access Condition을 자동으로 적용합니다.

**예시**: 프로젝트 2의 Access Condition이 `Modality=CT`인 경우
- 사용자 요청: `?modality=MR`
- 실제 QIDO 요청: `?modality=CT` (Access Condition이 우선)

### 4. RBAC 필터링

각 Study에 대해 RBAC 평가를 수행하여 사용자가 접근 가능한지 확인합니다.

```rust
// RBAC 평가
let result = evaluator.evaluate_study_uid(user_id, project_id, &study_uid).await;

// project_data_access 확인
let has_data_access = can_access_study(user_id, project_id, &study_uid, pool).await;

// 접근 가능한 경우만 포함
if result.allowed && has_data_access {
    studies.push(study);
}
```

### 5. Extension Fields 추가

배치 쿼리를 사용하여 Extension Fields를 효율적으로 조회합니다.

```rust
// 모든 Study UID 수집
let all_study_uids: Vec<String> = studies.iter()
    .filter_map(|s| extract_study_uid(s))
    .collect();

// 배치로 report_status 조회
let report_status_cache = fetch_report_status_batch(&all_study_uids, &all_project_ids).await;

// 배치로 review 조회
let review_cache = fetch_review_batch(&all_study_uids, &all_project_ids).await;

// 각 Study에 Extension Fields 추가
for study in studies {
    study["extensions"] = build_extensions(study_uid, &caches);
}
```

### 6. 중복 제거

같은 Study가 여러 프로젝트에 속한 경우 중복을 제거하고, `project_ids` 배열에 모든 프로젝트 ID를 포함합니다.

### 7. 페이지네이션 적용

최종 결과에 페이지네이션을 적용하여 반환합니다.

---

## Study List View

`view` 파라미터를 사용하면 사전 정의된 View를 통해 필터링, 정렬, 표시 필드를 제어할 수 있습니다.

### View 구조

```json
{
  "id": "research",
  "name": "Research View",
  "description": "연구용 Study 목록",
  "fields": [
    {
      "source": "dicom",
      "key": "StudyDate",
      "tag": "00080020",
      "label": "검사 날짜",
      "sortable": true,
      "filterable": true
    },
    {
      "source": "extension",
      "key": "subjectNo",
      "label": "피험자 번호",
      "sortable": true,
      "filterable": true
    }
  ],
  "default_sort": {
    "field": "StudyDate",
    "order": "desc"
  },
  "default_filters": {
    "Modality": "CT"
  }
}
```

### View 사용 예시

```bash
# "research" View 사용
curl -X GET "http://localhost:8080/api/me/dicom/studies?view=research&page=1&page_size=10" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 성능 최적화

### 1. 병렬 QIDO 요청

여러 프로젝트의 QIDO 요청을 병렬로 처리하여 응답 시간을 단축합니다.

**개선 전**: 10개 프로젝트 × 1초 = 10초
**개선 후**: max(10개 프로젝트) = 1초

### 2. 배치 Extension Fields 조회

모든 Study의 Extension Fields를 한 번의 쿼리로 조회합니다.

**개선 전**: 100개 Study × 2개 필드 × 10ms = 2초
**개선 후**: 1개 배치 쿼리 × 50ms = 0.05초

### 3. 중복 제거

HashSet을 사용하여 효율적으로 중복을 제거합니다.

---

## 권한 요구사항

### 필수 권한

- **인증된 사용자**: 로그인한 사용자만 접근 가능
- **프로젝트 멤버**: 조회하려는 프로젝트의 멤버여야 함

### 권한별 동작

| 역할 | 동작 |
|------|------|
| **일반 사용자** | 자신이 속한 프로젝트의 Study만 조회 가능 |
| **프로젝트 관리자** | 해당 프로젝트의 모든 Study 조회 가능 |
| **SUPER_ADMIN** | 모든 프로젝트의 Study 조회 가능 (단, `/api/admin/dicom/studies` 사용 권장) |

---

## 관련 API

- `GET /api/me/dicom/series` - 사용자의 모든 Series 조회
- `GET /api/me/dicom/studies/{study_uid}/series` - 특정 Study의 Series 조회
- `GET /api/admin/dicom/studies` - 관리자용 전체 Study 조회 (RBAC 필터링 없음)
- `GET /api/study-list-views` - Study List View 목록 조회
- `GET /api/study-list-views/{view_id}` - 특정 View 상세 조회

---

## 참고 사항

### Access Condition 우선순위

1. **프로젝트 Access Condition** (최우선)
2. **사용자 쿼리 파라미터**
3. **View 기본 필터**

### 페이지네이션 우선순위

1. **DICOMweb 표준** (`limit`, `offset`)
2. **일반 페이지네이션** (`page`, `page_size`)

### 날짜 형식

- **단일 날짜**: `20240715`
- **날짜 범위**: `20240101-20241231`
- **DICOM 표준**: `YYYYMMDD` 형식

---

## 예제 시나리오

### 시나리오 1: 연구원이 CT Study 조회

```bash
# 요청
GET /api/me/dicom/studies?modality=CT&study_date=20240101-20241231&page=1&page_size=20

# 동작
1. 사용자가 속한 프로젝트 조회 (예: 프로젝트 2, 5)
2. 각 프로젝트의 Access Condition 적용
3. 병렬로 QIDO 요청 실행
4. RBAC 필터링 수행
5. Extension Fields 추가
6. 중복 제거 및 페이지네이션 적용
```

### 시나리오 2: 의사가 승인된 리포트만 조회

```bash
# 요청
GET /api/me/dicom/studies?report_status=approved&page=1&page_size=10

# 동작
1. 모든 Study 조회
2. Extension Fields에서 report_status 필터링
3. "approved" 상태인 Study만 반환
```

---

## 문제 해결

### Q: 응답이 느립니다

**A**: 다음을 확인하세요:
- 프로젝트 수가 많은 경우 `project_id` 파라미터로 특정 프로젝트만 조회
- `page_size`를 줄여서 요청
- QIDO 서버 상태 확인

### Q: 일부 Study가 보이지 않습니다

**A**: 다음을 확인하세요:
- RBAC 권한 확인
- `project_data_access` 테이블에 할당되어 있는지 확인
- Access Condition이 너무 제한적이지 않은지 확인

### Q: Extension Fields가 표시되지 않습니다

**A**: 다음을 확인하세요:
- `view` 파라미터에 Extension Fields가 포함되어 있는지 확인
- 데이터베이스에 해당 데이터가 존재하는지 확인

---

## 변경 이력

| 날짜 | 버전 | 변경 내용 |
|------|------|----------|
| 2026-01-08 | 1.0.0 | 초기 문서 작성 |
| 2026-01-08 | 1.1.0 | 병렬 QIDO 요청 및 배치 Extension Fields 조회 추가 |

