# 📚 PACS Extension Server 문서

## 📋 개요
PACS Extension Server 프로젝트의 모든 문서를 체계적으로 정리한 디렉토리입니다.

## 🗂️ 문서 구조

### 📋 TODO 문서 (`/todo/`)
프로젝트 진행 상황과 남은 작업들을 정리한 문서들입니다.

- **[MASK_UPLOAD_TODO.md](../MASK_UPLOAD_TODO.md)** - 마스크 업로드 시스템 TODO
- **[implementation_plan.md](todo/implementation_plan.md)** - 구현 계획서
- **[design.md](todo/design.md)** - 시스템 설계 문서
- **[worklist.md](todo/worklist.md)** - 작업 목록
- **[code_implementation_guide.md](todo/code_implementation_guide.md)** - 코드 구현 가이드
- **[implementation_checklist.md](todo/implementation_checklist.md)** - 구현 체크리스트

### 🔧 기술 문서 (`/technical/`)
구현된 기능들의 상세한 기술 문서들입니다.

#### 🗄️ 데이터베이스 관련
- **[DATABASE_SCHEMA_MASK_UPLOAD.md](technical/DATABASE_SCHEMA_MASK_UPLOAD.md)** - 마스크 업로드 데이터베이스 스키마
- **[REPOSITORY_IMPLEMENTATION.md](technical/REPOSITORY_IMPLEMENTATION.md)** - Repository 구현

#### ☁️ Object Storage 관련
- **[OBJECT_STORAGE_INTEGRATION.md](technical/OBJECT_STORAGE_INTEGRATION.md)** - Object Storage 연동
- **[AWS_S3_INTEGRATION_GUIDE.md](technical/AWS_S3_INTEGRATION_GUIDE.md)** - AWS S3 연동 가이드
- **[AWS_SETUP_GUIDE.md](technical/AWS_SETUP_GUIDE.md)** - AWS 설정 가이드

#### 📋 API 및 DTO 관련
- **[DTO_DESIGN_MASK_UPLOAD.md](technical/DTO_DESIGN_MASK_UPLOAD.md)** - DTO 설계
- **[ANNOTATION_API_GUIDE.md](technical/ANNOTATION_API_GUIDE.md)** - 어노테이션 API 가이드

#### 🌐 웹 서버 관련
- **[CORS_DEVELOPMENT_GUIDE.md](technical/CORS_DEVELOPMENT_GUIDE.md)** - CORS 개발 가이드
- **[CACHE_HEADERS.md](technical/CACHE_HEADERS.md)** - 캐시 헤더 구현
- **[CACHE_REVIEW.md](technical/CACHE_REVIEW.md)** - 캐시 구현 검토

#### 📊 성능 및 최적화
- **[TRANSACTION_OPTIMIZATION.md](technical/TRANSACTION_OPTIMIZATION.md)** - 트랜잭션 최적화
- **[TRANSACTION_REVIEW_FINAL.md](technical/TRANSACTION_REVIEW_FINAL.md)** - 트랜잭션 최종 검토

#### 📝 프로젝트 관리
- **[CHANGELOG.md](technical/CHANGELOG.md)** - 변경 이력
- **[TECHNICAL_DOCUMENTATION_TODO.md](technical/TECHNICAL_DOCUMENTATION_TODO.md)** - 기술 문서 TODO
- **[MASK_UPLOAD_SYSTEM_IMPLEMENTATION.md](technical/MASK_UPLOAD_SYSTEM_IMPLEMENTATION.md)** - 마스크 업로드 시스템 구현

## 🚀 주요 기능별 문서

### 1. 어노테이션 시스템
- **API 가이드**: [ANNOTATION_API_GUIDE.md](technical/ANNOTATION_API_GUIDE.md)
- **데이터베이스 스키마**: 기존 `annotation_annotation` 테이블 사용

### 2. 마스크 업로드 시스템 (개발 중)
- **전체 TODO**: [MASK_UPLOAD_TODO.md](../MASK_UPLOAD_TODO.md)
- **데이터베이스 스키마**: [DATABASE_SCHEMA_MASK_UPLOAD.md](technical/DATABASE_SCHEMA_MASK_UPLOAD.md)
- **Repository 구현**: [REPOSITORY_IMPLEMENTATION.md](technical/REPOSITORY_IMPLEMENTATION.md)
- **Object Storage 연동**: [OBJECT_STORAGE_INTEGRATION.md](technical/OBJECT_STORAGE_INTEGRATION.md)
- **DTO 설계**: [DTO_DESIGN_MASK_UPLOAD.md](technical/DTO_DESIGN_MASK_UPLOAD.md)

