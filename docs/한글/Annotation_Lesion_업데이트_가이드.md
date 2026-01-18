# Annotation에 Lesion 정보 할당 가이드

## 📋 요구사항

Annotation에 다음 Lesion 정보를 할당할 수 있어야 함:
- **Target lesion** 1, 2, 3, 4, 5
- **Non-target lesion** 1, 2, 3, 4, 5
- **Target new lesion** 1, 2, 3, 4, 5
- **Non-target new lesion** 1, 2, 3, 4, 5

## 🎯 사용자 워크플로우

1. **뷰어에서 Annotation 생성** (측정선 그리기)
2. **Annotation 우클릭** 또는 **우측 사이드바**에서 "Lesion 할당" 선택
3. **Lesion 선택**:
   - 기존 Lesion 선택 (Target Lesion 1, 2, 3...)
   - 또는 새 Lesion 생성

---

## 📚 용어 설명

### 1. Lesion Type (병변 타입)

현재 DB에는 **3가지 타입**만 있음:
- `TARGET`: 측정 가능한 병변 (Baseline에서 발견)
- `NON_TARGET`: 측정 불가능한 병변 (Baseline에서 발견)
- `NEW`: 새로 발견된 병변 (Follow-up에서 발견)

⚠️ **문제**: Target new lesion vs Non-target new lesion 구분 불가
- 현재는 `description` 필드에 "Target new" 또는 "Non-target new" 명시 필요
- 향후 개선 필요 (아래 TODO 참조)

### 2. Baseline TimePoint

- **Baseline**: 임상시험의 첫 평가 시점 (기준선)
- **Follow-up**: Baseline 이후의 평가 시점 (TP1, TP2, TP3...)

**`baseline_timepoint_id`**:
- Target/Non-target lesion이 **처음 발견된 Baseline TimePoint의 ID**
- Target/Non-target lesion은 반드시 `baseline_timepoint_id` 필요
- New lesion은 `baseline_timepoint_id`가 NULL (Follow-up에서 발견되므로)

**예시**:
```
Subject 1:
  - Baseline (id: 1, name: "BL")     ← baseline_timepoint_id
  - TP1 (id: 2, name: "TP1")
  - TP2 (id: 3, name: "TP2")

Target Lesion 1:
  - baseline_timepoint_id: 1  (Baseline에서 처음 발견)
  - TP1에서 측정: 25.5mm
  - TP2에서 측정: 20.3mm

New Lesion 1:
  - baseline_timepoint_id: NULL  (TP2에서 새로 발견)
  - TP2에서 측정: 15.0mm
```

### 3. Organ Site (장기 위치)

병변이 위치한 장기/부위:
- `Liver` (간)
- `Lung` (폐)
- `Lymph Node` (림프절)
- `Bone` (뼈)
- `Soft Tissue` (연조직)
- 기타 자유 입력

---

## ✅ 현재 API 구조

### 2. RECIST Lesion API

#### 2.1 Subject의 Lesion 목록 조회 (사이드바용)

**엔드포인트**:
```http
GET /api/recist-lesions/subjects/{subject_id}?lesion_type=TARGET
```

**Query Parameters**:
- `lesion_type` (optional): `TARGET`, `NON_TARGET`, `NEW`

**Response**:
```json
[
  {
    "id": 1,
    "lesion_type": "TARGET",
    "lesion_number": 1,  // ✅ 자동 생성 (Subject 내 순번)
    "organ_site": "Liver",
    "description": "Right lobe lesion",
    "baseline_timepoint_id": 1,
    "created_at": "2026-01-18T10:00:00Z"
  },
  {
    "id": 2,
    "lesion_type": "TARGET",
    "lesion_number": 2,
    "organ_site": "Lung",
    "description": "Left lung lesion",
    "baseline_timepoint_id": 1,
    "created_at": "2026-01-18T10:00:00Z"
  }
]
```

**사용 예시**:
```javascript
// 사이드바에서 Lesion 목록 표시
const lesions = await fetch(`/api/recist-lesions/subjects/${subjectId}?lesion_type=TARGET`);
// → "Target Lesion 1 (Liver)", "Target Lesion 2 (Lung)" 표시
```

#### 2.2 Lesion 생성 (새 Lesion 추가)

**엔드포인트**:
```http
POST /api/recist-lesions/subjects/{subject_id}
```

