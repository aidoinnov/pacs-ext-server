# 🚀 Annotation API 구현 로드맵

**목표:** DICOM Viewer Annotation 시스템의 단계별 개발 계획  
**현재 상태:** 1차 개발 완료 (기본 CRUD + 권한 체크)  
**다음 단계:** 2차 개발 (버전 관리 + 실시간 동기화)

---

## 📈 전체 개발 단계

```
┌─────────────────────────────────────────────────────────────┐
│ 1차 개발 (완료) ✅                                            │
│ - 기본 CRUD                                                  │
│ - Query Parameter 기반 조회                                   │
│ - RBAC 기반 권한 체크                                         │
│ - 다양한 필터링                                               │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 2차 개발 (계획 중) 📅                                         │
│ - Version Conflict 처리                                      │
│ - HEAD 요청 (캐시 검증)                                       │
│ - WebSocket 실시간 동기화                                     │
│ - Collaborative Lock                                        │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 3차 개발 (향후) 🔮                                            │
│ - History / Audit Trail                                     │
│ - Advanced Conflict Resolution                              │
│ - Performance Optimization                                  │
└─────────────────────────────────────────────────────────────┘
```

---

## ✅ 1차 개발 (완료)

### 목표
- 단일 사용자 기반 안정적인 CRUD
- 권한 기반 필터링
- 다양한 조회 옵션

### 구현된 기능

#### 1.1 기본 CRUD ✅
```
POST   /api/annotations              → Create
GET    /api/annotations?...          → Read (다양한 필터)
PUT    /api/annotations/{id}         → Update
DELETE /api/annotations/{id}         → Delete
```

#### 1.2 Query Parameter 기반 조회 ✅
```
?series_instance_uid=...    → Series 레벨
?study_instance_uid=...     → Study 레벨
?sop_instance_uid=...       → Instance 레벨
?project_id=...             → 프로젝트 필터
?user_id=...                → 사용자 필터
?level=study|series|instance → DICOM 계층 필터
?viewer_software=...        → Viewer 필터
```

#### 1.3 RBAC 기반 권한 체크 ✅
```
user_id + project_id 조합으로 권한 확인
↓
ANNOTATION:READ_ALL 권한 있음?
├─ YES → 모든 어노테이션 반환
└─ NO  → 본인 어노테이션만 반환
```

#### 1.4 응답 포맷 표준화 ✅
```json
{
  "annotations": [
    {
      "id": 243,
      "project_id": 2,
      "user_id": 1,
      "study_instance_uid": "...",
      "series_instance_uid": "...",
      "sop_instance_uid": "...",
      "tool_name": "ROI",
      "data": {...},
      "created_at": "2025-11-07T10:22:00Z",
      "updated_at": "2025-11-07T10:22:00Z",
      "user_name": "John Doe"
    }
  ],
  "total": 1
}
```

### 테스트 상태
- ✅ 통합 테스트: 5/5 통과
- ✅ 권한 체크: 정상 작동
- ✅ 필터링: 모든 옵션 정상 작동

---

## 📅 2차 개발 (계획 중)

### 목표
- 버전 충돌 처리
- 캐시 최적화
- 실시간 동기화 기초

### 2.1 Version Conflict 처리 (우선순위: 높음)

**현재 상태:** Last-Write-Wins (덮어쓰기)
```
User A: PUT /api/annotations/1 (v1 → v2)
User B: PUT /api/annotations/1 (v1 → v2)  ← 충돌!
결과: User B의 변경사항이 User A를 덮어씀
```

**개선 방안:** Optimistic Locking
```
PUT /api/annotations/1
{
  "baseVersion": 1,  ← 클라이언트가 알고 있는 버전
  "updates": {...}
}

서버 검증:
├─ 현재 버전 == baseVersion? 
│  ├─ YES → 업데이트 (v2로 증가)
│  └─ NO  → 409 Conflict 반환
└─ 클라이언트: 최신 데이터 다시 조회 후 재시도
```

**구현 계획:**
- [ ] `baseVersion` 필드 추가
- [ ] 버전 충돌 감지 로직
- [ ] 409 Conflict 응답 처리
- [ ] 클라이언트 재시도 로직

### 2.2 HEAD 요청 (우선순위: 중간)

**목적:** 버전 정보만 조회 (대역폭 절감)

