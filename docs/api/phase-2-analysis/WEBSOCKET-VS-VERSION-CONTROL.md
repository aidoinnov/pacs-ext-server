# 🔄 WebSocket vs Version Control 비교 분석

**목적:** 2-3 (WebSocket)과 2-1 (Version Control)의 차이점 및 상호작용 이해  
**대상:** 백엔드 개발자, 아키텍트

---

## 🎯 핵심 차이점

| 항목 | Version Control (2-1) | WebSocket (2-3) |
|------|----------------------|-----------------|
| **목적** | 데이터 무결성 보장 | 실시간 동기화 |
| **문제 해결** | 동시 편집 시 덮어쓰기 방지 | 사용자 간 변경사항 실시간 공유 |
| **기술** | Optimistic Locking | 이벤트 브로드캐스팅 |
| **응답 시간** | 즉시 (HTTP) | 실시간 (WebSocket) |
| **필수 여부** | ✅ 필수 | 📅 권장 |
| **개발 난이도** | 중간 | 높음 |
| **개발 기간** | 3-4일 | 5-7일 |

---

## 📊 상황별 비교

### 상황 1: 단일 사용자 편집

```
User A: 어노테이션 수정

Version Control (2-1):
├─ PUT /api/annotations/1 (baseVersion: 1)
├─ 서버: 버전 확인 (1 == 1) ✅
├─ 업데이트 (v1 → v2)
└─ 200 OK 반환

WebSocket (2-3):
├─ 필요 없음 (다른 사용자 없음)
└─ 선택사항
```

### 상황 2: 두 사용자 동시 편집 (Version Control 없음)

```
User A: PUT /api/annotations/1 (v1 → v2)
User B: PUT /api/annotations/1 (v1 → v2)

결과: ❌ 데이터 손실!
└─ User B의 변경사항이 User A를 덮어씀
```

### 상황 3: 두 사용자 동시 편집 (Version Control 있음)

```
User A: PUT /api/annotations/1 (baseVersion: 1)
User B: PUT /api/annotations/1 (baseVersion: 1)

서버 처리:
├─ User A 요청 먼저 도착
│  ├─ 버전 확인 (1 == 1) ✅
│  ├─ 업데이트 (v1 → v2)
│  └─ 200 OK 반환
│
└─ User B 요청 나중 도착
   ├─ 버전 확인 (2 != 1) ❌
   ├─ 409 Conflict 반환
   └─ User B: 최신 데이터 조회 후 재시도

결과: ✅ 데이터 무결성 보장!
```

### 상황 4: 두 사용자 동시 편집 (WebSocket 있음)

```
User A: 어노테이션 수정 시작
├─ WebSocket 연결 (project_id=2 구독)
└─ annotation_updated 이벤트 발송

User B: 같은 어노테이션 보고 있음
├─ WebSocket 연결 (project_id=2 구독)
└─ annotation_updated 이벤트 수신
   └─ UI 자동 업데이트 (실시간!)

결과: ✅ 실시간 동기화!
```

### 상황 5: 두 사용자 동시 편집 (Version Control + WebSocket)

```
User A: 어노테이션 수정
├─ PUT /api/annotations/1 (baseVersion: 1)
├─ 서버: 버전 확인 (1 == 1) ✅
├─ 업데이트 (v1 → v2)
├─ 200 OK 반환
└─ WebSocket: annotation_updated 브로드캐스트

User B: 같은 어노테이션 보고 있음
├─ WebSocket: annotation_updated 이벤트 수신
├─ UI 자동 업데이트 (실시간!)
└─ 최신 데이터 표시

결과: ✅ 데이터 무결성 + 실시간 동기화!
```

---

## 🔄 Version Control (2-1) 상세

### 문제: 동시 편집 시 덮어쓰기

```
Timeline:
T1: User A 조회 (v1)
T2: User B 조회 (v1)
T3: User A 수정 (v1 → v2)
T4: User B 수정 (v1 → v2)  ← User A의 변경사항 덮어씀!

결과: User A의 변경사항 손실 ❌
```

### 해결책: Optimistic Locking

```
T1: User A 조회 (v1)
T2: User B 조회 (v1)
T3: User A 수정 (baseVersion: 1)
    ├─ 서버: 현재 버전 == 1? YES ✅
    ├─ 업데이트 (v1 → v2)
    └─ 200 OK

T4: User B 수정 (baseVersion: 1)
    ├─ 서버: 현재 버전 == 1? NO ❌
    ├─ 409 Conflict 반환
    └─ User B: 최신 데이터 조회 후 재시도

결과: 데이터 무결성 보장 ✅
```

### 구현 흐름

```
1. 클라이언트: 데이터 조회
   GET /api/annotations/1
   ↓
   {
     "id": 1,
     "version": 1,  ← 버전 정보 포함
     "description": "Original"
   }

2. 사용자: 데이터 수정

3. 클라이언트: 수정된 데이터 전송
   PUT /api/annotations/1
   {
     "baseVersion": 1,  ← 클라이언트가 알고 있는 버전
     "updates": {
       "description": "Modified"
     }
   }

4. 서버: 버전 검증
   ├─ 현재 버전 == baseVersion?
   │  ├─ YES → 업데이트 (v1 → v2)
   │  └─ NO  → 409 Conflict

5. 클라이언트: 응답 처리
   ├─ 200 OK → 완료
   └─ 409 Conflict → 재시도
```

### 장점
- ✅ 간단한 구현
- ✅ 데이터 무결성 보장
- ✅ 낮은 서버 부하
- ✅ 확장성 좋음

