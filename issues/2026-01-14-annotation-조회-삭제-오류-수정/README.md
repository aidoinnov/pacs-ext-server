# Annotation 조회/삭제 오류 수정

## 📋 이슈 개요

**날짜**: 2026-01-14  
**작업자**: aido  
**우선순위**: 🔴 High (기능 동작 불가)

### 문제 상황
Annotation Snapshot API 구현 후, 기존 Annotation 조회 및 삭제 기능이 동작하지 않는 문제 발생

### 증상
- ❌ Annotation 조회 실패
- ❌ Annotation 삭제 실패
- ❌ 모든 Annotation 관련 API 오류

---

## 🔍 원인 분석

### 1. DB 스키마 변경
**Migration 036, 037**에서 `annotation_annotation` 테이블에 새로운 컬럼 추가:
```sql
-- 036_add_snapshot_image_to_annotations.sql
ALTER TABLE annotation_annotation
ADD COLUMN snapshot_image_key VARCHAR(512),
ADD COLUMN snapshot_status VARCHAR(50),
ADD COLUMN snapshot_uploaded_at TIMESTAMP;

-- 037_fix_snapshot_uploaded_at_type.sql
ALTER TABLE annotation_annotation
ALTER COLUMN snapshot_uploaded_at TYPE TIMESTAMP WITH TIME ZONE;
```

### 2. Entity 업데이트
`Annotation` Entity에 새 필드 추가:
```rust
pub struct Annotation {
    // 기존 필드들...
    
    // 새로 추가된 필드
    pub snapshot_image_key: Option<String>,
    pub snapshot_status: Option<SnapshotUploadStatus>,
    pub snapshot_uploaded_at: Option<DateTime<Utc>>,
}
```

### 3. 문제의 핵심
**Repository의 SELECT 쿼리가 업데이트되지 않음!**

#### 기존 쿼리 (오류 발생)
```rust
sqlx::query_as::<_, Annotation>(
    "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid,
            tool_name, tool_version, data, is_shared, created_at, updated_at,
            viewer_software, description, measurement_values, label, version
     FROM annotation_annotation
     WHERE id = $1"
)
```

**문제점**:
- Entity는 `snapshot_image_key`, `snapshot_status`, `snapshot_uploaded_at` 필드를 가지고 있음
- SELECT 쿼리는 이 필드들을 조회하지 않음
- sqlx의 `query_as!` 매크로가 필드 불일치로 인해 실패

---

## ✅ 해결 방법

### 모든 SELECT 쿼리에 snapshot 필드 추가

#### 수정 후 쿼리
```rust
sqlx::query_as::<_, Annotation>(
    "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid,
            tool_name, tool_version, data, is_shared,
            snapshot_image_key, snapshot_status, snapshot_uploaded_at,  // ✅ 추가
            created_at, updated_at, version,
            viewer_software, description, measurement_values, label
     FROM annotation_annotation
     WHERE id = $1"
)
```

---

## 🔧 수정 파일

### `pacs-server/src/infrastructure/repositories/annotation_repository_impl.rs`

#### 수정된 메서드 (총 15개)

1. ✅ `find_by_id()` - ID로 조회
2. ✅ `find_by_project_id()` - 프로젝트 ID로 조회
3. ✅ `find_by_user_id()` - 사용자 ID로 조회
4. ✅ `find_by_study_uid()` - Study UID로 조회
5. ✅ `find_by_series_uid()` - Series UID로 조회
6. ✅ `find_by_instance_uid()` - Instance UID로 조회
7. ✅ `find_by_project_and_study()` - 프로젝트 + Study로 조회
8. ✅ `find_by_project_and_series()` - 프로젝트 + Series로 조회
9. ✅ `find_by_project_and_instance()` - 프로젝트 + Instance로 조회
10. ✅ `create()` - 생성 (RETURNING 절)
11. ✅ `update()` - 업데이트 (기존 데이터 조회 + RETURNING 절)
12. ✅ `update_with_version_check()` - 버전 체크 업데이트 (기존 데이터 조회 + RETURNING 절)
13. ✅ `update_snapshot_info()` - Snapshot 정보 업데이트 (RETURNING 절)
14. ✅ `delete()` - 삭제 (기존 데이터 조회)
15. ✅ `find_by_project_and_series_paginated()` - 페이지네이션 조회

