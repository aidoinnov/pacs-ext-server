# 📌 Annotation Data 필드 전략

## 🎯 문제 상황

요약 목록(Summary)에는 `annotation_data` 필드가 없는데, **캔버스에 annotation을 그리기 위해 필요한 데이터**를 놓칠까봐 걱정됨.

---

## 💡 해결책: 2단계 로딩 전략!

### 요약 목록에 포함된 정보 (사이드바 표시용)

```json
{
  "id": 1,
  "type": "rectangle",
  "label": "Tumor",
  "color": "#FF0000",
  "tool_name": "Rectangle Tool",
  "measurements": {
    "width": 100,
    "height": 100,
    "area": 10000
  },
  "created_by_name": "Dr. Kim",
  "study_instance_uid": "1.2.3.4.5",
  "series_instance_uid": "1.2.3.4.5.6",
  "sop_instance_uid": "1.2.3.4.5.6.7",
  "version": 2
}
```

**이 정보는 사이드바에 목록으로 표시하기 위한 정보입니다!** ✅

### 캔버스에 그리기 위해 필요한 정보 (annotation_data)

```json
{
  "type": "rectangle",
  "coordinates": [100, 100, 200, 200],  // ← 캔버스에 그리기 위해 필수!
  "label": "Tumor",
  "color": "#FF0000",
  "tool_name": "Rectangle Tool",
  "description": "Suspicious lesion",
  "measurements": {
    "width": 100,
    "height": 100,
    "area": 10000
  },
  "metadata": {
    "confidence": 0.95,
    "reviewer": "Dr. Park",
    "notes": "Follow-up needed"
  }
}
```

**`coordinates` 필드가 없으면 캔버스에 그릴 수 없습니다!** ⚠️

---

## 📊 annotation_data vs 요약 정보

### annotation_data 구조 (전체)

```json
{
  "type": "rectangle",
  "coordinates": [100, 100, 200, 200],
  "label": "Tumor",
  "color": "#FF0000",
  "tool_name": "Rectangle Tool",
  "description": "Suspicious lesion",
  "measurements": {
    "width": 100,
    "height": 100,
    "area": 10000
  },
  "metadata": {
    "confidence": 0.95,
    "reviewer": "Dr. Park",
    "notes": "Follow-up needed"
  }
}
```

### 요약 정보 (필수 필드만)

```json
{
  "type": "rectangle",
  "label": "Tumor",
  "color": "#FF0000",
  "tool_name": "Rectangle Tool",
  "measurements": {
    "width": 100,
    "height": 100,
    "area": 10000
  }
}
```

**차이점:**
- ❌ `coordinates` 제외 (목록에서 필요 없음)
- ❌ `description` 제외 (목록에서 필요 없음)
- ❌ `metadata` 제외 (목록에서 필요 없음)
- ✅ `type`, `label`, `color`, `tool_name`, `measurements` 포함

---

## 🔄 데이터 흐름

### Step 1: 요약 목록 조회 (Series 선택)

```
GET /api/annotations/summary?series_instance_uid={uid}

응답 (50KB):
{
  "annotations": [
    {
      "id": 1,
      "type": "rectangle",
      "label": "Tumor",
      "color": "#FF0000",
      "tool_name": "Rectangle Tool",
      "measurements": {...},
      "created_by_name": "Dr. Kim",
      "study_instance_uid": "1.2.3.4.5",
      "series_instance_uid": "1.2.3.4.5.6",
      "sop_instance_uid": "1.2.3.4.5.6.7",
      "version": 2
    },
    // ... 19개 더
  ]
}

사이드바에 목록 표시:
├─ Annotation 타입: rectangle ✅
├─ 라벨: Tumor ✅
├─ 색상: #FF0000 ✅
├─ 도구: Rectangle Tool ✅
├─ 측정값: 100x100 ✅
├─ 작성자: Dr. Kim ✅
└─ UID 정보: 표시 ✅
```

**사이드바 목록 표시에는 이 정보로 충분합니다!** ✅

---

### Step 2: 사용자가 Annotation 선택 (캔버스에 그리기)

