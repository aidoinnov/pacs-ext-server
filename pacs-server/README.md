# 🏥 PACS Extension Server

## 📋 개요
PACS (Picture Archiving and Communication System) Extension Server는 의료 영상 어노테이션 및 마스크 업로드 기능을 제공하는 Rust 기반 웹 서버입니다.

## ✨ 주요 기능

### 🏷️ 어노테이션 시스템
- **CRUD 작업**: 어노테이션 생성, 조회, 수정, 삭제
- **RESTful API**: 표준 HTTP 메서드 지원
- **Swagger 문서**: 자동 생성된 API 문서
- **데이터 검증**: 입력 데이터 유효성 검사
- **뷰어 소프트웨어 필터링**: OHIF Viewer, DICOM Viewer 등으로 필터링 ✨
- **측정값 지원**: 구조화된 측정 데이터 저장 및 관리 ✨

### 🎭 마스크 업로드 시스템 ✅
- **Object Storage 연동**: AWS S3 및 MinIO 지원
- **Signed URL**: 보안적인 직접 업로드
- **마스크 그룹 관리**: 관련 마스크들을 그룹화
- **메타데이터 저장**: 파일 정보 및 통계
- **완전한 API**: 14개 엔드포인트 구현 완료

### 👥 사용자 관리
- **JWT 인증**: 토큰 기반 인증 시스템
- **권한 관리**: 역할 기반 접근 제어
- **프로젝트 관리**: 사용자별 프로젝트 할당
- **역할-권한 매트릭스**: 표 형태로 역할과 권한 관계 관리 ✨

### 🌐 웹 서버 기능
- **CORS 지원**: 크로스 오리진 요청 처리
- **캐시 헤더**: 성능 최적화
- **에러 처리**: 일관된 에러 응답
- **로깅**: 구조화된 로그 시스템

### 🔒 데이터 일관성
- **원자적 트랜잭션**: 데이터베이스 작업의 원자성 보장
- **자동 롤백**: 트랜잭션 실패 시 자동 복구
- **동시성 제어**: Race condition 방지
- **데이터 무결성**: 외래키 제약조건 및 비즈니스 규칙 준수

## 🏗️ 아키텍처

### Clean Architecture
```
Presentation Layer (Controllers)
├── Annotation Controller
├── User Controller
├── Project Controller
├── Mask Controller ✅
└── Role Permission Matrix Controller ✨

Application Layer (Use Cases)
├── Annotation Use Case
├── User Use Case
├── Project Use Case
├── Mask Use Case ✅
└── Role Permission Matrix Use Case ✨

Domain Layer (Entities & Services)
├── Annotation Entity
├── User Entity
├── Project Entity
├── Mask Entity
└── Business Logic

Infrastructure Layer (Repositories & External)
├── PostgreSQL Repository
├── Object Storage Service
├── JWT Service
└── Configuration
```

### 기술 스택
- **Backend**: Rust + Actix Web
- **Database**: PostgreSQL + SQLx
- **Authentication**: JWT
- **Object Storage**: AWS S3 / MinIO
- **Documentation**: Swagger/OpenAPI
- **Testing**: Rust built-in testing

## 🚀 빠른 시작

### 1. 필수 요구사항
- Rust 1.70+
- PostgreSQL 13+
- Git

### 2. 설치 및 실행
```bash
# 저장소 클론
git clone <repository-url>
cd pacs-ext-server/pacs-server

# 의존성 설치
cargo build

# 환경 변수 설정
cp .env.example .env
# .env 파일을 편집하여 데이터베이스 URL 등 설정

# 데이터베이스 마이그레이션
sqlx migrate run

# 서버 실행
cargo run
```

### 3. API 문서 확인
- **Swagger UI**: http://localhost:8080/swagger-ui/
- **Health Check**: http://localhost:8080/health

## ⚙️ 설정

### 환경 변수
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

