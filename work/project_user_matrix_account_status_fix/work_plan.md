# Project User Matrix API account_status 에러 수정 작업 계획

## 📋 작업 개요

**작업명**: Project User Matrix API account_status 에러 수정  
**작업일**: 2025-01-23  
**작업자**: AI Assistant  
**상태**: ✅ 완료  

## 🎯 목표

Project User Matrix API (`GET /api/project-user-matrix`)에서 발생하는 `account_status` 컬럼 관련 에러를 해결하여 정상적인 매트릭스 데이터를 출력할 수 있도록 수정

## 🔍 문제 분석

### 발생한 에러
```
Database error: no column found for name: account_status
```

### 원인 분석
1. **데이터베이스**: `security_user` 테이블에 `account_status` 컬럼이 존재함
2. **엔티티**: `User` 엔티티에 `account_status` 필드가 정의되어 있음
3. **SQL 쿼리**: `user_service.rs`의 `get_users_with_filter` 메서드에서 `account_status` 컬럼을 SELECT 하지 않음
4. **SQLx 매핑**: 쿼리 결과와 엔티티 구조 불일치로 인한 에러 발생

## 📝 작업 계획

### 1단계: SQL 쿼리 수정
- **파일**: `pacs-server/src/domain/services/user_service.rs`
- **작업**: `get_users_with_filter` 메서드의 SELECT 쿼리 수정
- **내용**: User 엔티티의 모든 필드를 SELECT 하도록 수정

### 2단계: 필터링 조건 추가
- **작업**: 삭제된 사용자 제외 조건 추가
- **내용**: `WHERE account_status != 'DELETED'` 조건 추가

### 3단계: COUNT 쿼리 수정
- **작업**: 총 개수 조회 쿼리에도 동일한 필터링 조건 적용

### 4단계: 테스트 및 검증
- **API 테스트**: 수정된 API 엔드포인트 호출 테스트
- **데이터 검증**: 매트릭스 데이터 정상 출력 확인

## 🔧 기술적 세부사항

### 수정 전 쿼리
```sql
SELECT id, keycloak_id, username, email, full_name, organization, department, phone, created_at, updated_at
FROM security_user
WHERE ($1::int[] IS NULL OR id = ANY($1))
ORDER BY username
LIMIT $2 OFFSET $3
```

### 수정 후 쿼리
```sql
SELECT id, keycloak_id, username, email, full_name, organization, department, phone, 
       created_at, updated_at, account_status, email_verified, 
       email_verification_token, email_verification_expires_at, 
       approved_by, approved_at, suspended_at, suspended_reason, deleted_at
FROM security_user
WHERE ($1::int[] IS NULL OR id = ANY($1))
  AND account_status != 'DELETED'
ORDER BY username
LIMIT $2 OFFSET $3
```

## 📊 예상 결과

1. **에러 해결**: 500 Internal Server Error → 200 OK
2. **데이터 정확성**: 모든 사용자 상태 정보 정상 조회
3. **필터링**: 삭제된 사용자 제외, 활성/대기 상태 사용자만 조회
4. **성능**: 기존 성능 유지

## ✅ 완료 체크리스트

- [x] SQL 쿼리 수정 (SELECT 절에 모든 User 필드 추가)
- [x] 필터링 조건 추가 (삭제된 사용자 제외)
- [x] COUNT 쿼리 수정 (동일한 필터링 조건 적용)
- [x] 서버 재시작 및 컴파일 확인
- [x] API 테스트 및 응답 검증
- [x] 기술 문서 작성
- [x] CHANGELOG 업데이트
- [x] Git 커밋 및 푸시

## 📚 참고 자료

- [Project User Matrix API 문서](../../docs/api/project-user-matrix-api-complete.md)
- [User 엔티티 정의](../../src/domain/entities/user.rs)
- [UserService 구현체](../../src/domain/services/user_service.rs)