### 3. 웹 서버 기능
- **CORS 설정**: [CORS_DEVELOPMENT_GUIDE.md](technical/CORS_DEVELOPMENT_GUIDE.md)
- **캐시 헤더**: [CACHE_HEADERS.md](technical/CACHE_HEADERS.md)
- **성능 최적화**: [TRANSACTION_OPTIMIZATION.md](technical/TRANSACTION_OPTIMIZATION.md)

## 📊 현재 진행 상황

### ✅ 완료된 기능
- [x] 어노테이션 시스템 (CRUD, API, Swagger)
- [x] 사용자 인증 및 권한 관리
- [x] 프로젝트 관리 시스템
- [x] CORS 설정 및 캐시 헤더
- [x] 마스크 업로드 데이터베이스 스키마
- [x] Object Storage 연동 (S3/MinIO)
- [x] Repository 구현체
- [x] DTO 설계 및 구현

### 🚧 진행 중인 기능
- [ ] 마스크 업로드 Use Case 구현
- [ ] 마스크 업로드 Service 구현
- [ ] 마스크 업로드 API 엔드포인트
- [ ] 마스크 업로드 컨트롤러

### 📋 예정된 기능
- [ ] 마스크 업로드 통합 테스트
- [ ] 성능 테스트 및 최적화
- [ ] 모니터링 및 로깅 시스템

## 🔧 개발 환경 설정

### 필수 요구사항
- Rust 1.70+
- PostgreSQL 13+
- Redis (선택사항)

### 환경 변수 설정
```bash
# 데이터베이스
DATABASE_URL=postgresql://user:password@localhost:5432/pacs_db

# JWT
JWT_SECRET=your-secret-key
JWT_EXPIRATION_HOURS=24

# Object Storage (마스크 업로드용)
APP_OBJECT_STORAGE__PROVIDER=s3
APP_OBJECT_STORAGE__BUCKET_NAME=pacs-masks
APP_OBJECT_STORAGE__REGION=us-east-1
APP_OBJECT_STORAGE__ACCESS_KEY=your-access-key
APP_OBJECT_STORAGE__SECRET_KEY=your-secret-key

# CORS
CORS_ENABLED=true
CORS_ALLOWED_ORIGINS=["http://localhost:3000"]
```

## 🚀 실행 방법

### 1. 데이터베이스 마이그레이션
```bash
sqlx migrate run
```

### 2. 서버 실행
```bash
cargo run
```

### 3. API 문서 확인
- Swagger UI: http://localhost:8080/swagger-ui/
- Health Check: http://localhost:8080/health

## 📚 참고 자료

### Rust 관련
- [Actix Web 문서](https://actix.rs/docs/)
- [SQLx 문서](https://docs.rs/sqlx/latest/sqlx/)
- [Serde 문서](https://serde.rs/)

### 데이터베이스 관련
- [PostgreSQL 문서](https://www.postgresql.org/docs/)
- [SQLx 마이그레이션](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)

### Object Storage 관련
- [AWS S3 Rust SDK](https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/)
- [MinIO 문서](https://docs.min.io/)

## 🤝 기여 방법

1. 이슈 생성 또는 기존 이슈 확인
2. 기능 브랜치 생성 (`git checkout -b feature/amazing-feature`)
3. 변경사항 커밋 (`git commit -m 'Add amazing feature'`)
4. 브랜치에 푸시 (`git push origin feature/amazing-feature`)
5. Pull Request 생성

## 📝 문서 작성 가이드

### 문서 구조
- **제목**: 이모지와 함께 명확한 제목
- **개요**: 문서의 목적과 범위
- **상세 내용**: 구현 세부사항
- **예시 코드**: 실제 사용 예시
- **참고 자료**: 관련 링크

### 코드 블록
```rust
// Rust 코드 예시
pub struct Example {
    pub field: String,
}
```

```sql
-- SQL 예시
SELECT * FROM table_name;
```

---
**최종 업데이트**: 2025-10-07  
**작성자**: AI Assistant  
**버전**: 1.0