### 데이터베이스 스키마
```sql
-- 어노테이션 테이블
CREATE TABLE annotation_annotation (
    id SERIAL PRIMARY KEY,
    study_uid TEXT NOT NULL,
    series_uid TEXT NOT NULL,
    instance_uid TEXT NOT NULL,
    project_id INTEGER NOT NULL,
    annotation_data JSONB NOT NULL,
    created_by INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 마스크 그룹 테이블
CREATE TABLE annotation_mask_group (
    id SERIAL PRIMARY KEY,
    annotation_id INTEGER NOT NULL REFERENCES annotation_annotation(id),
    group_name TEXT,
    model_name TEXT,
    version TEXT,
    modality TEXT,
    slice_count INTEGER DEFAULT 1,
    mask_type TEXT DEFAULT 'segmentation',
    description TEXT,
    created_by INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 마스크 테이블
CREATE TABLE annotation_mask (
    id SERIAL PRIMARY KEY,
    mask_group_id INTEGER NOT NULL REFERENCES annotation_mask_group(id),
    slice_index INTEGER,
    sop_instance_uid TEXT,
    label_name TEXT,
    file_path TEXT NOT NULL,
    mime_type TEXT DEFAULT 'image/png',
    file_size BIGINT,
    checksum TEXT,
    width INTEGER,
    height INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## 📚 API 사용 예시

### 어노테이션 생성
```bash
curl -X POST http://localhost:8080/api/annotations \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <jwt-token>" \
  -d '{
    "study_uid": "1.2.3.4.5.6.7.8.9.10",
    "series_uid": "1.2.3.4.5.6.7.8.9.11",
    "instance_uid": "1.2.3.4.5.6.7.8.9.12",
    "project_id": 1,
    "annotation_data": {
      "type": "polygon",
      "coordinates": [[100, 200], [150, 250], [200, 200]]
    }
  }'
```

### 측정값이 포함된 어노테이션 생성
```bash
curl -X POST http://localhost:8080/api/annotations \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <jwt-token>" \
  -d '{
    "study_uid": "1.2.3.4.5.6.7.8.9.10",
    "series_uid": "1.2.3.4.5.6.7.8.9.11",
    "instance_uid": "1.2.3.4.5.6.7.8.9.12",
    "project_id": 1,
    "annotation_data": {
      "type": "measurement",
      "points": [[0, 0], [100, 100]]
    },
    "description": "폐 결절 크기 측정",
    "tool_name": "Measurement Tool",
    "tool_version": "2.1.0",
    "viewer_software": "OHIF Viewer",
    "measurement_values": [
      {
        "id": "m1",
        "type": "raw",
        "values": [42.3, 18.7],
        "unit": "mm"
      },
      {
        "id": "m2",
        "type": "mean",
        "values": [30.5],
        "unit": "mm"
      }
    ]
  }'
```

### 어노테이션 조회
```bash
# 특정 어노테이션 조회
curl -X GET http://localhost:8080/api/annotations/1 \
  -H "Authorization: Bearer <jwt-token>"

# 뷰어 소프트웨어로 필터링
curl -X GET "http://localhost:8080/api/annotations?viewer_software=OHIF%20Viewer" \
  -H "Authorization: Bearer <jwt-token>"

# 사용자별 뷰어 소프트웨어 필터링
curl -X GET "http://localhost:8080/api/annotations?user_id=123&viewer_software=DICOM%20Viewer" \
  -H "Authorization: Bearer <jwt-token>"

# 프로젝트별 필터링
curl -X GET "http://localhost:8080/api/annotations?project_id=456&viewer_software=OHIF%20Viewer" \
  -H "Authorization: Bearer <jwt-token>"
```

### 마스크 그룹 생성
```bash
curl -X POST http://localhost:8080/api/annotations/1/mask-groups \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <jwt-token>" \
  -d '{
    "group_name": "Liver Segmentation v1.0",
    "model_name": "UNet3D",
    "version": "1.0.0",
    "modality": "CT",
    "slice_count": 100,
    "mask_type": "segmentation",
    "description": "간 분할을 위한 AI 모델 결과"
  }'
