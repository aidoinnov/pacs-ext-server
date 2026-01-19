# RECIST Lesion 간소화 작업 (방안 2: 하이브리드)

## 📋 핵심 아이디어

**사용자는 간단하게, 서버는 자동으로 추적**

- 사용자: Annotation에 `lesion_type` + `lesion_number`만 입력
- 서버: 자동으로 `recist_lesion` 테이블에 추적 (분석용)

---

## 📋 변경 사항 요약

### 1. Annotation에 lesion 정보 추가 (사용자 입력)
- ✅ `annotation_annotation.lesion_type` 추가
- ✅ `annotation_annotation.lesion_number` 추가

### 2. Lesion 테이블 간소화 (서버 자동 관리)
- ❌ `recist_lesion.organ_site` 삭제
- ❌ `recist_lesion.baseline_timepoint_id` 삭제
- ❌ `recist_lesion.project_id` 삭제
- ❌ `recist_lesion_annotation_map` 테이블 전체 삭제
- ✅ `recist_lesion` 테이블 유지 (Subject별 Lesion 추적용)

### 3. ENUM 타입 확장
- ✅ `TARGET_NEW`, `NON_TARGET_NEW` 추가

---

## 🗄️ 최종 DB 구조

### annotation_annotation (사용자가 직접 입력)
```sql
ALTER TABLE annotation_annotation
ADD COLUMN lesion_type VARCHAR(20),     -- TARGET, NON_TARGET, TARGET_NEW, NON_TARGET_NEW
ADD COLUMN lesion_number INTEGER;       -- 1, 2, 3, 4, 5

CREATE INDEX idx_annotation_lesion_type ON annotation_annotation(lesion_type) WHERE lesion_type IS NOT NULL;
```

### recist_lesion (서버가 자동 관리 - 분석/추적용)
```sql
CREATE TABLE recist_lesion (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    subject_id INTEGER NOT NULL REFERENCES project_subject(id) ON DELETE CASCADE,
    lesion_type VARCHAR(20) NOT NULL,    -- TARGET, NON_TARGET, TARGET_NEW, NON_TARGET_NEW
    lesion_number INTEGER NOT NULL,      -- 1, 2, 3, 4, 5
    description TEXT,                    -- 선택사항
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Subject 내에서 (lesion_type, lesion_number) 유일
    CONSTRAINT uq_subject_lesion UNIQUE (subject_id, lesion_type, lesion_number)
);

CREATE INDEX idx_recist_lesion_subject ON recist_lesion(subject_id);
```

---

## 🔧 Migration 작업

### Migration 파일: `042_simplify_recist_lesion.sql`

```sql
-- 1. annotation_annotation에 lesion 정보 추가
ALTER TABLE annotation_annotation
ADD COLUMN IF NOT EXISTS lesion_type VARCHAR(20),
ADD COLUMN IF NOT EXISTS lesion_number INTEGER;

-- 2. 기존 매핑 데이터 이전 (있다면)
UPDATE annotation_annotation a
SET
    lesion_type = l.lesion_type::text,
    lesion_number = l.lesion_number
FROM recist_lesion_annotation_map m
JOIN recist_lesion l ON l.id = m.lesion_id
WHERE a.id = m.annotation_id;

-- 3. 매핑 테이블 삭제
DROP TABLE IF EXISTS recist_lesion_annotation_map;

-- 4. recist_lesion 불필요한 컬럼 삭제
ALTER TABLE recist_lesion DROP COLUMN IF EXISTS organ_site;
ALTER TABLE recist_lesion DROP COLUMN IF EXISTS baseline_timepoint_id;
ALTER TABLE recist_lesion DROP COLUMN IF EXISTS project_id;

-- 5. recist_lesion 제약 조건 변경
ALTER TABLE recist_lesion DROP CONSTRAINT IF EXISTS uq_subject_lesion_number;
ALTER TABLE recist_lesion
    ADD CONSTRAINT uq_subject_lesion UNIQUE (subject_id, lesion_type, lesion_number);

-- 6. lesion_type을 VARCHAR로 변경 (ENUM 제거)
ALTER TABLE recist_lesion
    ALTER COLUMN lesion_type TYPE VARCHAR(20);

-- 7. 인덱스 추가
CREATE INDEX IF NOT EXISTS idx_annotation_lesion_type
    ON annotation_annotation(lesion_type) WHERE lesion_type IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_recist_lesion_subject
    ON recist_lesion(subject_id);
```

---

## 📡 API 변경

### Before (복잡):
```http
# 1. Study → Subject 조회
GET /api/studies/by-uid/{study_uid}

# 2. Lesion 목록 조회
GET /api/recist-lesions/subjects/1

# 3. Lesion 생성
POST /api/recist-lesions/subjects/1
{
  "lesion_type": "TARGET",
  "baseline_timepoint_id": 1,
  "organ_site": "Liver"
}

# 4. Annotation 연결
POST /api/recist-lesions/1/annotations
{
  "lesion_id": 1,
  "annotation_id": 123,
  "timepoint_id": 1,
  "measured_length_mm": 25.5
}
```

