# 프로젝트 데이터 접근 관리 API 기술 문서

## 📋 개요

프로젝트 데이터 접근 관리 API는 프로젝트 참여자가 프로젝트에 포함된 DICOM Study 데이터에 대한 접근 상태를 조회하고 수정할 수 있는 RESTful API입니다.

## 🏗️ 아키텍처

### Clean Architecture 패턴

```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                       │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │            ProjectDataAccessController                  │ │
│  │  - GET /api/projects/{id}/data-access                  │ │
│  │  - PUT /api/projects/{id}/data-access/{data_id}        │ │
│  │  - GET /api/projects/{id}/data-access/matrix           │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │            ProjectDataAccessUseCase                     │ │
│  │  - get_project_data_access()                           │ │
│  │  - update_data_access_status()                         │ │
│  │  - get_data_access_matrix()                            │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                    DTOs                                 │ │
│  │  - ProjectDataAccessDto                                │ │
│  │  - ProjectDataAccessMatrixDto                          │ │
│  │  - UpdateDataAccessStatusRequest                       │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                    Domain Layer                             │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                    Entities                             │ │
│  │  - ProjectData                                         │ │
│  │  - ProjectDataAccess                                   │ │
│  │  - DataAccessStatus                                    │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                  Repository Traits                      │ │
│  │  - ProjectDataRepository                               │ │
│  │  - ProjectDataAccessRepository                         │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                  Service Traits                         │ │
│  │  - ProjectDataService                                  │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                  Infrastructure Layer                       │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                Repository Implementations               │ │
│  │  - ProjectDataRepositoryImpl                           │ │
│  │  - ProjectDataAccessRepositoryImpl                     │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                Service Implementations                  │ │
│  │  - ProjectDataServiceImpl                              │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                    Database Layer                           │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                    PostgreSQL                           │ │
│  │  - project_data table                                  │ │
│  │  - project_data_access table                           │ │
│  │  - data_access_status_enum                             │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 🗄️ 데이터베이스 스키마

### 테이블 구조

#### `project_data` 테이블
```sql
CREATE TABLE project_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    study_uid VARCHAR(255) NOT NULL,
    patient_id VARCHAR(255) NOT NULL,
    patient_name VARCHAR(255) NOT NULL,
    study_date DATE,
    study_description TEXT,
    modality VARCHAR(50),
    series_count INTEGER DEFAULT 0,
    instance_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

#### `project_data_access` 테이블
```sql
CREATE TABLE project_data_access (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_data_id UUID NOT NULL REFERENCES project_data(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_status data_access_status_enum NOT NULL DEFAULT 'PENDING',
    granted_by UUID REFERENCES users(id),
    granted_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(project_data_id, user_id)
);
```

#### `data_access_status_enum` 열거형
```sql
CREATE TYPE data_access_status_enum AS ENUM (
    'APPROVED',
    'DENIED', 
    'PENDING'
);
```

### 인덱스
```sql
-- 성능 최적화를 위한 인덱스
CREATE INDEX idx_project_data_project_id ON project_data(project_id);
CREATE INDEX idx_project_data_study_uid ON project_data(study_uid);
CREATE INDEX idx_project_data_patient_id ON project_data(patient_id);
CREATE INDEX idx_project_data_access_user_id ON project_data_access(user_id);
CREATE INDEX idx_project_data_access_status ON project_data_access(access_status);
CREATE INDEX idx_project_data_access_project_data_id ON project_data_access(project_data_id);
```

## 🔧 API 엔드포인트

### 1. 데이터 접근 상태 조회

**엔드포인트**: `GET /api/projects/{project_id}/data-access`

**쿼리 파라미터**:
- `page` (optional): 페이지 번호 (기본값: 1)
- `limit` (optional): 페이지 크기 (기본값: 20)
- `search` (optional): 검색어 (study_uid, patient_id, patient_name)
- `user_search` (optional): 사용자 검색어 (username, email)
- `status` (optional): 상태 필터 (APPROVED, DENIED, PENDING)

**응답 예시**:
```json
{
  "data": [
    {
      "id": "uuid",
      "project_data": {
        "id": "uuid",
        "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
        "patient_id": "P001",
        "patient_name": "John Doe",
        "study_date": "2025-01-27",
        "study_description": "Chest X-ray",
        "modality": "CR",
        "series_count": 1,
        "instance_count": 1
      },
      "user": {
        "id": "uuid",
        "username": "john_doe",
        "email": "john@example.com",
        "full_name": "John Doe"
      },
      "access_status": "APPROVED",
      "granted_by": {
        "id": "uuid",
        "username": "admin",
        "email": "admin@example.com"
      },
      "granted_at": "2025-01-27T10:00:00Z",
      "created_at": "2025-01-27T09:00:00Z",
      "updated_at": "2025-01-27T10:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "total_pages": 5
  }
}
```

### 2. 데이터 접근 상태 수정

