# 설정 화면 권한·Capability 백엔드 요구사항

## 1. 개요

설정 페이지 사이드바 및 라우트 접근을 **역할(role)** 과 **권한(permission)/capability** 기반으로 제어하기 위해, 백엔드에서 “현재 사용자가 가진 권한·capability 목록”을 제공해 주실 수 있는 API를 요청합니다.

- **현재**: `GET /api/users/me` 응답의 `global_role_names`(예: `["ADMIN"]`, `["USER"]`)만 사용하여, ADMIN 전용 메뉴(접근·역할, 스터디 할당)를 노출/가드하고 있음.
- **목표**: “스터디 할당”을 **ADMIN + 프로젝트 관리자**에게 공통으로 노출하고, 향후 다른 메뉴도 **capability/permission** 단위로 제어할 수 있도록 함.

**제공 방식**: permission과 capability는 **따로 조회하는 API**로 제공해 주셔도 됩니다. 다만 **둘 다 제공**해 주시길 바라며, 프론트에서는 필요한 API를 각각 호출한 뒤 두 결과를 합쳐서 “이 사용자가 가진 권한/코드 목록”으로 사용합니다.

---

## 2. 프론트엔드 현재 동작 요약

| 구분 | 노출 대상 | 근거 |
|------|-----------|------|
| 내 작업 (참여중인 프로젝트, 프로필, 알림) | 모든 로그인 사용자 | 제한 없음 |
| 프로젝트 목록 | 모든 로그인 사용자 | 제한 없음 |
| 스터디 할당 | **ADMIN만** (임시) | `global_role_names`에 `ADMIN` 포함 시 |
| 접근·역할 (역할 할당, 사용자 관리, 권한 매트릭스) | **ADMIN만** | `global_role_names`에 `ADMIN` 포함 시 |

- 스터디 할당은 **프로젝트 관리자 + ADMIN**으로 확장하려고 하며, 이를 위해 **permission/capability** 근거가 필요함.

---

## 3. 백엔드 요구사항

### 3.1. 제공 방식: 따로 조회 가능, 둘 다 제공 요청

- **Permission**과 **Capability**는 서로 다른 API로 **따로 조회**해 주셔도 됩니다.
- 단, 프론트에서 “메뉴/기능 노출 여부”를 판단할 때 **permission 결과와 capability 결과를 함께** 쓰기 때문에, **둘 다** 제공해 주시길 바랍니다.
  - 예: `GET /api/users/me/permissions` → permission 코드 목록  
  - 예: `GET /api/users/me/capabilities` (또는 기존 Role API의 “내 capability 목록” 엔드포인트) → capability 코드 목록  
  - 프론트는 두 API를 각각 호출한 뒤, “이 사용자가 가진 코드 목록”을 합쳐서 사용합니다.

아래는 **구현 형태 예시**이며, 백엔드 설계에 맞게 **분리된 엔드포인트**로 제공하셔도 됩니다.

---

### 3.2. 옵션 A: 별도 엔드포인트로 제공 (따로 조회)

#### A-1. 현재 사용자 Permission 목록

- **API**: `GET /api/users/me/permissions` (또는 `GET /api/permissions/me` 등)
- **인증**: Authorization: Bearer \<token\>
- **응답**: 현재 사용자가 가진 **권한 코드(permission code)** 배열.

**응답 예시:**

```json
{
  "permissions": [
    "settings.study_assignment",
    "project.view"
  ]
}
```

- **`permissions`**: `string[]` — 글로벌 역할 + 프로젝트 역할(예: 프로젝트 관리자)을 조합해 부여된 권한 코드만 내려주시면 됨.

#### A-2. 현재 사용자 Capability 목록

- **API**: `GET /api/users/me/capabilities` (또는 기존 Role/Capability 도메인의 “현재 사용자 capability” 조회 API)
- **인증**: Authorization: Bearer \<token\>
- **응답**: 현재 사용자의 역할들에 연결된 **capability 코드** 배열.

**응답 예시:**

```json
{
  "capability_codes": [
    "STUDY_ASSIGNMENT_MANAGE",
    "PROJECT_VIEW"
  ]
}
```

