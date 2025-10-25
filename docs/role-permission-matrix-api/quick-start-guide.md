# Role-Permission Matrix API 빠른 시작 가이드

## 🚀 개요

이 문서는 현재 구현된 Role-Permission Matrix API를 빠르게 이해하고 사용하는 방법을 안내합니다. Clean Architecture 패턴을 따르는 Rust 백엔드 서버의 권한 관리 시스템입니다.

## 🏗️ 현재 구현된 아키텍처

```
pacs-server/
├── src/
│   ├── domain/                    # 도메인 계층
│   │   ├── entities/
│   │   │   ├── role.rs           # 롤 엔티티
│   │   │   ├── permission.rs     # 권한 엔티티
│   │   │   └── role_permission.rs # 롤-권한 관계
│   │   ├── repositories/
│   │   │   ├── role_repository.rs
│   │   │   └── permission_repository.rs
│   │   └── services/
│   │       └── access_control_service.rs
│   ├── application/               # 애플리케이션 계층
│   │   ├── use_cases/
│   │   │   └── role_permission_matrix_use_case.rs
│   │   └── dto/
│   │       └── role_permission_matrix_dto.rs
│   ├── infrastructure/            # 인프라스트럭처 계층
│   │   ├── repositories/
│   │   │   ├── role_repository_impl.rs
│   │   │   └── permission_repository_impl.rs
│   │   └── external/
│   │       └── keycloak_client.rs
│   └── presentation/              # 프레젠테이션 계층
│       └── controllers/
│           └── role_permission_matrix_controller.rs
```

## 🎯 핵심 기능

### 1. 글로벌 롤-권한 매트릭스 조회
- 모든 글로벌 롤과 권한의 관계를 매트릭스 형태로 조회
- 권한을 카테고리별로 그룹화하여 제공
- 각 롤에 할당된 권한 상태를 한 번에 확인

### 2. 권한 할당/제거
- 특정 롤에 특정 권한을 할당하거나 제거
- ON/OFF 토글 방식으로 간단한 권한 관리
- 실시간 권한 상태 변경

## 🔧 기술 스택

- **언어**: Rust
- **웹 프레임워크**: Actix Web
- **데이터베이스**: PostgreSQL + SQLx
- **인증**: JWT + Keycloak
- **문서화**: OpenAPI (Swagger)
- **아키텍처**: Clean Architecture

## 📊 데이터베이스 스키마

### 핵심 테이블

```sql
-- 롤 테이블
CREATE TABLE security_roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    scope VARCHAR(20) NOT NULL DEFAULT 'GLOBAL',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 권한 테이블
CREATE TABLE security_permissions (
    id SERIAL PRIMARY KEY,
    resource_type VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(resource_type, action)
);

-- 롤-권한 할당 테이블
CREATE TABLE security_role_permission (
    id SERIAL PRIMARY KEY,
    role_id INTEGER NOT NULL REFERENCES security_roles(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES security_permissions(id) ON DELETE CASCADE,
    assigned BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(role_id, permission_id)
);
```

## 🚀 API 사용법

### 1. 서버 시작

```bash
# 의존성 설치
cargo build

# 서버 실행
cargo run

# 또는 환경변수와 함께 실행
DATABASE_URL="postgres://user:pass@localhost:5432/db" cargo run
```

### 2. API 엔드포인트

#### 2.1 글로벌 롤-권한 매트릭스 조회

```bash
GET /api/roles/global/permissions/matrix
Authorization: Bearer <jwt-token>
```

**응답 예시:**
```json
{
  "roles": [
    {
      "id": 1,
      "name": "Admin",
      "description": "시스템 관리자",
      "scope": "GLOBAL"
    },
    {
      "id": 2,
      "name": "User",
      "description": "일반 사용자",
      "scope": "GLOBAL"
    }
  ],
  "permissions_by_category": {
    "USER": [
      {
        "id": 1,
        "resource_type": "USER",
        "action": "CREATE",
        "description": "사용자 생성"
      },
      {
        "id": 2,
        "resource_type": "USER",
        "action": "READ",
        "description": "사용자 조회"
      }
    ],
    "PROJECT": [
      {
        "id": 3,
        "resource_type": "PROJECT",
        "action": "CREATE",
        "description": "프로젝트 생성"
      }
    ]
  },
  "assignments": [
    {
      "role_id": 1,
      "permission_id": 1,
      "assigned": true
    },
    {
      "role_id": 1,
      "permission_id": 2,
      "assigned": true
    },
    {
      "role_id": 2,
      "permission_id": 2,
      "assigned": true
    }
  ]
}
```

