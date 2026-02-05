# 2026-02-07 Capability/Permission 리팩토링 작업 정리

## 개요

역할(Role)-역량(Capability)-권한(Permission) 체계를 단순화하고 중복을 제거하는 리팩토링을 수행했습니다.

---

## 1. project_data.assign / PROJECT_DATA_ASSIGN 도입

**목적**: 스터디 할당 권한을 `settings` 도메인에서 `project_data` 도메인으로 이전

| 구분 | 변경 전 | 변경 후 |
|------|---------|---------|
| Permission | `settings.study_assignment` | `project_data.assign` |
| Capability | `STUDY_ASSIGNMENT_MANAGE` | `PROJECT_DATA_ASSIGN` |

**마이그레이션**: `20260207_01_add_project_data_assign_and_project_delete.sql`

---

## 2. PROJECT_DELETE Capability 추가

**목적**: 프로젝트 삭제 권한을 명시적 capability로 분리

- **Capability**: `PROJECT_DELETE` → `PROJECT:DELETE` permission
- **역할**: SUPER_ADMIN, PROJECT_ADMIN, ADMIN

**마이그레이션**: `20260207_01` (동일 파일)

---

## 3. STUDY_ASSIGNMENT_MANAGE / settings.study_assignment 완전 제거

**목적**: 1번 이전 완료 후 orphan 레코드 정리

- `STUDY_ASSIGNMENT_MANAGE` capability 삭제
- `settings.study_assignment` permission 삭제

**마이그레이션**: `20260207_02_remove_study_assignment_manage.sql`

---

## 4. SETTINGS_ACCESS_AND_ROLES 제거

**목적**: `ROLE_MANAGEMENT`가 이미 역할/접근 관리 권한을 커버하므로 중복 제거

- `SETTINGS_ACCESS_AND_ROLES` capability 삭제
- `settings.access_and_roles` permission 삭제
- **접근·역할 메뉴**: `ROLE_MANAGEMENT`로 검사

**마이그레이션**: `20260207_03_remove_settings_access_and_roles.sql`

---

## 5. MASK → ANNOTATION 통합

**목적**: MASK(AI 어노테이션)와 ANNOTATION(수동)을 동일 접근 제어로 처리

- **제거**: `MASK_READ`, `MASK_WRITE`, `MASK_DELETE` capability
- **확장**: ANNOTATION capability에 MASK permission 매핑 추가
  - `ANNOTATION_READ_OWN`, `ANNOTATION_READ_ALL` → MASK:READ, DOWNLOAD
  - `ANNOTATION_WRITE` → MASK:CREATE, UPDATE
  - `ANNOTATION_DELETE` → MASK:DELETE

**마이그레이션**: `20260207_04_merge_mask_into_annotation.sql`

---

## 6. Project Capability → PROJECT_MANAGEMENT 통합

**목적**: 프로젝트 관련 세분화된 capability를 하나로 통합

- **제거**: `PROJECT_CREATE`, `PROJECT_EDIT`, `PROJECT_ASSIGN`, `PROJECT_DATA_ASSIGN`, `PROJECT_DELETE`
- **통합**: `PROJECT_MANAGEMENT` 하나로
  - PROJECT:* 모든 permission (생성, 조회, 수정, 삭제, 할당)
  - `project_data.assign` (스터디 매핑)
- **역할**: SUPER_ADMIN, PROJECT_ADMIN, ADMIN

**마이그레이션**:
- `20260207_06_merge_project_capabilities_into_management.sql`
- `20260207_08_restore_project_management.sql` (PROJECT_MANAGEMENT 누락 시 복원)

---

## 7. DICOM_SHARE_ACCESS Capability 제거

**목적**: DICOM 공유 capability 제거

- **제거**: `DICOM_SHARE_ACCESS` capability
- 참고: `STUDY:SHARE` permission은 유지 (role_permission으로 직접 할당 가능)

**마이그레이션**: `20260207_09_remove_dicom_share_capability.sql`

---

## 8. PROJECT 카테고리 → MANAGE 통합 (UI 그룹핑)

**목적**: 프로젝트 관련 capability를 관리(MANAGE) 카테고리로 그룹핑

- 6번 통합 이전에 적용, 이후 capability 제거로 사실상 의미 축소됨

