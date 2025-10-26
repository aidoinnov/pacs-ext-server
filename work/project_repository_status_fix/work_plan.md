# 프로젝트 Repository Status 컬럼 에러 수정 - 작업 계획

## 📋 작업 개요
- **작업명**: 프로젝트 Repository Status 컬럼 에러 수정
- **작업일**: 2025-01-26
- **작업자**: AI Assistant
- **우선순위**: 높음 (API 에러 해결)

## 🎯 목표
`PUT /api/projects/{project_id}/users/{user_id}/role` API 호출 시 발생하는 **"no column found for name: status"** 에러를 완전히 해결

## 🔍 문제 분석
- **원인**: `Project` 엔티티에는 `status: ProjectStatus` 필드가 있지만, `project_repository_impl.rs`의 SQL 쿼리들이 이 컬럼을 SELECT하지 않음
- **영향**: 프로젝트 관련 모든 API에서 데이터베이스 매핑 에러 발생
- **심각도**: 높음 (API 기능 완전 중단)

## 📝 작업 계획

### 1단계: 문제 확인 및 분석
- [x] 에러 로그 분석
- [x] `Project` 엔티티 구조 확인
- [x] `project_repository_impl.rs` SQL 쿼리 분석
- [x] 데이터베이스 스키마 확인

### 2단계: 코드 수정
- [x] `find_by_id` 함수 SQL 쿼리 수정
- [x] `find_by_name` 함수 SQL 쿼리 수정
- [x] `find_all` 함수 SQL 쿼리 수정
- [x] `find_active` 함수 SQL 쿼리 수정
- [x] `create` 함수 SQL 쿼리 수정
- [x] `update` 함수 SQL 쿼리 수정

### 3단계: 테스트 및 검증
- [x] 서버 컴파일 확인
- [x] 서버 재시작
- [x] API 엔드포인트 테스트
- [x] 에러 해결 확인

### 4단계: 문서화 및 Git 업데이트
- [x] 작업 문서 작성
- [x] CHANGELOG 업데이트
- [x] Git 커밋 및 푸시

## 🔧 수정 내용 상세

### 파일: `pacs-server/src/infrastructure/repositories/project_repository_impl.rs`

#### 수정된 함수들:
1. **`find_by_id`** (19-28번째 줄)
   - 변경 전: `SELECT id, name, description, is_active, created_at`
   - 변경 후: `SELECT id, name, description, is_active, status, created_at`

2. **`find_by_name`** (30-38번째 줄)
   - 변경 전: `SELECT id, name, description, is_active, created_at`
   - 변경 후: `SELECT id, name, description, is_active, status, created_at`

3. **`find_all`** (41-48번째 줄)
   - 변경 전: `SELECT id, name, description, is_active, created_at`
   - 변경 후: `SELECT id, name, description, is_active, status, created_at`

4. **`find_active`** (51-59번째 줄)
   - 변경 전: `SELECT id, name, description, is_active, created_at`
   - 변경 후: `SELECT id, name, description, is_active, status, created_at`

5. **`create`** (62-71번째 줄)
   - 변경 전: `RETURNING id, name, description, is_active, created_at`
   - 변경 후: `RETURNING id, name, description, is_active, status, created_at`

6. **`update`** (74-85번째 줄)
   - 변경 전: `RETURNING id, name, description, is_active, created_at`
   - 변경 후: `RETURNING id, name, description, is_active, status, created_at`

## ✅ 성공 기준
- [x] "no column found for name: status" 에러 완전 해결
- [x] `PUT /api/projects/{project_id}/users/{user_id}/role` API 정상 작동
- [x] HTTP 200 OK 응답 확인
- [x] 기존 API 기능에 영향 없음

## 🚨 위험 요소 및 대응 방안
- **위험**: 다른 API에 영향 가능성
- **대응**: 모든 관련 함수를 일괄 수정하여 일관성 유지

## 📊 예상 효과
- 프로젝트 관련 모든 API 정상화
- 사용자 경험 개선
- 시스템 안정성 향상