#### 2.2 권한 할당/제거

```bash
PUT /api/roles/{role_id}/permissions/{permission_id}
Authorization: Bearer <jwt-token>
Content-Type: application/json

{
  "assign": true
}
```

**응답 예시:**
```json
{
  "success": true,
  "message": "Permission assigned successfully"
}
```

### 3. Swagger UI 사용

서버 실행 후 브라우저에서 접속:
```
http://localhost:8080/swagger-ui/
```

## 🧪 테스트

### 1. 단위 테스트 실행

```bash
# 모든 단위 테스트
cargo test

# 특정 테스트만 실행
cargo test role_permission_matrix

# 테스트 커버리지 확인
cargo test --coverage
```

### 2. 통합 테스트 실행

```bash
# 통합 테스트 (데이터베이스 필요)
cargo test --test integration_tests

# 성능 테스트
cargo test --test performance_tests
```

### 3. API 테스트 예시

```bash
# cURL을 사용한 API 테스트
curl -X GET "http://localhost:8080/api/roles/global/permissions/matrix" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

curl -X PUT "http://localhost:8080/api/roles/1/permissions/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"assign": true}'
```

## 🔐 인증 및 권한

### 1. JWT 토큰 획득

```bash
# Keycloak을 통한 로그인 (예시)
curl -X POST "http://localhost:8080/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "password"
  }'
```

### 2. 필요한 권한

- **매트릭스 조회**: `ROLE_MANAGEMENT` 권한
- **권한 할당/제거**: `ROLE_MANAGEMENT` 권한

## 🐳 Docker 실행

### 1. Docker Compose 사용

```yaml
# docker-compose.yml
version: '3.8'
services:
  pacs-server:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://user:pass@db:5432/pacs
      - KEYCLOAK_URL=http://keycloak:8080
    depends_on:
      - db
      - keycloak

  db:
    image: postgres:15
    environment:
      - POSTGRES_DB=pacs
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
    volumes:
      - postgres_data:/var/lib/postgresql/data

  keycloak:
    image: quay.io/keycloak/keycloak:latest
    environment:
      - KEYCLOAK_ADMIN=admin
      - KEYCLOAK_ADMIN_PASSWORD=admin
    ports:
      - "8081:8080"

volumes:
  postgres_data:
```

```bash
# Docker Compose 실행
docker-compose up -d

# 로그 확인
docker-compose logs -f pacs-server
```

## 🔧 설정

### 1. 환경 변수

```bash
# .env 파일
DATABASE_URL=postgres://user:password@localhost:5432/pacs_db
KEYCLOAK_URL=http://localhost:8080
KEYCLOAK_REALM=dcm4che
KEYCLOAK_CLIENT_ID=pacs-server
KEYCLOAK_CLIENT_SECRET=your-client-secret
JWT_SECRET=your-jwt-secret
```

### 2. 설정 파일

```toml
# config/default.toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[database]
host = "localhost"
port = 5432
username = "pacs_user"
password = "pacs_password"
database = "pacs_db"
max_connections = 10

[keycloak]
url = "http://localhost:8080"
realm = "dcm4che"
client_id = "pacs-server"
client_secret = ""

[jwt]
secret = "your-secret-key"
expiration_hours = 24
```

## 🚨 트러블슈팅

### 1. 데이터베이스 연결 오류

```bash
# 데이터베이스 연결 확인
psql "postgres://user:password@localhost:5432/pacs_db"

# 마이그레이션 실행
cargo run --bin migrate
```

### 2. JWT 토큰 오류

```bash
# JWT 시크릿 확인
echo $JWT_SECRET

# Keycloak 연결 확인
curl http://localhost:8080/realms/dcm4che/.well-known/openid_configuration
```

### 3. 권한 오류

```bash
# 사용자 권한 확인
curl -X GET "http://localhost:8080/api/users/me/permissions" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## 📚 다음 단계

1. **프로젝트별 롤-권한 관리** 구현
2. **사용자별 권한 조회** API 추가
3. **권한 검증 미들웨어** 구현
4. **권한 관리 UI** 개발
5. **권한 감사 로그** 시스템 구축

## 🔗 관련 문서

- [API 참조](api-reference.md)
- [사용자 가이드](user-guide.md)
- [API 예시](api-examples.md)
- [다음 단계 구현 가이드](next-steps-implementation-guide.md)

---

이 가이드를 통해 Role-Permission Matrix API를 빠르게 이해하고 사용할 수 있습니다. 추가 질문이나 도움이 필요하시면 언제든 문의해주세요! 🚀
