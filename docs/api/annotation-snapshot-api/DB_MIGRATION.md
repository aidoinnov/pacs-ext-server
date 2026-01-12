# 데이터베이스 마이그레이션 가이드

> **작성일**: 2026-01-11
> **최종 업데이트**: 2026-01-12
> **상태**: ✅ 완료

---

## 📋 개요

어노테이션 스냅샷 이미지 저장을 위한 데이터베이스 스키마 변경 가이드입니다.

---

## 🗄️ 마이그레이션 파일

### 1. 036_add_snapshot_image_to_annotations.sql

**파일 위치**: `pacs-server/migrations/036_add_snapshot_image_to_annotations.sql`

**설명**: 스냅샷 이미지 관련 컬럼 및 ENUM 타입 추가

**변경 사항**:
- ENUM 타입 `snapshot_upload_status` 생성
- 컬럼 3개 추가: `snapshot_image_key`, `snapshot_status`, `snapshot_uploaded_at`
- 인덱스 2개 추가: `idx_annotation_snapshot_image_key`, `idx_annotation_snapshot_status`

```sql
-- Migration: Add snapshot image support to annotations
-- Created: 2026-01-11
-- Description: S3에 저장된 어노테이션 스냅샷 이미지 경로 및 상태를 저장하기 위한 컬럼 추가

-- Step 1: 스냅샷 상태 ENUM 타입 생성 (이미 존재하면 무시)
DO $$ BEGIN
    CREATE TYPE snapshot_upload_status AS ENUM (
        'pending',      -- URL 생성됨, 업로드 대기 중
        'uploading',    -- 업로드 진행 중
        'completed',    -- 업로드 완료
        'failed'        -- 업로드 실패
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Step 2: 스냅샷 관련 컬럼 추가
ALTER TABLE annotation_annotation
ADD COLUMN IF NOT EXISTS snapshot_image_key VARCHAR(512) NULL,
ADD COLUMN IF NOT EXISTS snapshot_status snapshot_upload_status NULL DEFAULT NULL,
ADD COLUMN IF NOT EXISTS snapshot_uploaded_at TIMESTAMPTZ NULL;

-- Step 3: 컬럼 주석 추가
COMMENT ON COLUMN annotation_annotation.snapshot_image_key IS 'S3에 저장된 스냅샷 이미지의 object key';
COMMENT ON COLUMN annotation_annotation.snapshot_status IS '스냅샷 업로드 상태 (pending/uploading/completed/failed)';
COMMENT ON COLUMN annotation_annotation.snapshot_uploaded_at IS '스냅샷 업로드 완료 시간';

-- Step 4: 인덱스 추가 (이미지가 있는 어노테이션 조회 최적화)
CREATE INDEX IF NOT EXISTS idx_annotation_snapshot_image_key
ON annotation_annotation(snapshot_image_key)
WHERE snapshot_image_key IS NOT NULL;

-- Step 5: 인덱스 추가 (업로드 상태별 조회 최적화)
CREATE INDEX IF NOT EXISTS idx_annotation_snapshot_status
ON annotation_annotation(snapshot_status)
WHERE snapshot_status IS NOT NULL;
```

### 2. 037_fix_snapshot_uploaded_at_type.sql

**파일 위치**: `pacs-server/migrations/037_fix_snapshot_uploaded_at_type.sql`

**설명**: `snapshot_uploaded_at` 컬럼 타입을 TIMESTAMPTZ로 변경 (다른 날짜 컬럼과 일관성 유지)

**변경 사항**:
- `snapshot_uploaded_at` 타입 변경: TIMESTAMP → TIMESTAMPTZ
- 컬럼 주석 업데이트

```sql
-- Migration: Fix snapshot_uploaded_at column type
-- Created: 2026-01-12
-- Description: TIMESTAMP를 TIMESTAMPTZ로 변경하여 다른 날짜 컬럼과 일관성 유지

-- Step 1: 기존 컬럼 타입 변경
ALTER TABLE annotation_annotation
ALTER COLUMN snapshot_uploaded_at TYPE TIMESTAMPTZ;

-- Step 2: 컬럼 주석 업데이트
COMMENT ON COLUMN annotation_annotation.snapshot_uploaded_at IS '스냅샷 업로드 완료 시간 (UTC)';
```

**변경 이유**:
- 기존 `created_at`, `updated_at` 컬럼은 모두 `TIMESTAMPTZ` 사용
- 타임존 정보 포함으로 글로벌 서비스 대응
- 데이터 일관성 유지

---

## 📊 스키마 변경 요약

### 추가된 컬럼

| 컬럼명 | 타입 | NULL | 기본값 | 설명 |
|--------|------|------|--------|------|
| `snapshot_image_key` | VARCHAR(512) | YES | NULL | S3 object key |
| `snapshot_status` | snapshot_upload_status | YES | NULL | 업로드 상태 |
| `snapshot_uploaded_at` | TIMESTAMPTZ | YES | NULL | 업로드 완료 시간 (UTC) |

### 추가된 ENUM 타입

**snapshot_upload_status**:
- `pending` - URL 생성됨, 업로드 대기 중
- `uploading` - 업로드 진행 중
- `completed` - 업로드 완료
- `failed` - 업로드 실패