```
사용자가 사이드바 목록에서 annotation 클릭
    ↓
캔버스에 annotation을 그려야 함
    ↓
⚠️ 문제: 요약에는 coordinates가 없음!
    ↓
GET /api/annotations/{id}
응답 (전체 annotation_data 포함):
{
  "id": 1,
  "annotation_data": {
    "type": "rectangle",
    "coordinates": [100, 100, 200, 200],  // ← 이것이 필수!
    "label": "Tumor",
    "color": "#FF0000",
    "tool_name": "Rectangle Tool",
    "description": "Suspicious lesion",
    "measurements": {...},
    "metadata": {...}
  },
  "version": 2
}
    ↓
캔버스에 그리기:
├─ coordinates 사용: [100, 100, 200, 200] ✅
├─ type 사용: rectangle ✅
├─ color 사용: #FF0000 ✅
└─ label 사용: Tumor ✅
```

**캔버스에 그리려면 annotation_data의 coordinates가 필수입니다!** ⚠️

---

## 🎨 시퀀스 다이어그램

### 전체 흐름

```
┌─────────────┐                    ┌──────────┐                    ┌────────────┐
│  Frontend   │                    │ Backend  │                    │ Database   │
└──────┬──────┘                    └────┬─────┘                    └─────┬──────┘
       │                                │                                │
       │ 1. Series 선택                 │                                │
       │ (사이드바 목록 필요)            │                                │
       │                                │                                │
       ├─────────────────────────────────>                               │
       │ GET /api/annotations/summary    │                               │
       │ ?series_instance_uid={uid}      │                               │
       │                                │                                │
       │                                ├───────────────────────────────>│
       │                                │ SELECT id, type, label,        │
       │                                │ color, tool_name,              │
       │                                │ measurements, ...              │
       │                                │ (annotation_data 제외)         │
       │                                │                                │
       │                                │<───────────────────────────────┤
       │                                │ 요약 정보 반환                  │
       │                                │ (50KB)                         │
       │<─────────────────────────────────                               │
       │ 응답: AnnotationSummary[]       │                               │
       │ {                              │                               │
       │   id, type, label, color,      │                               │
       │   tool_name, measurements,     │                               │
       │   created_by_name, UIDs, ...   │                               │
       │ }                              │                               │
       │                                │                               │
       │ 2. 사이드바에 목록 표시         │                               │
       │ (요약 정보로 충분)              │                               │
       │ ✅ 완료                         │                               │
       │                                │                               │
       │ 3. 사용자가 annotation 선택     │                               │
       │ (캔버스에 그려야 함)            │                               │
       │                                │                               │
       ├─────────────────────────────────>                               │
       │ GET /api/annotations/{id}       │                               │
       │ (전체 데이터 필요)              │                               │
       │                                │                                │
       │                                ├───────────────────────────────>│
       │                                │ SELECT * FROM annotation       │
       │                                │ (annotation_data 포함)         │
       │                                │                                │
       │                                │<───────────────────────────────┤
       │                                │ 전체 정보 반환                  │
       │                                │ (500KB)                        │
       │<─────────────────────────────────                               │
       │ 응답: AnnotationDetail          │                               │
       │ {                              │                               │
       │   id, annotation_data: {       │                               │
       │     type, coordinates,         │                               │
       │     label, color, ...          │                               │
       │   }, version                   │                               │
       │ }                              │                               │
       │                                │                               │
       │ 4. 캔버스에 그리기              │                               │
       │ (coordinates 사용)              │                               │
       │ ✅ 완료                         │                               │
       │                                │                               │
```

---

### Step 1: 사이드바 목록 조회 (요약 정보)

