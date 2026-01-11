# User Study List API

사용자가 접근 가능한 모든 프로젝트의 Study 목록을 통합 조회하는 API 문서입니다.

---

## Endpoint

```http
GET /api/me/dicom/studies
```

---

## 인증 / 권한

### 인증 방식
- JWT Bearer Token (`Authorization: Bearer <token>`)
- 또는 Keycloak Token (`X-Keycloak-Token: <token>`)

### 권한 처리
- 사용자가 속한 프로젝트의 Study만 조회 가능
- 프로젝트별 RBAC 평가 적용
- `project_data_access` 테이블 기반 접근 제어

---

## Request Parameters

### Query Parameters

| 파라미터 | 타입 | 필수 | 기본값 | 설명 |
|---------|------|------|--------|------|
| `project_id` | integer | ❌ | - | 특정 프로젝트만 필터링 (없으면 모든 프로젝트) |
| `page` | integer | ❌ | 1 | 페이지 번호 (1부터 시작) |
| `page_size` | integer | ❌ | 50 | 페이지당 항목 수 (1~200) |
| `modality` | string | ❌ | - | Modality 필터 (예: "CT", "MR") |
| `patient_id` | string | ❌ | - | Patient ID 필터 |
| `study_date` | string | ❌ | - | Study Date 범위 (예: "20240101-20241231") |
| `accession_number` | string | ❌ | - | Accession Number 필터 |
| `patient_name` | string | ❌ | - | Patient Name 필터 |

### 예제 요청