**Request**:
```json
{
  "lesion_type": "TARGET",  // TARGET, NON_TARGET, NEW
  "baseline_timepoint_id": 1,  // Baseline TimePoint ID (NEW는 null)
  "organ_site": "Liver",  // 선택사항
  "description": "Right lobe lesion"  // 선택사항
}
```

**Response**:
```json
{
  "id": 1,
  "project_id": 1,
  "subject_id": 1,
  "lesion_type": "TARGET",
  "lesion_number": 1,  // ✅ 자동 생성
  "baseline_timepoint_id": 1,
  "organ_site": "Liver",
  "description": "Right lobe lesion",
  "created_at": "2026-01-18T10:00:00Z",
  "updated_at": "2026-01-18T10:00:00Z"
}
```

**비즈니스 규칙**:
- Target Lesion은 **최대 5개**까지만 허용
- Non-target Lesion은 제한 없음
- `lesion_number`는 자동 생성 (Subject 내 순번)

#### 2.3 Annotation을 Lesion에 할당

**엔드포인트**:
```http
POST /api/recist-lesions/{lesion_id}/annotations
```

**Request**:
```json
{
  "lesion_id": 1,
  "annotation_id": 123,
  "timepoint_id": 1,  // 현재 Study가 속한 TimePoint ID
  "measured_length_mm": 25.5  // 선택사항 (자동 계산 가능)
}
```

**Response**:
```json
{
  "message": "Annotation linked successfully"
}
```

**사용 예시**:
```javascript
// Annotation 우클릭 → "Target Lesion 1에 할당"
await fetch(`/api/recist-lesions/1/annotations`, {
  method: 'POST',
  body: JSON.stringify({
    lesion_id: 1,
    annotation_id: annotationId,
    timepoint_id: currentTimepointId,
    measured_length_mm: calculatedLength
  })
});
```

#### 2.4 Lesion 상세 조회 (TimePoint별 측정값 포함)

**엔드포인트**:
```http
GET /api/recist-lesions/{id}
```

**Response**:
```json
{
  "id": 1,
  "lesion_type": "TARGET",
  "lesion_number": 1,
  "organ_site": "Liver",
  "baseline_timepoint_id": 1,
  "annotations": [
    {
      "timepoint_id": 1,
      "timepoint_name": "BL",
      "annotation_id": 123,
      "measured_length_mm": 25.5,
      "measured_at": "2026-01-18T10:00:00Z"
    },
    {
      "timepoint_id": 2,
      "timepoint_name": "TP1",
      "annotation_id": 456,
      "measured_length_mm": 20.3,
      "measured_at": "2026-02-18T10:00:00Z"
    }
  ]
}
```

#### 2.5 Lesion 수정

**엔드포인트**:
```http
PUT /api/recist-lesions/{id}
```

**Request**:
```json
{
  "lesion_type": "NON_TARGET",
  "organ_site": "Lung",
  "description": "Updated description"
}
```

---

## 🎯 실제 사용 워크플로우

### 시나리오 1: Baseline에서 Target Lesion 측정

#### 사전 준비
```http
# Subject 확인
GET /api/subjects?project_id=1
→ subject_id: 1

# Baseline TimePoint 확인
GET /api/timepoints?subject_id=1
→ [
    {"id": 1, "name": "BL", "visit_type": "Baseline"},
    {"id": 2, "name": "TP1", "visit_type": "Visit"}
  ]
```

#### 1단계: 뷰어에서 Annotation 생성 (사용자 작업)

사용자가 뷰어에서 측정선을 그림:
```http
POST /api/annotations
{
  "study_instance_uid": "1.2.3.4.5",
  "series_instance_uid": "1.2.3.4.5.6",
  "sop_instance_uid": "1.2.3.4.5.6.7",
  "annotation_data": {
    "type": "line",
    "points": [[100, 100], [150, 150]]
  },
  "measurement_values": [
    {"id": "m1", "type": "length", "values": [25.5], "unit": "mm"}
  ]
}
```
→ 응답: `annotation_id: 123`

#### 2단계: Annotation 우클릭 → "Lesion 할당" (사용자 작업)

**옵션 A: 기존 Lesion에 할당**

사이드바에서 Lesion 목록 조회:
```http
GET /api/recist-lesions/subjects/1?lesion_type=TARGET
→ [
    {"id": 1, "lesion_number": 1, "organ_site": "Liver"},
    {"id": 2, "lesion_number": 2, "organ_site": "Lung"}
  ]
```