```
프론트엔드                          백엔드                          데이터베이스
    │                                │                                │
    ├─ Series 선택                   │                                │
    │                                │                                │
    ├──────────────────────────────────>                               │
    │ GET /api/annotations/summary    │                               │
    │ ?series_instance_uid=1.2.3.4.5.6                               │
    │                                │                                │
    │                                ├───────────────────────────────>│
    │                                │ SELECT                         │
    │                                │   id, type, label, color,      │
    │                                │   tool_name, measurements,     │
    │                                │   created_by_name,             │
    │                                │   study_instance_uid,          │
    │                                │   series_instance_uid,         │
    │                                │   sop_instance_uid,            │
    │                                │   version                      │
    │                                │ FROM annotation_annotation     │
    │                                │ WHERE series_instance_uid = ?  │
    │                                │ (annotation_data 제외!)        │
    │                                │                                │
    │                                │<───────────────────────────────┤
    │                                │ 요약 정보 반환                  │
    │<─────────────────────────────────                               │
    │ 응답 (50KB):                    │                               │
    │ [                              │                               │
    │   {                            │                               │
    │     id: 1,                     │                               │
    │     type: "rectangle",         │                               │
    │     label: "Tumor",            │                               │
    │     color: "#FF0000",          │                               │
    │     tool_name: "Rectangle",    │                               │
    │     measurements: {...},       │                               │
    │     created_by_name: "Dr. Kim",│                               │
    │     version: 2                 │                               │
    │   },                           │                               │
    │   ...                          │                               │
    │ ]                              │                               │
    │                                │                               │
    ├─ 사이드바에 목록 표시           │                               │
    │ ✅ 완료 (annotation_data 불필요)│                               │
    │                                │                               │
```

---

### Step 2: 캔버스에 그리기 (전체 데이터 + Version 검사)

```
프론트엔드                          백엔드                          데이터베이스
    │                                │                                │
    ├─ 사이드바에서 annotation 선택   │                                │
    │ (annotation.id = 1)            │                                │
    │ summary.version = 1            │                                │
    │                                │                                │
    ├──────────────────────────────────>                               │
    │ GET /api/annotations/1          │                               │
    │ (전체 데이터 필요!)             │                               │
    │                                │                                │
    │                                ├───────────────────────────────>│
    │                                │ SELECT *                       │
    │                                │ FROM annotation_annotation     │
    │                                │ WHERE id = 1                   │
    │                                │ (annotation_data 포함!)        │
    │                                │                                │
    │                                │<───────────────────────────────┤
    │                                │ 전체 정보 반환                  │
    │<─────────────────────────────────                               │
    │ 응답 (500KB):                   │                               │
    │ {                              │                               │
    │   id: 1,                       │                               │
    │   annotation_data: {           │                               │
    │     type: "rectangle",         │                               │
    │     coordinates: [100, 100,    │ ← 캔버스에 그리기 위해 필수!   │
    │                    200, 200],  │                               │
    │     label: "Tumor",            │                               │
    │     color: "#FF0000",          │                               │
    │     tool_name: "Rectangle",    │                               │
    │     description: "...",        │                               │
    │     measurements: {...},       │                               │
    │     metadata: {...}            │                               │
    │   },                           │                               │
    │   version: 2  ← 새로운 버전!   │                               │
    │ }                              │                               │
    │                                │                               │
    ├─ Version 검사                   │                               │
    │ if (summary.version !== detail.version) {                      │
    │   console.warn('⚠️ 버전 불일치!');                             │
    │   console.log(`Summary: v1, Detail: v2`);                      │
    │   // 최신 버전 사용 (detail.version = 2)                       │
    │ }                              │                               │
    │                                │                               │
    ├─ 캔버스에 그리기                │                               │
    │ drawRectangle(                 │                               │
    │   coordinates: [100, 100, 200, 200],                           │
    │   color: "#FF0000"             │                               │
    │ )                              │                               │
    │ ✅ 완료 (annotation_data 필요!)  │                               │
    │                                │                               │
```

---

### Step 3: 수정 시 Version 검사 (Optimistic Locking)

