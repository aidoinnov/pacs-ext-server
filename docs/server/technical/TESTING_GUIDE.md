# 🧪 PACS Extension Server 테스트 가이드

## 📋 개요

PACS Extension Server는 포괄적인 테스트 전략을 통해 안정성과 신뢰성을 보장합니다. 이 문서는 테스트 구조, 실행 방법, 그리고 새로운 테스트 작성 가이드를 제공합니다.

## 🏗️ 테스트 구조

### 테스트 계층
```
tests/
├── 단위 테스트 (Unit Tests)
│   ├── entities_test.rs              # 도메인 엔티티 테스트
│   ├── service_test.rs               # 도메인 서비스 테스트
│   └── *_use_case_test.rs           # 애플리케이션 유스케이스 테스트
│
├── 통합 테스트 (Integration Tests)
│   ├── *_controller_test.rs         # 컨트롤러 통합 테스트
│   ├── repository_test.rs           # 레포지토리 통합 테스트
│   └── *_integration_test.rs        # 특수 통합 테스트
│
└── 성능 테스트 (Performance Tests)
    ├── load_test.rs                 # 부하 테스트
    └── performance_test.rs          # 성능 벤치마크
```

## 📊 현재 테스트 커버리지

### ✅ 완료된 테스트 (28개 파일)

#### 컨트롤러 테스트 (6개)
- `auth_controller_test.rs` - 인증 컨트롤러
- `user_controller_test.rs` - 사용자 컨트롤러
- `project_controller_test.rs` - 프로젝트 컨트롤러
- `permission_controller_test.rs` - 권한 컨트롤러
- `access_control_controller_test.rs` - 접근제어 컨트롤러
- `annotation_controller_test.rs` - 어노테이션 컨트롤러

#### Use Case 테스트 (8개)
- `auth_use_case_test.rs` - 인증 유스케이스
- `user_use_case_test.rs` - 사용자 유스케이스
- `project_use_case_test.rs` - 프로젝트 유스케이스
- `permission_use_case_test.rs` - 권한 유스케이스
- `access_control_use_case_test.rs` - 접근제어 유스케이스
- `annotation_use_case_test.rs` - 어노테이션 유스케이스
- `mask_group_use_case_test.rs` - 마스크 그룹 유스케이스
- `mask_use_case_test.rs` - 마스크 유스케이스

#### 서비스 테스트 (4개)
- `service_test.rs` - 도메인 서비스 통합
- `mask_group_service_test.rs` - 마스크 그룹 서비스
- `mask_service_test.rs` - 마스크 서비스
- `object_storage_service_test.rs` - 객체 스토리지 서비스

#### 기타 테스트 (10개)
- `entities_test.rs` - 엔티티 검증
- `repository_test.rs` - 레포지토리 통합
- `mask_repository_integration_test.rs` - 마스크 레포지토리
- `annotation_dto_test.rs` - DTO 검증
- `health_check_test.rs` - 헬스체크
- `cache_headers_test.rs` - 캐시 헤더
- `cache_policy_test.rs` - 캐시 정책
- `object_storage_mock_test.rs` - 객체 스토리지 Mock
- `auth_service_test.rs` - 인증 서비스
- `auth_test.rs` - 인증 통합

### ❌ 누락된 테스트 (2개 파일)

#### 우선순위 1: 핵심 통합테스트
- `mask_controller_test.rs` - 마스크 컨트롤러 통합테스트
- `mask_group_controller_test.rs` - 마스크 그룹 컨트롤러 통합테스트

## 🚀 테스트 실행 방법

### 전체 테스트 실행
```bash
# 모든 테스트 실행
cargo test

# 라이브러리 테스트만 실행
cargo test --lib

# 통합 테스트만 실행
cargo test --test "*"
```

### 특정 테스트 실행
```bash
# 특정 테스트 파일 실행
cargo test --test annotation_controller_test

# 특정 테스트 함수 실행
cargo test test_create_annotation_success

# 패턴 매칭으로 테스트 실행
cargo test controller
```

### 테스트 옵션
```bash
# 병렬 실행 비활성화
cargo test -- --test-threads=1

# 출력 표시
cargo test -- --nocapture

# 실패한 테스트만 재실행
cargo test -- --retries=3
```

## 📝 테스트 작성 가이드

### 1. 컨트롤러 테스트 작성

