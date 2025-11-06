# RBAC 계층적 권한 상속 (Hierarchical Permission Inheritance)

## 📋 개요

PACS Extension Server의 RBAC 시스템은 **계층적 권한 상속(Hierarchical Inheritance)** 방식을 사용합니다.

상위 레벨(Study)에 대한 접근 권한이 있으면, 하위 레벨(Series, Instance)에 대한 접근도 자동으로 허용됩니다.

---

## 🏗️ DICOM 데이터 계층 구조

```
Study (연구)
  └── Series (시리즈)
        └── Instance (인스턴스/이미지)
```

---

## ✅ 권한 상속 규칙

### 1️⃣ Study 레벨 권한
**`project_data` 테이블에 `study_id`만 있는 경우:**

```sql
-- 예시 데이터
project_id: 2
resource_level: 'STUDY'
study_id: 242
series_id: NULL
instance_id: NULL
```

**효과:**
- ✅ 해당 Study의 **모든 Series** 접근 가능
- ✅ 해당 Study의 **모든 Instance** 접근 가능

**사용 사례:**
- 프로젝트에 전체 Study를 할당할 때
- 연구 전체에 대한 접근 권한 부여

---

### 2️⃣ Series 레벨 권한
**`project_data` 테이블에 `study_id` + `series_id`가 있는 경우:**

```sql
-- 예시 데이터
project_id: 2
resource_level: 'SERIES'
study_id: 242
series_id: 216
instance_id: NULL
```

**효과:**
- ✅ 해당 Series의 **모든 Instance** 접근 가능
- ❌ 같은 Study의 **다른 Series**는 접근 불가 (별도 권한 필요)

**사용 사례:**
- Study 중 특정 Series만 선택적으로 할당
- CT 시리즈만 허용하고 MRI 시리즈는 제외

---

### 3️⃣ Instance 레벨 권한
**`project_data` 테이블에 `study_id` + `series_id` + `instance_id`가 있는 경우:**

```sql
-- 예시 데이터
project_id: 2
resource_level: 'INSTANCE'
study_id: 242
series_id: 216
instance_id: 45
```

**효과:**
- ✅ 해당 **특정 Instance만** 접근 가능
- ❌ 같은 Series의 **다른 Instance**는 접근 불가 (별도 권한 필요)

**사용 사례:**
- 특정 이미지만 선택적으로 할당
- 샘플 이미지 제공

---

## 🔄 권한 상속 체인

### Instance 접근 시:
```
1. Instance 레벨 명시적 거부 체크 → 거부되면 차단
2. Instance 레벨 명시적 승인 체크 → 승인되면 허용
3. ⭐ 상위 Series 권한 상속 체크
   - Series가 거부되면 Instance도 거부
   - Series가 허용되면 Instance도 허용
4. 상위를 찾을 수 없으면 거부
```

### Series 접근 시:
```
1. Series 레벨 명시적 거부 체크 → 거부되면 차단
2. Series 레벨 명시적 승인 체크 → 승인되면 허용
3. ⭐ 상위 Study 권한 상속 체크
   - Study가 거부되면 Series도 거부
   - Study가 허용되면 Series도 허용
4. 상위를 찾을 수 없으면 거부
```

### Study 접근 시:
```
1. 프로젝트 멤버십 체크 → 멤버가 아니면 거부
2. Study 레벨 명시적 거부 체크 → 거부되면 차단
3. Study 레벨 명시적 승인 체크 → 승인되면 허용
4. 기관 기반 접근 체크 (같은 기관 또는 기관 간 허용)
5. 룰 기반 조건 평가 (DICOM 태그 매칭)
6. 기본값: 프로젝트 멤버면 허용
```

---

## 💡 실제 예시

### 예시 1: Study 레벨 할당
```sql
-- project_data 테이블
INSERT INTO project_data (project_id, resource_level, study_id, series_id, instance_id)
VALUES (2, 'STUDY', 242, NULL, NULL);
```

**결과:**
- Study UID `1.2.410.200022.500.200612201921171.113378644` 접근 ✅
- 해당 Study의 모든 Series 접근 ✅
- 해당 Study의 모든 Instance 접근 ✅

---

