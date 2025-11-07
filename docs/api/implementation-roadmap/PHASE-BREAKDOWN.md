# 📊 Annotation API 단계별 분석

**목적:** 각 개발 단계의 상세 분석 및 구현 계획  
**대상:** 백엔드 팀, 프론트엔드 팀

---

## 🎯 Phase 1: 기본 CRUD (완료) ✅

### 1.1 현재 상태

**구현 기간:** 2025-11-01 ~ 2025-11-07 (약 1주)  
**상태:** ✅ 완료 및 테스트 통과

### 1.2 구현된 엔드포인트

```
POST   /api/annotations
GET    /api/annotations
GET    /api/annotations/{id}
PUT    /api/annotations/{id}
DELETE /api/annotations/{id}
```

### 1.3 주요 기능

#### 1.3.1 조회 (GET)
- **Series 레벨:** `?series_instance_uid=...&project_id=...`
- **Study 레벨:** `?study_instance_uid=...&project_id=...`
- **Instance 레벨:** `?sop_instance_uid=...`
- **필터링:** level, viewer_software, user_id
- **권한 체크:** RBAC 기반 (READ_ALL 권한 확인)

#### 1.3.2 생성 (POST)
- 모든 필드 포함 (project_id, user_id, UIDs, tool_name, data)
- 201 Created 응답
- 생성된 annotation 반환

#### 1.3.3 수정 (PUT)
- 수정할 필드만 포함
- 200 OK 응답
- 수정된 annotation 반환
- **주의:** 버전 충돌 처리 없음 (Last-Write-Wins)

#### 1.3.4 삭제 (DELETE)
- 204 No Content 응답
- 즉시 삭제 (복구 불가)

### 1.4 데이터 모델

```
annotation_annotation 테이블
├─ id (PK)
├─ project_id (FK)
├─ user_id (FK)
├─ study_instance_uid
├─ series_instance_uid
├─ sop_instance_uid
├─ tool_name
├─ tool_version
├─ data (JSON)
├─ is_shared
├─ created_at
├─ updated_at
├─ viewer_software
├─ description
└─ measurement_values (JSON)
```

### 1.5 권한 체크 로직

```
GET /api/annotations?series_instance_uid=...&project_id=2&user_id=1

1. user_id=1이 project_id=2의 멤버인가?
   └─ NO → 401 Unauthorized

2. user_id=1이 ANNOTATION:READ_ALL 권한이 있는가?
   ├─ YES → 모든 어노테이션 반환
   └─ NO  → 본인 어노테이션만 반환
```

### 1.6 테스트 결과

```
✅ 통합 테스트: 5/5 통과
✅ 권한 체크: 정상 작동
✅ 필터링: 모든 옵션 정상 작동
✅ 응답 포맷: 표준화됨
```

### 1.7 한계

- ❌ 버전 충돌 처리 없음
- ❌ 동시 편집 불가
- ❌ 실시간 동기화 불가
- ❌ 변경 이력 추적 불가

---

## 📅 Phase 2: 버전 관리 + 실시간 동기화 (계획 중)

### 2.1 목표

- 버전 충돌 처리
- 캐시 최적화
- 실시간 동기화 기초

### 2.2 세부 기능

#### 2.2.1 Version Conflict 처리 (우선순위: 높음)

**문제:**
```
User A: PUT /api/annotations/1 (v1 → v2)
User B: PUT /api/annotations/1 (v1 → v2)
결과: User B의 변경사항이 User A를 덮어씀 (데이터 손실!)
```

**해결책: Optimistic Locking**
```
PUT /api/annotations/1
{
  "baseVersion": 1,
  "updates": {
    "description": "Updated by User A"
  }
}

서버 검증:
├─ 현재 버전 == baseVersion?
│  ├─ YES → 업데이트 (v2로 증가)
│  └─ NO  → 409 Conflict 반환
```

**구현 계획:**
- [ ] `version` 필드 추가 (auto-increment)
- [ ] `baseVersion` 검증 로직
- [ ] 409 Conflict 응답 처리
- [ ] 클라이언트 재시도 로직

**예상 개발 시간:** 3-4일

#### 2.2.2 HEAD 요청 (우선순위: 중간)

**목적:** 버전 정보만 조회 (대역폭 절감)

**현재:**
```
GET /api/annotations?series_instance_uid=...
→ 전체 annotation 데이터 반환 (1-10MB)
```

**개선:**
```
HEAD /api/annotations?series_instance_uid=...
→ 응답 헤더만 반환 (1KB)
  - Last-Modified: 2025-11-07T10:22:00Z
  - Annotation-Version: 13
  - Content-Length: 0
```

**구현 계획:**
- [ ] HEAD 요청 핸들러 추가
- [ ] 응답 헤더 최적화
- [ ] 캐시 검증 로직

**예상 개발 시간:** 1-2일

#### 2.2.3 WebSocket 실시간 동기화 (우선순위: 중간)

**목적:** 다중 사용자 동시 편집 지원

