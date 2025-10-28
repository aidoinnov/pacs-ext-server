# Keycloak 사용자 삭제 기능 구현

## 작업 개요

Keycloak을 사용한 사용자 관리 시스템에서 사용자 삭제 API를 구현합니다. Service Account 방식으로 Keycloak에 접근하여 사용자를 삭제하는 기능을 추가합니다.

## 작업 목표

1. Service Account 방식으로 Keycloak 인증 구현
2. Keycloak 사용자 삭제 API 구현
3. 데이터베이스와 Keycloak 간 원자적 트랜잭션 보장
4. 명확한 에러 메시지 제공

## 작업 범위

### 1. Keycloak Client 수정

- Client credentials grant type 구현
- Service account 토큰 획득 방식 변경
- Admin API 접근 권한 설정

### 2. 사용자 삭제 로직 구현

- Keycloak 사용자 삭제
- PACS DB 사용자 삭제
- 트랜잭션 롤백 처리

### 3. 에러 처리 개선

- 존재하지 않는 사용자 처리
- 명확한 에러 메시지

## 예상 작업 시간

- 개발: 2시간
- 테스트: 1시간
- 문서화: 1시간
- **총 예상 시간: 4시간**

## 우선순위

**높음** - 사용자 관리 기능의 핵심 부분