```http
GET /api/me/dicom/studies?project_id=2&page=1&page_size=10&modality=CT
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

---

## Response

### 성공 응답 (200 OK)

DICOMweb JSON 형식의 Study 배열을 반환합니다.

```json
[
  {
    "00080005": {"Value": ["ISO_IR 100"], "vr": "CS"},
    "00080020": {"Value": ["20240115"], "vr": "DA"},
    "00080030": {"Value": ["093012"], "vr": "TM"},
    "00080050": {"Value": ["ACC123456"], "vr": "SH"},
    "00080061": {"Value": ["CT"], "vr": "CS"},
    "00081030": {"Value": ["Chest CT"], "vr": "LO"},
    "00100010": {"Value": [{"Alphabetic": "DOE^JOHN"}], "vr": "PN"},
    "00100020": {"Value": ["P123456"], "vr": "LO"},
    "0020000D": {"Value": ["1.2.840.113619.2.55.3.604688433.1234"], "vr": "UI"},
    "00201206": {"Value": ["3"], "vr": "IS"},
    "00201208": {"Value": ["245"], "vr": "IS"}
  }
]
```

### 주요 DICOM 태그

| 태그 | 이름 | 설명 |
|------|------|------|
| `0020000D` | StudyInstanceUID | Study 고유 식별자 |
| `00080020` | StudyDate | Study 날짜 (YYYYMMDD) |
| `00080030` | StudyTime | Study 시간 (HHMMSS) |
| `00081030` | StudyDescription | Study 설명 |
| `00100010` | PatientName | 환자 이름 |
| `00100020` | PatientID | 환자 ID |
| `00080061` | ModalitiesInStudy | Study에 포함된 Modality 목록 |
| `00201206` | NumberOfStudyRelatedSeries | Series 개수 |
| `00201208` | NumberOfStudyRelatedInstances | Instance 개수 |

---

## 페이지네이션

### 동작 방식

**메모리 기반 페이지네이션**을 사용합니다:

1. QIDO 서버에서 충분한 데이터 가져오기 (limit ≥ 100)
2. 모든 프로젝트의 데이터를 통합
3. Study Date 기준 내림차순 정렬 (최신순)
4. 메모리에서 offset 계산: `offset = (page - 1) * page_size`
5. 요청된 페이지의 데이터만 반환: `결과[offset : offset + page_size]`

### 예제

전체 Study가 25개일 때:
- `page=1, page_size=10` → 1~10번째 Study (10개)
- `page=2, page_size=10` → 11~20번째 Study (10개)
- `page=3, page_size=10` → 21~25번째 Study (5개)
- `page=4, page_size=10` → 빈 배열 (0개)

### 왜 메모리 페이지네이션인가?

- QIDO 서버가 `offset` 파라미터를 제대로 지원하지 않음
- 여러 프로젝트의 데이터를 통합해야 하므로 서버 측 페이지네이션 불가능
- 정렬 및 중복 제거를 위해 전체 데이터 필요

---

## 오류 응답

### 401 Unauthorized

```json
{
  "error": "Invalid or missing authorization token"
}
```

### 400 Bad Request

```json
{
  "error": "Invalid parameter: page_size must be between 1 and 200"
}
```

---

## 내부 처리 흐름

1. **사용자 인증 및 프로젝트 조회**
   - JWT 토큰에서 사용자 ID 추출
   - 사용자가 속한 프로젝트 목록 조회
   - `project_id` 파라미터가 있으면 해당 프로젝트만 필터링

2. **프로젝트별 QIDO 호출 (병렬)**
   - 각 프로젝트의 Access Condition 적용
   - QIDO 서버에 `/rs/studies` 요청
   - Bearer 토큰 전달
   - `limit` 파라미터만 전달 (offset 제외)

3. **RBAC 필터링**
   - 각 Study에 대해 RBAC 평가
   - `project_data_access` 테이블 확인
   - 접근 가능한 Study만 선택

4. **통합 및 정렬**
   - 모든 프로젝트의 Study 통합
   - Study Date 기준 내림차순 정렬
   - 중복 제거 (StudyInstanceUID 기준)

5. **페이지네이션 적용**
   - 메모리에서 offset 계산
   - 요청된 페이지의 데이터만 반환

---

## 성능 최적화

### QIDO Limit 계산

단일 프로젝트:
```
limit = max(offset + page_size * 10, 100)
limit = min(limit, 500)
```

여러 프로젝트:
```
limit = max(offset + page_size * 프로젝트수 * 10, 100)
limit = min(limit, 500)
```

### 병렬 처리
- 여러 프로젝트의 QIDO 호출을 병렬로 처리
- `tokio::spawn`을 사용한 비동기 처리

---

## 구현 세부사항

### QIDO 파라미터 처리

**제거되는 파라미터:**
- `page` - 메모리 페이지네이션에서 사용
- `page_size` - 메모리 페이지네이션에서 사용
- `offset` - QIDO 서버가 지원하지 않음
- `report_status` - QIDO 파라미터가 아님

**QIDO에 전달되는 파라미터:**
- `limit` - 충분히 큰 값으로 설정
- `modality` - Modality 필터
- `patient_id` - Patient ID 필터
- `study_date` - Study Date 범위
- 기타 DICOM 검색 파라미터

### 파라미터 파싱

URL 쿼리 파라미터는 문자열로 전달되므로 파싱 필요:

```rust
let page = query.extra
    .get("page")
    .and_then(|v| {
        v.as_i64()
            .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
    })
    .unwrap_or(1)
    .max(1);
```

---

## 테스트 예제

### Python 테스트 스크립트

```python
import requests

# 로그인
login_resp = requests.post('http://localhost:8080/api/auth/login', json={
    'username': 'your-username',
    'password': 'your-password'
})
token = login_resp.json()['token']

# Study 목록 조회
resp = requests.get('http://localhost:8080/api/me/dicom/studies', params={
    'project_id': 2,
    'page': 1,
    'page_size': 10
}, headers={'Authorization': f'Bearer {token}'})

studies = resp.json()
print(f"반환된 Study 수: {len(studies)}")
```

---

## 관련 API

- `GET /api/me/dicom/series` - 사용자 Series 목록 조회
- `GET /api/me/dicom/studies/{study_uid}/series` - 특정 Study의 Series 조회
- `POST /api/v1/viewer/studies/meta` - Viewer Study Meta Batch API

---

## 변경 이력

### 2026-01-02
- 메모리 기반 페이지네이션 구현
- 문자열 파라미터 파싱 지원 추가
- QIDO에 offset 전달하지 않도록 수정
- QIDO limit 최적화