---

## 📊 수정 전후 비교

### Before (오류 발생)
```rust
// ❌ snapshot 필드 누락
"SELECT id, project_id, user_id, study_uid, series_uid, instance_uid,
        tool_name, tool_version, data, is_shared, created_at, updated_at,
        viewer_software, description, measurement_values, label, version
 FROM annotation_annotation
 WHERE id = $1"
```

**결과**: 
- sqlx 매핑 실패
- `MissingColumn` 에러
- API 500 Internal Server Error

### After (정상 동작)
```rust
// ✅ snapshot 필드 포함
"SELECT id, project_id, user_id, study_uid, series_uid, instance_uid,
        tool_name, tool_version, data, is_shared,
        snapshot_image_key, snapshot_status, snapshot_uploaded_at,
        created_at, updated_at, version,
        viewer_software, description, measurement_values, label
 FROM annotation_annotation
 WHERE id = $1"
```

**결과**:
- ✅ 정상 매핑
- ✅ API 200 OK
- ✅ 모든 기능 정상 동작

---

## 🎯 핵심 교훈

### 1. Entity 변경 시 체크리스트
- [ ] Entity 구조체 수정
- [ ] DB Migration 작성
- [ ] **Repository의 모든 SELECT 쿼리 업데이트** ⭐ 중요!
- [ ] RETURNING 절이 있는 INSERT/UPDATE 쿼리 업데이트
- [ ] 테스트 실행

### 2. sqlx의 컴파일 타임 체크
- `query_as!` 매크로는 컴파일 타임에 DB 스키마를 체크
- 필드 불일치 시 컴파일 에러 발생
- 하지만 `query_as::<_, T>` 사용 시 런타임 에러 발생 가능

### 3. 예방 방법
```rust
// ✅ 권장: 컴파일 타임 체크
let annotation = sqlx::query_as!(
    Annotation,
    "SELECT * FROM annotation_annotation WHERE id = $1",
    id
)
.fetch_optional(&self.pool)
.await?;

// ⚠️ 주의: 런타임 체크
let annotation = sqlx::query_as::<_, Annotation>(
    "SELECT * FROM annotation_annotation WHERE id = $1"
)
.bind(id)
.fetch_optional(&self.pool)
.await?;
```

---

## ✅ 검증

### 테스트 시나리오
1. ✅ Annotation 생성
2. ✅ Annotation 조회 (ID, Project, User, Study, Series, Instance)
3. ✅ Annotation 업데이트
4. ✅ Annotation 삭제
5. ✅ Annotation History 생성

### 테스트 결과
```bash
# 모든 E2E 테스트 통과
✅ test_annotation_level_filtering_refactored.py
✅ test_annotation_version_conflict_refactored.py
✅ test_annotation_head_request_refactored.py
✅ test_annotation_snapshot_e2e_refactored.py
✅ test_annotation_permission_filtering_refactored.py
✅ test_annotation_permission_management_refactored.py
```

---

## 📝 관련 이슈

### 선행 작업
- [Annotation Snapshot API 구현](../../docs/api/annotation-snapshot-api/README.md)
- [Migration 036](../../pacs-server/migrations/036_add_snapshot_image_to_annotations.sql)
- [Migration 037](../../pacs-server/migrations/037_fix_snapshot_uploaded_at_type.sql)

### 영향받은 기능
- Annotation CRUD 전체
- Annotation History
- Annotation Permissions
- Annotation List/Summary

---

## 🚀 배포 체크리스트

- [x] 코드 수정 완료
- [x] 로컬 테스트 통과
- [x] E2E 테스트 통과
- [ ] 스테이징 배포
- [ ] 프로덕션 배포

---

## 📖 참고 자료

- [sqlx Documentation](https://docs.rs/sqlx/)
- [Annotation Entity](../../pacs-server/src/domain/entities/annotation.rs)
- [Annotation Repository](../../pacs-server/src/infrastructure/repositories/annotation_repository_impl.rs)

