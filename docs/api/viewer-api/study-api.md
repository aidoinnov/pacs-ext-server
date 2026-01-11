좋아. 이건 **개발 요청서로 그대로 넘겨도 되는 수준**으로
**API 스펙 + 설계 의도 + 구현 가이드**까지 포함해서 정리해줄게.

아래 내용은 **“왜 이 API가 필요한지”를 모르는 개발자도 바로 이해**할 수 있게 썼어.

---

# API Spec — Viewer Study Meta Batch API

## 1. API 개요

### API 명

**Viewer Study Meta Batch API**

### 목적

Viewer에서 **이미 선택된 Study들의 메타데이터를 한 번의 호출로 조회**하기 위한 API이다.
PACS 탐색용 API(`/api/v1/me/studies`)와 달리, 본 API는 **Viewer 초기화 및 렌더링을 위한 최소 메타데이터 제공**을 목적으로 한다.

본 API는 QIDO-RS를 직접 호출하지 않고,
**기존 QIDO Proxy(RBAC 적용)** 를 내부적으로 재사용하는 **Backend-for-Frontend(BFF)** 역할을 수행한다.

---

## 2. Endpoint 정의

```http
POST /api/v1/viewer/studies/meta
```

---

## 3. 인증 / 권한

### 인증 방식

* 기존 PACS API와 동일
* JWT (`Authorization: Bearer <token>`) 또는 Session Cookie

### 권한 처리

* Viewer API 자체에서는 별도 RBAC 로직을 두지 않음
* **내부에서 호출되는 QIDO Proxy Service에서 RBAC 적용**
* 접근 권한이 없는 Study는 결과에서 제외하거나 오류 처리

---

## 4. Request Spec

### Headers

```http
Authorization: Bearer <access_token>
Content-Type: application/json
```

### Body

```json
{
  "studyUIDs": [
    "1.2.840.113619.2.55.3.604688433.1234",
    "1.2.840.113619.2.55.3.604688433.5678"
  ]
}
```

### Request 필드 설명

| 필드명       | 타입       | 필수 | 설명                           |
| --------- | -------- | -- | ---------------------------- |
| studyUIDs | string[] | ✅  | 조회할 StudyInstanceUID 목록      |
| maxCount  | number   | ❌  | 최대 조회 개수 (기본값: 20, 서버 제한 적용) |

> ⚠️ URL 길이 제한을 회피하기 위해 **Query Parameter가 아닌 Body 사용**

---

## 5. Response Spec

### 성공 응답 (200 OK)

```json
{
  "studies": [
    {
      "studyUID": "1.2.840.113619.2.55.3.604688433.1234",
      "studyDate": "20240115",
      "studyTime": "093012",
      "studyDescription": "Chest CT",
      "patientName": "DOE^JOHN",
      "patientId": "P123456",
      "modalitiesInStudy": ["CT"],
      "numberOfSeries": 3,
      "numberOfInstances": 245
    },
    {
      "studyUID": "1.2.840.113619.2.55.3.604688433.5678",
      "studyDate": "20240110",
      "studyTime": "141500",
      "studyDescription": "Abdomen CT",
      "patientName": "DOE^JOHN",
      "patientId": "P123456",
      "modalitiesInStudy": ["CT"],
      "numberOfSeries": 2,
      "numberOfInstances": 180
    }
  ]
}
```

---

## 6. Study Meta DTO 정의

### ViewerStudyMeta

| 필드명               | 타입       | 설명                   |
| ----------------- | -------- | -------------------- |
| studyUID          | string   | StudyInstanceUID     |
| studyDate         | string   | StudyDate (YYYYMMDD) |
| studyTime         | string   | StudyTime (HHMMSS)   |
| studyDescription  | string   | StudyDescription     |
| patientName       | string   | PatientName          |
| patientId         | string   | PatientID            |
| modalitiesInStudy | string[] | Modality 목록          |
| numberOfSeries    | number   | Series 개수            |
| numberOfInstances | number   | Instance 개수          |

> ⚠️ Viewer 렌더링에 필요한 **최소 필드만 포함**
> ⚠️ Patient 전체 정보 API로 확장하지 않음

---

## 7. 오류 응답

### 400 Bad Request

```json
{
  "error": "INVALID_REQUEST",
  "message": "studyUIDs is required and must be a non-empty array"
}
```

### 403 Forbidden

```json
{
  "error": "FORBIDDEN",
  "message": "Access denied for one or more studies"
}
```

### 404 Not Found

```json
{
  "error": "NOT_FOUND",
  "message": "No accessible studies found"
}
```

---

## 8. 내부 처리 흐름 (구현 가이드)

### 처리 순서

1. Request Body 검증
2. studyUID 목록 deduplication
3. 캐시 조회 (`study:{studyUID}`)
4. Cache miss 시:

   * 공통 QIDO Proxy Service 호출
   * `GET /dicom-web/studies/{studyUID}`
5. QIDO 응답 → Viewer DTO 변환
6. 결과 캐싱 (TTL 적용)
7. Viewer에 일괄 반환

---

## 9. 캐시 전략

### Cache Key

```
study:{studyUID}
```

### TTL

* 기본: 10 ~ 30분
* Viewer 접근 시 TTL 연장 가능

### 캐시 미스 처리

* 일부 study 실패 시:

  * 실패한 study만 제외하고 반환
  * 전체 실패 시 404 반환

---

## 10. 설계 의도 요약 (개발 요청서용 문장)

> 본 API는 Viewer 초기 로딩 성능 개선을 위해,
> 이미 선택된 StudyInstanceUID 목록에 대한 메타데이터를
> 단일 요청으로 조회하는 Viewer 전용 Batch API이다.
>
> 기존 QIDO Proxy 및 RBAC 로직을 재사용하며,
> Viewer는 QIDO-RS를 직접 호출하지 않는다.

---

## 11. API 사용 예 (Viewer)

```ts
await fetch("/api/v1/viewer/studies/meta", {
  method: "POST",
  headers: {
    "Authorization": `Bearer ${token}`,
    "Content-Type": "application/json"
  },
  body: JSON.stringify({
    studyUIDs: selectedStudyUIDs
  })
})
```

---

## 12. 확장 예정 API (후속)

* `POST /api/v1/viewer/series/meta`
* `POST /api/v1/viewer/instances/meta`
* `GET  /api/v1/viewer/selections/{selectionId}`

---

### 최종 한 줄 요약

> **`/viewer/studies/meta`는
> 탐색 API가 아니라 Viewer 성능을 위한 Batch View API이며,
> 기존 QIDO Proxy 위에 얇게 얹는 BFF 계층이다.**

---

원하면 다음으로:

* `/viewer/series/meta` 스펙
* OpenAPI(Swagger) YAML
* 구현 체크리스트 (PR 기준)
* 성능 테스트 시나리오

중에서 바로 이어서 만들어줄게.
