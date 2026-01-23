# DICOM QIDO-RS API - Redis 캐싱 클라이언트 가이드

## 개요

DICOM QIDO-RS API는 **Redis 캐싱**을 활용하여 Dcm4chee 서버 부하를 줄이고 응답 속도를 개선합니다.
이 문서는 클라이언트 개발자가 캐싱 메커니즘을 이해하고 최적으로 활용하는 방법을 설명합니다.

## 기본 정보

- **Base URL**: `http://localhost:8080/api`
- **Content-Type**: `application/json`
- **캐싱 방식**: Redis (서버 사이드 캐싱)
- **캐시 TTL**: 60초 (기본값, 환경변수로 설정 가능)

---

## 🎯 캐싱 전략 요약

### **핵심 개념**

1. **서버 사이드 캐싱**: Redis에 QIDO-RS 응답 저장
2. **TTL 기반**: 60초 후 자동 만료
3. **투명한 캐싱**: 클라이언트는 캐시 여부를 신경 쓸 필요 없음
4. **프로젝트별 격리**: 각 프로젝트의 데이터는 독립적으로 캐싱

### **장점**

- ✅ **응답 속도 개선**: Cache HIT 시 0.1~0.2초 (MISS 대비 30~50% 개선)
- ✅ **Dcm4chee 부하 절감**: 동일 요청 시 QIDO-RS 호출 생략
- ✅ **동시 요청 처리**: 여러 사용자가 동시에 조회해도 안전
- ✅ **자동 만료**: 60초 후 자동으로 최신 데이터 조회

---

## 📡 캐싱이 적용된 API 엔드포인트

### 1. Series 목록 조회 (사용자 관점)

#### 요청

```http
GET /api/me/dicom/studies/{study_uid}/series?project_id={project_id}
Authorization: Bearer {token}
```

#### 응답

**Cache MISS (첫 요청)**:
```json
[
  {
    "0020000E": {"Value": ["1.2.840.113619.2.55.3.2831164527.123.1234567890.1"]},
    "00080060": {"Value": ["CT"]},
    "0020000D": {"Value": ["1.2.410.200022.500.202205101053010.12252192375"]},
    ...
  }
]
```

**Cache HIT (60초 이내 재요청)**:
- 동일한 JSON 응답
- 응답 시간: ~0.1~0.15초 (MISS 대비 30~50% 빠름)

---

### 2. Series 목록 조회 (일반)

#### 요청

```http
GET /api/dicom/studies/{study_uid}/series?project_id={project_id}
Authorization: Bearer {token}
```

#### 동작

- Series 엔드포인트와 동일한 캐싱 메커니즘
- 프로젝트별 독립적인 캐시

---

### 3. Studies 목록 조회

#### 요청

```http
GET /api/me/dicom/studies?project_id={project_id}
Authorization: Bearer {token}
```

#### 응답

**Cache MISS (첫 요청)**:
```json
[
  {
    "0020000D": {"Value": ["1.2.410.200022.500.202205101053010.12252192375"]},
    "00100010": {"Value": [{"Alphabetic": "홍길동"}]},
    "00080020": {"Value": ["20220510"]},
    ...
  }
]
```

**Cache HIT (60초 이내 재요청)**:
- 동일한 JSON 응답
- 응답 시간: ~0.2~0.3초 (MISS 대비 15~30% 빠름)

---

## 🚀 클라이언트 구현 가이드

### **시나리오 1: 일반적인 조회**

캐싱은 서버에서 자동으로 처리되므로 특별한 처리 불필요.

```javascript
// Series 조회
const response = await fetch(
  `/api/me/dicom/studies/${studyUid}/series?project_id=${projectId}`,
  {
    headers: {
      'Authorization': `Bearer ${token}`
    }
  }
);

const series = await response.json();
console.log('Series count:', series.length);

// 60초 이내 재요청 시 자동으로 캐시 사용
const response2 = await fetch(
  `/api/me/dicom/studies/${studyUid}/series?project_id=${projectId}`,
  {
    headers: {
      'Authorization': `Bearer ${token}`
    }
  }
);
// ✅ 더 빠른 응답 (Cache HIT)
```

---

### **시나리오 2: Studies 목록 조회**

```javascript
// Studies 조회
const response = await fetch(
  `/api/me/dicom/studies?project_id=${projectId}`,
  {
    headers: {
      'Authorization': `Bearer ${token}`
    }
  }
);

const studies = await response.json();
console.log('Studies count:', studies.length);
```

---

### **시나리오 3: 프로젝트별 독립적인 캐시**

```javascript
// 프로젝트 2의 Series 조회 (Cache MISS)
const series_p2 = await fetch(
  `/api/me/dicom/studies/${studyUid}/series?project_id=2`,
  { headers: { 'Authorization': `Bearer ${token}` } }
);

// 프로젝트 3의 Series 조회 (Cache MISS - 다른 캐시 키)
const series_p3 = await fetch(
  `/api/me/dicom/studies/${studyUid}/series?project_id=3`,
  { headers: { 'Authorization': `Bearer ${token}` } }
);

// 프로젝트 2 재조회 (Cache HIT)
const series_p2_again = await fetch(
  `/api/me/dicom/studies/${studyUid}/series?project_id=2`,
  { headers: { 'Authorization': `Bearer ${token}` } }
);
```

