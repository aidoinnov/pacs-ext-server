# PACS Extension Server - 인증 및 권한 관리

## 개요

PACS Extension Server의 인증(Authentication) 및 권한 관리(Authorization) 시스템에 대한 문서입니다.

## 문서 구조

### 1. AuthGuard JWT 인증 구현 계획
- **파일**: `AUTHGUARD_IMPLEMENTATION_PLAN.md`
- **내용**: JWT 기반 인증 시스템 구현 계획
- **범위**: Phase 1 - 사용자 신원 확인 및 토큰 검증
- **상태**: 계획 완료, 구현 대기

### 2. RBAC 권한 제어 구현 계획
- **파일**: `RBAC_IMPLEMENTATION_PLAN.md`
- **내용**: Role 기반 접근 제어 시스템 구현 계획
- **범위**: Phase 2 - 사용자 권한 및 리소스 접근 제어
- **상태**: 계획 완료, Phase 1 완료 후 구현

## 구현 단계

### Phase 1: JWT 인증 (AuthGuard)
1. **목표**: 모든 API 엔드포인트에서 JWT 토큰 검증
2. **주요 기능**:
   - JWT 토큰 검증
   - 사용자 신원 확인
   - 화이트리스트 경로 지원
   - Claims 추출 및 주입

3. **구현 범위**:
   - AuthGuard Middleware
   - Claims Extractor
   - 컨트롤러 수정
   - 설정 및 통합

### Phase 2: 권한 제어 (RBAC)
1. **목표**: Role 기반 리소스 접근 제어
2. **주요 기능**:
   - 사용자 Role 관리
   - Permission 기반 접근 제어
   - 리소스 소유자 확인
   - 프로젝트별 권한 격리

3. **구현 범위**:
   - Permission Guard
   - Role-Permission 매핑
   - 권한 검증 로직
   - 데이터베이스 스키마 확장

## 현재 상태

### 완료된 작업
- [x] AuthGuard 구현 계획 수립
- [x] RBAC 구현 계획 수립
- [x] 기술 문서 작성
- [x] 아키텍처 설계

### 진행 예정
- [ ] AuthGuard 구현 (Phase 1)
- [ ] 단위 테스트 작성
- [ ] 통합 테스트 작성
- [ ] RBAC 구현 (Phase 2)

## 기술 스택

### 인증 (Phase 1)
- **JWT**: JSON Web Token 기반 인증
- **Actix-Web**: Middleware 및 Extractor
- **SQLx**: 사용자 정보 조회
- **Serde**: JSON 직렬화/역직렬화

### 권한 제어 (Phase 2)
- **RBAC**: Role-Based Access Control
- **PostgreSQL**: 권한 데이터 저장
- **SQLx**: 권한 정보 조회
- **Actix-Web**: Permission Guard

## 보안 고려사항

### 인증 보안
1. **토큰 검증**: 완전한 JWT 토큰 검증
2. **에러 처리**: 클라이언트에는 간단한 메시지
3. **로깅**: 모든 인증 실패 이벤트 로깅
4. **화이트리스트**: 최소한의 경로만 인증 없이 허용

### 권한 보안
1. **최소 권한 원칙**: 필요한 최소한의 권한만 부여
2. **권한 격리**: 프로젝트별 권한 격리
3. **감사 로그**: 권한 변경 및 접근 시도 로깅
4. **정기 검토**: 권한 설정 정기 검토

## API 엔드포인트 분류

### 공개 엔드포인트 (인증 불필요)
- `GET /health` - 서버 상태 확인
- `GET /api-docs/*` - OpenAPI 문서
- `GET /swagger-ui/*` - Swagger UI
- `POST /api/auth/login` - 로그인
- `POST /api/users` - 회원가입

### 보호된 엔드포인트 (인증 필수)
- `GET|POST|PUT|DELETE /api/annotations/*` - 어노테이션 관리
- `GET|POST|PUT|DELETE /api/annotations/{id}/mask-groups/*` - 마스크 그룹 관리
- `GET|POST|PUT|DELETE /api/annotations/{id}/mask-groups/{id}/masks/*` - 마스크 관리
- `GET|PUT|DELETE /api/users/*` - 사용자 관리 (회원가입 제외)
- `GET|POST|PUT|DELETE /api/projects/*` - 프로젝트 관리
- `GET|POST|PUT|DELETE /api/permissions/*` - 권한 관리
- `GET|POST|PUT|DELETE /api/access-control/*` - 접근 제어 관리

## 설정

### AuthGuard 설정
```toml
[auth_guard]
enabled = true
whitelist_paths = ["/health", "/api-docs", "/swagger-ui"]
```

### JWT 설정
```toml
[jwt]
secret = "your-secret-key"
expiration_hours = 24
```

## 테스트

### 단위 테스트
- AuthGuard Middleware 테스트
- Claims Extractor 테스트
- Permission Guard 테스트 (Phase 2)
- 권한 검증 로직 테스트 (Phase 2)

### 통합 테스트
- 인증 플로우 테스트
- 권한 기반 접근 제어 테스트 (Phase 2)
- 에러 처리 테스트
- 성능 테스트

## 모니터링

### 로그
- 인증 실패 이벤트
- 권한 거부 이벤트
- 토큰 만료 이벤트
- 권한 변경 이벤트

### 메트릭
- 인증 성공/실패율
- 권한 검증 성능
- 토큰 사용 통계
- API 접근 패턴

## 문제 해결

### 일반적인 문제
1. **토큰 만료**: 클라이언트에서 토큰 갱신 필요
2. **권한 부족**: 관리자에게 권한 요청
3. **인증 실패**: 토큰 형식 및 서명 확인
4. **성능 이슈**: 권한 캐싱 및 쿼리 최적화

### 디버깅
1. **로그 확인**: 인증/권한 관련 로그 분석
2. **토큰 검증**: JWT 토큰 내용 확인
3. **권한 확인**: 사용자 Role 및 Permission 확인
4. **네트워크**: 클라이언트-서버 통신 확인

## 참고 자료

- [JWT 공식 문서](https://jwt.io/)
- [Actix-Web Middleware](https://actix.rs/docs/middleware/)
- [RBAC 모델](https://en.wikipedia.org/wiki/Role-based_access_control)
- [OAuth 2.0 표준](https://tools.ietf.org/html/rfc6749)

## 연락처

문서에 대한 질문이나 개선 사항이 있으시면 개발팀에 문의해주세요.