### 추가된 인덱스

| 인덱스명 | 컬럼 | 조건 | 목적 |
|----------|------|------|------|
| `idx_annotation_snapshot_image_key` | snapshot_image_key | WHERE snapshot_image_key IS NOT NULL | 이미지가 있는 어노테이션 조회 최적화 |
| `idx_annotation_snapshot_status` | snapshot_status | WHERE snapshot_status IS NOT NULL | 업로드 상태별 조회 최적화 |

---

## 🚀 마이그레이션 실행

### 방법 1: 직접 실행

```bash
cd pacs-server

# 036 마이그레이션 실행
psql -h localhost -p 5456 -U pacs_extension_admin -d pacs_extension \
  -f migrations/036_add_snapshot_image_to_annotations.sql

# 037 마이그레이션 실행
psql -h localhost -p 5456 -U pacs_extension_admin -d pacs_extension \
  -f migrations/037_fix_snapshot_uploaded_at_type.sql
```

### 방법 2: 스크립트 실행

```bash
cd pacs-server

# 036 마이그레이션 실행
bash scripts/migration/036_migration.sh
```

**036_migration.sh 내용**:
```bash
#!/bin/bash
psql -h localhost -p 5456 -U pacs_extension_admin -d pacs_extension \
  -f migrations/036_add_snapshot_image_to_annotations.sql
```

---

## ✅ 검증

### 방법 1: 검증 스크립트 실행

```bash
cd pacs-server
bash scripts/migration/036_validation.sh
```

**실행 결과 예시**:
```
======================================
 Snapshot Schema Validation Started
======================================
🔐 PostgreSQL password for user 'pacs_extension_admin': ****

[1] Column validation...
✅ PASS - column 'snapshot_image_key' exists
✅ PASS - column 'snapshot_status' exists
✅ PASS - column 'snapshot_uploaded_at' exists

[2] ENUM snapshot_upload_status validation...
✅ PASS - enum 'pending' exists
✅ PASS - enum 'uploading' exists
✅ PASS - enum 'completed' exists
✅ PASS - enum 'failed' exists

[3] Snapshot index validation...
✅ PASS - snapshot related index exists

======================================
 Validation Summary
======================================
✅ PASS: 8
❌ FAIL: 0

🎉 ALL CHECKS PASSED
```

### 방법 2: 수동 검증

```bash
# 컬럼 확인
psql -h localhost -p 5456 -U pacs_extension_admin -d pacs_extension -c "
  SELECT column_name, data_type, is_nullable
  FROM information_schema.columns
  WHERE table_name = 'annotation_annotation'
    AND column_name LIKE 'snapshot%';
"

# ENUM 타입 확인
psql -h localhost -p 5456 -U pacs_extension_admin -d pacs_extension -c "
  SELECT enumlabel
  FROM pg_enum
  WHERE enumtypid = 'snapshot_upload_status'::regtype
  ORDER BY enumsortorder;
"

# 인덱스 확인
psql -h localhost -p 5456 -U pacs_extension_admin -d pacs_extension -c "
  SELECT indexname, indexdef
  FROM pg_indexes
  WHERE tablename = 'annotation_annotation'
    AND indexname LIKE '%snapshot%';
"
```

---

## 🔄 롤백 (필요시)

### 037 롤백

```sql
-- snapshot_uploaded_at 타입을 TIMESTAMP로 되돌림
ALTER TABLE annotation_annotation
ALTER COLUMN snapshot_uploaded_at TYPE TIMESTAMP;
```

### 036 롤백

```sql
-- 인덱스 삭제
DROP INDEX IF EXISTS idx_annotation_snapshot_status;
DROP INDEX IF EXISTS idx_annotation_snapshot_image_key;

-- 컬럼 삭제
ALTER TABLE annotation_annotation
DROP COLUMN IF EXISTS snapshot_uploaded_at,
DROP COLUMN IF EXISTS snapshot_status,
DROP COLUMN IF EXISTS snapshot_image_key;

-- ENUM 타입 삭제 (주의: 다른 테이블에서 사용 중이면 실패)
DROP TYPE IF EXISTS snapshot_upload_status;
```

---

## 📝 마이그레이션 히스토리

| 번호 | 파일명 | 작성일 | 설명 | 상태 |
|------|--------|--------|------|------|
| 036 | 036_add_snapshot_image_to_annotations.sql | 2026-01-11 | 스냅샷 컬럼 및 ENUM 추가 | ✅ 완료 |
| 037 | 037_fix_snapshot_uploaded_at_type.sql | 2026-01-12 | TIMESTAMPTZ 타입 변경 | ✅ 완료 |

---

## 🔗 관련 문서

- [README.md](./README.md) - 프로젝트 개요
- [WORKLOG.md](./WORKLOG.md) - 구현 작업 로그
- [API_SPEC.md](./API_SPEC.md) - API 명세서
- [ARCHITECTURE.md](./ARCHITECTURE.md) - 아키텍처 설계

---

**최종 업데이트**: 2026-01-12
**작성자**: AI Assistant
