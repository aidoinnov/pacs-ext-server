# 작업 계획: 사용자 프로젝트 목록 API에 기한 정보 추가

## 작업 개요
- **작업명**: 사용자 프로젝트 목록 API에 기한 정보 추가
- **작업 기간**: 2025-01-27
- **담당자**: AI Assistant
- **상태**: 완료

## 작업 목적
특정 사용자가 속한 프로젝트 목록을 반환하는 API (`GET /api/users/{user_id}/projects`)의 응답에 프로젝트 기한 정보(start_date, end_date)를 추가하여 클라이언트에서 프로젝트 마감일을 확인할 수 있도록 합니다.

## 배경 및 문제점
- 기존 API는 사용자의 프로젝트 목록과 역할 정보만 제공
- 프로젝트 기한(start_date, end_date) 정보가 포함되지 않아 클라이언트에서 마감일 관리 불가
- 사용자 요청: "특정 사용자가 속한 프로젝트 목록을 받아올 API가 어떤거지? 롤이랑 기한은 꼭나야하거든?"

## 요구사항
1. 기존 API 응답 구조에 `start_date`, `end_date` 필드 추가
2. 데이터베이스에서 프로젝트 기한 정보 조회
3. 롤 정보와 함께 기한 정보를 포함한 완전한 프로젝트 목록 제공

## 작업 범위

### 수정된 파일
1. `pacs-server/src/application/dto/project_user_dto.rs`
   - `ProjectWithRoleResponse` 구조체에 `start_date`, `end_date` 필드 추가

2. `pacs-server/src/domain/services/user_service.rs`
   - `get_user_projects_with_roles` 메서드의 SQL 쿼리 수정
   - 프로젝트 테이블에서 `start_date`, `end_date` 컬럼 조회 추가
   - DTO 변환 로직에 기한 정보 포함

### 영향을 받지 않는 파일
- Controller, Use Case는 수정 불필요 (DTO만 변경되므로 자동 반영)
- API 경로 및 엔드포인트는 동일 유지

## 작업 단계
1. DTO 수정 (`project_user_dto.rs`)
   - `ProjectWithRoleResponse`에 `start_date: Option<String>`, `end_date: Option<String>` 추가

2. Service Layer 수정 (`user_service.rs`)
   - SQL 쿼리에 `p.start_date`, `p.end_date` 추가
   - 쿼리 결과 타입 수정 (튜플 구조 변경)
   - DTO 변환 로직에 기한 정보 포함

3. 컴파일 검증
   - `cargo check`로 컴파일 에러 확인 및 수정

4. 문서화
   - 변경 로그 업데이트
   - 작업 폴더에 문서 정리

## 예상 결과

### 수정 전 응답 구조
```json
{
  "projects": [
    {
      "project_id": 1,
      "project_name": "심장 질환 연구 프로젝트",
      "description": "...",
      "is_active": true,
      "role_id": 2,
      "role_name": "PROJECT_ADMIN",
      "role_scope": "PROJECT"
    }
  ],
  "total_count": 1,
  "page": 1,
  "page_size": 20,
  "total_pages": 1
}
```

### 수정 후 응답 구조
```json
{
  "projects": [
    {
      "project_id": 1,
      "project_name": "심장 질환 연구 프로젝트",
      "description": "...",
      "is_active": true,
      "start_date": "2025-01-01",
      "end_date": "2025-12-31",
      "role_id": 2,
      "role_name": "PROJECT_ADMIN",
      "role_scope": "PROJECT"
    }
  ],
  "total_count": 1,
  "page": 1,
  "page_size": 20,
  "total_pages": 1
}
```

## 성공 기준
- ✅ API 엔드포인트 정상 작동
- ✅ 응답에 start_date, end_date 필드 포함
- ✅ 기존 기능 유지 (역할 정보 포함)
- ✅ 컴파일 에러 없음
- ✅ 기존 클라이언트와 호환성 유지

## 위험 요인 및 대응
1. **기존 클라이언트와의 호환성**
   - 새로운 필드 추가는 하위 호환 가능 (Optional 필드)
   - 기존 클라이언트는 무시할 수 있음

2. **데이터베이스 스키마**
   - security_project 테이블에 이미 start_date, end_date 컬럼 존재
   - NULL 값 처리 필요 (Option<String>으로 처리)

## 관련 정보
- **API 엔드포인트**: `GET /api/users/{user_id}/projects`
- **관련 문서**: `docs/api/user-crud-api-complete.md`, `docs/api/project-user-role-management-api.md`