- **`capability_codes`**: `string[]` — 글로벌/프로젝트 역할–capability 매트릭스에서 유도한 목록.

프론트는 위 두 API를 **각각 호출**한 뒤, `permissions`와 `capability_codes`를 합쳐서 “노출 가능한 코드 목록”으로 사용합니다.

---

### 3.3. 옵션 B: 기존 GET /api/users/me 응답에 함께 포함

- **API**: `GET /api/users/me`
- **응답 확장**: `permissions: string[]`, `capability_codes: string[]` 를 **같이** 넣어 주시는 방식입니다. (따로 조회하지 않고 한 번에 받고 싶을 때 사용)

**응답 예시 (확장 분만):**

```json
{
  "id": 336,
  "keycloak_id": "...",
  "username": "alice",
  "email": "alice@example.com",
  "global_role_names": ["USER"],
  "created_at": "2025-10-30T12:34:56Z",
  "updated_at": null,

  "permissions": ["settings.study_assignment", "project.view"],
  "capability_codes": ["STUDY_ASSIGNMENT_MANAGE", "PROJECT_VIEW"]
}
```

- permission과 capability를 **같이** 넣어 주시면, 프론트는 한 번의 호출로 둘 다 사용할 수 있습니다.

---

### 3.4. 스터디 할당용 권한 코드 (필수 요청)

- **권한 코드 이름(예시)**: `settings.study_assignment`  
  (또는 capability 기반이면 예: `STUDY_ASSIGNMENT_MANAGE` 등 백엔드 규칙에 맞게 정의)
- **부여 대상**  
  - 글로벌 역할 **ADMIN**  
  - 프로젝트 역할 **프로젝트 관리자** (해당 프로젝트에 한해 관리 권한이 있는 역할)
- **의도**  
  - 위 권한을 가진 사용자에게만 설정 사이드바에 “스터디 할당” 메뉴 노출 및 `/setting/series-project-mapping` 접근 허용.

백엔드에서 “프로젝트 관리자”를 어떤 역할명/코드로 두는지는 자유이며, 최종적으로 **위 권한 코드 하나**만 위 두 경우에 true 로 내려주시면 됩니다.

---

### 3.5. (선택) 권한 코드 목록 문서화

- 프론트에서 사용할 **권한 코드 목록**을 백엔드/문서에서 한 번 정의해 두면, 이후 설정/기능 추가 시 일관되게 사용 가능합니다.
- 예시 (설정 화면 위주):

| 코드 | 설명 | 부여 대상 예시 |
|------|------|----------------|
| `settings.study_assignment` | 스터디 할당 메뉴 및 페이지 접근 | ADMIN, 프로젝트 관리자 |
| `settings.access_and_roles` | 역할 할당·사용자 관리·권한 매트릭스 | ADMIN |
| (추가 시) | … | … |

---

## 4. 프론트엔드 연동 방식 (요약)

- **따로 조회**하는 경우: `GET /api/users/me/permissions`, `GET /api/users/me/capabilities` (또는 동등한 별도 API)를 각각 호출한 뒤, 응답의 **permission 목록**과 **capability 목록**을 합쳐서 “사용자가 가진 코드 목록”으로 사용합니다.
- **같이 넣어 주시는** 경우: `GET /api/users/me` 에 `permissions`와 `capability_codes`를 모두 포함해 주시면, 한 번의 호출로 둘 다 사용합니다.
- 노출/가드 로직:
  1. **사이드바**: `requiredPermission` / `requiredCapability`가 있는 메뉴는, 위에서 합친 목록에 해당 코드가 있을 때만 노출.
  2. **라우트 가드**: `/setting/series-project-mapping` 등은 해당 권한/코드가 있을 때만 접근 허용, 없으면 `/setting/profile` 로 리다이렉트.
- `global_role_names`는 당분간 유지해도 되며, “ADMIN 전용” 메뉴는 기존처럼 `global_role_names`에 `ADMIN` 포함 여부로 계속 처리 가능합니다.

---

## 5. 정리

