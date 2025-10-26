# 데이터 접근 권한 상태 업데이트 API 수정 작업 계획

## 작업 목적
프로젝트 데이터 접근 권한의 상태(status)를 업데이트하는 API에서 발생한 데이터베이스 오류를 수정하고, 자동 권한 부여 기능을 구현합니다.

## 발생한 문제들

### 1. Status 필드 타입 오류
- **에러**: `column "status" is of type data_access_status_enum but expression is of type integer`
- **원인**: status 필드를 동적 쿼리에서 enum 타입으로 캐스팅하지 않아 발생
- **해결**: `status = $1::data_access_status_enum` 형태로 타입 명시

### 2. 프로젝트에 멤버 추가 시 자동 권한 부여 미구현
- **문제**: 사용자를 프로젝트에 추가해도 데이터 접근 권한이 자동으로 부여되지 않음
- **해결**: `ProjectUserUseCase.add_member_to_project`에서 `grant_default_access_to_user` 호출

### 3. 바인딩 파라미터 불일치 오류
- **에러**: `bind message supplies 5 parameters, but prepared statement requires 6`
- **원인**: 동적 쿼리 구성과 바인딩 순서가 불일치
- **해결**: 동적 쿼리를 단일 prepared statement로 재구현

### 4. NULL 컬럼 디코딩 오류 (project_id)
- **에러**: `unexpected null; try decoding as an Option`
- **원인**: RETURNING 절에서 `project_id`를 반환하려 하지만 테이블에 해당 컬럼이 없음
- **해결**: `0 as project_id`로 기본값 설정

### 5. NULL 컬럼 디코딩 오류 (study_id)
- **에러**: `unexpected null; try decoding as an Option`
- **원인**: 기존 데이터에 `study_id`가 NULL일 수 있으나 entity에서 `i32`로 정의됨
- **해결**: `study_id`를 `Option<i32>`로 변경하여 NULL 허용

### 6. 매트릭스 정렬 문제
- **문제**: 매트릭스가 생성/수정 시마다 순서가 달라짐
- **원인**: `ORDER BY created_at DESC` 사용 및 HashSet으로 인한 순서 불일치
- **해결**: 
  - 데이터는 `ORDER BY id ASC`로 변경
  - 사용자는 ID로 정렬하여 안정적인 순서 제공

## 작업 범위

### 1. Repository 레이어 수정
- `pacs-server/src/infrastructure/repositories/project_data_access_repository_impl.rs`
  - `update_by_project_data_and_user` 메서드 완전 재구현
  - 단일 prepared statement로 간소화
  - status 필드를 enum 타입으로 바인딩

### 2. Domain 레이어 수정
- `pacs-server/src/domain/entities/project_data.rs`
  - `study_id` 필드를 `Option<i32>`로 변경
  - 기존 데이터 호환성 확보

### 3. Use Case 레이어 수정
- `pacs-server/src/application/use_cases/project_user_use_case.rs`
  - `ProjectUserUseCase`에 `ProjectDataService` 의존성 추가
  - `add_member_to_project`에서 자동 권한 부여 로직 추가
  - 제네릭 타입에 `D: ProjectDataService` 추가

- `pacs-server/src/application/use_cases/project_data_access_use_case.rs`
  - users 목록을 ID로 정렬하여 안정적인 순서 제공

### 4. Controller 레이어 수정
- `pacs-server/src/presentation/controllers/project_user_controller.rs`
  - 모든 API 핸들러에 `D` 제네릭 타입 추가
  - `ProjectDataService` import 추가

### 5. Infrastructure 레이어 수정
- `pacs-server/src/infrastructure/repositories/project_data_repository_impl.rs`
  - 정렬 기준을 `created_at DESC`에서 `id ASC`로 변경

### 6. Application 레이어 수정
- `pacs-server/src/main.rs`
  - `ProjectUserUseCase` 생성 시 `project_data_service` 전달

## 작업 우선순위
1. 데이터베이스 오류 수정 (High)
2. 자동 권한 부여 기능 구현 (High)
3. 매트릭스 정렬 개선 (Medium)
4. 타입 안전성 개선 (Medium)

## 예상 작업 시간
- 데이터베이스 오류 수정: 2시간
- 자동 권한 부여 구현: 2시간
- 매트릭스 정렬 개선: 1시간
- 테스트 및 검증: 1시간
- **총 예상 시간**: 6시간

