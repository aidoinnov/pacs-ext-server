# ⚡ Phase 2 빠른 가이드

**목적:** Phase 2의 4가지 기능을 빠르게 이해  
**대상:** 모든 팀원  
**읽는 시간:** 5-10분

---

## 🎯 Phase 2 한눈에 보기

```
Phase 2 (2-3주)
│
├─ 2-1: Version Control (3-4일) ⭐ 필수
│   └─ 문제: 동시 편집 시 데이터 손실
│   └─ 해결: Optimistic Locking
│   └─ 결과: 데이터 무결성 보장 ✅
│
├─ 2-2: HEAD 요청 (1-2일) ⭐ 권장
│   └─ 문제: 대역폭 낭비
│   └─ 해결: 헤더만 반환
│   └─ 결과: 성능 최적화 ✅
│
├─ 2-3: WebSocket (5-7일) ⭐ 권장
│   └─ 문제: 실시간 동기화 없음
│   └─ 해결: 이벤트 브로드캐스팅
│   └─ 결과: 실시간 동기화 ✅
│
└─ 2-4: Collaborative Lock (3-4일) ⭐ 권장
    └─ 문제: 동시 수정 방지 안 됨
    └─ 해결: Lock 메커니즘
    └─ 결과: 협업 지원 ✅
```

---

## 📊 기능별 비교표

| 기능 | 문제 | 해결책 | 개발 기간 | 필수 |
|------|------|--------|---------|------|
| **2-1** | 동시 편집 시 덮어쓰기 | Optimistic Locking | 3-4일 | ✅ |
| **2-2** | 대역폭 낭비 | HEAD 요청 | 1-2일 | 📅 |
| **2-3** | 실시간 동기화 없음 | WebSocket | 5-7일 | 📅 |
| **2-4** | 동시 수정 방지 안 됨 | Lock 메커니즘 | 3-4일 | 📅 |

---

## 🔍 각 기능 상세

### 2-1️⃣ Version Control (필수)

**문제:**
```
User A: 수정 (v1 → v2)
User B: 수정 (v1 → v2)  ← User A 덮어씀!
결과: 데이터 손실 ❌
```

**해결:**
```
PUT /api/annotations/1
{
  "baseVersion": 1,  ← 클라이언트 버전
  "updates": {...}
}

서버: 버전 확인
├─ 1 == 1? YES → 업데이트 ✅
└─ 1 != 2? NO → 409 Conflict ❌
```

**개발 항목:**
- [ ] `version` 필드 추가
- [ ] `baseVersion` 검증
- [ ] 409 Conflict 응답
- [ ] 클라이언트 재시도 로직

**개발 기간:** 3-4일

---

### 2-2️⃣ HEAD 요청 (권장)

**문제:**
```
GET /api/annotations?series_instance_uid=...
→ 전체 데이터 반환 (1-10MB) 낭비!
```

**해결:**
```
HEAD /api/annotations?series_instance_uid=...
→ 헤더만 반환 (1KB)
  - Last-Modified: 2025-11-07T10:22:00Z
  - Annotation-Version: 13
  - Content-Length: 0
```

**개발 항목:**
- [ ] HEAD 핸들러 추가
- [ ] 응답 헤더 최적화
- [ ] 캐시 검증 로직

**개발 기간:** 1-2일

---

### 2-3️⃣ WebSocket (권장)

**문제:**
```
User A: 수정
User B: 여전히 이전 데이터 표시
→ 수동 새로고침 필요 ❌
```

**해결:**
```
User A: 수정
├─ PUT /api/annotations/1
└─ WebSocket: annotation_updated 브로드캐스트

User B: 자동 업데이트 ✅
├─ WebSocket: annotation_updated 수신
└─ UI 자동 업데이트
```

**개발 항목:**
- [ ] WebSocket 서버 구축
- [ ] 이벤트 브로드캐스팅
- [ ] 클라이언트 구독 관리
- [ ] 재연결 처리

**개발 기간:** 5-7일

---

### 2-4️⃣ Collaborative Lock (권장)

