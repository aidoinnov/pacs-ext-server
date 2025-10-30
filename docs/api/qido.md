## DICOMweb QIDO API (Gateway)

이 문서는 PACS Extension Gateway의 QIDO API를 정의합니다. 게이트웨이는 DICOMweb 호환 요청/응답을 유지하며, RBAC 필터링을 사후 적용(post-filter)합니다. 사용자 쿼리 파라미터는 RBAC에서 파생된 파라미터보다 우선합니다.

### Base Path
- Base: `/api/dicom`

### 공통 사항
- Authorization: `Bearer <Keycloak Access Token>` (필수)
- Content-Type: `application/json`
- Upstream: 요청은 Dcm4chee QIDO-RS로 전달되며, 응답 JSON을 그대로 반환하되 RBAC으로 필터링됩니다.
- 페이지네이션: `page`, `page_size`를 지원하며, 사용자 입력에 `limit`/`offset`이 없는 경우 `limit = page_size`, `offset = (page-1)*page_size`로 변환되어 upstream으로 전달됩니다.

### 지원 필터 (사용자 alias → DICOMweb tag)
- `modality` → `00080060` (Modality)
- `patient_id` → `00100020` (PatientID)
- `study_date` → `00080020` (StudyDate)
  - 단일 `YYYYMMDD`, 범위 `YYYYMMDD-YYYYMMDD` 지원
- `accession_number` → `00080050` (AccessionNumber)
- `patient_name` → `00100010` (PatientName)

운영자/연산자:
- EQ(정확히 일치), CONTAINS(부분일치) 맵핑 지원. 사용자가 alias로 전달하면 게이트웨이가 DICOMweb 파라미터로 변환합니다.

정렬/기타:
- 기본적으로 DICOMweb 표준 파라미터를 그대로 전달합니다. 별도 정렬 파라미터가 필요한 경우 추후 확장.

RBAC 병합 규칙
- 사용자 입력 우선: 같은 태그에 대해 사용자 입력이 있으면 RBAC 규칙에서 파생된 값보다 우선합니다.
- RBAC 파생 파라미터는 사용자가 지정하지 않은 태그에 한해 추가됩니다.

에러 응답
- 401: 토큰 누락/유효하지 않음
- 403: RBAC 결과상 접근 불가
- 5xx: 상류(dcm4chee) 오류 또는 게이트웨이 내부 오류

---

### GET /api/dicom/studies
설명: Study 목록을 QIDO로 조회합니다.

Query Parameters:
- 필터: `modality`, `patient_id`, `study_date`, `accession_number`, `patient_name`
- 페이지네이션: `page` (기본 1), `page_size` (기본 50)
- DICOMweb native: `limit`, `offset` (사용자가 직접 지정 시 그대로 사용)

Request 예시:
```
GET /api/dicom/studies?modality=CT&study_date=20240101-20241231&page=1&page_size=25
Authorization: Bearer <token>
```

Response (요약 예시):
```json
[
  {
    "0020000D": { "vr": "UI", "Value": ["1.2.840.113619.2.55.3.283116435.780.1730234.1"] },
    "00080060": { "vr": "CS", "Value": ["CT"] },
    "00100020": { "vr": "LO", "Value": ["PAT123"] },
    "00080020": { "vr": "DA", "Value": ["20240715"] }
  }
]
```

---

### GET /api/dicom/studies/{study_uid}/series
설명: 지정한 Study 내 Series 목록을 조회합니다.

Path Parameters:
- `study_uid` (필수)

Query Parameters:
- 필터: `modality`, `patient_id`(일반적으로 study-level), `study_date`
- 페이지네이션: `page`, `page_size` 또는 `limit`, `offset`

Request 예시:
```
GET /api/dicom/studies/1.2.3.4/series?modality=CT&page=2&page_size=50
Authorization: Bearer <token>
```

Response (요약 예시):
```json
[
  {
    "0020000E": { "vr": "UI", "Value": ["1.2.3.4.5"] },
    "00080060": { "vr": "CS", "Value": ["CT"] }
  }
]
```

---

### GET /api/dicom/studies/{study_uid}/series/{series_uid}/instances
설명: 지정한 Series 내 Instance 목록을 조회합니다.

Path Parameters:
- `study_uid` (필수)
- `series_uid` (필수)

Query Parameters:
- 필터: `modality`(일반적으로 series-level), `patient_id`, `study_date`
- 페이지네이션: `page`, `page_size` 또는 `limit`, `offset`

Request 예시:
```
GET /api/dicom/studies/1.2.3.4/series/1.2.3.4.5/instances?page=1&page_size=100
Authorization: Bearer <token>
```

Response (요약 예시):
```json
[
  {
    "00080018": { "vr": "UI", "Value": ["1.2.3.4.5.6"] }
  }
]
```

---

### 동작 요약
- 게이트웨이 파라미터 병합: 사용자 입력 > RBAC 파생값
- 페이지네이션 변환: `limit`/`offset` 미지정 시 `page`/`page_size` 기반으로 변환
- 응답: DICOMweb QIDO JSON 포맷 그대로 반환, 단 RBAC으로 미허용 항목 제거

### 테스트 및 호환성
- 단위 테스트: 파라미터 파싱, 날짜 형식, 병합 우선순위 확인
- 모킹 통합 테스트: mock upstream로 `limit`/`offset` 및 필터 전달 확인
- 실서버 연동: Keycloak 토큰 릴레이로 dcm4chee 보호 자원 접근 확인