```

## 🧪 테스트

### 테스트 실행
```bash
# 모든 테스트 실행
cargo test

# 단위 테스트만 실행
cargo test --lib

# 통합 테스트 실행
cargo test --test annotation_use_case_test --test mask_group_controller_test --test service_test --test mask_controller_test --test annotation_controller_test -- --test-threads=1
```

### 테스트 커버리지
- **단위 테스트**: 49개 테스트, 100% 통과 ✅
- **통합 테스트**: 85개 테스트, 100% 통과 ✅
- **총 테스트**: 134개 테스트, 100% 통과 ✅

### 테스트 카테고리
- **Domain Entities**: 16개 테스트 (mask, mask_group)
- **Application Services**: 2개 테스트 (signed_url_service)
- **Infrastructure**: 25개 테스트 (auth, config, middleware, external)
- **API Controllers**: 20개 테스트 (annotation, mask_group, mask)
- **Service Layer**: 52개 테스트 (user, project, permission, access_control, annotation)
- **Use Cases**: 7개 테스트 (annotation business logic)
- **Role Permission Matrix**: 12개 테스트 (단위 6개 + 통합 6개) ✨

### 통합 테스트 세부사항
- **annotation_controller_test**: 4개 테스트 (API 엔드포인트)
- **annotation_use_case_test**: 7개 테스트 (비즈니스 로직)
- **mask_controller_test**: 8개 테스트 (마스크 API)
- **mask_group_controller_test**: 8개 테스트 (마스크 그룹 API)
- **service_test**: 52개 테스트 (서비스 레이어)
- **role_permission_matrix_integration_tests**: 6개 테스트 (매트릭스 API) ✨

## 📊 성능

### 벤치마크 결과
- **어노테이션 생성**: ~50ms
- **어노테이션 조회**: ~20ms
- **마스크 업로드**: ~200ms (1MB 파일)
- **동시 사용자**: 100+ (예상)

### 최적화 전략
- 데이터베이스 인덱스 최적화
- 쿼리 성능 모니터링
- 캐시 헤더 활용
- Object Storage 직접 업로드

## 🔒 보안

### 인증 및 권한
- JWT 토큰 기반 인증
- 역할 기반 접근 제어 (RBAC)
- API 키 관리

### 데이터 보안
- SQL 인젝션 방지 (SQLx 사용)
- XSS 방지 (입력 검증)
- CORS 설정
- HTTPS 강제 (프로덕션)

### Object Storage 보안
- Signed URL TTL 제한
- IAM 정책으로 접근 제어
- 파일 타입 검증
- 악성 파일 스캔 (향후 구현)

## 📈 모니터링

### 로그 레벨
- **ERROR**: 시스템 오류
- **WARN**: 경고 사항
- **INFO**: 일반 정보
- **DEBUG**: 디버깅 정보

### 메트릭
- API 응답 시간
- 데이터베이스 쿼리 성능
- Object Storage 사용량
- 에러 발생률

## 🚀 배포

### Docker (향후 구현)
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/pacs-server /usr/local/bin/
EXPOSE 8080
CMD ["pacs-server"]
```

### 환경별 설정
- **Development**: 로컬 데이터베이스, MinIO
- **Staging**: AWS RDS, S3
- **Production**: AWS RDS, S3, CloudFront

## 📚 문서

### 기술 문서
- [전체 문서 목록](docs/README.md)
- [API 가이드](docs/technical/ANNOTATION_API_GUIDE.md)
- [데이터베이스 스키마](docs/technical/DATABASE_SCHEMA_MASK_UPLOAD.md)
- [Object Storage 연동](docs/technical/OBJECT_STORAGE_INTEGRATION.md)
- [트랜잭션 처리 최적화](docs/technical/TRANSACTION_OPTIMIZATION_FINAL.md) ✨
- [뷰어 소프트웨어 필터링](docs/VIEWER_SOFTWARE_FILTERING.md) ✨
- [어노테이션 측정값 기능](docs/ANNOTATION_MEASUREMENT_VALUES.md) ✨