**문제:**
```
User A: 편집 중
User B: 동시에 편집 시작
→ 충돌 가능성 ❌
```

**해결:**
```
User A: 편집 시작
├─ Lock 획득
└─ 다른 사용자는 읽기만 가능

User B: 편집 시도
└─ "User A가 편집 중" 메시지

User A: 편집 완료
└─ Lock 해제

User B: 편집 가능
```

**개발 항목:**
- [ ] Lock 테이블 생성
- [ ] Lock 획득/해제 로직
- [ ] 타임아웃 처리 (5분)
- [ ] 편집자 표시 (Presence)

**개발 기간:** 3-4일

---

## 🔄 Version Control vs WebSocket

### Version Control (2-1)
```
목적: 데이터 무결성
방식: Optimistic Locking
응답: HTTP (즉시)
필수: ✅ YES
```

### WebSocket (2-3)
```
목적: 실시간 동기화
방식: 이벤트 브로드캐스팅
응답: WebSocket (실시간)
필수: 📅 NO (권장)
```

### 통합 효과
```
Version Control + WebSocket
= 데이터 무결성 + 실시간 동기화
= 최고의 협업 환경 ✅
```

---

## 📈 개발 타임라인

### Week 3 (3-4일)
```
2-1: Version Control
├─ Optimistic Locking (1-2일)
├─ 버전 충돌 감지 (1-2일)
└─ 409 Conflict 처리 (1일)
```

### Week 3-4 (1-2일)
```
2-2: HEAD 요청
├─ HEAD 핸들러 (0.5일)
├─ 응답 헤더 최적화 (0.5일)
└─ 캐시 검증 (1일)
```

### Week 4-5 (5-7일)
```
2-3: WebSocket
├─ WebSocket 서버 (2-3일)
├─ 이벤트 브로드캐스팅 (2-3일)
├─ 클라이언트 구독 (1-2일)
└─ 재연결 처리 (1-2일)
```

### Week 5 (3-4일)
```
2-4: Collaborative Lock
├─ Lock 테이블 (0.5일)
├─ Lock 획득/해제 (1-2일)
├─ 타임아웃 처리 (1일)
└─ Presence (1일)
```

---

## 🎯 우선순위

### 1순위 (필수)
```
2-1: Version Control
└─ 데이터 무결성 보장
└─ 다른 기능의 기초
```

### 2순위 (권장)
```
2-2: HEAD 요청 (독립적)
2-3: WebSocket (2-1 완료 후)
2-4: Collaborative Lock (2-1 완료 후)
```

### 병렬 개발 가능
```
Week 3: 2-1 (필수)
Week 4: 2-2 (병렬) + 2-3 시작
Week 5: 2-3 계속 + 2-4 (병렬)
```

---

## 💡 핵심 포인트

### Version Control (2-1)
- ✅ 필수 기능
- ✅ 데이터 무결성 보장
- ✅ 간단한 구현
- ❌ 실시간 동기화 없음

### WebSocket (2-3)
- 📅 권장 기능
- ✅ 실시간 동기화
- ✅ 협업 지원
- ❌ 복잡한 구현

### 통합
- ✅ 최고의 협업 환경
- ✅ 데이터 무결성 + 실시간 동기화
- ✅ 사용자 경험 최고

---

## 📚 상세 문서

더 자세한 정보는 다음 문서를 참고하세요:

1. **PHASE-2-DETAILED-BREAKDOWN.md**
   - 각 기능의 상세 구현 계획
   - 코드 예제
   - 예상 시간

2. **WEBSOCKET-VS-VERSION-CONTROL.md**
   - Version Control vs WebSocket 비교
   - 통합 흐름
   - 개발 전략

3. **PHASE-BREAKDOWN.md**
   - Phase 2 전체 개요
   - 데이터 모델 변경
   - API 변경사항

---

## 🚀 다음 단계

1. **이 문서 읽기** (5-10분)
2. **상세 문서 읽기** (30-60분)
3. **팀 회의에서 공유**
4. **개발 계획 수립**
5. **개발 시작** (Week 3)

---

**Happy Coding! 🎉**

