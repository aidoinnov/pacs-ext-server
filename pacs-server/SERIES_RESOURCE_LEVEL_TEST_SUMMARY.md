# Series API resource_level 필터링 및 페이지네이션 테스트 요약

## 작성된 테스트

### 1. 단위 테스트
**파일**: `tests/dicom_gateway_series_unit_test.rs`

- 쿼리 구조 검증
  - `resource_level='SERIES'` 케이스 쿼리 구조 확인
  - `resource_level='STUDY'` 케이스 쿼리 구조 확인
  - UNION 쿼리 구조 확인

**실행 방법**:
```bash
cargo test dicom_gateway_series_unit_test --lib
```

### 2. 통합 테스트
**파일**: `tests/dicom_gateway_series_resource_level_test.rs`

- `get_allowed_series_uids` 함수 직접 테스트
- 실제 DB를 사용한 통합 테스트
- resource_level별 필터링 검증

**실행 방법**:
```bash
cargo test dicom_gateway_series_resource_level_test --test dicom_gateway_series_resource_level_test -- --ignored
```

### 3. 통합 테스트 (컨트롤러)
**파일**: `tests/dicom_gateway_series_integration_test.rs`

- 실제 서버 및 DB를 사용한 통합 테스트
- API 엔드포인트 테스트

**실행 방법**:
```bash
cargo test dicom_gateway_series_integration_test --test dicom_gateway_series_integration_test -- --ignored
```

### 4. E2E 테스트 (Python)
**파일**: `test_series_resource_level_e2e.py`

#### 테스트 항목

1. **resource_level='SERIES' 필터링 테스트**
   - project_data에 5개 레코드가 있을 때 5개 Series만 반환되는지 확인
   - 중복 확인

2. **페이지네이션 테스트**
   - page_size에 따라 올바른 개수 반환
   - 페이지 간 중복 없음
   - 전체 Series와 페이지네이션 결과 일치

3. **다양한 페이지 크기 테스트**
   - page_size=1, 2, 3, 5, 10, 20 등 다양한 크기 테스트

4. **엣지 케이스 테스트**
   - page=0 처리
   - page_size=0 처리
   - 음수 page 처리
   - 매우 큰 page_size 처리

**실행 방법**:
```bash
python3 test_series_resource_level_e2e.py
```

## 테스트 전 준비사항

1. **서버 재시작 필수**
   ```bash
   # 변경사항 적용을 위해 서버 재시작
   cargo build --release
   # 서버 재시작
   ```

2. **DB 터널 확인**
   ```bash
   ./scripts/start-db-tunnels.sh
   ```

3. **환경 변수 확인**
   - `.env` 파일의 DB 연결 정보 확인
   - Keycloak 인증 정보 확인

## 예상 결과

### 수정 전
- Series 개수: 11개 (잘못됨 - study의 모든 series 반환)
- 페이지네이션: 작동하지 않음

### 수정 후
- Series 개수: 5개 (올바름 - resource_level='SERIES'인 경우 series_id로 직접 조회)
- 페이지네이션: 정상 작동 (필터링 후 메모리에서 페이지네이션 적용)

## 테스트 실행 순서

1. **서버 재시작** (변경사항 적용)
2. **단위 테스트 실행**
   ```bash
   cargo test dicom_gateway_series_unit_test --lib
   ```
3. **Python E2E 테스트 실행**
   ```bash
   python3 test_series_resource_level_e2e.py
   ```
4. **통합 테스트 실행** (선택사항)
   ```bash
   cargo test dicom_gateway_series_resource_level_test --test dicom_gateway_series_resource_level_test -- --ignored
   ```

## 참고

- 현재 테스트가 실패하는 이유: 서버가 재시작되지 않아 변경사항이 적용되지 않음
- 서버 재시작 후 테스트를 다시 실행하면 통과할 것으로 예상됨

