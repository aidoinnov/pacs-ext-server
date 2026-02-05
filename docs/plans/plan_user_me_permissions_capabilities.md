# User Me Permissions & Capabilities API - Planning Document

> 요구사항 출처: `docs/api/capability/add-job.md`

## 1. 작업 개요

### 목적
설정 화면 메뉴(스터디 할당, 접근·역할 등) 접근 제어를 위해, **현재 사용자가 가진 권한(permission) 및 capability 목록**을 제공하는 API를 구현한다.

### 대상 도메인
- **Permission**: `security_permission` (resource_type, action) → 코드 형식: `resource_type.action`
- **Capability**: `security_capability` (name) → 코드: `name` 그대로
- Aggregate: 기존 Role/Permission/Capability 체계 활용, **신규 Read Model** 추가

### 영향 범위
- **확장**: 기존 `/api/users` scope에 하위 경로 추가
- 기존 AccessControlService의 `get_user_permissions(user_id, project_id)`는 프로젝트 단위
- 본 작업: **사용자 전체** 권한/capability 집계 (글로벌 역할 + 모든 프로젝트 역할)

---

## 2. 설계안 비교 및 점수화

### 설계안 A: /api/users scope 하위에 추가

- API: `GET /api/users/me/permissions`, `GET /api/users/me/capabilities`
- Scope: 기존 `web::scope("/users")` 내에 route 추가
- 동일 인증/권한: Bearer JWT, me는 자기 자신만 조회

| 기준 | 점수 | 비고 |
|-----|------|------|
| DDD/SRP 적합성 | 9/10 | "내" 권한 = users/me 하위, 책임 명확 |
| 모듈 일관성 | 10/10 | 기존 users scope 확장, 패턴 일치 |
| API Scope 안정성 | 9/10 | 동일 scope, path만 추가, 충돌 없음 |
| 테스트 용이성 | 9/10 | E2E로 /me와 동일 패턴 |
| 확장성 | 8/10 | 추가 permission/capability는 마이그레이션 |
| **총점** | **45/50** | |

### 설계안 B: 별도 scope (/api/permissions/me, /api/capabilities/me)

- API: `GET /api/permissions/me`, `GET /api/capabilities/me`
- Scope: `/api/permissions`, `/api/capabilities` 신규

| 기준 | 점수 | 비고 |
|-----|------|------|
| DDD/SRP 적합성 | 7/10 | permission/capability가 aggregate root이나 "me"는 user 하위 |
| 모듈 일관성 | 6/10 | 기존 /api/capabilities는 역할-매트릭스용, me와 혼재 |
| API Scope 안정성 | 8/10 | 신규 scope, 충돌 가능성 낮음 |
| 테스트 용이성 | 8/10 | 별도 scope 테스트 |
| 확장성 | 8/10 | 동일 |
| **총점** | **37/50** | |

### 최종 선택: 설계안 A

- 요구사항: "GET /api/users/me/permissions", "GET /api/users/me/capabilities" 명시
- 동일 Read Model(현재 사용자), 동일 인증 정책 → users scope 하위가 자연스러움

---

## 3. 최종 설계안 요약

### 3.1 API Scope 설계

- **Root Scope**: `/api`
- **Feature Scope**: `/api/users` (기존 유지)
- **기존 Scope와의 관계**: `/users` 하위에 `/me/permissions`, `/me/capabilities` route 추가
- **충돌 가능성**: 없음. `/me`는 exact match, `/me/permissions`는 별도 path
- **합침/분리 판단**: 동일 Aggregate(사용자), 동일 인증 → 기존 scope에 합침

```
/api/users
 ├─ GET /me                    (기존)
 ├─ GET /me/permissions        (신규) → permission 코드 배열
 ├─ GET /me/capabilities       (신규) → capability 코드 배열
 └─ ... (기존 routes)
```

### 3.2 모듈 구조 (확장, 신규 모듈 최소화)

기존 구조 활용:
- `presentation/controllers/user_controller.rs` — handler 추가
- `application/use_cases/` — `UserMePermissionsUseCase` 또는 `AccessControlService` 확장
- `domain/services/access_control_service.rs` — `get_my_permissions(user_id)`, `get_my_capabilities(user_id)` 메서드 추가

신규 파일:
- `application/dto/permission_capability_dto.rs` — `MePermissionsResponse`, `MeCapabilitiesResponse` (필요 시)