| 항목 | 내용 |
|------|------|
| **제공 방식** | permission / capability **따로 조회**해도 됨. 단, **둘 다** 제공해 주시길 요청 (프론트에서 함께 사용). |
| **따로 조회 시 예시** | `GET /api/users/me/permissions`, `GET /api/users/me/capabilities` (또는 동등 API) |
| **한 번에 제공 시** | `GET /api/users/me` 에 `permissions: string[]`, `capability_codes: string[]` 함께 포함 |
| **우선 적용 권한** | `settings.study_assignment` (스터디 할당: ADMIN + 프로젝트 관리자) |
| **선택** | 권한 코드 목록 문서화, 역할별 부여 규칙 정리 |

---

## 6. 결론: 추가할 Permission / Capability

| 구분 | 코드 | 용도 | 부여 대상 |
|------|------|------|-----------|
| **Permission** | `settings.study_assignment` | 스터디 할당 메뉴·페이지 접근 | ADMIN, 프로젝트 관리자 |
| **Permission** | `settings.access_and_roles` | 역할 할당·사용자 관리·권한 매트릭스 | ADMIN |
| **Capability** | `STUDY_ASSIGNMENT_MANAGE` | 스터디 할당 (capability 체계 사용 시) | ADMIN, 프로젝트 관리자 |
| **Capability** | `SETTINGS_ACCESS_AND_ROLES` | 접근·역할 메뉴 (capability 체계 사용 시) | ADMIN |

- Permission **또는** Capability 중 백엔드에서 쓰는 체계 하나만 정해서 위 코드만 추가해 주시면 됨.
- 역할명(예: 프로젝트 관리자)은 백엔드 규칙에 맞게 정의.

---

## 7. 역할 생성·수정 시 범위(scope) 선택

- 역할을 만들거나 수정할 때 **범위를 반드시 선택**할 수 있도록 해 주시길 요청합니다.
  - **글로벌**: 시스템 전역 역할 (예: ADMIN, USER). 사용자에게 한 번 부여하면 전체 시스템에 적용.
  - **프로젝트별**: 특정 프로젝트 안에서만 의미하는 역할 (예: 프로젝트 관리자, 리더). 프로젝트 멤버로 지정할 때 해당 프로젝트용으로 부여.
- 이미 Role/Capability 모델에 `scope: 'GLOBAL' | 'PROJECT' | 'USER'` 등이 있다면, 역할 생성/수정 폼에서 “범위: 글로벌 / 프로젝트” 선택이 가능하도록 API·UI 지원을 요청합니다.

---

이 요구사항이 반영되면, 프론트엔드에서 스터디 할당을 “ADMIN + 프로젝트 관리자”에게 공통으로 노출·접근 허용하고, 이후 다른 설정 메뉴도 permission/capability 기반으로 확장할 수 있습니다.

---

## 8. 구현 완료 API (2026-02-05, 2026-02-07 갱신)

### GET /api/users/me/permissions

**인증**: Bearer JWT (필수)

**Response 200:**
```json
{
  "permissions": ["project_data.assign", "ROLE.READ", "PROJECT:DELETE", "PROJECT", "READ", ...]
}
```

### GET /api/users/me/capabilities

**인증**: Bearer JWT (필수)

**Response 200:**
```json
{
  "capability_codes": ["PROJECT_DATA_ASSIGN", "PROJECT_DELETE", "ROLE_MANAGEMENT", ...]
}
```

- **Permission 코드 형식**: resource_type.action (예: project_data.assign)
- **Capability 코드**: security_capability.name (예: PROJECT_DATA_ASSIGN, PROJECT_DELETE)
- **집계 범위**: 글로벌 역할 + 모든 프로젝트 역할
- **변경 이력 (2026-02-07)**: `settings.study_assignment`/`STUDY_ASSIGNMENT_MANAGE` → `project_data.assign`/`PROJECT_DATA_ASSIGN` 이전, `PROJECT_DELETE` capability 추가, `SETTINGS_ACCESS_AND_ROLES` 제거 (접근·역할은 `ROLE_MANAGEMENT` 사용)