```
프론트엔드                          백엔드                          데이터베이스
    │                                │                                │
    ├─ 사용자가 annotation 수정       │                                │
    │ currentVersion = 2             │                                │
    │ (detail에서 얻은 버전)          │                                │
    │                                │                                │
    ├──────────────────────────────────>                               │
    │ PUT /api/annotations/1          │                               │
    │ {                              │                               │
    │   "base_version": 2,           │ ← 현재 버전 사용               │
    │   "annotation_data": {...}     │                               │
    │ }                              │                               │
    │                                │                                │
    │                                ├───────────────────────────────>│
    │                                │ SELECT version                 │
    │                                │ FROM annotation_annotation     │
    │                                │ WHERE id = 1                   │
    │                                │                                │
    │                                │<───────────────────────────────┤
    │                                │ 현재 version = 2               │
    │                                │                                │
    │                                ├─ Version 검사                  │
    │                                │ if (base_version == current) { │
    │                                │   ✅ 일치! 수정 진행            │
    │                                │ } else {                       │
    │                                │   ❌ 불일치! 409 반환          │
    │                                │ }                              │
    │                                │                                │
    │                                ├───────────────────────────────>│
    │                                │ UPDATE annotation_annotation   │
    │                                │ SET annotation_data = ...,     │
    │                                │     version = version + 1      │
    │                                │ WHERE id = 1 AND               │
    │                                │       version = 2              │
    │                                │                                │
    │                                │<───────────────────────────────┤
    │                                │ 수정 완료                       │
    │<─────────────────────────────────                               │
    │ 응답 (200 OK):                  │                               │
    │ {                              │                               │
    │   id: 1,                       │                               │
    │   annotation_data: {...},      │                               │
    │   version: 3  ← 증가!          │                               │
    │ }                              │                               │
    │                                │                               │
    │ ✅ 수정 완료                    │                               │
    │                                │                               │
```

---

### Step 4: Version 충돌 시나리오

```
프론트엔드 A                        백엔드                          프론트엔드 B
    │                                │                                │
    ├─ annotation 조회               │                                │
    │ version = 1                    │                                │
    │                                │                                ├─ annotation 조회
    │                                │                                │ version = 1
    │                                │                                │
    ├─ 수정 시작                      │                                ├─ 수정 시작
    │                                │                                │
    │                                │                                ├──────────────────────>
    │                                │                                │ PUT /api/annotations/1
    │                                │                                │ base_version: 1
    │                                │                                │
    │                                │<──────────────────────────────┤
    │                                │ ✅ 수정 완료                    │
    │                                │ version: 1 → 2                 │
    │                                │                                │
    ├──────────────────────────────────>                               │
    │ PUT /api/annotations/1          │                               │
    │ base_version: 1                │                               │
    │ (구버전!)                       │                               │
    │                                │                                │
    │                                ├─ Version 검사                  │
    │                                │ base_version (1) ≠ current (2) │
    │                                │ ❌ 불일치!                     │
    │<──────────────────────────────────                               │
    │ 응답 (409 Conflict):            │                               │
    │ {                              │                               │
    │   "error": "Version Conflict", │                               │
    │   "current_version": 2,        │                               │
    │   "client_version": 1          │                               │
    │ }                              │                               │
    │                                │                               │
    ├─ 최신 버전 조회                 │                               │
    │ GET /api/annotations/1          │                               │
    │                                │                                │
    │                                ├───────────────────────────────>│
    │                                │ SELECT * WHERE id = 1          │
    │                                │                                │
    │                                │<───────────────────────────────┤
    │<──────────────────────────────────                               │
    │ 응답: version = 2               │                               │
    │                                │                               │
    ├─ 재시도                         │                               │
    │ PUT /api/annotations/1          │                               │
    │ base_version: 2                │ ← 최신 버전 사용               │
    │                                │                               │
    │                                ├─ Version 검사                  │
    │                                │ base_version (2) == current (2)│
    │                                │ ✅ 일치!                       │
    │                                │                               │
    │<──────────────────────────────────                               │
    │ 응답 (200 OK):                  │                               │
    │ version: 3                     │                               │
    │ ✅ 수정 완료                    │                               │
    │                                │                               │
```

---

## 📋 필드별 용도 정리

### 요약 목록에서 필요한 필드

| 필드 | 출처 | 용도 | 요약에 포함 |
|------|------|------|-----------|
| `id` | DB | 식별자 | ✅ |
| `type` | annotation_data | 타입 표시 | ✅ |
| `label` | annotation_data | 라벨 표시 | ✅ |
| `color` | annotation_data | 색상 표시 | ✅ |
| `tool_name` | annotation_data | 도구 표시 | ✅ |
| `measurements` | annotation_data | 측정값 표시 | ✅ |
| `created_by_name` | security_user | 작성자 표시 | ✅ |
| `study_instance_uid` | DB | UID 표시 | ✅ |
| `series_instance_uid` | DB | UID 표시 | ✅ |
| `sop_instance_uid` | DB | UID 표시 | ✅ |
| `version` | DB | 버전 관리 | ✅ |
| `created_at` | DB | 생성 시간 표시 | ✅ |
| `updated_at` | DB | 수정 시간 표시 | ✅ |