**구조:**
```
Client A (Viewer 1)
    ↓
    ├─→ WebSocket Server
    ↓
Client B (Viewer 2)

이벤트:
- annotation_created
- annotation_updated
- annotation_deleted
```

**구현 계획:**
- [ ] WebSocket 서버 구축 (Actix-web)
- [ ] 이벤트 브로드캐스팅
- [ ] 클라이언트 구독 관리
- [ ] 재연결 처리

**예상 개발 시간:** 5-7일

#### 2.2.4 Collaborative Lock (우선순위: 중간)

**목적:** 동일 annotation 동시 수정 방지

**구조:**
```
User A: 편집 시작
  ├─ Lock 획득 (annotation_id=1)
  └─ 다른 사용자는 읽기만 가능

User B: 편집 시도
  └─ "User A가 편집 중입니다" 메시지

User A: 편집 완료
  └─ Lock 해제

User B: 편집 가능
```

**구현 계획:**
- [ ] `annotation_lock` 테이블 생성
- [ ] Lock 획득/해제 로직
- [ ] 타임아웃 처리 (5분)
- [ ] 편집자 표시 (Presence)

**예상 개발 시간:** 3-4일

### 2.3 데이터 모델 변경

```
annotation_annotation 테이블 (기존)
├─ ... (기존 필드)
└─ version (NEW) ← auto-increment

annotation_lock 테이블 (NEW)
├─ id (PK)
├─ annotation_id (FK)
├─ user_id (FK)
├─ locked_at
└─ expires_at
```

### 2.4 API 변경

**수정 요청:**
```
PUT /api/annotations/1
{
  "baseVersion": 1,  ← NEW
  "updates": {
    "description": "Updated"
  }
}
```

**응답:**
```json
{
  "id": 1,
  "version": 2,  ← NEW
  "updated_at": "2025-11-07T11:32:00Z",
  ...
}
```

**오류 응답:**
```json
409 Conflict
{
  "error": "VersionConflict",
  "message": "Server version is 2, client baseVersion is 1",
  "currentVersion": 2
}
```

### 2.5 예상 개발 시간

- Version Conflict: 3-4일
- HEAD 요청: 1-2일
- WebSocket: 5-7일
- Collaborative Lock: 3-4일
- **총 예상:** 2-3주

---

## 🔮 Phase 3: 감사 추적 + 최적화 (향후)

### 3.1 목표

- 변경 이력 추적
- 고급 충돌 해결
- 성능 최적화

### 3.2 세부 기능

#### 3.2.1 History / Audit Trail

**데이터 모델:**
```
annotation_history 테이블
├─ id (PK)
├─ annotation_id (FK)
├─ version
├─ changed_by (user_id)
├─ changed_at
├─ previous_data (JSON)
├─ new_data (JSON)
└─ change_type (CREATE/UPDATE/DELETE)
```

**API:**
```
GET /api/annotations/1/history
→ 모든 버전 이력 반환

GET /api/annotations/1/history/2
→ 특정 버전 데이터 반환
```

#### 3.2.2 Advanced Conflict Resolution

**방식:**
- 3-way merge
- Operational Transformation (OT)
- CRDT (Conflict-free Replicated Data Type)

#### 3.2.3 Performance Optimization

**목표:**
- 캐싱 전략 개선
- 쿼리 최적화
- 인덱싱 추가

### 3.3 예상 개발 시간

- History/Audit: 3-4일
- Advanced Merge: 5-7일
- Performance: 2-3일
- **총 예상:** 3-4주

---

## 📈 전체 타임라인

```
Week 1-2: Phase 1 (완료) ✅
├─ 기본 CRUD
├─ Query Parameter
├─ RBAC 권한 체크
└─ 테스트

Week 3-5: Phase 2 (계획 중) 📅
├─ Version Conflict (3-4일)
├─ HEAD 요청 (1-2일)
├─ WebSocket (5-7일)
└─ Collaborative Lock (3-4일)

Week 6-9: Phase 3 (향후) 🔮
├─ History/Audit (3-4일)
├─ Advanced Merge (5-7일)
└─ Performance (2-3일)
```

---

## 🎯 우선순위

### 높음 (Phase 2 - 1순위)
1. Version Conflict 처리 (데이터 무결성)
2. HEAD 요청 (성능)

### 중간 (Phase 2 - 2순위)
3. WebSocket 실시간 동기화 (UX)
4. Collaborative Lock (협업)

### 낮음 (Phase 3)
5. History/Audit Trail (감사)
6. Advanced Conflict Resolution (고급)
7. Performance Optimization (최적화)

---

## 📝 결론

**현재 상태:** 1차 개발 완료 ✅
- 기본 CRUD 작동
- 권한 체크 정상
- 테스트 통과

**다음 단계:** 2차 개발 (2-3주)
- Version Conflict 처리 (필수)
- WebSocket 실시간 동기화 (권장)

**장기 계획:** 3차 개발 (3-4주)
- 감사 추적
- 성능 최적화

