# Keycloak 인증 시스템 개선

## 📌 개요

이 폴더는 2025년 10월부터 11월까지 진행된 Keycloak 인증 시스템 개선 작업에 대한 문서를 포함합니다.

**주요 파일**: `pacs-server/src/infrastructure/external/keycloak_client.rs`

---

## 📁 문서 구조

```
2025-10-keycloak-인증-개선/
├── README.md           # 이 파일
├── 작업계획.md         # 프로젝트 계획 및 일정
├── 작업내용.md         # 상세 작업 내용 및 변경사항
└── 기술문서.md         # 기술 아키텍처 및 구현 상세
```

---

## 🎯 프로젝트 목표

### 주요 개선사항
1. ✅ **토큰 갱신 API 구현**
   - Refresh token을 사용한 access token 갱신
   - 재로그인 없이 세션 유지

2. ✅ **Service Account 기반 관리 작업**
   - Password grant → Client credentials 전환
   - Admin 계정 정보 노출 위험 제거

3. ✅ **관리자 승인 기반 회원가입**
   - 이메일 인증 단계 제거
   - 관리자 승인 방식으로 보안 강화

4. ✅ **인증 프록시 구조**
   - 클라이언트가 Keycloak과 직접 통신하지 않음
   - Username/password 기반 로그인

5. ✅ **사용자 관리 기능 강화**
   - Username 기반 사용자 삭제
   - 비밀번호 재설정 API
   - 멱등성 보장

---

## 📅 작업 타임라인

| 날짜 | 작업 내용 | 커밋 |
|------|-----------|------|
| 2025-10-25 | 토큰 갱신 API 구현 | `0941571` |
| 2025-10-27 | 회원가입 이메일 인증 우회 | `ad4e3d7` |
| 2025-10-28 | Admin 토큰 획득 방식 개선 | `9f194e2` ~ `642c146` |
| 2025-10-28 | Service account 사용자 삭제 | `5e716be` |
| 2025-10-28 | 회원가입 정책 변경 | `416e71d` |
| 2025-11-10 | 인증 API 프록시 구현 | `81416f7` |
| 2025-11-13 | 전반적인 리팩토링 | `cf96ef0` |

---

## 🔑 주요 변경사항

### 1. 새로운 API 엔드포인트

#### 토큰 갱신
```http
POST /api/auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

#### 로그인 (변경됨)
```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "user@example.com",
  "password": "password123"
}
```

#### 비밀번호 재설정
```http
PUT /api/users/{user_id}/reset-password
Content-Type: application/json

{
  "new_password": "newPassword123"
}
```

---

### 2. 주요 메서드 추가

| 메서드 | 설명 |
|--------|------|
| `authenticate_user()` | Username/password로 Keycloak 인증 |
| `refresh_access_token()` | Refresh token으로 access token 갱신 |
| `reset_user_password()` | 관리자 권한으로 비밀번호 재설정 |
| `delete_user_by_username()` | Username으로 사용자 삭제 |

---

### 3. 동작 변경

| 항목 | 이전 | 이후 |
|------|------|------|
| Admin 토큰 획득 | Password grant | Client credentials |
| 회원가입 | 이메일 인증 필수 | 관리자 승인 필요 |
| 로그인 | Keycloak ID | Username/password |
| 사용자 삭제 404 | 에러 | 성공으로 처리 |

---

## 📊 영향 범위

### 변경된 파일
1. `pacs-server/src/infrastructure/external/keycloak_client.rs` (주요)
2. `pacs-server/src/domain/services/auth_service.rs`
3. `pacs-server/src/application/use_cases/auth_use_case.rs`
4. `pacs-server/src/application/dto/auth_dto.rs`
5. `pacs-server/src/presentation/controllers/auth_controller.rs`

### 추가된 테스트
- `pacs-server/tests/keycloak_client_refresh_token_test.rs`
- `pacs-server/tests/auth_service_refresh_token_test.rs`
- `pacs-server/tests/auth_use_case_refresh_token_test.rs`
- `pacs-server/tests/auth_controller_refresh_token_test.rs`
- `pacs-server/tests/refresh_token_integration_test.rs`
- `pacs-server/tests/refresh_token_performance_test.rs`

---

## 🔒 보안 개선

### 1. Service Account 사용
- Admin 계정 정보 노출 위험 제거
- Client credentials만으로 관리 작업 수행
- Keycloak 권장 방식 준수

### 2. 토큰 보안
- Access token: 5분 유효
- Refresh token: 30분 유효
- Refresh token rotation 구현

### 3. 관리자 승인
- 무분별한 회원가입 방지
- 스팸 계정 차단
- 서비스 남용 방지

---

## 🚀 사용 방법

### 환경 변수 설정
```bash
# .env
KEYCLOAK_URL=https://keycloak.example.com
KEYCLOAK_REALM=dcm4che
KEYCLOAK_CLIENT_ID=pacs-server
KEYCLOAK_CLIENT_SECRET=your-client-secret
```

### Keycloak 설정
1. Clients → pacs-server 선택
2. Settings 탭
   - Access Type: `confidential`
   - Service Accounts Enabled: `ON`
3. Service Account Roles 탭
   - realm-management → realm-admin 역할 할당

---

## 📚 관련 문서

### 프로젝트 문서
- [작업계획.md](./작업계획.md) - 프로젝트 계획 및 일정
- [작업내용.md](./작업내용.md) - 상세 작업 내용
- [기술문서.md](./기술문서.md) - 기술 아키텍처

### 외부 문서
- [docs/api/AUTH_API.md](../../api/AUTH_API.md) - 인증 API 상세 문서
- [work/token_refresh_api/](../../../work/token_refresh_api/) - 토큰 갱신 작업 문서
- [work/email_verification_bypass/](../../../work/email_verification_bypass/) - 이메일 인증 우회 작업 문서

---

## 🔄 마이그레이션 가이드

### 클라이언트 측 변경

#### 1. 로그인 API
```javascript
// ❌ 이전
POST /api/auth/login
{ "keycloak_id": "user-uuid-123" }

// ✅ 이후
POST /api/auth/login
{ "username": "user@example.com", "password": "password123" }
```

#### 2. 토큰 갱신 (신규)
```javascript
// ✅ 새로운 기능
POST /api/auth/refresh
{ "refresh_token": "..." }
```

#### 3. 회원가입 후 처리
```javascript
// ✅ 사용자에게 안내
"회원가입이 완료되었습니다. 관리자 승인 후 이용 가능합니다."
```

---

## ✅ 완료 항목

- [x] 토큰 갱신 API 구현
- [x] Service account 전환
- [x] 관리자 승인 방식 구현
- [x] 인증 프록시 구현
- [x] 사용자 관리 기능 강화
- [x] 단위/통합/성능 테스트 작성
- [x] API 문서 작성
- [x] 마이그레이션 가이드 작성

---

## 🎯 향후 계획

### 단기 (1개월)
- [ ] 토큰 캐싱 구현
- [ ] Rate limiting 구현
- [ ] E2E 테스트 작성

### 중기 (3개월)
- [ ] Token revocation 구현
- [ ] 배치 작업 최적화
- [ ] 감사 로그 구현

### 장기 (6개월)
- [ ] Multi-factor authentication
- [ ] Social login 지원
- [ ] SSO 통합

---

## 📞 문의

프로젝트 관련 문의사항은 Backend Team에 연락해주세요.

---

**최종 업데이트**: 2025-11-13