### 상세 정보에서만 필요한 필드

| 필드 | 출처 | 용도 | 요약에 포함 |
|------|------|------|-----------|
| `coordinates` | annotation_data | 캔버스에 그리기 | ❌ |
| `description` | annotation_data | 상세 설명 | ❌ |
| `metadata` | annotation_data | 추가 정보 | ❌ |
| `annotation_data` (전체) | DB | 모든 정보 | ❌ |

---

## 🔍 annotation_data 필드가 없어도 되는 이유

### 1. 목록 표시에 필요한 정보는 모두 추출됨

```typescript
// 요약에서 추출된 정보로 충분
const summary = {
  type: "rectangle",           // ✅ 있음
  label: "Tumor",              // ✅ 있음
  color: "#FF0000",            // ✅ 있음
  tool_name: "Rectangle Tool", // ✅ 있음
  measurements: {...}          // ✅ 있음
};

// 목록 렌더링에 필요한 모든 정보 포함!
```

### 2. 상세 정보가 필요하면 별도 요청

```typescript
// 사용자가 annotation 선택
async function onAnnotationSelected(summary: AnnotationSummary) {
  // 상세 정보 필요 시에만 요청
  const detail = await fetch(`/api/annotations/${summary.id}`);
  const fullData = await detail.json();
  
  // 이제 annotation_data 전체 사용 가능
  const { coordinates, description, metadata } = fullData.annotation_data;
  
  // 캔버스에 그리기, 상세 정보 표시 등
}
```

### 3. 성능 최적화

```
요약 목록 (annotation_data 제외):
├─ 응답 크기: 50KB
├─ 로드 시간: 200-300ms
└─ 메모리: 1MB

전체 데이터 (annotation_data 포함):
├─ 응답 크기: 500KB
├─ 로드 시간: 2-3초
└─ 메모리: 10MB

개선율: 90% 감소! 🚀
```

---

## ✅ 체크리스트: 요약에 포함된 필드

### 필수 필드 (모두 포함됨)

- [x] `id` - Annotation 식별자
- [x] `type` - Annotation 타입 (rectangle, polygon, etc.)
- [x] `label` - Annotation 라벨
- [x] `color` - Annotation 색상
- [x] `tool_name` - 도구 이름
- [x] `measurements` - 측정값 (width, height, area, perimeter)
- [x] `created_by_name` - 작성자 이름
- [x] `study_instance_uid` - Study UID
- [x] `series_instance_uid` - Series UID
- [x] `sop_instance_uid` - SOP Instance UID
- [x] `version` - 버전 정보
- [x] `created_at` - 생성 시간
- [x] `updated_at` - 수정 시간

### 제외된 필드 (필요 없음)

- [ ] `coordinates` - 목록에서 필요 없음 (상세 정보에서만)
- [ ] `description` - 목록에서 필요 없음 (상세 정보에서만)
- [ ] `metadata` - 목록에서 필요 없음 (상세 정보에서만)
- [ ] `annotation_data` (전체) - 필요한 필드만 추출됨

---

## ⚠️ Version 검사 (중요!)

### Version 검사가 필요한 시점

```
Step 1: 사이드바 목록 조회
GET /api/annotations/summary
응답: version = 1

Step 2: 사용자가 annotation 선택
GET /api/annotations/{id}
응답: version = 1 (또는 2, 3, ...)

⚠️ 문제: 사이드바의 version과 상세 정보의 version이 다를 수 있음!

예시:
- 사이드바 조회 시: version = 1
- 다른 사용자가 수정: version = 1 → 2
- 상세 정보 조회 시: version = 2

→ 버전 불일치 감지!
```

### Version 검사 로직

