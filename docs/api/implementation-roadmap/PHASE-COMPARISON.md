# 📊 Phase별 기능 비교표

**목적:** 각 개발 단계의 기능을 한눈에 비교  
**대상:** 모든 팀원

---

## 🎯 전체 기능 비교

| 기능 | Phase 1 ✅ | Phase 2 📅 | Phase 3 🔮 | 설명 |
|------|-----------|-----------|-----------|------|
| **기본 CRUD** | ✅ | ✅ | ✅ | Create, Read, Update, Delete |
| **Query Parameter** | ✅ | ✅ | ✅ | series_uid, study_uid, sop_uid 등 |
| **RBAC 권한 체크** | ✅ | ✅ | ✅ | READ_ALL 권한 확인 |
| **필터링** | ✅ | ✅ | ✅ | level, viewer_software 등 |
| **응답 포맷** | ✅ | ✅ | ✅ | { annotations: [...], total: N } |
| **Version Conflict** | ❌ | ✅ | ✅ | Optimistic Locking |
| **HEAD 요청** | ❌ | ✅ | ✅ | 캐시 검증 |
| **WebSocket** | ❌ | ✅ | ✅ | 실시간 동기화 |
| **Collaborative Lock** | ❌ | ✅ | ✅ | 동시 수정 방지 |
| **History/Audit** | ❌ | ❌ | ✅ | 변경 이력 추적 |
| **Advanced Merge** | ❌ | ❌ | ✅ | 3-way merge, OT, CRDT |
| **Performance** | ⚠️ | ⚠️ | ✅ | 캐싱, 인덱싱 최적화 |

---

## 📋 Phase 1: 기본 CRUD (완료) ✅

### 구현된 기능

```
✅ POST   /api/annotations              (Create)
✅ GET    /api/annotations              (Read - 다양한 필터)
✅ GET    /api/annotations/{id}         (Read - 단일)
✅ PUT    /api/annotations/{id}         (Update)
✅ DELETE /api/annotations/{id}         (Delete)
```

### Query Parameter 옵션

```
✅ ?series_instance_uid=...    (Series 레벨)
✅ ?study_instance_uid=...     (Study 레벨)
✅ ?sop_instance_uid=...       (Instance 레벨)
✅ ?project_id=...             (프로젝트 필터)
✅ ?user_id=...                (사용자 필터)
✅ ?level=study|series|instance (DICOM 계층)
✅ ?viewer_software=...        (Viewer 필터)
```

### 권한 체크

```
✅ user_id + project_id 조합 검증
✅ ANNOTATION:READ_ALL 권한 확인
✅ 권한 없으면 본인 어노테이션만 반환
```

### 응답 포맷

```json
✅ {
  "annotations": [...],
  "total": N
}
```

### 한계

```
❌ 버전 충돌 처리 없음
❌ 동시 편집 불가
❌ 실시간 동기화 불가
❌ 변경 이력 추적 불가
```

### 개발 기간

```
📅 약 1주 (2025-11-01 ~ 2025-11-07)
```

---

## 📅 Phase 2: 버전 관리 + 실시간 동기화 (계획 중)

### 추가될 기능

#### 2.1 Version Conflict 처리

```
❌ Phase 1: Last-Write-Wins (덮어쓰기)
✅ Phase 2: Optimistic Locking

PUT /api/annotations/1
{
  "baseVersion": 1,  ← NEW
  "updates": {...}
}

응답:
{
  "version": 2,  ← NEW
  ...
}

오류:
409 Conflict
{
  "error": "VersionConflict",
  "currentVersion": 2
}
```

#### 2.2 HEAD 요청

```
❌ Phase 1: 미지원
✅ Phase 2: 지원

HEAD /api/annotations?series_instance_uid=...

응답 헤더:
- Last-Modified: 2025-11-07T10:22:00Z
- Annotation-Version: 13
- Content-Length: 0
```

#### 2.3 WebSocket 실시간 동기화

```
❌ Phase 1: 미지원
✅ Phase 2: 지원

ws://localhost:8080/ws/annotations

이벤트:
- annotation_created
- annotation_updated
- annotation_deleted
```

#### 2.4 Collaborative Lock

