# Database Migrations

PACS Extension Server의 데이터베이스 마이그레이션 파일들입니다.

## 📋 마이그레이션 목록

### 001_initial_schema.sql
- **생성일**: 2025-10-16
- **목적**: 초기 데이터베이스 스키마 생성
- **내용**:
  - ENUM 타입 생성 (condition_type_enum, resource_level_enum, grant_action_enum)
  - Security 스키마: 사용자, 프로젝트, 역할, 권한, 그룹 관리
  - Viewer 스키마: Hanging Protocol 관리
  - Annotation 스키마: DICOM 어노테이션 관리
  - 전체 인덱스 및 주석 생성

**생성되는 테이블**:
- `security_user` - 사용자 정보 (Keycloak 연동)
- `security_project` - 프로젝트 관리
- `security_role` - 역할 정의
- `security_permission` - 권한 정의
- `security_role_permission` - 역할-권한 매핑
- `security_user_project` - 사용자-프로젝트 멤버십
- `security_project_role` - 프로젝트-역할 매핑
- `security_project_permission` - 프로젝트-권한 매핑
- `security_access_condition` - DICOM 태그 기반 접근 조건
- `security_role_access_condition` - 역할-접근조건 매핑
- `security_project_access_condition` - 프로젝트-접근조건 매핑
- `security_group` - 사용자 그룹
- `security_grant_log` - 권한 부여 이력
- `security_access_log` - 접근 로그
- `security_user_group` - 사용자-그룹 매핑
- `security_group_role` - 그룹-역할 매핑
- `viewer_hanging_protocol` - Hanging Protocol 정의
- `viewer_hp_condition` - HP 조건
- `viewer_hp_layout` - HP 레이아웃
- `viewer_hp_viewport` - HP 뷰포트
- `annotation_annotation` - DICOM 어노테이션
- `annotation_annotation_history` - 어노테이션 히스토리

### 002_initial_seed_data.sql
- **생성일**: 2025-10-16
- **목적**: 초기 시드 데이터 추가
- **내용**:
  - 기본 역할 생성 (SUPER_ADMIN, PROJECT_ADMIN, RESEARCHER, VIEWER, ANNOTATOR)
  - 기본 권한 생성 (USER, PROJECT, STUDY, SERIES, INSTANCE, ANNOTATION, MASK, HANGING_PROTOCOL)
  - 역할-권한 매핑

**역할별 권한**:
- **SUPER_ADMIN**: 모든 권한
- **PROJECT_ADMIN**: 프로젝트 내 모든 권한
- **RESEARCHER**: 읽기, 쓰기, 어노테이션 권한
- **ANNOTATOR**: 어노테이션 작성 권한
- **VIEWER**: 읽기 전용 권한

### 003_add_mask_tables.sql
- **생성일**: 2025-10-07
- **목적**: 마스크 업로드 기능을 위한 테이블 추가
- **내용**:
  - `annotation_mask_group` - 마스크 그룹 관리
  - `annotation_mask` - 개별 마스크 파일 정보
  - 관련 인덱스 및 주석

**주요 기능**:
- AI 모델 기반 세그멘테이션 마스크 저장
- 슬라이스별, 라벨별 마스크 관리
- S3/MinIO 객체 저장소 연동
- DICOM SOP Instance UID 매핑

## 🚀 마이그레이션 실행 방법

### 1. SQLx CLI 사용
```bash
# 마이그레이션 실행
sqlx migrate run

# 마이그레이션 되돌리기
sqlx migrate revert

# 마이그레이션 상태 확인
sqlx migrate info
```

### 2. Makefile 사용
```bash
# 마이그레이션 실행
make db-migrate

# 데이터베이스 리셋 (주의: 모든 데이터 삭제)
make db-reset

# PostgreSQL 쉘 접속
make db-shell
```

### 3. Docker Compose 사용
```bash
# 전체 스택 실행 (자동으로 마이그레이션 실행됨)
make compose-up

# 데이터베이스만 실행
docker-compose up -d postgres

# 마이그레이션 수동 실행
docker-compose exec pacs-server sqlx migrate run
```

## 📝 마이그레이션 작성 규칙

1. **파일명 형식**: `{번호}_{설명}.sql`
   - 예: `004_add_study_metadata.sql`
   - 번호는 3자리 숫자로 통일

2. **파일 헤더 포함**:
   ```sql
   -- Migration: {설명}
   -- Created: {날짜}
   -- Description: {상세 설명}
   ```

3. **트랜잭션 처리**:
   - SQLx는 각 마이그레이션을 자동으로 트랜잭션으로 실행
   - 마이그레이션 실패 시 자동 롤백

4. **롤백 고려**:
   - 가능한 경우 역마이그레이션 작성
   - `{번호}_{설명}.down.sql` 파일 생성

5. **데이터 무결성**:
   - 외래 키 제약조건 명시
   - NOT NULL 제약조건 신중히 사용
   - 기본값 설정 권장

6. **인덱스 생성**:
   - 조회 성능이 중요한 컬럼에 인덱스 추가
   - 복합 인덱스 고려

7. **주석 작성**:
   - COMMENT ON 문으로 테이블/컬럼 설명 추가
   - 한글 설명 권장

## ⚠️ 주의사항

1. **프로덕션 환경**:
   - 마이그레이션 전 반드시 백업
   - 테스트 환경에서 먼저 검증
   - 마이그레이션 순서 엄격히 준수

2. **성능 영향**:
   - 대용량 데이터가 있을 경우 마이그레이션 시간 고려
   - 인덱스 생성 시 CONCURRENTLY 옵션 고려
   - 테이블 잠금 최소화

3. **호환성**:
   - PostgreSQL 버전 호환성 확인
   - SQLx 버전과 호환되는 SQL 문법 사용

4. **롤백 불가능한 경우**:
   - 데이터 삭제 마이그레이션
   - 컬럼 타입 변경 (데이터 손실 가능)
   - 제약조건 추가 (기존 데이터와 충돌)

## 🔍 트러블슈팅

### 마이그레이션 실패 시
```bash
# 마이그레이션 상태 확인
sqlx migrate info

# 마이그레이션 히스토리 확인
psql -U admin -d pacs_db -c "SELECT * FROM _sqlx_migrations ORDER BY installed_on DESC;"

# 수동으로 마이그레이션 되돌리기
sqlx migrate revert

# 데이터베이스 완전 리셋
make db-reset
```

### 중복 마이그레이션 오류
- `_sqlx_migrations` 테이블에서 중복 항목 제거
- 마이그레이션 파일 번호 확인

### 외래 키 제약조건 오류
- 참조되는 테이블이 먼저 생성되었는지 확인
- 데이터 정합성 확인

## 📚 참고 문서

- [SQLx Migration Guide](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Clean Architecture Database Pattern](../docs/technical/clean-architecture.md)

