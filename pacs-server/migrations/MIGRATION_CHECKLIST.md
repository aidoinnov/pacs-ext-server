# 마이그레이션 체크리스트

## ✅ 수정 완료 사항

### 1. 새로 생성한 마이그레이션 파일
- ✅ `001_initial_schema.sql` - 전체 데이터베이스 스키마 생성
- ✅ `002_initial_seed_data.sql` - 초기 역할/권한 시드 데이터
- ✅ `003_add_mask_tables.sql` - 마스크 테이블 (기존 유지)
- ✅ `migrations/README.md` - 마이그레이션 문서

### 2. 삭제한 파일
- ✅ `004_add_updated_at_columns.sql` - 중복으로 삭제 (003에서 이미 추가됨)

### 3. 001_initial_schema.sql 수정 사항

#### 추가한 `created_at` 필드 (13개 테이블)
- ✅ `security_role_permission` - 역할-권한 매핑
- ✅ `security_user_project` - 사용자-프로젝트 멤버십
- ✅ `security_project_role` - 프로젝트-역할 매핑
- ✅ `security_project_permission` - 프로젝트-권한 매핑
- ✅ `security_access_condition` - 접근 조건
- ✅ `security_role_access_condition` - 역할-접근조건 매핑
- ✅ `security_project_access_condition` - 프로젝트-접근조건 매핑
- ✅ `security_user_group` - 사용자-그룹 매핑
- ✅ `security_group_role` - 그룹-역할 매핑
- ✅ `viewer_hanging_protocol` - Hanging Protocol
- ✅ `viewer_hp_condition` - HP 조건
- ✅ `viewer_hp_layout` - HP 레이아웃
- ✅ `viewer_hp_viewport` - HP 뷰포트

## 🔍 검증 완료 항목

### Rust 엔티티와 SQL 스키마 일치 확인
- ✅ `User` ↔ `security_user`
- ✅ `Project` ↔ `security_project`
- ✅ `Role` ↔ `security_role`
- ✅ `Permission` ↔ `security_permission` (created_at 없음 - 정상)
- ✅ `AccessCondition` ↔ `security_access_condition` (created_at 추가됨)
- ✅ `Group` ↔ `security_group`
- ✅ `GrantLog` ↔ `security_grant_log`
- ✅ `AccessLog` ↔ `security_access_log`
- ✅ `HangingProtocol` ↔ `viewer_hanging_protocol` (created_at 추가됨)
- ✅ `HpCondition` ↔ `viewer_hp_condition` (created_at 추가됨)
- ✅ `HpLayout` ↔ `viewer_hp_layout` (created_at 추가됨)
- ✅ `HpViewport` ↔ `viewer_hp_viewport` (created_at 추가됨)
- ✅ `Annotation` ↔ `annotation_annotation`
- ✅ `AnnotationHistory` ↔ `annotation_annotation_history`
- ✅ `MaskGroup` ↔ `annotation_mask_group`
- ✅ `Mask` ↔ `annotation_mask`

### 관계 테이블 (relations.rs) 검증
- ✅ `UserProject` ↔ `security_user_project` (created_at 추가됨)
- ✅ `ProjectRole` ↔ `security_project_role` (created_at 추가됨)
- ✅ `RolePermission` ↔ `security_role_permission` (created_at 추가됨)
- ✅ `ProjectPermission` ↔ `security_project_permission` (created_at 추가됨)
- ✅ `RoleAccessCondition` ↔ `security_role_access_condition` (created_at 추가됨)
- ✅ `ProjectAccessCondition` ↔ `security_project_access_condition` (created_at 추가됨)
- ✅ `UserGroup` ↔ `security_user_group` (created_at 추가됨)
- ✅ `GroupRole` ↔ `security_group_role` (created_at 추가됨)

## 📊 테이블 통계

### 생성되는 테이블 (총 23개)
- **Security 스키마**: 14개 테이블
  - `security_user`
  - `security_project`
  - `security_role`
  - `security_permission`
  - `security_role_permission`
  - `security_user_project`
  - `security_project_role`
  - `security_project_permission`
  - `security_access_condition`
  - `security_role_access_condition`
  - `security_project_access_condition`
  - `security_group`
  - `security_grant_log`
  - `security_access_log`
  - `security_user_group`
  - `security_group_role`

- **Viewer 스키마**: 4개 테이블
  - `viewer_hanging_protocol`
  - `viewer_hp_condition`
  - `viewer_hp_layout`
  - `viewer_hp_viewport`

- **Annotation 스키마**: 2개 테이블 (001에서 생성)
  - `annotation_annotation`
  - `annotation_annotation_history`

- **Mask 스키마**: 2개 테이블 (003에서 생성)
  - `annotation_mask_group`
  - `annotation_mask`

### ENUM 타입 (3개)
- ✅ `condition_type_enum` (ALLOW, DENY, LIMIT)
- ✅ `resource_level_enum` (STUDY, SERIES, INSTANCE)
- ✅ `grant_action_enum` (GRANT, REVOKE)

### 인덱스
- ✅ Security 인덱스: 15개
- ✅ Viewer 인덱스: 5개
- ✅ Annotation 인덱스: 6개
- ✅ Mask 인덱스: 4개 (003에서 추가)
- **총**: 30개 인덱스

## 🚀 실행 방법

```bash
# 방법 1: Makefile 사용 (권장)
make compose-down-volumes  # 기존 데이터 삭제
make compose-up-build      # 새로 시작
make db-migrate           # 마이그레이션 실행

# 방법 2: SQLx CLI 직접 사용
sqlx database drop -y
sqlx database create
sqlx migrate run

# 방법 3: 전체 스택 재시작
docker-compose down -v
docker-compose up -d --build
```

## ⚠️ 주의사항

1. **기존 데이터베이스가 있다면 반드시 백업하세요!**
2. `sqlx database reset -y` 명령은 모든 데이터를 삭제합니다.
3. 마이그레이션은 순서대로 실행됩니다: 001 → 002 → 003
4. 003은 001의 `annotation_annotation` 테이블에 의존합니다.

## 📝 변경 이력

- 2025-10-16: 001, 002 마이그레이션 생성, 004 삭제, created_at 필드 추가
- 2025-10-07: 003 마스크 테이블 생성

## 🔗 관련 문서

- [마이그레이션 README](./README.md)
- [Makefile 사용법](../makefile)
- [Docker Compose 설정](../docker-compose.yaml)