### 단점
- ❌ 사용자가 충돌 해결해야 함
- ❌ 재시도 필요
- ❌ 실시간 동기화 없음

---

## 🔌 WebSocket (2-3) 상세

### 문제: 사용자 간 변경사항 공유 안 됨

```
User A: 어노테이션 수정
├─ PUT /api/annotations/1
└─ 200 OK

User B: 같은 어노테이션 보고 있음
├─ 여전히 이전 데이터 표시
└─ 수동으로 새로고침 필요 ❌
```

### 해결책: WebSocket 실시간 동기화

```
User A: 어노테이션 수정
├─ PUT /api/annotations/1
├─ 200 OK
└─ WebSocket: annotation_updated 브로드캐스트

User B: 같은 어노테이션 보고 있음
├─ WebSocket: annotation_updated 이벤트 수신
├─ UI 자동 업데이트
└─ 최신 데이터 표시 ✅
```

### 구현 흐름

```
1. 클라이언트: WebSocket 연결
   ws://localhost:8080/ws/annotations
   ↓
   {
     "action": "subscribe",
     "project_id": 2
   }

2. 서버: 클라이언트 등록
   ├─ sessions[client_id] = connection
   └─ subscriptions[project_id].add(client_id)

3. User A: 어노테이션 수정
   PUT /api/annotations/1
   ↓
   서버: annotation_updated 이벤트 생성
   ↓
   WebSocket: 모든 구독자에게 브로드캐스트
   ├─ User A: 수정 완료 확인
   └─ User B: 자동 업데이트

4. 클라이언트: 이벤트 수신
   {
     "type": "annotation_updated",
     "annotation_id": 1,
     "user_id": 1,
     "user_name": "User A",
     "changes": {
       "description": "Modified"
     },
     "new_version": 2
   }
   ↓
   UI 업데이트
```

### 장점
- ✅ 실시간 동기화
- ✅ 사용자 경험 향상
- ✅ 자동 업데이트
- ✅ 협업 지원

### 단점
- ❌ 복잡한 구현
- ❌ 높은 서버 부하
- ❌ 연결 관리 필요
- ❌ 재연결 처리 필요

---

## 🔗 Version Control + WebSocket 통합

### 통합 흐름

```
1. 클라이언트: 데이터 조회
   GET /api/annotations/1
   ↓
   {
     "id": 1,
     "version": 1,
     "description": "Original"
   }

2. 클라이언트: WebSocket 연결
   ws://localhost:8080/ws/annotations
   ↓
   구독 시작 (project_id=2)

3. User A: 데이터 수정
   PUT /api/annotations/1
   {
     "baseVersion": 1,
     "updates": {"description": "Modified"}
   }
   ↓
   서버: 버전 검증 (1 == 1) ✅
   ↓
   업데이트 (v1 → v2)
   ↓
   200 OK + WebSocket 브로드캐스트

4. User B: WebSocket 이벤트 수신
   {
     "type": "annotation_updated",
     "annotation_id": 1,
     "new_version": 2,
     "changes": {"description": "Modified"}
   }
   ↓
   UI 자동 업데이트
   ↓
   다음 수정 시 baseVersion: 2 사용
```

### 이점
- ✅ 데이터 무결성 (Version Control)
- ✅ 실시간 동기화 (WebSocket)
- ✅ 최적의 협업 환경
- ✅ 사용자 경험 최고

---

## 📈 개발 순서

### 권장 순서

```
1️⃣ Version Control (2-1) 먼저 ⭐ 필수
   └─ 데이터 무결성 보장
   └─ 다른 기능의 기초

2️⃣ WebSocket (2-3) 나중 ⭐ 권장
   └─ Version Control 위에 구축
   └─ 실시간 동기화 추가
```

### 이유

```
Version Control 없이 WebSocket만 있으면:
├─ 실시간 동기화는 되지만
└─ 동시 편집 시 데이터 손실 가능 ❌

WebSocket 없이 Version Control만 있으면:
├─ 데이터 무결성은 보장되지만
└─ 실시간 동기화 없음 (수동 새로고침 필요)

둘 다 있으면:
├─ 데이터 무결성 ✅
└─ 실시간 동기화 ✅
```

---

## 🎯 구현 전략

### Phase 2-1: Version Control (필수)

**목표:** 데이터 무결성 보장

```
Week 3 (3-4일):
├─ Optimistic Locking 구현
├─ 버전 충돌 감지
└─ 409 Conflict 처리

결과: 동시 편집 시 데이터 손실 방지 ✅
```

### Phase 2-3: WebSocket (권장)

**목표:** 실시간 동기화 추가

```
Week 4-5 (5-7일):
├─ WebSocket 서버 구축
├─ 이벤트 브로드캐스팅
├─ 클라이언트 구독 관리
└─ 재연결 처리

결과: 실시간 동기화 + 협업 지원 ✅
```

### 통합 테스트

```
Week 5 (1-2일):
├─ Version Control + WebSocket 통합 테스트
├─ 동시 편집 시나리오 테스트
└─ 성능 테스트

결과: 완벽한 협업 환경 ✅
```

---

## 📝 결론

### Version Control (2-1)
- **필수:** 데이터 무결성 보장
- **개발 기간:** 3-4일
- **복잡도:** 중간
- **우선순위:** 1순위

### WebSocket (2-3)
- **권장:** 실시간 동기화
- **개발 기간:** 5-7일
- **복잡도:** 높음
- **우선순위:** 2순위

### 통합
- **최고의 협업 환경**
- **데이터 무결성 + 실시간 동기화**
- **총 개발 기간:** 2-3주