사용자가 "Target Lesion 1" 선택 → API 호출:
```http
POST /api/recist-lesions/1/annotations
{
  "lesion_id": 1,
  "annotation_id": 123,
  "timepoint_id": 1,  // Baseline TimePoint
  "measured_length_mm": 25.5
}
```

**옵션 B: 새 Lesion 생성 후 할당**

사용자가 "새 Target Lesion 생성" 선택 → 모달 표시:
- Lesion Type: Target
- Organ Site: Liver (선택사항)
- Description: Right lobe lesion (선택사항)

API 호출:
```http
# 1. Lesion 생성
POST /api/recist-lesions/subjects/1
{
  "lesion_type": "TARGET",
  "baseline_timepoint_id": 1,
  "organ_site": "Liver",
  "description": "Right lobe lesion"
}
→ 응답: {"id": 3, "lesion_number": 3}

# 2. Annotation 할당
POST /api/recist-lesions/3/annotations
{
  "lesion_id": 3,
  "annotation_id": 123,
  "timepoint_id": 1,
  "measured_length_mm": 25.5
}
```

---

### 시나리오 2: Follow-up에서 동일 Lesion 재측정

#### 1단계: TP1에서 Annotation 생성
```http
POST /api/annotations
{
  "study_instance_uid": "1.2.3.4.6",  // TP1의 Study
  "annotation_data": {...}
}
→ annotation_id: 456
```

#### 2단계: 기존 Lesion에 할당
```http
POST /api/recist-lesions/1/annotations
{
  "lesion_id": 1,  // 동일 Lesion (Target Lesion 1)
  "annotation_id": 456,
  "timepoint_id": 2,  // TP1 TimePoint
  "measured_length_mm": 20.3
}
```

---

### 시나리오 3: Follow-up에서 New Lesion 발견

#### 1단계: Annotation 생성
```http
POST /api/annotations
{
  "study_instance_uid": "1.2.3.4.7",  // TP2의 Study
  "annotation_data": {...}
}
→ annotation_id: 789
```

#### 2단계: New Lesion 생성 및 할당
```http
# 1. New Lesion 생성
POST /api/recist-lesions/subjects/1
{
  "lesion_type": "NEW",
  "baseline_timepoint_id": null,  // ✅ NEW는 null
  "organ_site": "Bone",
  "description": "Target new lesion"  // ⚠️ Target/Non-target 구분
}
→ 응답: {"id": 10, "lesion_number": 10}

# 2. Annotation 할당
POST /api/recist-lesions/10/annotations
{
  "lesion_id": 10,
  "annotation_id": 789,
  "timepoint_id": 3,  // TP2 TimePoint
  "measured_length_mm": 15.0
}
```

---

## 📊 프론트엔드 구현 가이드

### 1. 사이드바: Lesion 목록 표시

```javascript
// Subject의 모든 Lesion 조회
async function loadLesions(subjectId) {
  const response = await fetch(`/api/recist-lesions/subjects/${subjectId}`);
  const lesions = await response.json();

  // Lesion 타입별 그룹화
  const targetLesions = lesions.filter(l => l.lesion_type === 'TARGET');
  const nonTargetLesions = lesions.filter(l => l.lesion_type === 'NON_TARGET');
  const newLesions = lesions.filter(l => l.lesion_type === 'NEW');

  // UI 렌더링
  renderLesionGroup('Target Lesions', targetLesions);
  renderLesionGroup('Non-target Lesions', nonTargetLesions);
  renderLesionGroup('New Lesions', newLesions);
}

function renderLesionGroup(title, lesions) {
  return `
    <div class="lesion-group">
      <h3>${title}</h3>
      ${lesions.map(l => `
        <div class="lesion-item" data-lesion-id="${l.id}">
          ${l.lesion_type} Lesion ${l.lesion_number}
          ${l.organ_site ? `(${l.organ_site})` : ''}
        </div>
      `).join('')}
    </div>
  `;
}
```

### 2. Annotation 우클릭 메뉴

