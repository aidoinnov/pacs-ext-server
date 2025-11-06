# User-Centered Matrix API 구현 작업 계획

## 작업 개요

기존의 프로젝트 중심 매트릭스 API(`GET /api/project-user-matrix`)와 함께 사용할 수 있는 새로운 사용자 중심 매트릭스 API(`GET /api/user-project-matrix`)를 구현합니다.

## 요구사항 분석

### 1. 기존 API와의 차이점
- **기존 API**: 프로젝트 중심 (행: 프로젝트, 열: 사용자)
- **신규 API**: 사용자 중심 (행: 사용자, 열: 프로젝트)

### 2. 주요 기능 요구사항
- 이중 페이지네이션 (사용자 페이지네이션 + 프로젝트 페이지네이션)
- 사용자 정렬 (username, email, created_at 기준)
- 사용자 검색 (username, email로 검색)
- 필터링 (role_id, project_ids, user_ids)
- 매트릭스 형태의 응답 구조

## 구현 계획

### Phase 1: DTO 설계 및 정의
- `ProjectRoleCell`: 각 셀의 프로젝트-역할 정보
- `UserProjectMatrixRow`: 사용자 행 (사용자 정보 + 프로젝트 역할 목록)
- `UserProjectMatrixResponse`: 최종 응답 구조
- `ProjectInfo`: 프로젝트 기본 정보 (열 헤더용)
- `UserProjectMatrixPagination`: 이중 페이지네이션 정보
- `UserProjectMatrixQueryParams`: 쿼리 파라미터

### Phase 2: Service Layer 확장
- `UserService`에 정렬 지원 메서드 추가
- `get_users_with_sorting()` 메서드 구현
- 동적 SQL 쿼리 구성 (정렬, 검색, 필터링)

### Phase 3: Use Case 구현
- `UserProjectMatrixUseCase` 구조체 생성
- `get_matrix()` 메서드 구현
- 비즈니스 로직 오케스트레이션

### Phase 4: Controller 구현
- `user_project_matrix_controller.rs` 생성
- `get_matrix()` 핸들러 함수 구현
- OpenAPI 문서화

### Phase 5: 라우팅 및 통합
- `main.rs`에 라우팅 설정 추가
- OpenAPI 스키마 등록
- 모듈 구조 정리

### Phase 6: 테스트 및 검증
- API 엔드포인트 테스트
- 다양한 쿼리 파라미터 조합 테스트
- 성능 검증

## 기술적 고려사항

### 1. 성능 최적화
- 동적 SQL 쿼리 구성으로 불필요한 데이터 조회 방지
- 페이지네이션을 통한 대용량 데이터 처리
- 인덱스 활용을 위한 쿼리 최적화

### 2. 확장성
- 기존 API와의 호환성 유지
- 모듈화된 구조로 유지보수성 확보
- Clean Architecture 패턴 준수

### 3. 사용자 경험
- 직관적인 API 응답 구조
- 상세한 에러 메시지 제공
- 완전한 OpenAPI 문서화

## 예상 작업 시간

- **Phase 1**: 1시간 (DTO 설계 및 정의)
- **Phase 2**: 2시간 (Service Layer 확장)
- **Phase 3**: 2시간 (Use Case 구현)
- **Phase 4**: 1시간 (Controller 구현)
- **Phase 5**: 1시간 (라우팅 및 통합)
- **Phase 6**: 1시간 (테스트 및 검증)

**총 예상 시간**: 8시간

## 성공 기준

1. ✅ API 엔드포인트 정상 작동
2. ✅ 모든 쿼리 파라미터 정상 처리
3. ✅ 이중 페이지네이션 정상 작동
4. ✅ 정렬 및 검색 기능 정상 작동
5. ✅ OpenAPI 문서 완전성
6. ✅ 기존 API와의 호환성 유지
7. ✅ 성능 요구사항 충족

## 리스크 및 대응 방안

### 1. 라우팅 충돌
- **리스크**: 기존 컨트롤러와의 라우팅 충돌
- **대응**: 명확한 경로 구분 및 라우팅 순서 조정

### 2. 성능 이슈
- **리스크**: 대용량 데이터 처리 시 성능 저하
- **대응**: 페이지네이션 및 쿼리 최적화

### 3. OpenAPI 충돌
- **리스크**: 유사한 함수명으로 인한 OpenAPI 생성 오류
- **대응**: 명시적 import 및 함수명 구분