**엔드포인트**: `PUT /api/projects/{project_id}/data-access/{data_id}`

**요청 본문**:
```json
{
  "access_status": "APPROVED"
}
```

**응답 예시**:
```json
{
  "id": "uuid",
  "project_data": {
    "id": "uuid",
    "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
    "patient_id": "P001",
    "patient_name": "John Doe"
  },
  "user": {
    "id": "uuid",
    "username": "john_doe",
    "email": "john@example.com"
  },
  "access_status": "APPROVED",
  "granted_by": {
    "id": "uuid",
    "username": "admin",
    "email": "admin@example.com"
  },
  "granted_at": "2025-01-27T10:00:00Z",
  "updated_at": "2025-01-27T10:00:00Z"
}
```

### 3. 데이터 접근 매트릭스 조회

**엔드포인트**: `GET /api/projects/{project_id}/data-access/matrix`

**쿼리 파라미터**:
- `page` (optional): 페이지 번호 (기본값: 1)
- `limit` (optional): 페이지 크기 (기본값: 20)
- `search` (optional): 검색어
- `user_search` (optional): 사용자 검색어

**응답 예시**:
```json
{
  "data": [
    {
      "project_data": {
        "id": "uuid",
        "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
        "patient_id": "P001",
        "patient_name": "John Doe"
      },
      "user_access": [
        {
          "user": {
            "id": "uuid",
            "username": "john_doe",
            "email": "john@example.com"
          },
          "access_status": "APPROVED",
          "granted_at": "2025-01-27T10:00:00Z"
        }
      ]
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "total_pages": 5
  }
}
```

## 🔐 보안 및 인증

### 인증 요구사항
- 모든 API 엔드포인트는 JWT 토큰 인증이 필요합니다
- 사용자는 해당 프로젝트의 참여자여야 합니다

### 권한 부여
- 프로젝트 참여자만 데이터 접근 상태를 조회할 수 있습니다
- 프로젝트 참여자만 자신의 데이터 접근 상태를 수정할 수 있습니다

### 데이터 보호
- 개인정보(patient_id, patient_name)는 인증된 사용자만 접근 가능합니다
- 모든 데이터 접근은 로그로 기록됩니다

## 🧪 테스트

### 단위 테스트
- **Repository 테스트**: 데이터 접근 로직 검증
- **Service 테스트**: 비즈니스 로직 검증
- **Use Case 테스트**: 오케스트레이션 로직 검증
- **Controller 테스트**: HTTP 요청 처리 검증

### 통합 테스트
- **API 엔드포인트 테스트**: 전체 API 플로우 검증
- **데이터베이스 연동 테스트**: 실제 DB 연동 검증
- **페이지네이션 테스트**: 대량 데이터 처리 검증
- **검색 기능 테스트**: 검색 로직 검증

### 성능 테스트
- **응답 시간 테스트**: API 응답 시간 측정
- **동시성 테스트**: 동시 요청 처리 검증
- **메모리 사용량 테스트**: 메모리 효율성 검증

## 📊 성능 최적화

### 데이터베이스 최적화
- **인덱스 활용**: 자주 사용되는 컬럼에 인덱스 생성
- **쿼리 최적화**: N+1 문제 해결 및 효율적인 쿼리 작성
- **연결 풀**: 데이터베이스 연결 풀 최적화

### 애플리케이션 최적화
- **페이지네이션**: 대량 데이터를 효율적으로 처리
- **캐싱**: 자주 접근하는 데이터 캐싱
- **비동기 처리**: I/O 작업의 비동기 처리

## 🚀 배포 및 운영

### 환경 변수
```bash
# 데이터베이스 설정
DATABASE_URL=postgresql://user:password@localhost:5432/pacs_db

# 서버 설정
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# 로깅 설정
RUST_LOG=info
```

### Docker 배포
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/pacs-server /usr/local/bin/
CMD ["pacs-server"]
```

### 모니터링
- **로그 모니터링**: 구조화된 로그를 통한 모니터링
- **메트릭 수집**: API 응답 시간, 에러율 등 메트릭 수집
- **알림 설정**: 에러 발생 시 알림 설정

## 🔄 확장성

### 수평적 확장
- **로드 밸런싱**: 여러 인스턴스 간 로드 분산
- **데이터베이스 샤딩**: 대량 데이터 처리를 위한 샤딩
- **캐시 클러스터**: Redis 클러스터를 통한 캐시 확장

### 수직적 확장
- **리소스 증설**: CPU, 메모리 증설
- **데이터베이스 튜닝**: 데이터베이스 성능 튜닝
- **애플리케이션 최적화**: 코드 최적화 및 성능 개선

## 📚 참고 자료

- [Rust 공식 문서](https://doc.rust-lang.org/)
- [Actix-web 문서](https://actix.rs/)
- [SQLx 문서](https://docs.rs/sqlx/)
- [Utoipa 문서](https://docs.rs/utoipa/)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
