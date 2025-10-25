# 사용자 회원가입 및 계정 삭제 API 구현 계획

## 📋 프로젝트 개요
PACS Extension Server에 사용자 회원가입 및 계정 삭제 기능을 구현하여 Keycloak과 연동된 사용자 생명주기 관리를 제공합니다.

## 🎯 목표
- Keycloak과 연동된 사용자 회원가입 시스템 구축
- 이메일 인증 및 관리자 승인 프로세스 구현
- 계정 삭제 기능 구현
- 감사 로그 시스템 구축
- Clean Architecture 패턴 적용

## 📅 작업 일정
- **시작일**: 2025-10-25
- **완료일**: 2025-10-25
- **소요시간**: 1일

## 🏗️ 아키텍처 설계

### Clean Architecture 4계층 구조
```
Presentation Layer (Controllers)
    ↓
Application Layer (Use Cases, DTOs)
    ↓
Domain Layer (Entities, Services, Repositories)
    ↓
Infrastructure Layer (Database, External Services)
```

### 주요 컴포넌트
1. **Domain Layer**
   - `User` 엔티티 확장 (계정 상태, 감사 로그 필드)
   - `UserRegistrationService` 트레이트
   - `UserAccountStatus` 열거형

2. **Application Layer**
   - `UserRegistrationUseCase`
   - DTOs (SignupRequest, VerifyEmailRequest, etc.)

3. **Infrastructure Layer**
   - `UserRegistrationServiceImpl`
   - `KeycloakClient`
   - `S3ObjectStorageService`

4. **Presentation Layer**
   - `auth_controller` (회원가입 API 통합)

## 🔧 기술 스택
- **Backend**: Rust (Actix-web)
- **Database**: PostgreSQL
- **Authentication**: Keycloak
- **Object Storage**: AWS S3
- **Documentation**: OpenAPI/Swagger

## 📊 데이터베이스 스키마 변경

### 1. 사용자 테이블 확장
```sql
-- security_user 테이블에 계정 상태 필드 추가
ALTER TABLE security_user ADD COLUMN account_status user_account_status_enum;
ALTER TABLE security_user ADD COLUMN email_verified BOOLEAN DEFAULT FALSE;
ALTER TABLE security_user ADD COLUMN email_verification_token VARCHAR(255);
ALTER TABLE security_user ADD COLUMN email_verification_expires_at TIMESTAMP;
ALTER TABLE security_user ADD COLUMN approved_by INTEGER;
ALTER TABLE security_user ADD COLUMN approved_at TIMESTAMP;
ALTER TABLE security_user ADD COLUMN suspended_at TIMESTAMP;
ALTER TABLE security_user ADD COLUMN suspended_reason TEXT;
ALTER TABLE security_user ADD COLUMN deleted_at TIMESTAMP;
```

### 2. 감사 로그 테이블 생성
```sql
CREATE TABLE security_user_audit_log (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    action VARCHAR(50) NOT NULL,
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## 🔌 API 엔드포인트 설계

### 1. 회원가입 API
- **POST** `/api/auth/signup`
- **기능**: 사용자 회원가입 처리
- **요청**: SignupRequest
- **응답**: SignupResponse

### 2. 이메일 인증 API
- **POST** `/api/auth/verify-email`
- **기능**: 이메일 인증 처리
- **요청**: VerifyEmailRequest
- **응답**: VerifyEmailResponse

### 3. 사용자 승인 API
- **POST** `/api/auth/admin/users/approve`
- **기능**: 관리자가 사용자 승인
- **요청**: ApproveUserRequest
- **응답**: ApproveUserResponse

### 4. 계정 삭제 API
- **DELETE** `/api/auth/users/{user_id}`
- **기능**: 사용자 계정 삭제
- **응답**: DeleteAccountResponse

## 🔐 보안 요구사항

### 1. 비밀번호 정책
- 최소 8자 이상
- 대문자, 소문자, 숫자, 특수문자 포함
- 일반적인 패턴 금지

### 2. 이메일 검증
- 유효한 이메일 형식 검증
- 중복 이메일 주소 방지

### 3. 감사 로깅
- 모든 사용자 액션 기록
- IP 주소 및 User-Agent 추적
- JSON 형태의 상세 정보 저장

## 🧪 테스트 계획

### 1. 단위 테스트
- Service Layer 테스트
- Use Case Layer 테스트
- Controller Layer 테스트

### 2. 통합 테스트
- API 엔드포인트 테스트
- 데이터베이스 연동 테스트
- Keycloak 연동 테스트

### 3. 성능 테스트
- 동시 회원가입 처리 테스트
- 대용량 감사 로그 처리 테스트

## 📈 성공 지표
- [ ] 회원가입 API 정상 작동
- [ ] 이메일 인증 프로세스 완료
- [ ] 관리자 승인 시스템 구축
- [ ] 계정 삭제 기능 구현
- [ ] 감사 로그 시스템 구축
- [ ] Keycloak 연동 완료
- [ ] S3 Object Storage 연동 완료
- [ ] 모든 테스트 통과

## 🚀 배포 계획
1. 개발 환경에서 테스트 완료
2. 스테이징 환경에서 통합 테스트
3. 프로덕션 환경 배포
4. 모니터링 및 로그 확인

## 📝 문서화
- API 문서 (OpenAPI/Swagger)
- 기술 문서
- 사용자 가이드
- 운영 가이드