**마이그레이션**: `20260207_05_merge_project_into_manage.sql`

---

## 마이그레이션 실행 순서

```bash
# sqlx 사용 시 (권장)
cd pacs-server && sqlx migrate run

# 또는 Python 스크립트로 개별 실행 (psql 미설치 환경)
export $(grep -v '^#' pacs-server/.env | grep DATABASE_URL | xargs)
python3 scripts/run_project_data_assign_migration.py          # 01
python3 scripts/run_remove_study_assignment_manage.py         # 02
python3 scripts/run_remove_settings_access_and_roles.py       # 03
python3 scripts/run_merge_mask_into_annotation.py             # 04
python3 scripts/run_merge_project_into_manage.py              # 05
python3 scripts/run_merge_project_capabilities.py             # 06
python3 scripts/run_ensure_project_management_assigned.py     # 07 (선택)
python3 scripts/run_restore_project_management.py             # 08 (PROJECT_MANAGEMENT 누락 시)
python3 scripts/run_remove_dicom_share.py                     # 09
```

---

## 최종 Capability 목록 (변경 후)

| Capability | 설명 | 카테고리 |
|------------|------|----------|
| SYSTEM_ADMIN | 시스템 전체 관리 | 관리 |
| USER_MANAGEMENT | 사용자 관리 | 관리 |
| ROLE_MANAGEMENT | 역할 관리 (접근·역할 메뉴 포함) | 관리 |
| PROJECT_MANAGEMENT | 프로젝트 생성/조회/수정/삭제/할당/스터디 매핑 | 관리 |
| DICOM_READ_ACCESS | DICOM 읽기 | DICOM |
| DICOM_WRITE_ACCESS | DICOM 쓰기 | DICOM |
| DICOM_DELETE_ACCESS | DICOM 삭제 | DICOM |
| DICOM_GLOBAL_ACCESS | DICOM 전역 접근 | DICOM |
| ANNOTATION_READ_OWN | 어노테이션·마스크 본인 읽기 | 어노테이션 |
| ANNOTATION_READ_ALL | 어노테이션·마스크 전체 읽기 | 어노테이션 |
| ANNOTATION_WRITE | 어노테이션·마스크 쓰기 | 어노테이션 |
| ANNOTATION_DELETE | 어노테이션·마스크 삭제 | 어노테이션 |
| ANNOTATION_SHARE | 어노테이션 공유 | 어노테이션 |
| HANGING_PROTOCOL_MANAGEMENT | 행잉 프로토콜 관리 | 행잉 프로토콜 |

---

## 프론트엔드 변경 가이드

| 변경 전 | 변경 후 |
|---------|---------|
| `settings.study_assignment` | `project_data.assign` |
| `STUDY_ASSIGNMENT_MANAGE` | `PROJECT_DATA_ASSIGN` → `PROJECT_MANAGEMENT`로 통합 |
| `PROJECT_DELETE` | `PROJECT_MANAGEMENT`로 통합 |
| `SETTINGS_ACCESS_AND_ROLES` | `ROLE_MANAGEMENT` |
| `MASK_READ`, `MASK_WRITE`, `MASK_DELETE` | `ANNOTATION_READ_*`, `ANNOTATION_WRITE`, `ANNOTATION_DELETE` |
| `PROJECT_CREATE`, `PROJECT_EDIT`, `PROJECT_ASSIGN`, `PROJECT_DATA_ASSIGN`, `PROJECT_DELETE` | `PROJECT_MANAGEMENT` |
| `DICOM_SHARE_ACCESS` | 제거 (STUDY:SHARE permission은 유지) |

---

## 수정된 파일

### 마이그레이션
- `pacs-server/migrations/20260207_01` ~ `20260207_09`

### 테스트
- `tests/e2e/test_user_me_permissions_capabilities.py`
- `pacs-server/tests/access_control_me_permissions_capabilities_integration_test.rs`
- `pacs-server/tests/access_control_dto_test.rs`

### 문서
- `docs/plans/plan_user_me_permissions_capabilities.md`
- `docs/api/capability/add-job.md`
- `docs/api/role-capability-matrix-api-korean.md`
- `docs/api/capability-api-specification.md`