---

### **시나리오 4: 쿼리 파라미터별 독립적인 캐시**

```javascript
// 기본 조회 (Cache MISS)
const series1 = await fetch(
  `/api/me/dicom/studies/${studyUid}/series?project_id=2`
);

// 다른 파라미터 (Cache MISS - 다른 캐시 키)
const series2 = await fetch(
  `/api/me/dicom/studies/${studyUid}/series?project_id=2&includefield=all`
);

// 기본 조회 재요청 (Cache HIT)
const series3 = await fetch(
  `/api/me/dicom/studies/${studyUid}/series?project_id=2`
);
```

---

## 🔍 캐시 동작 이해하기

### **캐시 키 생성 규칙**

#### Series 엔드포인트
```
qido:series:{study_uid}:p{project_id}:h{params_hash}
```

**예시**:
```
qido:series:1.2.410.200022.500.202205101053010.12252192375:p2:h3a4b5c6d
```

#### Studies 엔드포인트
```
qido:studies:p{project_id}:h{params_hash}
```

**예시**:
```
qido:studies:p2:h7e8f9g0h
```

### **캐시 만료**

- **TTL**: 60초 (환경변수 `QIDO_CACHE_TTL_SEC`로 설정 가능)
- **자동 만료**: 60초 후 자동으로 삭제
- **재조회**: 만료 후 첫 요청은 Cache MISS → 새로운 캐시 생성

---

## ⚠️ 주의사항

### 1. **캐시는 투명하게 동작**

클라이언트는 캐시 여부를 알 필요 없음. 서버가 자동으로 처리.

```javascript
// ✅ 올바른 사용
const response = await fetch(url);
const data = await response.json();

// ❌ 불필요한 처리
// 캐시 여부를 체크하거나 특별한 헤더를 추가할 필요 없음
```

### 2. **60초 이내 데이터는 캐시됨**

최신 데이터가 필요한 경우 60초 대기 또는 서버 재시작 필요.

```javascript
// 데이터 업로드 후 즉시 조회
await uploadDicomData(studyUid);

// ⚠️ 60초 이내면 이전 캐시 반환 가능
const series = await fetch(`/api/me/dicom/studies/${studyUid}/series`);
```

### 3. **프로젝트별 독립적인 캐시**

같은 Study라도 프로젝트가 다르면 다른 캐시 사용.

```javascript
// 프로젝트 2의 캐시
const series_p2 = await fetch(url + '?project_id=2');

// 프로젝트 3의 캐시 (독립적)
const series_p3 = await fetch(url + '?project_id=3');
```

---

## 📊 성능 비교

### **Series 엔드포인트**

| 상황 | 응답 시간 | 개선율 |
|------|----------|--------|
| Cache MISS | 0.22초 | - |
| Cache HIT | 0.14초 | ⚡ **36% 개선** |

### **Studies 엔드포인트**

| 상황 | 응답 시간 | 개선율 |
|------|----------|--------|
| Cache MISS | 0.29초 | - |
| Cache HIT | 0.24초 | ⚡ **17% 개선** |

### **동시 요청 (10개)**

| 메트릭 | 값 |
|--------|-----|
| 총 소요 시간 | ~1.0초 |
| 평균 응답 시간 | ~0.10초 |

---

## 🧪 캐시 동작 확인 방법

### **서버 로그 확인**

```bash
tail -f backend.log | grep -E "(⚡|🔄)"
```

**출력 예시**:
```
[INFO] 🔄 Cache MISS - study=1.2.410..., project=2
[INFO] ⚡ Cache HIT - study=1.2.410..., project=2
[INFO] 🔄 Cache MISS - studies for project=2
[INFO] ⚡ Cache HIT - studies for project=2
```

- **🔄 Cache MISS**: 첫 요청 또는 캐시 만료 후 요청
- **⚡ Cache HIT**: 캐시에서 데이터 반환

---

## 🎯 요약

| 상황 | 동작 | 응답 시간 |
|------|------|----------|
| 첫 요청 | Cache MISS → QIDO 호출 | 0.2~0.3초 |
| 60초 이내 재요청 | Cache HIT → Redis 조회 | 0.1~0.2초 |
| 60초 후 재요청 | Cache MISS → QIDO 호출 | 0.2~0.3초 |
| 다른 프로젝트 | Cache MISS → 독립적인 캐시 | 0.2~0.3초 |

**핵심**: 
- ✅ 클라이언트는 특별한 처리 불필요
- ✅ 서버가 자동으로 캐싱 처리
- ✅ 60초 이내 동일 요청은 자동으로 빠른 응답