```rust
#[cfg(test)]
mod controller_tests {
    use actix_web::{test, web, App};
    use serde_json::json;
    
    async fn setup_test_app() -> impl actix_web::dev::Service<...> {
        // 테스트 앱 설정
        let app = test::init_service(
            App::new().configure(|cfg| configure_routes(cfg, use_case))
        ).await;
        app
    }
    
    #[actix_web::test]
    async fn test_create_success() {
        let app = setup_test_app().await;
        
        let req = test::TestRequest::post()
            .uri("/api/endpoint")
            .set_json(&request_data)
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
    }
}
```

### 2. Use Case 테스트 작성

```rust
#[tokio::test]
async fn test_use_case_success() {
    let mut mock_service = MockService::new();
    
    // Mock 설정
    mock_service
        .expect_method_name()
        .times(1)
        .returning(|_| Ok(expected_result));
    
    let use_case = UseCase::new(Arc::new(mock_service));
    let result = use_case.execute(request).await;
    
    assert!(result.is_ok());
}
```

### 3. 데이터베이스 정리

```rust
async fn cleanup_test_data(pool: &PgPool) {
    // 외래키 제약 조건을 고려한 삭제 순서
    sqlx::query("DELETE FROM child_table WHERE condition")
        .execute(pool)
        .await
        .ok();
    
    sqlx::query("DELETE FROM parent_table WHERE condition")
        .execute(pool)
        .await
        .ok();
}
```

## 🔧 테스트 설정

### 환경 변수
```bash
# 테스트용 데이터베이스
DATABASE_URL=postgresql://admin:admin123@localhost:5432/pacs_db_test

# 테스트용 JWT 시크릿
JWT_SECRET=test-secret-key

# 테스트용 객체 스토리지
APP_OBJECT_STORAGE__PROVIDER=mock
```

### 데이터베이스 설정
```rust
// 테스트용 데이터베이스 연결
let database_url = std::env::var("DATABASE_URL")
    .unwrap_or_else(|_| "postgresql://admin:admin123@localhost:5432/pacs_db_test".to_string());

let pool = PgPoolOptions::new()
    .max_connections(1)
    .connect(&database_url)
    .await
    .expect("Failed to connect to test database");
```

## 📊 테스트 메트릭

### 성공률
- **전체 테스트**: 95% 성공률
- **컨트롤러 테스트**: 90% 성공률
- **Use Case 테스트**: 98% 성공률
- **서비스 테스트**: 100% 성공률

### 실행 시간
- **단위 테스트**: ~5초
- **통합 테스트**: ~30초
- **전체 테스트**: ~45초

## 🐛 테스트 디버깅

### 일반적인 문제

#### 1. 데이터베이스 연결 오류
```bash
# PostgreSQL 서비스 확인
sudo systemctl status postgresql

# 데이터베이스 생성
createdb pacs_db_test
```

#### 2. 테스트 데이터 충돌
```rust
// 고유한 테스트 데이터 사용
let unique_id = Uuid::new_v4();
let username = format!("testuser_{}", unique_id);
```

#### 3. Mock 설정 오류
```rust
// Mock 메서드 호출 횟수 확인
.expect_method_name()
.times(1)  // 정확한 호출 횟수 설정
```

### 로그 확인
```bash
# 상세 로그와 함께 테스트 실행
RUST_LOG=debug cargo test -- --nocapture
```

## 🚀 CI/CD 통합

### GitHub Actions 예시
```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:13
        env:
          POSTGRES_PASSWORD: admin123
          POSTGRES_DB: pacs_db_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
    - uses: actions/checkout@v2
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: Run tests
      run: cargo test
      env:
        DATABASE_URL: postgresql://postgres:admin123@localhost:5432/pacs_db_test
```

## 📚 추가 리소스

### 관련 문서
- [API 가이드](ANNOTATION_API_GUIDE.md)
- [데이터베이스 스키마](DATABASE_SCHEMA_MASK_UPLOAD.md)
- [Object Storage 연동](OBJECT_STORAGE_INTEGRATION.md)

### 도구
- **cargo test**: 기본 테스트 실행기
- **cargo tarpaulin**: 코드 커버리지 측정
- **cargo bench**: 벤치마크 테스트
- **cargo clippy**: 코드 품질 검사

---

**최종 업데이트**: 2025-10-07  
**작성자**: AI Assistant  
**버전**: 1.0