### After (간단):
```http
# Annotation 생성/업데이트만!
PUT /api/annotations/123
{
  "lesion_type": "TARGET",
  "lesion_number": 1,
  "label": "Liver lesion"  // 선택사항
}

# 서버가 자동으로:
# 1. Study → Subject 조회
# 2. recist_lesion 테이블에 (subject_id, TARGET, 1) 생성 또는 조회
# 3. 분석/추적 데이터 자동 관리
```

---

## 🎯 사용 시나리오

### 1. Baseline에서 Target Lesion 측정

```javascript
// 1. Annotation 생성 (뷰어에서)
const annotation = await createAnnotation({
  study_uid: "...",
  annotation_data: {...},
  measurement_values: [{type: "length", values: [25.5], unit: "mm"}]
});
// → annotation_id: 123

// 2. 우클릭 → "Target Lesion 1" 선택
await fetch('/api/annotations/123', {
  method: 'PUT',
  headers: {'Content-Type': 'application/json'},
  body: JSON.stringify({
    lesion_type: 'TARGET',
    lesion_number: 1,
    label: 'Liver lesion'  // 선택사항
  })
});

// 끝! 서버가 자동으로:
// - Study → Subject 조회
// - recist_lesion 테이블에 (subject_id, TARGET, 1) 생성 또는 조회
```

### 2. Follow-up에서 동일 Lesion 재측정

```javascript
// TP1에서 Target Lesion 1 다시 측정
const annotation = await createAnnotation({
  study_uid: "TP1_study",
  annotation_data: {...},
  measurement_values: [{type: "length", values: [20.3], unit: "mm"}]
});

// 같은 Target Lesion 1로 설정
await fetch(`/api/annotations/${annotation.id}`, {
  method: 'PUT',
  headers: {'Content-Type': 'application/json'},
  body: JSON.stringify({
    lesion_type: 'TARGET',
    lesion_number: 1  // 같은 번호
  })
});
```

### 3. New Lesion 발견

```javascript
// TP2에서 새 병변 발견
const annotation = await createAnnotation({
  study_uid: "TP2_study",
  annotation_data: {...}
});

// Target New Lesion 1로 설정
await fetch(`/api/annotations/${annotation.id}`, {
  method: 'PUT',
  headers: {'Content-Type': 'application/json'},
  body: JSON.stringify({
    lesion_type: 'TARGET_NEW',
    lesion_number: 1,
    label: 'New liver lesion'
  })
});
```

---

## 📊 조회 예시

### Subject의 Target Lesion 1 추적

```sql
-- Subject 1의 Target Lesion 1 시간에 따른 변화
SELECT
    tp.name AS timepoint,
    tp.order_index,
    a.id AS annotation_id,
    a.lesion_type,
    a.lesion_number,
    a.measurement_values->0->'values'->0 AS size_mm,
    a.created_at
FROM annotation_annotation a
JOIN subject_timepoint_study_map m ON m.study_uid = a.study_uid
JOIN subject_timepoint tp ON tp.id = m.timepoint_id
WHERE m.subject_id = 1
  AND a.lesion_type = 'TARGET'
  AND a.lesion_number = 1
ORDER BY tp.order_index;
```

**결과**:
```
timepoint | annotation_id | lesion_type | lesion_number | size_mm | created_at
----------|---------------|-------------|---------------|---------|------------
BL        | 123           | TARGET      | 1             | 25.5    | 2026-01-01
TP1       | 456           | TARGET      | 1             | 20.3    | 2026-02-01
TP2       | 789           | TARGET      | 1             | 18.0    | 2026-03-01
```

### Subject의 모든 Lesion 요약

```sql
-- Subject 1의 모든 Lesion 목록 (recist_lesion 테이블)
SELECT
    lesion_type,
    lesion_number,
    description,
    created_at,
    (SELECT COUNT(*) FROM annotation_annotation a
     JOIN subject_timepoint_study_map m ON m.study_uid = a.study_uid
     WHERE m.subject_id = l.subject_id
       AND a.lesion_type = l.lesion_type
       AND a.lesion_number = l.lesion_number) AS annotation_count
FROM recist_lesion l
WHERE subject_id = 1
ORDER BY lesion_type, lesion_number;
```

**결과**:
```
lesion_type  | lesion_number | description   | annotation_count
-------------|---------------|---------------|------------------
TARGET       | 1             | Liver lesion  | 3
TARGET       | 2             | Lung lesion   | 2
NON_TARGET   | 1             | Bone lesion   | 2
TARGET_NEW   | 1             | New liver     | 1
```