### 개발 가이드
- [구현 계획서](docs/todo/implementation_plan.md)
- [코드 구현 가이드](docs/todo/code_implementation_guide.md)
- [CORS 개발 가이드](docs/technical/CORS_DEVELOPMENT_GUIDE.md)

## 🤝 기여하기

### 개발 워크플로우
1. 이슈 생성 또는 기존 이슈 확인
2. 기능 브랜치 생성 (`git checkout -b feature/amazing-feature`)
3. 코드 작성 및 테스트
4. 변경사항 커밋 (`git commit -m 'Add amazing feature'`)
5. 브랜치에 푸시 (`git push origin feature/amazing-feature`)
6. Pull Request 생성

### 코딩 스타일
- Rust 표준 스타일 가이드 준수
- `cargo fmt` 및 `cargo clippy` 사용
- 테스트 코드 작성 필수
- 문서 주석 작성

## 📝 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.

## 📞 지원

### 문제 신고
- GitHub Issues를 통해 버그 신고
- 상세한 재현 단계 포함
- 로그 및 환경 정보 제공

### 기능 요청
- GitHub Discussions 활용
- 명확한 사용 사례 설명
- 우선순위 논의

## 🔄 변경 이력

자세한 변경 이력은 [CHANGELOG.md](docs/technical/CHANGELOG.md)를 참조하세요.

### 주요 버전
- **v1.0.0-beta.5**: 역할-권한 매트릭스 API (2024-12-19) ✨
  - 역할과 권한 간의 관계를 매트릭스 형태로 조회
  - 개별 권한 할당/제거 API 구현
  - 글로벌/프로젝트별 역할 지원
  - 완전한 테스트 커버리지 (12개 테스트)
  - OpenAPI 문서화 완료

- **v1.0.0-beta.4**: 어노테이션 측정값 기능 (2025-01-27) ✨
  - 구조화된 측정 데이터 저장 및 관리
  - JSONB 기반 유연한 측정값 스키마
  - 다양한 측정 타입 및 단위 지원
  - 포괄적인 API 문서화 및 테스트
  - 성능 최적화된 JSONB 인덱싱

- **v1.0.0-beta.3**: 뷰어 소프트웨어 필터링 기능 (2025-01-27) ✨
  - 뷰어 소프트웨어별 어노테이션 필터링 지원
  - API 라우팅 404 오류 수정
  - 포괄적인 테스트 커버리지 (15+ 새 테스트)
  - 동적 테스트 데이터 생성 및 정리
  - 완전한 기술 문서화

- **v1.0.0-beta.2**: 통합 테스트 컴파일 수정 (2025-01-27) ✅
  - 9개 통합 테스트 파일 컴파일 오류 해결
  - 서비스 생성자 패턴 표준화
  - DTO 필드 완성도 개선
  - 100% 테스트 컴파일 성공

- **v1.0.0-beta.1**: 트랜잭션 처리 최적화 (2025-10-11) ✅
  - 122개 테스트 모두 통과
  - 원자적 트랜잭션 처리 구현
  - 데이터 일관성 및 무결성 보장
  - TIMESTAMPTZ 타입 지원

- **v1.0.0-beta**: 베타 릴리스 (완전한 기능 구현) ✅
  - 118개 테스트 모두 통과
  - Annotation & Mask 관리 시스템 완성
  - 완전한 API 엔드포인트 구현
  - 프로덕션 준비 완료

### 다음 버전 계획
- **v1.1.0**: 성능 최적화 및 대용량 파일 지원
- **v1.2.0**: 웹 대시보드 및 사용자 인터페이스
- **v1.3.0**: AI 통합 및 자동 마스크 생성

---
**최종 업데이트**: 2024-12-19  
**작성자**: AI Assistant  
**버전**: 1.0.0-beta.5
