# Project User Matrix API 기술 문서

## 개요

Project User Matrix API는 프로젝트와 사용자 간의 역할 관계를 매트릭스 형태로 조회할 수 있는 API입니다. 각 셀은 특정 사용자가 특정 프로젝트에서 담당하는 역할을 나타냅니다.

## 주요 기능

### 1. 매트릭스 조회
- 프로젝트와 사용자의 모든 조합에 대한 역할 정보 조회
- 프로젝트별, 사용자별 페이지네이션 지원
- 상태별 프로젝트 필터링
- 특정 프로젝트/사용자 ID 목록으로 필터링

### 2. 페이지네이션
- 프로젝트 페이지네이션: `project_page`, `project_page_size`
- 사용자 페이지네이션: `user_page`, `user_page_size`
- 각각 독립적으로 페이지네이션 적용

### 3. 필터링
- 프로젝트 상태 필터: `project_status` (PREPARING, IN_PROGRESS, COMPLETED, ON_HOLD, CANCELLED)
- 특정 프로젝트 ID 목록: `project_ids`
- 특정 사용자 ID 목록: `user_ids`

## API 엔드포인트

### GET /api/project-user-matrix

**Query Parameters:**
- `project_page` (optional): 프로젝트 페이지 번호 (기본값: 1)
- `project_page_size` (optional): 프로젝트 페이지 크기 (기본값: 10, 최대: 50)
- `user_page` (optional): 사용자 페이지 번호 (기본값: 1)
- `user_page_size` (optional): 사용자 페이지 크기 (기본값: 10, 최대: 50)
- `project_status` (optional): 프로젝트 상태 필터 (배열)
- `project_ids` (optional): 특정 프로젝트 ID 목록 (배열)
- `user_ids` (optional): 특정 사용자 ID 목록 (배열)

**Response:**
```json
{
  "matrix": [
    {
      "project_id": 1,
      "project_name": "Project Alpha",
      "description": "Description",
      "status": "IN_PROGRESS",
      "user_roles": [
        {
          "user_id": 1,
          "username": "user1",
          "email": "user1@example.com",
          "role_id": 1,
          "role_name": "Admin"
        },
        {
          "user_id": 2,
          "username": "user2",
          "email": "user2@example.com",
          "role_id": null,
          "role_name": null
        }
      ]
    }
  ],
  "users": [
    {
      "user_id": 1,
      "username": "user1",
      "email": "user1@example.com"
    }
  ],
  "pagination": {
    "project_page": 1,
    "project_page_size": 10,
    "project_total_count": 37,
    "project_total_pages": 4,
    "user_page": 1,
    "user_page_size": 10,
    "user_total_count": 58,
    "user_total_pages": 6
  }
}
```

## 데이터베이스 스키마

### 프로젝트 상태 ENUM
```sql
CREATE TYPE project_status AS ENUM (
    'PREPARING',    -- 준비중
    'IN_PROGRESS',  -- 진행중
    'COMPLETED',    -- 완료
    'ON_HOLD',      -- 보류
    'CANCELLED'     -- 취소
);
```

### security_project 테이블 업데이트
```sql
ALTER TABLE security_project
ADD COLUMN status project_status NOT NULL DEFAULT 'PREPARING';
```

## 아키텍처

### Clean Architecture 구조
- **Domain**: 엔티티, 서비스 인터페이스, Repository 인터페이스
- **Application**: Use Case, DTO
- **Infrastructure**: Repository 구현체, 데이터베이스 연결
- **Presentation**: 컨트롤러, OpenAPI 문서화

### 주요 컴포넌트

#### Domain Layer
- `ProjectStatus` enum: 프로젝트 상태 정의
- `Project` entity: status 필드 추가
- `ProjectService` trait: 매트릭스 관련 메서드 추가
- `UserService` trait: 사용자 필터링 메서드 추가

#### Application Layer
- `ProjectUserMatrixUseCase`: 매트릭스 로직 오케스트레이션
- `MatrixQueryParams`: 쿼리 파라미터 DTO
- `ProjectUserMatrixResponse`: 응답 DTO
- `UserRoleCell`: 사용자-역할 셀 DTO

#### Infrastructure Layer
- Repository 구현체에서 매트릭스 쿼리 구현
- SQLx를 사용한 데이터베이스 접근

#### Presentation Layer
- `project_user_matrix_controller`: API 엔드포인트
- OpenAPI 문서화
- 에러 처리

## 성능 최적화

### 1. 데이터베이스 인덱스
```sql
CREATE INDEX idx_project_status ON security_project(status);
```

### 2. 쿼리 최적화
- CROSS JOIN을 사용한 매트릭스 생성
- LEFT JOIN으로 역할 정보 조인
- 적절한 WHERE 절로 필터링

### 3. 페이지네이션
- 프로젝트와 사용자 각각 독립적인 페이지네이션
- LIMIT/OFFSET 사용

## 테스트

### 1. 단위 테스트
- `ProjectUserMatrixUseCase` 테스트
- DTO 직렬화/역직렬화 테스트
- 서비스 메서드 테스트

### 2. 통합 테스트
- API 엔드포인트 테스트
- 데이터베이스 통합 테스트
- 페이지네이션 테스트
- 필터링 테스트

### 3. 성능 테스트
- 대용량 데이터 처리 테스트
- 응답 시간 측정
- 메모리 사용량 모니터링

## 보안 고려사항

### 1. 인증/인가
- JWT 토큰 기반 인증
- 역할 기반 접근 제어 (RBAC)
- 사용자별 데이터 접근 제한

### 2. 데이터 보호
- 민감한 정보 마스킹
- SQL 인젝션 방지 (SQLx 사용)
- 입력 검증

## 모니터링 및 로깅

### 1. 로깅
- 구조화된 로깅 (tracing)
- 요청/응답 로깅
- 에러 로깅

### 2. 메트릭
- API 응답 시간
- 데이터베이스 쿼리 시간
- 메모리 사용량

## 확장성

### 1. 캐싱
- Redis를 사용한 결과 캐싱
- TTL 기반 캐시 무효화

### 2. 분산 처리
- 대용량 데이터 처리를 위한 배치 처리
- 비동기 처리

## 문제 해결

### 1. 일반적인 문제
- 메모리 부족: 페이지네이션 크기 조정
- 느린 쿼리: 인덱스 추가, 쿼리 최적화
- 타임아웃: 비동기 처리, 타임아웃 설정

### 2. 디버깅
- 로그 레벨 조정
- SQL 쿼리 로깅
- 성능 프로파일링

## 참고 자료

- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Actix-web Documentation](https://actix.rs/docs/)
- [OpenAPI Specification](https://swagger.io/specification/)