```javascript
// Annotation 우클릭 시
function showAnnotationContextMenu(annotationId) {
  const menu = [
    {
      label: '기존 Lesion에 할당',
      submenu: await getLesionSubmenu(subjectId)
    },
    {
      label: '새 Target Lesion 생성',
      action: () => createNewLesion('TARGET', annotationId)
    },
    {
      label: '새 Non-target Lesion 생성',
      action: () => createNewLesion('NON_TARGET', annotationId)
    },
    {
      label: '새 New Lesion 생성',
      action: () => createNewLesion('NEW', annotationId)
    }
  ];

  showContextMenu(menu);
}

async function getLesionSubmenu(subjectId) {
  const lesions = await fetch(`/api/recist-lesions/subjects/${subjectId}`).then(r => r.json());

  return lesions.map(lesion => ({
    label: `${lesion.lesion_type} Lesion ${lesion.lesion_number} ${lesion.organ_site ? `(${lesion.organ_site})` : ''}`,
    action: () => assignToLesion(lesion.id, annotationId)
  }));
}
```

### 3. Lesion 할당 함수

```javascript
// 기존 Lesion에 할당
async function assignToLesion(lesionId, annotationId) {
  const currentTimepointId = getCurrentTimepointId();  // 현재 Study의 TimePoint
  const measuredLength = getMeasuredLength(annotationId);  // Annotation에서 측정값 추출

  const response = await fetch(`/api/recist-lesions/${lesionId}/annotations`, {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({
      lesion_id: lesionId,
      annotation_id: annotationId,
      timepoint_id: currentTimepointId,
      measured_length_mm: measuredLength
    })
  });

  if (response.ok) {
    showNotification('Lesion에 할당되었습니다');
    refreshLesionList();
  }
}

// 새 Lesion 생성 후 할당
async function createNewLesion(lesionType, annotationId) {
  // 1. 모달 표시 (Organ Site, Description 입력)
  const {organSite, description} = await showLesionCreateModal(lesionType);

  // 2. Baseline TimePoint 확인
  const baselineTimepointId = lesionType === 'NEW' ? null : await getBaselineTimepointId(subjectId);

  // 3. Lesion 생성
  const lesionResponse = await fetch(`/api/recist-lesions/subjects/${subjectId}`, {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({
      lesion_type: lesionType,
      baseline_timepoint_id: baselineTimepointId,
      organ_site: organSite,
      description: description
    })
  });

  const newLesion = await lesionResponse.json();

  // 4. Annotation 할당
  await assignToLesion(newLesion.id, annotationId);
}

// Baseline TimePoint 조회
async function getBaselineTimepointId(subjectId) {
  const timepoints = await fetch(`/api/timepoints?subject_id=${subjectId}`).then(r => r.json());
  const baseline = timepoints.find(tp => tp.visit_type === 'Baseline');
  return baseline?.id;
}
```

### 4. Lesion 상세 정보 표시