```
❌ Phase 1: 미지원
✅ Phase 2: 지원

User A: 편집 시작 → Lock 획득
User B: 편집 시도 → "User A가 편집 중" 메시지
User A: 편집 완료 → Lock 해제
User B: 편집 가능
```

### 데이터 모델 변경

```
annotation_annotation (기존)
├─ ... (기존 필드)
└─ version (NEW) ← auto-increment

annotation_lock (NEW)
├─ id
├─ annotation_id
├─ user_id
├─ locked_at
└─ expires_at
```

### 개발 기간

```
📅 약 2-3주
├─ Version Conflict: 3-4일
├─ HEAD 요청: 1-2일
├─ WebSocket: 5-7일
└─ Collaborative Lock: 3-4일
```

### 우선순위

```
1순위 (필수):
  - Version Conflict 처리 (데이터 무결성)
  - HEAD 요청 (성능)

2순위 (권장):
  - WebSocket (UX)
  - Collaborative Lock (협업)
```

---

## 🔮 Phase 3: 감사 추적 + 최적화 (향후)

### 추가될 기능

#### 3.1 History / Audit Trail

```
❌ Phase 1-2: 미지원
✅ Phase 3: 지원

GET /api/annotations/1/history
→ 모든 버전 이력 반환

GET /api/annotations/1/history/2
→ 특정 버전 데이터 반환

annotation_history 테이블:
├─ id
├─ annotation_id
├─ version
├─ changed_by
├─ changed_at
├─ previous_data
├─ new_data
└─ change_type (CREATE/UPDATE/DELETE)
```

#### 3.2 Advanced Conflict Resolution

```
❌ Phase 1-2: 기본 Optimistic Locking
✅ Phase 3: 고급 병합

방식:
- 3-way merge
- Operational Transformation (OT)
- CRDT (Conflict-free Replicated Data Type)
```

#### 3.3 Performance Optimization

```
⚠️ Phase 1-2: 기본 성능
✅ Phase 3: 최적화

개선 사항:
- 캐싱 전략 개선
- 쿼리 최적화
- 인덱싱 추가
- 배치 처리
```

### 개발 기간

```
📅 약 3-4주
├─ History/Audit: 3-4일
├─ Advanced Merge: 5-7일
└─ Performance: 2-3일
```

---

## 📈 타임라인

```
Week 1-2: Phase 1 ✅ (완료)
├─ 기본 CRUD
├─ Query Parameter
├─ RBAC 권한 체크
└─ 테스트 통과

Week 3-5: Phase 2 📅 (계획 중)
├─ Version Conflict (3-4일)
├─ HEAD 요청 (1-2일)
├─ WebSocket (5-7일)
└─ Collaborative Lock (3-4일)

Week 6-9: Phase 3 🔮 (향후)
├─ History/Audit (3-4일)
├─ Advanced Merge (5-7일)
└─ Performance (2-3일)
```

---

## 🎯 각 Phase의 목표

### Phase 1 목표 ✅
- ✅ 단일 사용자 기반 안정적인 CRUD
- ✅ 권한 기반 필터링
- ✅ 다양한 조회 옵션

### Phase 2 목표 📅
- 버전 충돌 처리
- 캐시 최적화
- 실시간 동기화 기초

### Phase 3 목표 🔮
- 감사 추적
- 고급 충돌 해결
- 성능 최적화

---

## 💡 의사결정 포인트

### Phase 2 진행 여부
```
필수 조건:
✅ Phase 1 완료 및 테스트 통과
✅ 프론트엔드 팀 준비 완료
✅ 리소스 확보

의존성:
- Version Conflict 처리 (필수)
- WebSocket 구축 (선택)
```

### Phase 3 진행 여부
```
필수 조건:
✅ Phase 2 완료
✅ 사용자 피드백 수집
✅ 성능 이슈 확인

의존성:
- History/Audit (선택)
- Advanced Merge (선택)
```

---

## 📝 결론

**현재:** Phase 1 완료 ✅
- 기본 기능 모두 구현
- 테스트 통과
- 프로덕션 준비 완료

**다음:** Phase 2 시작 (2-3주)
- Version Conflict 처리 (필수)
- WebSocket 실시간 동기화 (권장)

**장기:** Phase 3 계획 (3-4주)
- 감사 추적
- 성능 최적화

