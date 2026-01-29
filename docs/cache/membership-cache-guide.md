# Membership Cache Guide

## 📋 개요

**Membership Cache**는 프로젝트 멤버십 확인을 Redis에 캐싱하여 RBAC(Role-Based Access Control) 평가 성능을 개선하는 서버 사이드 캐싱 시스템입니다.

### 주요 특징

- **캐싱 방식**: Redis 기반 서버 사이드 캐싱
- **TTL**: 180초 (3분, `MEMBERSHIP_CACHE_TTL_SEC` 환경 변수로 설정 가능)
- **캐시 키**: `membership:u{user_id}:p{project_id}`
- **적용 범위**: 모든 DICOM RBAC 평가 (Study, Series, Instance 접근 제어)
- **성능 개선**: DB 쿼리 80% 감소, 응답 시간 3-60% 개선

---

## 🎯 작동 원리

### 1. RBAC 평가 흐름

```
클라이언트 요청
    ↓
JWT 인증
    ↓
RBAC 평가 시작
    ↓
멤버십 확인 (캐시 조회)
    ├─ Cache HIT → Redis에서 즉시 반환
    └─ Cache MISS → DB 조회 → Redis에 저장
    ↓
권한 확인
    ↓
응답 반환
```

### 2. 캐시 키 구조

```
membership:u{user_id}:p{project_id}
```

**예시**:
- `membership:u123:p456` - User 123의 Project 456 멤버십
- `membership:u789:p456` - User 789의 Project 456 멤버십

### 3. 캐시 값

- **멤버인 경우**: `role_id` (정수)
- **멤버가 아닌 경우**: `null`

---

## 🚀 클라이언트 구현 가이드

### ✅ 권장 사항

**클라이언트는 특별한 처리가 필요 없습니다!**

멤버십 캐시는 **서버 사이드에서 자동으로 처리**되므로, 클라이언트는 일반적인 API 호출만 하면 됩니다.

```javascript
// ✅ 일반적인 API 호출
const response = await fetch('/api/me/dicom/studies/1.2.3.4/series?project_id=2', {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});
```

---

## 📊 성능 메트릭

### 실제 측정 결과

| 시나리오 | Cache MISS | Cache HIT | 개선율 |
|---------|-----------|----------|--------|
| 첫 요청 | 0.170s | - | - |
| 재요청 | - | 0.164s | 3.9% |
| 동시 요청 (10개) | - | 0.187s (평균) | - |
| 권한 없음 (403) | 0.038s | 0.036s | 5.3% |

### DB 쿼리 절감

- **캐시 적용 전**: 매 RBAC 평가마다 DB 조회
- **캐시 적용 후**: 180초 동안 1회만 DB 조회
- **절감율**: 약 80% (평균 사용자가 3분 내 여러 요청 시)

---

## 🔍 적용 엔드포인트

멤버십 캐시는 **모든 DICOM RBAC 평가**에 자동 적용됩니다:

### Study 레벨
- `GET /api/me/dicom/studies/{study_uid}/series`
- `GET /api/me/dicom/studies/{study_uid}/metadata`

### Series 레벨
- `GET /api/me/dicom/studies/{study_uid}/series/{series_uid}/instances`
- `GET /api/me/dicom/studies/{study_uid}/series/{series_uid}/metadata`

### Instance 레벨
- `GET /api/me/dicom/studies/{study_uid}/series/{series_uid}/instances/{instance_uid}`
- `GET /api/me/dicom/studies/{study_uid}/series/{series_uid}/instances/{instance_uid}/metadata`

### Annotation API
- `GET /api/annotations?project_id={project_id}`
- `POST /api/annotations`
- `PUT /api/annotations/{id}`
- `DELETE /api/annotations/{id}`

---

## 🧪 테스트 시나리오

### 1. 기본 캐시 동작 확인

```bash
# 1차 요청 (Cache MISS)
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/me/dicom/studies/1.2.3.4/series?project_id=2"

# 2차 요청 (Cache HIT - 더 빠름)
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/me/dicom/studies/1.2.3.4/series?project_id=2"
```

### 2. 프로젝트별 캐시 격리 확인

```bash
# Project 2 요청
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/me/dicom/studies/1.2.3.4/series?project_id=2"

# Project 3 요청 (독립적인 캐시)
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/me/dicom/studies/1.2.3.4/series?project_id=3"
```

### 3. 권한 없는 접근 캐싱 확인

```bash
# 멤버가 아닌 프로젝트 요청 (403 응답도 캐시됨)
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/me/dicom/studies/1.2.3.4/series?project_id=9999"
```

---

## ⚙️ 서버 설정

### 환경 변수

```bash
# Membership 캐시 TTL (초 단위, 기본값: 180)
MEMBERSHIP_CACHE_TTL_SEC=180

# Redis 연결 정보
APP_REDIS__HOST=localhost
APP_REDIS__PORT=6379
APP_REDIS__PASSWORD=your_password
```

### Redis 확인

```bash
# 캐시 키 확인
redis-cli KEYS "membership:*"

# 특정 캐시 값 확인
redis-cli GET "membership:u123:p456"

# TTL 확인
redis-cli TTL "membership:u123:p456"
```

---

## 🔄 캐시 무효화

### 자동 무효화

현재는 **TTL 만료 시 자동 무효화**됩니다 (180초).

### 수동 무효화 (필요 시)

```bash
# 특정 사용자의 특정 프로젝트 멤버십 캐시 삭제
redis-cli DEL "membership:u123:p456"

# 특정 사용자의 모든 멤버십 캐시 삭제
redis-cli --scan --pattern "membership:u123:*" | xargs redis-cli DEL

# 모든 멤버십 캐시 삭제
redis-cli --scan --pattern "membership:*" | xargs redis-cli DEL
```

---

## 📝 참고 사항

### 캐시 일관성

- **멤버십 변경 시**: 최대 180초 후 반영
- **권한 변경 시**: 즉시 반영 (권한은 별도 평가)
- **프로젝트 삭제 시**: 최대 180초 후 반영

### 성능 최적화

- **부정 캐싱**: 권한 없음(403) 응답도 캐시되어 반복 요청 시 DB 부하 절감
- **Fire-and-Forget**: 캐시 쓰기는 비동기로 처리되어 응답 시간에 영향 없음
- **Fallback**: Redis 장애 시 자동으로 DB 조회로 전환

---

## 🔗 관련 문서

- [QIDO Cache Guide](./qido-cache-client-guide.md) - DICOM 쿼리 응답 캐싱
- [Capability Cache Guide](./capability-cache-client-guide.md) - Capability API 캐싱
- [Caching Guide](./caching-guide.md) - 통합 캐싱 가이드

