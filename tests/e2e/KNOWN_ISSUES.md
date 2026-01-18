# Known Issues in E2E Tests

## TimePoint Gateway Tests (test_05_subject_timepoint.py)

### Issue 1: assign_studies() Missing Required Fields

**Test**: `test_02_gateway_with_timepoint`, `test_04_gateway_timepoint_visit_no`

**Status**: ❌ BLOCKED - Backend Bug

**Error**:
```
Database error: error returned from database: there is no unique or exclusion constraint matching the ON CONFLICT specification
```

**Root Cause**:
The `assign_studies()` method in `timepoint_study_repository_impl.rs` is missing required fields in the INSERT statement.

**Current Code** (lines 150-154):
```rust
sqlx::query(
    "INSERT INTO subject_timepoint_study_map (timepoint_id, study_id, assigned_by)
     VALUES ($1, $2, $3)
     ON CONFLICT (study_id) DO UPDATE SET timepoint_id = $1, assigned_by = $3, assigned_at = NOW()"
)
```

**Issues**:
1. Missing `subject_id` and `project_id` fields (required by table schema)
2. `ON CONFLICT (study_id)` is invalid - table has `UNIQUE (subject_id, study_id)` constraint, not `UNIQUE (study_id)`

**Required Fix**:
```rust
// First, get subject_id and project_id from timepoint
let (subject_id, project_id) = sqlx::query_as::<_, (i32, i32)>(
    "SELECT subject_id, project_id FROM subject_timepoint WHERE id = $1"
)
.bind(timepoint_id)
.fetch_one(&self.pool)
.await?;

// Then insert with all required fields
sqlx::query(
    "INSERT INTO subject_timepoint_study_map (project_id, subject_id, timepoint_id, study_id, assigned_by)
     VALUES ($1, $2, $3, $4, $5)
     ON CONFLICT (subject_id, study_id) DO UPDATE 
     SET timepoint_id = $3, assigned_by = $5, assigned_at = NOW()"
)
.bind(project_id)
.bind(subject_id)
.bind(timepoint_id)
.bind(study_id)
.bind(user_id)
.execute(&self.pool)
.await?;
```

**File to Fix**: `pacs-server/src/infrastructure/repositories/timepoint_study_repository_impl.rs`

---

### Issue 2: include_timepoint Not Adding timepoint Field

**Test**: `test_03_gateway_timepoint_null_for_unassigned`

**Status**: ❌ BLOCKED - Backend Bug

**Expected Behavior**:
When `include_timepoint=true` is specified, the gateway API should always include a `timepoint` field in the `_ext` object, even when it's `null` for unassigned studies.

**Current Behavior**:
The `timepoint` field is only added when a study is assigned to a timepoint. For unassigned studies, the field is completely missing from `_ext`.

**Current Response** (unassigned study):
```json
{
  "_ext": {
    "project": {...},
    "report_status": "unread",
    "subject": {...}
    // timepoint field is missing!
  }
}
```

**Expected Response** (unassigned study):
```json
{
  "_ext": {
    "project": {...},
    "report_status": "unread",
    "subject": {...},
    "timepoint": null  // Should be present with null value
  }
}
```

**Root Cause**:
In `dicom_gateway_controller.rs` (lines 1061-1067), the code only adds the `timepoint` field when it exists in the cache:

```rust
// timepoint 추가 (캐시에서 가져오기)
if !timepoint_cache.is_empty() {
    if let Some(timepoint) = timepoint_cache.get(&study_uid) {
        ext.insert("timepoint".to_string(), serde_json::json!(timepoint));
    } else {
        ext.insert("timepoint".to_string(), serde_json::Value::Null);
    }
}
```

**Issue**: The outer `if !timepoint_cache.is_empty()` prevents adding `timepoint: null` when the cache is empty.

**Required Fix**:
```rust
// timepoint 추가 (include_timepoint=true일 때 항상 추가)
if query.include_timepoint.unwrap_or(false) {
    if let Some(timepoint) = timepoint_cache.get(&study_uid) {
        ext.insert("timepoint".to_string(), serde_json::json!(timepoint));
    } else {
        ext.insert("timepoint".to_string(), serde_json::Value::Null);
    }
}
```

**File to Fix**: `pacs-server/src/presentation/controllers/dicom_gateway_controller.rs`

---

## Summary

- **Total Tests**: 4
- **Passing**: 1 ✅
- **Skipped (Backend Bugs)**: 3 ⏭️

**Action Required**: Fix backend issues before these tests can pass.