**현재:** 전체 데이터 조회 필요
```
GET /api/annotations?series_instance_uid=...
→ 전체 annotation 데이터 반환 (무거움)
```

**개선:** HEAD 요청
```
HEAD /api/annotations?series_instance_uid=...
→ 응답 헤더만 반환
  - Last-Modified: 2025-11-07T10:22:00Z
  - Annotation-Version: 13
  - Content-Length: 0
```

**구현 계획:**
- [ ] HEAD 요청 핸들러 추가
- [ ] 응답 헤더 최적화
- [ ] 캐시 검증 로직

### 2.3 WebSocket 실시간 동기화 (우선순위: 중간)

**목적:** 다중 사용자 동시 편집 지원

**구조:**
```
Client A ──┐
           ├─→ WebSocket Server ──→ Client B
Client C ──┘                    ──→ Client D

이벤트:
- annotation_created
- annotation_updated
- annotation_deleted
```

**구현 계획:**
- [ ] WebSocket 서버 구축
- [ ] 이벤트 브로드캐스팅
- [ ] 클라이언트 구독 관리
- [ ] 재연결 처리

### 2.4 Collaborative Lock (우선순위: 중간)

**목적:** 동일 annotation 동시 수정 방지

**구조:**
```
User A: 편집 시작 → Lock 획득
User B: 편집 시도 → Lock 대기 또는 거부
User A: 편집 완료 → Lock 해제
User B: 편집 가능
```

**구현 계획:**
- [ ] Lock 테이블 생성
- [ ] Lock 획득/해제 로직
- [ ] 타임아웃 처리
- [ ] 편집자 표시 (Presence)

---

## 🔮 3차 개발 (향후)

### 목표
- 감사 추적
- 고급 충돌 해결
- 성능 최적화

### 3.1 History / Audit Trail

**목적:** 변경 이력 추적

**구조:**
```
annotation_history 테이블
├─ id
├─ annotation_id
├─ version
├─ changed_by (user_id)
├─ changed_at
├─ previous_data
├─ new_data
└─ change_type (CREATE/UPDATE/DELETE)
```

### 3.2 Advanced Conflict Resolution

**목적:** 자동 충돌 해결

**방식:**
- 3-way merge
- Operational Transformation (OT)
- CRDT (Conflict-free Replicated Data Type)

### 3.3 Performance Optimization

**목표:**
- 캐싱 전략 개선
- 쿼리 최적화
- 인덱싱 추가

---

## 📊 단계별 비교표

| 기능 | 1차 | 2차 | 3차 |
|------|-----|-----|-----|
| **기본 CRUD** | ✅ | ✅ | ✅ |
| **Query Parameter** | ✅ | ✅ | ✅ |
| **RBAC 권한 체크** | ✅ | ✅ | ✅ |
| **필터링** | ✅ | ✅ | ✅ |
| **Version Conflict** | ❌ | ✅ | ✅ |
| **HEAD 요청** | ❌ | ✅ | ✅ |
| **WebSocket** | ❌ | ✅ | ✅ |
| **Collaborative Lock** | ❌ | ✅ | ✅ |
| **History/Audit** | ❌ | ❌ | ✅ |
| **Advanced Merge** | ❌ | ❌ | ✅ |

---

## 🎯 다음 단계

### 즉시 (1주일 내)
- [ ] 1차 개발 최종 테스트
- [ ] 문서 정리
- [ ] 프론트엔드 팀 공유

### 단기 (2-3주)
- [ ] 2차 개발 계획 수립
- [ ] Version Conflict 처리 구현
- [ ] HEAD 요청 구현

### 중기 (1개월)
- [ ] WebSocket 실시간 동기화
- [ ] Collaborative Lock
- [ ] 통합 테스트

### 장기 (2개월 이상)
- [ ] History / Audit Trail
- [ ] Advanced Conflict Resolution
- [ ] 성능 최적화

---

## 📝 참고사항

**1차 개발의 장점:**
- ✅ 빠른 개발 (기본 기능 중심)
- ✅ 안정적 (단순한 로직)
- ✅ 유지보수 용이

**1차 개발의 한계:**
- ❌ 동시 편집 불가
- ❌ 버전 충돌 처리 없음
- ❌ 실시간 동기화 불가

**2차 개발의 필요성:**
- 다중 사용자 환경 지원
- 데이터 무결성 보장
- 사용자 경험 개선

