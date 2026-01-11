# Viewer BFF API

## 개요

Viewer BFF (Backend-for-Frontend) API는 DICOM Viewer 애플리케이션을 위한 최적화된 API입니다.

### 주요 특징

- **Batch 조회**: 여러 Study/Series의 메타데이터를 한 번의 요청으로 조회
- **RBAC 통합**: 사용자 권한에 따른 자동 필터링
- **성능 최적화**: Viewer에 필요한 필드만 선택적으로 반환
- **BFF 패턴**: 기존 QIDO-RS Proxy를 재사용하여 구현

## API 엔드포인트

### 1. Study Meta Batch API

여러 Study의 메타데이터를 한 번에 조회합니다.

#### Request

```http
POST /api/v1/viewer/studies/meta
Content-Type: application/json
Authorization: Bearer <JWT_TOKEN>

{
  "study_uids": [
    "1.2.840.113619.2.55.3.604688433.1234",
    "1.2.840.113619.2.55.3.604688433.5678"
  ],
  "max_count": 20
}
```

#### Response

```json
{
  "studies": [
    {
      "study_uid": "1.2.840.113619.2.55.3.604688433.1234",
      "study_date": "20240115",
      "study_time": "093012",
      "study_description": "Chest CT",
      "patient_name": "DOE^JOHN",
      "patient_id": "P123456",
      "modalities_in_study": ["CT"],
      "number_of_series": 3,
      "number_of_instances": 245
    }
  ]
}
```

#### 파라미터

- `study_uids` (required): 조회할 StudyInstanceUID 목록
- `max_count` (optional): 최대 조회 개수 (기본값: 20, 최대: 100)

#### 응답 코드

- `200 OK`: 성공
- `400 Bad Request`: 잘못된 요청 (빈 배열 등)
- `401 Unauthorized`: 인증 실패
- `404 Not Found`: 접근 가능한 Study가 없음

---

### 2. Series Meta Batch API

여러 Series의 메타데이터를 한 번에 조회합니다.

#### Request

```http
POST /api/v1/viewer/series/meta
Content-Type: application/json
Authorization: Bearer <JWT_TOKEN>

{
  "series_uids": [
    "1.2.840.113619.2.55.3.604688433.1234.1",
    "1.2.840.113619.2.55.3.604688433.1234.2"
  ],
  "max_count": 50
}
```

#### Response

```json
{
  "series": [
    {
      "series_uid": "1.2.840.113619.2.55.3.604688433.1234.1",
      "study_uid": "1.2.840.113619.2.55.3.604688433.1234",
      "series_number": 1,
      "series_description": "Axial T1",
      "modality": "MR",
      "number_of_instances": 120,
      "series_date": "20240115",
      "series_time": "093012",
      "body_part_examined": "BRAIN",
      "protocol_name": "T1_MPRAGE"
    }
  ]
}
```

#### 파라미터

- `series_uids` (required): 조회할 SeriesInstanceUID 목록
- `max_count` (optional): 최대 조회 개수 (기본값: 50, 최대: 200)

#### 응답 코드

- `200 OK`: 성공
- `400 Bad Request`: 잘못된 요청 (빈 배열 등)
- `401 Unauthorized`: 인증 실패
- `404 Not Found`: 접근 가능한 Series가 없음

---

## 권한 제어

### RBAC 기반 필터링

- 사용자가 속한 프로젝트의 RBAC 규칙에 따라 자동 필터링
- `project_data_access` 테이블의 접근 권한 확인
- 접근 권한이 없는 Study/Series는 응답에서 제외

### 권한 검증 흐름

1. JWT 토큰에서 사용자 ID 추출
2. 사용자가 속한 프로젝트 목록 조회
3. 각 Study/Series에 대해:
   - RBAC 규칙 평가
   - `project_data_access` 확인
   - 둘 다 통과한 경우만 응답에 포함

---

## 사용 예시

### cURL

```bash
# Study Meta 조회
curl -X POST http://localhost:8080/api/v1/viewer/studies/meta \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "study_uids": ["1.2.840.113619.2.55.3.604688433.1234"],
    "max_count": 20
  }'

# Series Meta 조회
curl -X POST http://localhost:8080/api/v1/viewer/series/meta \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "series_uids": ["1.2.840.113619.2.55.3.604688433.1234.1"],
    "max_count": 50
  }'
```

### 테스트 스크립트

```bash
./pacs-server/test_viewer_api.sh YOUR_JWT_TOKEN
```

---

## 구현 세부사항

### 아키텍처

- **Controller**: `viewer_controller.rs`
- **DTO**: `viewer_dto.rs`
- **QIDO Client**: 기존 `Dcm4cheeQidoClient` 재사용
- **RBAC**: `DicomRbacEvaluatorImpl` 사용

### DICOMweb 매핑

DICOM 태그를 DTO 필드로 변환:

| DICOM Tag | VR | DTO Field |
|-----------|----|-----------| 
| (0020,000D) | UI | study_uid |
| (0020,000E) | UI | series_uid |
| (0008,0020) | DA | study_date |
| (0008,0021) | DA | series_date |
| (0008,0030) | TM | study_time |
| (0008,0031) | TM | series_time |
| (0008,1030) | LO | study_description |
| (0008,103E) | LO | series_description |
| (0010,0010) | PN | patient_name |
| (0010,0020) | LO | patient_id |
| (0008,0060) | CS | modality |
| (0008,0061) | CS | modalities_in_study |
| (0020,0011) | IS | series_number |
| (0020,1206) | IS | number_of_series |
| (0020,1208) | IS | number_of_instances |
| (0020,1209) | IS | number_of_instances (series) |
| (0018,0015) | CS | body_part_examined |
| (0018,1030) | LO | protocol_name |

---

## 참고 자료

- [DICOMweb QIDO-RS Specification](https://www.dicomstandard.org/using/dicomweb/query-qido-rs)
- [DICOM Standard Part 6 - Data Dictionary](https://dicom.nema.org/medical/dicom/current/output/chtml/part06/chapter_6.html)
- [BFF Pattern](https://samnewman.io/patterns/architectural/bff/)

