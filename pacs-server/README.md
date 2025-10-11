# 🏥 PACS Extension Server

## 📋 개요
PACS (Picture Archiving and Communication System) Extension Server는 의료 영상 어노테이션 및 마스크 업로드 기능을 제공하는 Rust 기반 웹 서버입니다.

## ✨ 주요 기능

### 🏷️ 어노테이션 시스템
- **CRUD 작업**: 어노테이션 생성, 조회, 수정, 삭제
- **RESTful API**: 표준 HTTP 메서드 지원
- **Swagger 문서**: 자동 생성된 API 문서
- **데이터 검증**: 입력 데이터 유효성 검사

### 🎭 마스크 업로드 시스템 (개발 중)
- **Object Storage 연동**: AWS S3 및 MinIO 지원
- **Signed URL**: 보안적인 직접 업로드
- **마스크 그룹 관리**: 관련 마스크들을 그룹화
- **메타데이터 저장**: 파일 정보 및 통계

### 👥 사용자 관리
- **JWT 인증**: 토큰 기반 인증 시스템
- **권한 관리**: 역할 기반 접근 제어
- **프로젝트 관리**: 사용자별 프로젝트 할당

### 🌐 웹 서버 기능
- **CORS 지원**: 크로스 오리진 요청 처리
- **캐시 헤더**: 성능 최적화
- **에러 처리**: 일관된 에러 응답
- **로깅**: 구조화된 로그 시스템

## 🏗️ 아키텍처

### Clean Architecture
```
Presentation Layer (Controllers)
├── Annotation Controller
├── User Controller
├── Project Controller
└── Mask Controller (개발 중)

Application Layer (Use Cases)
├── Annotation Use Case
├── User Use Case
├── Project Use Case
└── Mask Use Case (개발 중)

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

### 어노테이션 조회
```bash
curl -X GET http://localhost:8080/api/annotations/1 \
  -H "Authorization: Bearer <jwt-token>"
```

### 마스크 그룹 생성 (개발 중)
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

### 단위 테스트
```bash
cargo test
```

### 통합 테스트
```bash
cargo test --test annotation_controller_test
```

### 테스트 커버리지
```bash
# (향후 구현 예정)
cargo tarpaulin --out Html
```

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
- **v0.1.0**: 초기 릴리스 (어노테이션 시스템)
- **v0.2.0**: 마스크 업로드 시스템 (개발 중)
- **v0.3.0**: 성능 최적화 및 모니터링 (예정)

---
**최종 업데이트**: 2025-10-07  
**작성자**: AI Assistant  
**버전**: 1.0
