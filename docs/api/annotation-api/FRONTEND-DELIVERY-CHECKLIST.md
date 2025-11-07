# ✅ 프론트엔드 팀 전달 체크리스트

## 🎯 개요

Annotation API가 완성되었습니다! 프론트엔드 팀에 전달할 준비가 완료되었습니다! 🚀

---

## 📚 프론트엔드 팀이 읽어야 할 문서

### 1️⃣ **필수 문서** (반드시 읽기)

| 문서 | 설명 | 대상 |
|------|------|------|
| **FRONTEND-INTEGRATION-GUIDE.md** | 전체 통합 전략 및 아키텍처 | 프론트엔드 리더 |
| **FRONTEND-API-SPEC.md** | 완전한 API 명세 (요청/응답) | 모든 개발자 |
| **ANNOTATION-DATA-FIELD-STRATEGY.md** | 데이터 필드 전략 및 시퀀스 다이어그램 | 모든 개발자 |

### 2️⃣ **참고 문서** (필요시 읽기)

| 문서 | 설명 | 대상 |
|------|------|------|
| **ANNOTATION-LIST-OPTIMIZATION.md** | 목록 최적화 전략 | 성능 최적화 담당자 |
| **VERSION-FIELD-EXPLANATION.md** | Version 필드의 3가지 용도 | 동시성 제어 담당자 |
| **IMPLEMENTATION-ROADMAP.md** | 구현 로드맵 및 일정 | 프로젝트 매니저 |

---

## 🔧 API 엔드포인트 요약

### 1. Study/Series 레벨 Annotation 조회

```http
GET /api/annotations?study_instance_uid={uid}&level=study,series&project_id={id}
```

**응답:**
- 요약 정보 (annotation_data 제외)
- 응답 크기: 50KB
- 로드 시간: 200-300ms
- 캐시: ETag, Last-Modified 헤더 포함

---

### 2. Instance 레벨 Annotation 조회

```http
GET /api/annotations?study_instance_uid={uid}&series_instance_uid={uid}&level=instance&project_id={id}
```

**응답:**
- 전체 정보 (annotation_data 포함)
- 응답 크기: 500KB
- 캐시: ETag, Last-Modified 헤더 포함

---

### 3. 특정 Annotation 조회

```http
GET /api/annotations/{id}
```

**응답:**
- 전체 정보 (annotation_data 포함)
- Version 필드 포함 (Optimistic Locking용)

---

### 4. HEAD 요청 (캐시 검증)

```http
HEAD /api/annotations/{id}
```

**응답:**
- 헤더만 반환 (본문 없음)
- ETag, Last-Modified 헤더 포함
- 304 Not Modified 가능

---

### 5. Annotation 생성

```http
POST /api/annotations
```

**요청:**
```json
{
  "study_instance_uid": "1.2.3.4.5",
  "series_instance_uid": "1.2.3.4.5.6",
  "sop_instance_uid": "1.2.3.4.5.6.7",
  "annotation_data": {
    "type": "rectangle",
    "coordinates": [100, 100, 200, 200],
    "label": "Tumor",
    "color": "#FF0000",
    "tool_name": "Rectangle Tool",
    "measurements": {...},
    "description": "..."
  }
}
```

---

### 6. Annotation 수정 (Optimistic Locking)

```http
PUT /api/annotations/{id}
```

**요청:**
```json
{
  "base_version": 2,
  "annotation_data": {...}
}
```

**응답:**
- 성공: 200 OK (version 증가)
- 충돌: 409 Conflict (버전 불일치)

---

### 7. Annotation 삭제

```http
DELETE /api/annotations/{id}
```

---

## 📊 2단계 로딩 전략

### Step 1: 사이드바 목록 표시

```
GET /api/annotations/summary?series_instance_uid={uid}

응답 (50KB):
- type, label, color, tool_name, measurements
- created_by_name
- study_instance_uid, series_instance_uid, sop_instance_uid
- version

⏱️ 200-300ms
✅ annotation_data 불필요
```

### Step 2: 캔버스에 그리기

```
GET /api/annotations/{id}

응답 (500KB):
- annotation_data (coordinates 포함!)
- version

⏱️ 필요할 때만
⚠️ annotation_data 필수!
```

---

## ⚠️ 중요 사항

### 1. Version 검사

```typescript
// 사이드바에서 조회한 version
const summaryVersion = summary.version;

// 상세 정보에서 조회한 version
const detailVersion = detail.version;

// 버전 불일치 감지
if (summaryVersion !== detailVersion) {
  console.warn('⚠️ 버전 불일치!');
  // 최신 버전 사용
}
```

### 2. Optimistic Locking

```typescript
// 수정 시 base_version 필수
PUT /api/annotations/{id}
{
  "base_version": currentVersion,  // ← 필수!
  "annotation_data": {...}
}

// 409 Conflict 시 처리
if (response.status === 409) {
  // 최신 버전 조회
  const latest = await GET /api/annotations/{id};
  // 재시도
  await PUT /api/annotations/{id} with latest.version;
}
```

### 3. 캐시 검증

```typescript
// HEAD 요청으로 캐시 검증
HEAD /api/annotations/{id}
If-None-Match: "2"
If-Modified-Since: Mon, 01 Jan 2024 00:00:00 +0000

// 304 Not Modified 응답 시 캐시 사용
if (response.status === 304) {
  // 캐시된 데이터 사용
}
```

---

## 🚀 구현 순서

1. ✅ **Step 1**: 요약 목록 API 호출 (사이드바 표시)
2. ✅ **Step 2**: 상세 정보 API 호출 (캔버스 그리기)
3. ✅ **Step 3**: Version 검사 로직 구현
4. ✅ **Step 4**: Optimistic Locking 처리
5. ✅ **Step 5**: 캐시 검증 (HEAD 요청)

---

## 📞 문의사항

API 명세에 대한 질문이나 추가 필요사항이 있으면 백엔드 팀에 문의하세요!

---

**프론트엔드 팀 준비 완료!** 🎉