### 예시 2: Series 레벨 할당
```sql
-- project_data 테이블
INSERT INTO project_data (project_id, resource_level, study_id, series_id, instance_id)
VALUES (2, 'SERIES', 242, 216, NULL);
```

**결과:**
- Study UID `1.2.410.200022.500.200612201921171.113378644` 접근 ✅ (Study는 project_data에 있으므로)
- Series UID `1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771` 접근 ✅
- 해당 Series의 모든 Instance (10개) 접근 ✅
- 같은 Study의 다른 Series 접근 ❌

---

### 예시 3: Instance 레벨 할당
```sql
-- project_data 테이블
INSERT INTO project_data (project_id, resource_level, study_id, series_id, instance_id)
VALUES (2, 'INSTANCE', 242, 216, 45);
```

**결과:**
- Study UID `1.2.410.200022.500.200612201921171.113378644` 접근 ✅
- Series UID `1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771` 접근 ✅
- Instance UID `1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771.1` 접근 ✅
- 같은 Series의 다른 Instance 접근 ❌

---

## 🔧 구현 위치

### 파일: `pacs-server/src/infrastructure/services/dicom_rbac_evaluator_impl.rs`

#### Instance 권한 상속 (lines 642-663)
```rust
// 3) instance가 포함된 series에 대한 권한 상속
let parent_series_id: Option<i32> =
    sqlx::query_scalar("SELECT series_id FROM project_data_instance WHERE id = $1")
        .bind(instance_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

if let Some(series_id) = parent_series_id {
    let series_result = self
        .evaluate_series_access(user_id, project_id, series_id)
        .await;
    // Series가 거부되면 Instance도 거부
    if !series_result.allowed {
        return series_result;
    }
    // Series가 허용되면 Instance도 허용 (상속)
    return RbacEvaluationResult {
        allowed: true,
        reason: Some("inherited_from_series".to_string()),
    };
}
```

#### Series 권한 상속 (lines 485-506)
```rust
// 3) series가 포함된 study에 대한 권한 상속
let parent_study_id: Option<i32> =
    sqlx::query_scalar("SELECT study_id FROM project_data_series WHERE id = $1")
        .bind(series_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

if let Some(study_id) = parent_study_id {
    let study_result = self
        .evaluate_study_access(user_id, project_id, study_id)
        .await;
    // Study가 거부되면 Series도 거부
    if !study_result.allowed {
        return study_result;
    }
    // Study가 허용되면 Series도 허용 (상속)
    return RbacEvaluationResult {
        allowed: true,
        reason: Some("inherited_from_study".to_string()),
    };
}
```

---

## 🎯 권한 상속 이유 (Reason) 코드

RBAC 평가 결과에 포함되는 `reason` 필드:

### 상속 관련:
- `inherited_from_study`: Study 레벨 권한에서 상속됨
- `inherited_from_series`: Series 레벨 권한에서 상속됨

### 명시적 권한:
- `explicit_study_approved`: Study 레벨 명시적 승인
- `explicit_series_approved`: Series 레벨 명시적 승인
- `explicit_instance_approved`: Instance 레벨 명시적 승인
- `explicit_study_denied`: Study 레벨 명시적 거부
- `explicit_series_denied`: Series 레벨 명시적 거부
- `explicit_instance_denied`: Instance 레벨 명시적 거부

### 기타:
- `project_member_default_access`: 프로젝트 멤버 기본 접근
- `same_institution`: 같은 기관
- `institution_cross_access`: 기관 간 접근 허용
- `rule_approved`: 룰 기반 조건 승인
- `rule_denied`: 룰 기반 조건 거부

---

## ⚠️ 주의사항

### 1. 명시적 거부가 최우선
상위 레벨에서 허용되어도, 하위 레벨에서 **명시적으로 거부**되면 접근이 차단됩니다.

```sql
-- Study 레벨 허용
INSERT INTO project_data (project_id, resource_level, study_id)
VALUES (2, 'STUDY', 242);

-- 특정 Instance 명시적 거부
INSERT INTO project_data_access (user_id, project_id, resource_level, instance_id, status)
VALUES (1, 2, 'INSTANCE', 45, 'DENIED');
```

**결과:**
- Study의 대부분 Instance 접근 ✅
- Instance ID 45는 접근 ❌ (명시적 거부)

