# 🎨 Annotation API - 프론트엔드 통합 가이드

## 📋 개요

이 문서는 프론트엔드에서 Annotation API를 효율적으로 사용하기 위한 통합 가이드입니다.

**핵심 전략:**
- 계층적 데이터 로딩 (Study → Series → Instance)
- 캐시 기반 최적화
- 버전 기반 동시성 제어
- HEAD 요청을 통한 대역폭 절약

---

## 🏗️ 아키텍처 개요

```
┌─────────────────────────────────────────────────────────┐
│                    프론트엔드 (React/Vue)                 │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│              Annotation 캐시 레이어                       │
│  ┌──────────────────────────────────────────────────┐   │
│  │ Study Level Cache    │ Series Level Cache        │   │
│  │ (version 추적)       │ (version 추적)            │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│              Annotation API (백엔드)                      │
│  ┌──────────────────────────────────────────────────┐   │
│  │ GET /annotations?level=study,series              │   │
│  │ GET /annotations?level=instance                  │   │
│  │ HEAD /annotations/{id}  (캐시 검증)              │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

---

## 📊 데이터 로딩 흐름

### Phase 1️⃣: Study/Series 로드 (초기 화면)

```
사용자가 Study 선택
    ↓
[1] GET /api/annotations?study_instance_uid={uid}&level=study,series
    ↓
응답: Study 레벨 + Series 레벨 Annotation
    ↓
캐시에 저장 (version 포함)
    ↓
화면에 표시
```

**요청:**
```bash
GET /api/annotations?study_instance_uid=1.2.3.4.5&level=study,series
```

**응답:**
```json
{
  "annotations": [
    {
      "id": 1,
      "study_instance_uid": "1.2.3.4.5",
      "series_instance_uid": "",
      "sop_instance_uid": "",
      "annotation_data": {...},
      "version": 1,
      "updated_at": "2024-01-01T00:00:00Z"
    },
    {
      "id": 2,
      "study_instance_uid": "1.2.3.4.5",
      "series_instance_uid": "1.2.3.4.5.6",
      "sop_instance_uid": "",
      "annotation_data": {...},
      "version": 2,
      "updated_at": "2024-01-01T00:01:00Z"
    }
  ],
  "total": 2
}
```

### Phase 2️⃣: Instance 로드 (인스턴스 선택 시)

```
사용자가 Instance 선택
    ↓
[1] HEAD /api/annotations/{id}
    (캐시된 버전과 비교)
    ↓
    ├─ 304 Not Modified → 캐시된 데이터 사용
    │
    └─ 200 OK → 새로운 버전 있음
        ↓
        [2] GET /api/annotations?series_instance_uid={uid}&level=instance
            ↓
            응답: Instance 레벨 Annotation
            ↓
            캐시 업데이트 (새로운 version)
            ↓
            화면에 표시
```

**Step 1: 캐시 검증 (HEAD 요청)**
```bash
HEAD /api/annotations/2
If-None-Match: "2"
```

**응답 (캐시 유효):**
```
HTTP/1.1 304 Not Modified
ETag: "2"
Last-Modified: Mon, 01 Jan 2024 00:01:00 +0000
```

**Step 2: 새로운 데이터 조회 (필요시)**
```bash
GET /api/annotations?series_instance_uid=1.2.3.4.5.6&level=instance
```

**응답:**
```json
{
  "annotations": [
    {
      "id": 3,
      "study_instance_uid": "1.2.3.4.5",
      "series_instance_uid": "1.2.3.4.5.6",
      "sop_instance_uid": "1.2.3.4.5.6.7",
      "annotation_data": {...},
      "version": 1,
      "updated_at": "2024-01-01T00:02:00Z"
    }
  ],
  "total": 1
}
```

---

## 💾 캐시 전략

### 캐시 구조

```javascript
// 프론트엔드 캐시 구조
const annotationCache = {
  study: {
    "1.2.3.4.5": {
      version: 1,
      etag: "\"1\"",
      lastModified: "Mon, 01 Jan 2024 00:00:00 +0000",
      data: [...],
      timestamp: 1704067200000
    }
  },
  series: {
    "1.2.3.4.5.6": {
      version: 2,
      etag: "\"2\"",
      lastModified: "Mon, 01 Jan 2024 00:01:00 +0000",
      data: [...],
      timestamp: 1704067260000
    }
  },
  instance: {
    "1.2.3.4.5.6.7": {
      version: 1,
      etag: "\"1\"",
      lastModified: "Mon, 01 Jan 2024 00:02:00 +0000",
      data: [...],
      timestamp: 1704067320000
    }
  }
}
```

### 캐시 검증 로직

```javascript
async function checkAnnotationCache(level, uid, cachedVersion) {
  try {
    // HEAD 요청으로 최신 버전 확인
    const response = await fetch(
      `/api/annotations/${cachedId}`,
      {
        method: 'HEAD',
        headers: {
          'If-None-Match': `"${cachedVersion}"`
        }
      }
    );

    if (response.status === 304) {
      // 캐시 유효 - 기존 데이터 사용
      return { valid: true, data: cachedData };
    } else if (response.status === 200) {
      // 새로운 버전 있음 - 전체 데이터 조회
      const newData = await fetchAnnotations(level, uid);
      return { valid: false, data: newData };
    }
  } catch (error) {
    console.error('캐시 검증 실패:', error);
    // 오류 시 전체 데이터 조회
    return { valid: false, data: await fetchAnnotations(level, uid) };
  }
}
```

---

## 🔄 버전 기반 동시성 제어

### Annotation 수정 시

```javascript
async function updateAnnotation(annotationId, newData, cachedVersion) {
  try {
    const response = await fetch(
      `/api/annotations/${annotationId}`,
      {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          base_version: cachedVersion,  // ← 중요!
          annotation_data: newData
        })
      }
    );

    if (response.status === 200) {
      // 업데이트 성공
      const updated = await response.json();
      updateCache(annotationId, updated);
      return { success: true, data: updated };
    } else if (response.status === 409) {
      // 버전 충돌 - 최신 버전 조회 후 재시도
      const conflict = await response.json();
      console.warn('버전 충돌:', conflict);
      
      // 최신 버전 조회
      const latest = await fetch(`/api/annotations/${annotationId}`).then(r => r.json());
      
      // 사용자에게 알림
      showConflictDialog(latest);
      return { success: false, conflict: true };
    }
  } catch (error) {
    console.error('업데이트 실패:', error);
    return { success: false, error };
  }
}
```

---

## 📡 API 엔드포인트 정리

### 1. Study/Series 레벨 조회

```bash
GET /api/annotations?study_instance_uid={uid}&level=study,series
```

**파라미터:**
- `study_instance_uid`: Study Instance UID
- `level`: `study,series` (쉼표로 구분)

**응답:** Study 레벨 + Series 레벨 Annotation 목록

### 2. Instance 레벨 조회

```bash
GET /api/annotations?series_instance_uid={uid}&level=instance
```

**파라미터:**
- `series_instance_uid`: Series Instance UID
- `level`: `instance`

**응답:** Instance 레벨 Annotation 목록

### 3. 캐시 검증 (HEAD 요청)

```bash
HEAD /api/annotations/{annotation_id}
If-None-Match: "{version}"
```

**응답:**
- `304 Not Modified`: 캐시 유효
- `200 OK`: 새로운 버전 있음

### 4. Annotation 수정

```bash
PUT /api/annotations/{annotation_id}
Content-Type: application/json