```javascript
// Lesion 클릭 시 상세 정보 표시
async function showLesionDetail(lesionId) {
  const lesion = await fetch(`/api/recist-lesions/${lesionId}`).then(r => r.json());

  const html = `
    <div class="lesion-detail">
      <h3>${lesion.lesion_type} Lesion ${lesion.lesion_number}</h3>
      <p>Organ Site: ${lesion.organ_site || 'N/A'}</p>
      <p>Description: ${lesion.description || 'N/A'}</p>

      <h4>Measurements</h4>
      <table>
        <thead>
          <tr>
            <th>TimePoint</th>
            <th>Length (mm)</th>
            <th>Measured At</th>
          </tr>
        </thead>
        <tbody>
          ${lesion.annotations.map(ann => `
            <tr>
              <td>${ann.timepoint_name}</td>
              <td>${ann.measured_length_mm?.toFixed(1) || 'N/A'}</td>
              <td>${new Date(ann.measured_at).toLocaleDateString()}</td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    </div>
  `;

  showModal(html);
}
```

---

## ⚠️ 현재 제약사항 및 TODO

### 1. NEW Lesion의 Target/Non-target 구분 불가 ❌

**현재 문제**:
- DB에는 `TARGET`, `NON_TARGET`, `NEW` 3가지 타입만 있음
- Target new lesion vs Non-target new lesion 구분 불가
- 현재는 `description` 필드에 "Target new" 또는 "Non-target new" 명시 필요

**임시 해결책** (현재 사용 가능):
```json
{
  "lesion_type": "NEW",
  "baseline_timepoint_id": null,
  "description": "Target new lesion"  // ⚠️ 문자열로 구분
}
```

**프론트엔드 처리**:
```javascript
// NEW Lesion 생성 시 모달에서 선택
const newLesionType = await showModal({
  title: 'New Lesion 생성',
  options: [
    {label: 'Target New Lesion', value: 'target'},
    {label: 'Non-target New Lesion', value: 'non-target'}
  ]
});

const description = newLesionType === 'target'
  ? 'Target new lesion'
  : 'Non-target new lesion';

await createLesion({
  lesion_type: 'NEW',
  description: description
});
```

**향후 개선 방안**:

#### 방안 1: `is_target` 필드 추가 (간단)
```sql
ALTER TABLE recist_lesion
ADD COLUMN is_target BOOLEAN DEFAULT NULL;
-- NEW Lesion만 사용 (TARGET/NON_TARGET은 NULL)
```

```json
{
  "lesion_type": "NEW",
  "is_target": true,  // true: Target new, false: Non-target new
  "baseline_timepoint_id": null
}
```

#### 방안 2: Lesion 타입 확장 (권장) ⭐
```sql
-- ENUM 타입 확장
ALTER TYPE recist_lesion_type_enum ADD VALUE 'TARGET_NEW';
ALTER TYPE recist_lesion_type_enum ADD VALUE 'NON_TARGET_NEW';
```

```json
{
  "lesion_type": "TARGET_NEW",  // 또는 "NON_TARGET_NEW"
  "baseline_timepoint_id": null
}
```

**권장**: 방안 2 (Lesion 타입 확장) - 가장 명확하고 구조화됨

### 2. Target Lesion 최대 5개 제한

**비즈니스 규칙**:
- RECIST 1.1 기준: Target Lesion은 최대 5개까지만 허용
- Non-target Lesion은 제한 없음

**프론트엔드 처리**:
```javascript
async function canCreateTargetLesion(subjectId) {
  const lesions = await fetch(`/api/recist-lesions/subjects/${subjectId}?lesion_type=TARGET`)
    .then(r => r.json());

  if (lesions.length >= 5) {
    showError('Target Lesion은 최대 5개까지만 생성할 수 있습니다');
    return false;
  }

  return true;
}
```

### 3. Baseline TimePoint 자동 조회

**현재**: 프론트엔드에서 Baseline TimePoint ID를 조회해야 함

**개선 방안**: API에서 자동 조회
```rust
// CreateRecistLesionRequest에서 baseline_timepoint_id를 Optional로 변경
// Target/Non-target Lesion 생성 시 자동으로 Subject의 Baseline TimePoint 조회
```

---

## 🎯 결론 및 권장사항

### ✅ 권장: RECIST Lesion API 사용

**이유**:
1. **구조화된 데이터**: Lesion 타입, 번호, TimePoint 자동 관리
2. **RECIST 1.1 준수**: 표준 기준에 맞는 병변 추적
3. **TimePoint별 추적**: 동일 Lesion의 시간에 따른 변화 추적
4. **측정값 관리**: Annotation별 측정값 저장 및 조회
5. **확장성**: 향후 RECIST 평가 자동화 가능

**사용자 워크플로우**:
1. 뷰어에서 Annotation 생성 (측정선 그리기)
2. Annotation 우클릭 → "Lesion 할당"
3. 기존 Lesion 선택 또는 새 Lesion 생성
4. 자동으로 현재 TimePoint에 연결

**프론트엔드 구현 복잡도**:
- ⚠️ 중간 (Lesion 목록 조회, 생성, 할당 3단계)
- ✅ 재사용 가능한 컴포넌트로 구현 가능

### ⚠️ 대안: Annotation `label` 필드만 사용 (비권장)

**사용 방법**:
```http
PUT /api/annotations/123
{"label": "Target Lesion 1"}
```

**장점**:
- ✅ 간단함 (API 1개만 사용)

**단점**:
- ❌ 단순 문자열 (구조화 안됨)
- ❌ Lesion 타입 구분 어려움
- ❌ Lesion 번호 자동 관리 불가
- ❌ TimePoint별 추적 불가
- ❌ RECIST 1.1 기준 미준수
- ❌ 향후 확장 불가

---

## 📚 관련 문서

- [RECIST Lesion 작업 계획](../target-lesion/작업계획.md)
- [RECIST Lesion 구현 요약](../target-lesion/IMPLEMENTATION_SUMMARY.md)
- [Annotation API 가이드](../server/technical/ANNOTATION_API_GUIDE.md)
- [API 엔드포인트 레퍼런스](../server/technical/API_ENDPOINTS_REFERENCE.md)

