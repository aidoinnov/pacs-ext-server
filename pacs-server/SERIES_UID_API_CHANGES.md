# Series UID 기반 API 변경 사항

## 변경 목적
DICOM 데이터를 식별할 때 Primary Key (i32) 대신 DICOM UID를 사용하도록 변경하여 DICOMweb과의 통일성을 확보합니다.

## 변경된 엔드포인트

### 전역 Series API (Note)

#### 변경 전
- `PUT /api/series/{series_id}/note` - `series_id: i32`
- `GET /api/series/{series_id}/note` - `series_id: i32`
- `GET /api/series/{series_id}/notes` - `series_id: i32`
- `DELETE /api/series/{series_id}/note` - `series_id: i32`

#### 변경 후
- `PUT /api/series/{series_uid}/note` - `series_uid: String` (DICOM Series Instance UID)
- `GET /api/series/{series_uid}/note` - `series_uid: String`
- `GET /api/series/{series_uid}/notes` - `series_uid: String`
- `DELETE /api/series/{series_uid}/note` - `series_uid: String`

## 구현 내용

### 1. 헬퍼 함수 추가
```rust
async fn find_series_id_by_uid(
    series_uid: &str,
    project_data_repo: &ProjectDataRepositoryImpl,
) -> Result<i32, ServiceError>
```
- Series UID로 Series ID를 조회하는 함수
- `project_data_series` 테이블에서 `series_uid`로 `id`를 찾음

### 2. 엔드포인트 수정
- 모든 전역 Series Note 엔드포인트에서:
  - `web::Path<i32>` → `web::Path<String>` 변경
  - `project_data_repo` 파라미터 추가
  - Series UID를 받아서 Series ID로 변환 후 기존 로직 사용

### 3. 라우트 설정 수정
- `configure_global_series_routes`에 `project_data_repo` 파라미터 추가
- `main.rs`에서 `project_data_repo` 전달

## 사용 예시

### 변경 전
```bash
PUT /api/series/123/note
```

### 변경 후
```bash
PUT /api/series/1.2.840.113619.2.311.168624790352053237183428645578553404611/note
```

## 에러 처리

- Series UID가 존재하지 않는 경우: `404 Not Found`
- DB 조회 실패: `500 Internal Server Error`

## 참고 사항

- 프로젝트 종속 API (`/api/project-data/{project_id}/series/{series_id}/note`)는 변경하지 않음
  - 프로젝트 컨텍스트 내에서는 여전히 Series ID 사용
- Report API는 아직 변경하지 않음 (향후 변경 예정)