{
  "base_version": 1,
  "annotation_data": {...}
}
```

**응답:**
- `200 OK`: 수정 성공
- `409 Conflict`: 버전 충돌

---

## ⚡ 성능 최적화 팁

### 1. 초기 로딩 최적화

```javascript
// ❌ 나쁜 예: 순차 요청
const study = await getStudyAnnotations();
const series = await getSeriesAnnotations();
const instance = await getInstanceAnnotations();

// ✅ 좋은 예: 병렬 요청
const [study, series] = await Promise.all([
  getStudyAnnotations(),
  getSeriesAnnotations()
]);
// instance는 사용자가 선택할 때 로드
```

### 2. 캐시 활용

```javascript
// ✅ 캐시 먼저 확인
if (cache.has(uid)) {
  const cached = cache.get(uid);
  
  // HEAD 요청으로 최신 버전 확인
  const isValid = await validateCache(cached.version);
  
  if (isValid) {
    return cached.data;  // 캐시 사용
  }
}

// 캐시 없거나 유효하지 않으면 조회
return await fetchAnnotations(uid);
```

### 3. 대역폭 절약

```javascript
// ✅ HEAD 요청으로 메타데이터만 확인
const response = await fetch(`/api/annotations/${id}`, {
  method: 'HEAD'
});

// 필요시에만 전체 데이터 조회
if (response.status === 200) {
  const data = await fetch(`/api/annotations/${id}`).then(r => r.json());
}
```

---

## 🚨 에러 처리

### 409 Conflict (버전 충돌)

```javascript
if (response.status === 409) {
  const conflict = await response.json();
  
  console.error('버전 충돌:', {
    currentVersion: conflict.current_version,
    clientVersion: conflict.client_version
  });
  
  // 사용자에게 알림
  showDialog({
    title: '편집 충돌',
    message: '다른 사용자가 이 항목을 수정했습니다.',
    action: '최신 버전 확인'
  });
}
```

### 304 Not Modified (캐시 유효)

```javascript
if (response.status === 304) {
  // 캐시된 데이터 사용
  return cachedData;
}
```

---

## 📝 구현 체크리스트

- [ ] 캐시 레이어 구현 (Study/Series/Instance)
- [ ] Study/Series 레벨 조회 구현
- [ ] Instance 레벨 조회 구현
- [ ] HEAD 요청 기반 캐시 검증 구현
- [ ] 버전 기반 동시성 제어 구현
- [ ] 409 Conflict 에러 처리
- [ ] 304 Not Modified 처리
- [ ] 성능 모니터링 (캐시 히트율)

---

## 🎯 예상 효과

| 항목 | 개선 효과 |
|------|---------|
| **네트워크 트래픽** | 50-70% 감소 (캐시 + HEAD 요청) |
| **응답 시간** | 80-90% 단축 (캐시 히트 시) |
| **동시성 제어** | 100% 안전 (버전 기반) |
| **사용자 경험** | 매우 향상 (빠른 로딩) |

---

## 📞 문의 및 피드백

백엔드 팀에 문의사항이 있으면 다음을 확인하세요:

1. **API 응답 형식**: `docs/api/annotation-api/API-REVIEW-SUMMARY.md`
2. **Phase 2 상세 분석**: `docs/api/phase-2-analysis/PHASE-2-DETAILED-BREAKDOWN.md`
3. **버전 충돌 처리**: `docs/api/phase-2-analysis/WEBSOCKET-VS-VERSION-CONTROL.md`

