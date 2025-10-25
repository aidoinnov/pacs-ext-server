# 토큰 갱신 API 구현 작업 계획

## 개요
Keycloak의 refresh token endpoint를 중계하여 access token을 갱신하는 API를 구현합니다. Keycloak에서 토큰 만료 정책을 관리하므로 별도의 저장소 없이 중계만 수행합니다.

## 작업 목표
- 사용자가 refresh token을 사용하여 새로운 access token을 발급받을 수 있도록 함
- Keycloak과의 통합을 통한 안전한 토큰 관리
- Clean Architecture 패턴을 준수한 구현

## 구현 단계

### 1. Keycloak Client 확장
- `refresh_access_token` 메서드 구현
- Keycloak의 `/realms/{realm}/protocol/openid-connect/token` endpoint 호출
- `grant_type=refresh_token` 파라미터 사용

### 2. DTO 확장
- 기존 `RefreshTokenRequest`, `RefreshTokenResponse` DTO 활용
- Keycloak 응답을 위한 내부 DTO 정의

### 3. Auth Service 확장
- `refresh_token_with_keycloak` 메서드 추가
- KeycloakClient를 주입받아 사용

### 4. Auth Use Case 확장
- `refresh_token` 메서드 추가
- 비즈니스 로직 오케스트레이션

### 5. Controller 라우트 추가
- `/api/auth/refresh` 엔드포인트 구현
- POST 요청 처리 및 에러 핸들링

### 6. OpenAPI 문서화
- API 스펙 문서화
- 요청/응답 예시 포함

### 7. 테스트 구현
- 단위 테스트: 각 계층별 테스트
- 통합 테스트: 전체 플로우 테스트
- 성능 테스트: 응답 시간 측정

## 기술 스택
- **Backend**: Rust, Actix-web
- **Authentication**: Keycloak
- **Testing**: Mockall, Mockito
- **Documentation**: OpenAPI/Swagger

## 예상 소요 시간
- 구현: 2-3시간
- 테스트: 1-2시간
- 문서화: 30분
- **총 소요 시간**: 4-5시간

## 성공 기준
- [ ] Keycloak과의 통합이 정상적으로 작동
- [ ] 모든 테스트가 통과
- [ ] API 문서가 완성
- [ ] 에러 처리가 적절히 구현됨
- [ ] Clean Architecture 패턴을 준수
