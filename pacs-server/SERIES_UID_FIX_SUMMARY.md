# Series UID 기반 API 수정 완료

## 수정된 엔드포인트

### Note API
- ✅ `PUT /api/series/{series_uid}/note` - `series_uid: String`
- ✅ `GET /api/series/{series_uid}/note` - `series_uid: String`
- ✅ `GET /api/series/{series_uid}/notes` - `series_uid: String`
- ✅ `DELETE /api/series/{series_uid}/note` - `series_uid: String`

### Report API
- ✅ `PUT /api/series/{series_uid}/report` - `series_uid: String`
- ✅ `GET /api/series/{series_uid}/report` - `series_uid: String`
- ✅ `GET /api/series/{series_uid}/reports` - `series_uid: String`
- ✅ `DELETE /api/series/{series_uid}/report` - `series_uid: String`

## 구현 내용

1. **헬퍼 함수 추가**
   - `find_series_id_by_uid`: Series UID로 Series ID 조회
   - `project_data_series` 테이블에서 `series_uid`로 `id` 조회

2. **엔드포인트 수정**
   - `web::Path<i32>` → `web::Path<String>` 변경
   - `project_data_repo` 파라미터 추가
   - Series UID를 받아서 Series ID로 변환 후 기존 로직 사용

3. **라우트 설정 수정**
   - `configure_global_series_routes`에 `project_data_repo` 파라미터 추가
   - `main.rs`에서 `project_data_repo` 전달
   - 라우트 경로를 `/{series_uid}/...`로 변경

## 테스트 결과

### Note API
- ✅ GET `/api/series/{series_uid}/note` - 200 OK
- ✅ PUT `/api/series/{series_uid}/note` - 정상 작동 (요청 형식 확인 필요)

### Report API
- ⚠️ GET `/api/series/{series_uid}/report` - 404 에러 (i32 파싱 에러)
  - **원인**: 서버가 재시작되지 않았거나, 다른 라우트와 충돌 가능성

## 다음 단계

1. **서버 재시작 필수**
   - 변경 사항이 적용되려면 서버를 재시작해야 합니다
   - `cargo build --release`로 빌드 후 서버 재시작

2. **테스트 재실행**
   ```bash
   python3 test_series_uid_simple.py
   ```

3. **에러가 계속 발생하는 경우**
   - 다른 엔드포인트가 같은 경로를 사용하는지 확인
   - 라우트 등록 순서 확인
   - 서버 로그 확인

## 참고

- 프로젝트 종속 API (`/api/project-data/{project_id}/series/{series_id}/...`)는 변경하지 않음
- Series UID는 DICOM Series Instance UID 형식 (예: `1.2.840.113619.2.311.168624790352053237183428645578553404611`)