### 2. 프로젝트 멤버십 필수
모든 권한 체크 전에 **프로젝트 멤버십**이 확인됩니다.

```sql
-- security_user_project 테이블에 레코드 필요
SELECT EXISTS(
    SELECT 1 FROM security_user_project 
    WHERE user_id = $1 AND project_id = $2
)
```

### 3. 데이터 동기화 필요
QIDO-RS로 조회한 Instance가 RBAC 평가를 통과하려면:
- `project_data_study` 테이블에 Study 존재
- `project_data_series` 테이블에 Series 존재
- `project_data_instance` 테이블에 Instance 존재
- `project_data` 테이블에 프로젝트 할당 레코드 존재

**동기화 방법:**
```bash
curl -X POST "http://localhost:8080/api/sync/run"
```

---

## 📊 데이터베이스 스키마

### project_data 테이블
```sql
CREATE TABLE project_data (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id),
    resource_level resource_level_enum NOT NULL DEFAULT 'STUDY',
    study_id INTEGER REFERENCES project_data_study(id),
    series_id INTEGER REFERENCES project_data_series(id),
    instance_id INTEGER REFERENCES project_data_instance(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 제약 조건
    CONSTRAINT chk_project_data_study_required CHECK (study_id IS NOT NULL),
    CONSTRAINT chk_project_data_study_level CHECK (
        (resource_level = 'STUDY' AND series_id IS NULL AND instance_id IS NULL) OR
        (resource_level = 'SERIES' AND series_id IS NOT NULL AND instance_id IS NULL) OR
        (resource_level = 'INSTANCE' AND series_id IS NOT NULL AND instance_id IS NOT NULL)
    )
);
```

### resource_level_enum
```sql
CREATE TYPE resource_level_enum AS ENUM ('STUDY', 'SERIES', 'INSTANCE');
```

---

## 🧪 테스트 방법

### 1. Study 레벨 권한 테스트
```bash
# Study 할당
psql -c "INSERT INTO project_data (project_id, resource_level, study_id) 
         VALUES (2, 'STUDY', 242);"

# Study 조회 (성공)
curl "http://localhost:8080/api/dicom/studies?project_id=2" \
  -H "Authorization: Bearer $TOKEN"

# Series 조회 (성공 - 상속)
curl "http://localhost:8080/api/dicom/studies/{studyUID}/series?project_id=2" \
  -H "Authorization: Bearer $TOKEN"

# Instance 조회 (성공 - 상속)
curl "http://localhost:8080/api/dicom/studies/{studyUID}/series/{seriesUID}/instances?project_id=2" \
  -H "Authorization: Bearer $TOKEN"
```

### 2. Series 레벨 권한 테스트
```bash
# Series 할당
psql -c "INSERT INTO project_data (project_id, resource_level, study_id, series_id) 
         VALUES (2, 'SERIES', 242, 216);"

# Series 조회 (성공)
curl "http://localhost:8080/api/dicom/studies/{studyUID}/series?project_id=2" \
  -H "Authorization: Bearer $TOKEN"

# Instance 조회 (성공 - 상속)
curl "http://localhost:8080/api/dicom/studies/{studyUID}/series/{seriesUID}/instances?project_id=2" \
  -H "Authorization: Bearer $TOKEN"
```

### 3. Instance 레벨 권한 테스트
```bash
# Instance 할당
psql -c "INSERT INTO project_data (project_id, resource_level, study_id, series_id, instance_id) 
         VALUES (2, 'INSTANCE', 242, 216, 45);"

# Instance 조회 (성공 - 해당 Instance만)
curl "http://localhost:8080/api/dicom/studies/{studyUID}/series/{seriesUID}/instances?project_id=2" \
  -H "Authorization: Bearer $TOKEN"
# 결과: 1개의 Instance만 반환 (Instance ID 45)
```

---

## 📚 관련 문서

- [QIDO-RS + RBAC API 문서](../api/dicom-qido-rbac-api.md)
- [Access Matrix API 문서](../api/data-access-matrix-api.md)
- [Project Data API 문서](../api/project-data-api.md)

---

**작성일**: 2025-11-06  
**버전**: 1.0  
**작성자**: PACS Extension Server Team