```
Step 1: 요약 목록 조회
GET /api/annotations/summary?series_instance_uid={uid}
응답:
{
  "annotations": [
    {
      "id": 1,
      "version": 1,  ← 캐시에 저장
      ...
    }
  ]
}

Step 2: 사용자가 annotation 선택
GET /api/annotations/{id}
응답:
{
  "id": 1,
  "annotation_data": {...},
  "version": 2  ← 새로운 버전!
}

Step 3: Version 검사
if (summaryVersion !== detailVersion) {
  console.warn('⚠️ 버전 불일치!');
  console.log(`Summary: v${summaryVersion}, Detail: v${detailVersion}`);

  // 선택지:
  // 1. 최신 버전 사용 (detail의 version)
  // 2. 사용자에게 알림
  // 3. 캐시 무효화
}
```

---

### 수정 시 Version 검사 (Optimistic Locking)

```
Step 1: 사이드바에서 annotation 선택
summary.version = 1

Step 2: 캔버스에 그리기
detail.version = 1

Step 3: 사용자가 annotation 수정
PUT /api/annotations/{id}
요청:
{
  "base_version": 1,  ← 사이드바 또는 상세 정보의 version
  "annotation_data": {...}
}

Step 4: 서버에서 Version 검사
SELECT version FROM annotation_annotation WHERE id = 1;
현재 version = 2 (다른 사용자가 수정함)

⚠️ base_version (1) ≠ 현재 version (2)
→ 409 Conflict 응답!

응답:
{
  "error": "Version Conflict",
  "current_version": 2,
  "client_version": 1
}

Step 5: 클라이언트에서 처리
// 최신 버전 조회
GET /api/annotations/{id}
응답: version = 2

// 최신 버전으로 재시도
PUT /api/annotations/{id}
요청:
{
  "base_version": 2,  ← 최신 버전 사용
  "annotation_data": {...}
}
```

---

## 🎯 결론

### annotation_data 필드의 역할

| 상황 | 필요한 데이터 | annotation_data | 이유 |
|------|-------------|-----------------|------|
| **사이드바 목록 표시** | 요약 정보 | ❌ 필요 없음 | type, label, color, tool_name, measurements로 충분 |
| **캔버스에 그리기** | 전체 정보 | ✅ 필수! | coordinates가 필요함 |
| **Version 검사** | 버전 정보 | ❌ 필요 없음 | version 필드만 필요 |

---

### 2단계 로딩 전략

#### Step 1: 요약 목록 조회 (사이드바 표시)
```
GET /api/annotations/summary?series_instance_uid={uid}

응답 (50KB):
- annotation_data 제외
- 사이드바 목록 표시에 필요한 정보만 포함
- 빠른 로드 (200-300ms)
```

#### Step 2: 전체 데이터 조회 (캔버스 그리기)
```
GET /api/annotations/{id}

응답 (500KB):
- annotation_data 포함 (coordinates 포함!)
- 캔버스에 그리기 위해 필수
- 필요할 때만 요청
```

---

### 성능 개선

| 항목 | 기존 | 최적화 | 개선율 |
|------|------|--------|--------|
| **응답 크기** | 500KB | 50KB | 90% 감소 |
| **로드 시간** | 2-3초 | 200-300ms | 90% 단축 |
| **메모리** | 10MB | 1MB | 90% 감소 |

---

### 핵심 포인트

✅ **사이드바 목록 표시**
- 요약 정보로 충분
- annotation_data 불필요
- 빠른 로드

⚠️ **캔버스에 그리기**
- annotation_data 필수!
- coordinates 필요
- 별도 요청 필요

---

## 📝 구현 흐름

```
1. Series 선택
   ↓
2. GET /api/annotations/summary
   ↓
3. 사이드바에 목록 표시 ✅
   (annotation_data 없이도 가능)
   ↓
4. 사용자가 annotation 선택
   ↓
5. GET /api/annotations/{id}
   ↓
6. 캔버스에 그리기 ✅
   (annotation_data의 coordinates 사용)
```

---

## 🚀 최종 정리

**annotation_data 필드가 없어도 사이드바 목록 표시에는 문제없습니다!** ✅

하지만 **캔버스에 그리려면 annotation_data가 필수**입니다! ⚠️

따라서 **2단계 로딩 전략**을 사용하면 됩니다:
1. 빠른 목록 로드 (요약 정보)
2. 필요할 때 상세 정보 로드 (전체 데이터)

