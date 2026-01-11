# View Selection 테스트 요약

## 📋 테스트 파일 목록

### 단위 테스트 (Unit Tests)

1. **`view_selection_entity_test.rs`** (4개 테스트)
   - `test_view_selection_new` - ViewSelection 생성 테스트
   - `test_view_selection_is_expired` - 만료 확인 테스트
   - `test_view_selection_extend_ttl` - TTL 연장 테스트
   - `test_selected_series_equality` - SelectedSeries 동등성 테스트

2. **`view_selection_use_case_test.rs`** (7개 테스트)
   - `test_create_selection_success` - Selection 생성 성공
   - `test_create_selection_empty_series` - 빈 Series 목록 검증
   - `test_get_selection_success` - Selection 조회 성공
   - `test_get_selection_not_found` - 존재하지 않는 Selection 조회
   - `test_extend_ttl_success` - TTL 연장 성공
   - `test_extend_ttl_not_found` - 존재하지 않는 Selection TTL 연장
   - `test_delete_selection_success` - Selection 삭제 성공

### 통합 테스트 (Integration Tests)

3. **`view_selection_repository_integration_test.rs`** (6개 테스트)
   - `test_save_and_find_selection` - Redis 저장 및 조회
   - `test_find_nonexistent_selection` - 존재하지 않는 Selection 조회
   - `test_extend_ttl` - TTL 연장 (Redis)
   - `test_extend_ttl_not_found` - 존재하지 않는 Selection TTL 연장
   - `test_delete_selection` - Selection 삭제 (Redis)
   - `test_expired_selection_auto_delete` - 만료된 Selection 자동 삭제

4. **`view_selection_controller_integration_test.rs`** (5개 테스트)
   - `test_create_view_selection_success` - API: Selection 생성
   - `test_get_view_selection_success` - API: Selection 조회
   - `test_get_view_selection_not_found` - API: 존재하지 않는 Selection 조회
   - `test_delete_view_selection_success` - API: Selection 삭제
   - `test_create_view_selection_empty_series` - API: 빈 Series 목록 검증

### E2E 테스트 (End-to-End Tests)

5. **`view_selection_e2e_test.rs`** (2개 테스트)
   - `test_view_selection_full_workflow` - 전체 플로우 (생성 → 조회 → TTL 연장 → 삭제)
   - `test_multi_study_series_selection` - 멀티 Study/Series 선택 시나리오

## 📊 테스트 통계

- **총 테스트 수**: 24개
- **단위 테스트**: 11개
- **통합 테스트**: 11개
- **E2E 테스트**: 2개

## 🧪 테스트 실행 방법

### 전체 테스트 실행
```bash
cargo test --test view_selection
```

### 특정 테스트 파일 실행
```bash
# 단위 테스트
cargo test --test view_selection_entity_test
cargo test --test view_selection_use_case_test

# 통합 테스트 (Redis 필요)
cargo test --test view_selection_repository_integration_test -- --ignored
cargo test --test view_selection_controller_integration_test -- --ignored

# E2E 테스트 (Redis 필요)
cargo test --test view_selection_e2e_test -- --ignored
```

### 특정 테스트 함수 실행
```bash
cargo test test_view_selection_new
cargo test test_create_selection_success
```

## ⚠️ 주의사항

1. **Redis 필요**: 통합 테스트와 E2E 테스트는 Redis가 실행 중이어야 합니다.
   - Redis가 없으면 테스트가 자동으로 스킵됩니다.
   - `#[ignore]` 어노테이션이 있는 테스트는 `--ignored` 플래그로 실행해야 합니다.

2. **환경 변수**: 
   - `APP_REDIS__URL` 또는 `REDIS_URL` 환경 변수가 설정되어 있어야 합니다.
   - 기본값: `redis://localhost:6379`

3. **테스트 격리**: 
   - 각 테스트는 독립적으로 실행됩니다.
   - Redis 키는 `test_view_selection:` 접두사를 사용하여 격리됩니다.
   - 테스트 후 자동으로 정리됩니다.

## ✅ 테스트 커버리지

### Domain 계층
- ✅ ViewSelection 엔티티 (생성, 만료 확인, TTL 연장)
- ✅ SelectedSeries 엔티티 (동등성)

### Application 계층
- ✅ ViewSelectionUseCase (생성, 조회, TTL 연장, 삭제)
- ✅ DTO 변환

### Infrastructure 계층
- ✅ ViewSelectionRepository (Redis 저장, 조회, TTL 연장, 삭제)
- ✅ 만료된 Selection 자동 삭제

### Presentation 계층
- ✅ ViewSelectionController API 엔드포인트
- ✅ 에러 처리 (404, 400)

### E2E 시나리오
- ✅ 전체 플로우 (생성 → 조회 → 삭제)
- ✅ 멀티 Study/Series 선택

## 🐍 Python E2E 시나리오 테스트

### `test_view_selection_e2e.py` (806줄)

**기본 테스트 (8개)**
- `test_create_selection_success` - Selection 생성 성공
- `test_create_selection_empty_series` - 빈 Series 목록 검증
- `test_get_selection_not_found` - 존재하지 않는 Selection 조회
- `test_selection_id_format` - Selection ID 형식 검증
- `test_multi_study_series_selection` - 멀티 Study/Series 선택
- `test_full_workflow` - 전체 플로우 (생성 → 조회 → TTL 연장 → 삭제)
- `test_large_series_list` - 대량 Series 선택 (10개)
- `test_unauthorized_access` - 인증 없이 접근 시도

**고급 시나리오 테스트 (4개)**
- `test_selection_persistence` - Selection 지속성 (여러 번 조회)
- `test_real_world_scenario` - 실제 Study/Series 데이터 사용
- `scenario_viewer_session_workflow` - Viewer Session 전체 워크플로우
- `scenario_multi_user_selection` - 여러 사용자 동시 Selection 생성
- `scenario_url_sharing` - URL 공유 및 상태 재현

**총 Python E2E 테스트: 12개**

### 실행 방법

```bash
# Python E2E 테스트 실행
python3 test_view_selection_e2e.py

# 또는 실행 권한이 있으면
./test_view_selection_e2e.py
```

### 요구사항

- Python 3.7+
- `requests` 라이브러리: `pip install requests`
- 서버가 `http://localhost:8080`에서 실행 중이어야 함
- Redis가 실행 중이어야 함 (Selection 저장용)
- 테스트 사용자 자동 생성 (또는 기존 사용자 사용 가능)

## 🚀 다음 단계

1. **성능 테스트**: 대량의 Selection 생성/조회 성능 테스트
2. **동시성 테스트**: 여러 사용자가 동시에 Selection 생성/조회
3. **TTL 만료 테스트**: 실제 시간 경과에 따른 만료 확인
4. **권한 검증 테스트**: Series 접근 권한 검증 통합
5. **부하 테스트**: 동시 100+ Selection 생성/조회

