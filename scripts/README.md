# PACS Extension Server Scripts

프로젝트 관리 및 마이그레이션을 위한 유틸리티 스크립트 모음

## 📋 목차

- [Subject 자동 생성 마이그레이션](#subject-자동-생성-마이그레이션)

---

## Subject 자동 생성 마이그레이션

### 개요

기존 프로젝트에 이미 할당된 Study들에 대해 Subject를 자동으로 생성하는 마이그레이션 도구입니다.

**사용 시나리오:**
- API 변경 전에 Study를 할당했지만 Subject가 없는 경우
- 대량의 Study가 할당되어 있어 수동으로 Subject를 생성하기 어려운 경우
- 기존 데이터를 새로운 Subject 자동 생성 로직에 맞추고 싶은 경우

### 설치

```bash
# Python 의존성 설치
pip install psycopg2-binary

# 또는 requirements.txt 사용
pip install -r scripts/requirements.txt
```

### 사용법

#### 1. 특정 프로젝트만 마이그레이션

```bash
python scripts/migrate_subjects.py --project-id 1
```

#### 2. 모든 활성 프로젝트 마이그레이션

```bash
python scripts/migrate_subjects.py --all-projects
```

#### 3. Dry-run (시뮬레이션)

실제로 생성하지 않고 어떤 Subject가 생성될지 미리 확인:

```bash
python scripts/migrate_subjects.py --project-id 1 --dry-run
```

#### 4. 커스텀 데이터베이스 URL

```bash
python scripts/migrate_subjects.py --all-projects \
  --db-url "postgresql://user:password@localhost:5432/pacs_extension"
```

### 동작 방식

1. **프로젝트 조회**: 지정된 프로젝트 또는 모든 활성 프로젝트 조회
2. **Study 조회**: 각 프로젝트에 할당된 Study 목록 조회 (patient_id가 있는 것만)
3. **Subject 확인**: Patient ID로 기존 Subject 찾기
   - 있으면 → 재사용 (로그만 출력)
   - 없으면 → 자동 생성
4. **Subject Code 생성**:
   - 1차: Patient ID 기반 (`P12345`)
   - 중복 시: Suffix 추가 (`P12345_1`, `P12345_2`, ...)
   - Fallback: 순차 번호 (`SUB001`, `SUB002`, ...)

### 출력 예시

#### Dry-run 모드
```
$ python scripts/migrate_subjects.py --all-projects --dry-run

2026-01-18 13:17:19 - INFO - Database: localhost:5456/pacs_extension
2026-01-18 13:17:19 - INFO - 🔍 DRY-RUN MODE: 실제 생성하지 않습니다
2026-01-18 13:17:19 - INFO - ✓ Database connected
2026-01-18 13:17:19 - INFO - Found 375 project(s) to migrate

============================================================
Project: Clinical Trial A (ID: 1)
============================================================
2026-01-18 13:17:19 - INFO - Found 15 studies with patient_id
2026-01-18 13:17:19 - INFO -   [DRY-RUN] Would create Subject: P12345 (Patient: P12345)
2026-01-18 13:17:19 - INFO -   [DRY-RUN] Would create Subject: P12346 (Patient: P12346)
2026-01-18 13:17:19 - INFO -   ✓ Reuse Subject: P12347 (Patient: P12347)
...

Summary:
  - Created: 12
  - Reused: 3
  - Total: 15

============================================================
✓ Migration completed successfully
============================================================
```

#### 실제 실행 모드
```
$ python scripts/migrate_subjects.py --project-id 1

2026-01-18 13:20:00 - INFO - Database: localhost:5456/pacs_extension
2026-01-18 13:20:00 - INFO - ✓ Database connected
2026-01-18 13:20:00 - INFO - Found 1 project(s) to migrate

============================================================
Project: Clinical Trial A (ID: 1)
============================================================
2026-01-18 13:20:01 - INFO - Found 15 studies with patient_id
2026-01-18 13:20:01 - INFO -   ✓ Created Subject: P12345 (ID: 101, Patient: P12345)
2026-01-18 13:20:01 - INFO -   ✓ Created Subject: P12346 (ID: 102, Patient: P12346)
2026-01-18 13:20:01 - INFO -   ✓ Reuse Subject: P12347 (Patient: P12347)
...

Summary:
  - Created: 12
  - Reused: 3
  - Total: 15

============================================================
✓ Migration completed successfully
============================================================
```

### 주의사항

⚠️ **백업 필수**: 마이그레이션 전에 반드시 데이터베이스 백업을 수행하세요.

```bash
# PostgreSQL 백업
pg_dump -U postgres pacs_extension > backup_$(date +%Y%m%d_%H%M%S).sql
```

⚠️ **Dry-run 먼저 실행**: 실제 마이그레이션 전에 `--dry-run`으로 결과를 확인하세요.

⚠️ **동시성**: 마이그레이션 중에는 다른 사용자가 Subject를 생성하지 않도록 주의하세요.

### 트러블슈팅

#### 1. `psycopg2` 설치 오류

```bash
# macOS
brew install postgresql
pip install psycopg2-binary

# Ubuntu/Debian
sudo apt-get install libpq-dev
pip install psycopg2-binary
```

#### 2. 데이터베이스 연결 실패

- DB URL이 올바른지 확인
- PostgreSQL이 실행 중인지 확인
- 방화벽/포트 설정 확인

#### 3. Subject Code 중복 오류

- 스크립트가 자동으로 suffix를 추가하므로 일반적으로 발생하지 않음
- 100번 이상 중복 시 순차 번호로 전환
- 1000번 이상 중복 시 에러 발생 (수동 확인 필요)

### 롤백

마이그레이션 후 문제가 발생한 경우:

```sql
-- 특정 프로젝트의 자동 생성된 Subject 삭제
DELETE FROM project_subject
WHERE project_id = 1
  AND created_at > '2026-01-18 10:00:00';  -- 마이그레이션 시작 시간

-- 또는 백업에서 복원
psql -U postgres pacs_extension < backup_20260118_100000.sql
```

---

## 기타 스크립트

추가 스크립트는 향후 이 디렉토리에 추가될 예정입니다.