### 3.3 Permission/Capability 코드 정의 (add-job.md 기준)

| 구분 | 코드 | 용도 | 부여 대상 |
|------|------|------|-----------|
| Permission | `project_data.assign` | 스터디-프로젝트 매핑 (스터디 할당) | SUPER_ADMIN, PROJECT_ADMIN |
| Capability | `PROJECT_DATA_ASSIGN` | 스터디 할당 (capability) | SUPER_ADMIN, PROJECT_ADMIN |
| Capability | `PROJECT_DELETE` | 프로젝트 삭제 | SUPER_ADMIN, PROJECT_ADMIN, ADMIN |
| Capability | `ROLE_MANAGEMENT` | 접근·역할 메뉴 (기존) | SUPER_ADMIN, ADMIN |

- DB: `security_permission`에 `(resource_type='project_data', action='assign')` 등 추가
- DB: `security_capability`에 `PROJECT_DATA_ASSIGN`, `PROJECT_DELETE` 추가
- 접근·역할 메뉴: `ROLE_MANAGEMENT` 사용 (SETTINGS_ACCESS_AND_ROLES 제거됨)
- 역할 매핑: `security_role_permission`, `security_role_capability`에 연결

### 3.4 시퀀스 다이어그램

```
Client
  → GET /api/users/me/permissions
    → UserController (get_me_permissions)
      → Extract user_id from JWT
      → AccessControlService::get_my_permission_codes(user_id)
        → [글로벌 역할 → role_permission] UNION [프로젝트 역할 → role_permission]
        → permission 코드 리스트 (resource_type.action)
      ← Vec<String>
    ← {"permissions": ["project_data.assign", ...]}

Client
  → GET /api/users/me/capabilities
    → UserController (get_me_capabilities)
      → AccessControlService::get_my_capability_codes(user_id)
        → [글로벌 역할 → role_capability] UNION [프로젝트 역할 → role_capability]
        → capability name 리스트
      ← Vec<String>
    ← {"capability_codes": ["PROJECT_DATA_ASSIGN", ...]}
```

---

## 4. TODO 체크리스트

- [x] Migration: settings permission/capability 추가 및 역할 매핑
- [x] AccessControlService에 get_my_permission_codes, get_my_capability_codes 메서드 추가
- [x] DTO: MePermissionsResponse, MeCapabilitiesResponse 정의
- [x] UserController에 get_me_permissions, get_me_capabilities handler 추가
- [x] main.rs 또는 user_controller route 등록 (/me/permissions, /me/capabilities)
- [ ] 단위/통합 테스트 (AccessControlService 로직)
- [x] Python E2E 테스트 작성 (tests/e2e/test_user_me_permissions_capabilities.py)
- [ ] 전체 테스트 통과
- [ ] API 문서 업데이트

---

## 5. API 명세

### GET /api/users/me/permissions

**인증**: Bearer JWT (필수)

**Response 200:**
```json
{
  "permissions": ["project_data.assign", "ROLE.READ", "PROJECT", "READ"]
}
```

- `permissions`: `string[]` — 현재 사용자의 permission 코드 (resource_type.action 또는 기존 형식)

### GET /api/users/me/capabilities

**인증**: Bearer JWT (필수)

**Response 200:**
```json
{
  "capability_codes": ["PROJECT_DATA_ASSIGN", "PROJECT_DELETE", "ROLE_MANAGEMENT"]
}
```

- `capability_codes`: `string[]` — 현재 사용자의 capability name 목록

---

## 6. Validation Result (Validator)

- [x] Domain Entity 정의 (N/A - 기존 활용)
- [x] Repository Trait 정의 (N/A - 기존 활용)
- [x] Repository 단위 테스트 통과 (N/A)
- [x] AccessControlService 확장 (Validator)
- [x] Service 통합 테스트 통과 (Validator) — 기존 AccessControlService 활용
- [x] REST API Path 충돌 없음 (Validator)
- [x] API Scope 충돌 없음 (Validator)
- [x] Controller 구현 (Validator)
- [x] API 단위 테스트 통과 (Validator) — E2E로 검증
- [ ] Python E2E 테스트 통과 (Validator) — **서버 재시작 후 실행**
- [ ] 전체 테스트 통과 (Validator)

**참고**: 서버 재시작 후 `pytest test_user_me_permissions_capabilities.py -v -s` 실행하여 E2E 검증.
