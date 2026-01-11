# API Spec — Viewer Series Meta Batch API

## 1. API 개요

### API 명

**Viewer Series Meta Batch API**

### 목적

Viewer에서 **이미 선택된 Series들의 메타데이터를 한 번의 호출로 조회**하기 위한 API이다.
Study Meta API와 유사하게, **Viewer 초기화 및 렌더링을 위한 최소 메타데이터 제공**을 목적으로 한다.

본 API는 QIDO-RS를 직접 호출하지 않고,
**기존 QIDO Proxy(RBAC 적용)** 를 내부적으로 재사용하는 **Backend-for-Frontend(BFF)** 역할을 수행한다.

---

## 2. Endpoint 정의

```http
POST /api/v1/viewer/series/meta
```

---

## 3. 인증 / 권한

### 인증 방식

* 기존 PACS API와 동일
* JWT (`Authorization: Bearer <token>`) 또는 Session Cookie

### 권한 처리

* Viewer API 자체에서는 별도 RBAC 로직을 두지 않음
* **내부에서 호출되는 QIDO Proxy Service에서 RBAC 적용**
* 접근 권한이 없는 Series는 결과에서 제외하거나 오류 처리

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
  "seriesUIDs": [
    "1.2.840.113619.2.55.3.604688433.1234.1",
    "1.2.840.113619.2.55.3.604688433.1234.2"
  ]
}
```

### Request 필드 설명

| 필드명        | 타입       | 필수 | 설명                           |
| ---------- | -------- | -- | ---------------------------- |
| seriesUIDs | string[] | ✅  | 조회할 SeriesInstanceUID 목록     |
| maxCount   | number   | ❌  | 최대 조회 개수 (기본값: 50, 서버 제한 적용) |

> ⚠️ URL 길이 제한을 회피하기 위해 **Query Parameter가 아닌 Body 사용**

---

## 5. Response Spec

### 성공 응답 (200 OK)

```json
{
  "series": [
    {
      "seriesUID": "1.2.840.113619.2.55.3.604688433.1234.1",
      "studyUID": "1.2.840.113619.2.55.3.604688433.1234",
      "seriesNumber": 1,
      "seriesDescription": "Axial T1",
      "modality": "MR",
      "numberOfInstances": 120,
      "seriesDate": "20240115",
      "seriesTime": "093012",
      "bodyPartExamined": "BRAIN",
      "protocolName": "T1_MPRAGE"
    },
    {
      "seriesUID": "1.2.840.113619.2.55.3.604688433.1234.2",
      "studyUID": "1.2.840.113619.2.55.3.604688433.1234",
      "seriesNumber": 2,
      "seriesDescription": "Axial T2",
      "modality": "MR",
      "numberOfInstances": 100,
      "seriesDate": "20240115",
      "seriesTime": "094500",
      "bodyPartExamined": "BRAIN",
      "protocolName": "T2_TSE"
    }
  ]
}
```

---

## 6. Series Meta DTO 정의

### ViewerSeriesMeta

| 필드명                | 타입     | 설명                      |
| ------------------ | ------ | ----------------------- |
| seriesUID          | string | SeriesInstanceUID       |
| studyUID           | string | StudyInstanceUID (부모)   |
| seriesNumber       | number | SeriesNumber            |
| seriesDescription  | string | SeriesDescription       |
| modality           | string | Modality                |
| numberOfInstances  | number | Instance 개수             |
| seriesDate         | string | SeriesDate (YYYYMMDD)   |
| seriesTime         | string | SeriesTime (HHMMSS)     |
| bodyPartExamined   | string | BodyPartExamined        |
| protocolName       | string | ProtocolName            |

> ⚠️ Viewer 렌더링에 필요한 **최소 필드만 포함**

---

## 7. 오류 응답

### 400 Bad Request

```json
{
  "error": "INVALID_REQUEST",
  "message": "seriesUIDs is required and must be a non-empty array"
}
```

### 403 Forbidden

```json
{
  "error": "FORBIDDEN",
  "message": "Access denied for one or more series"
}
```

### 404 Not Found

```json
{
  "error": "NOT_FOUND",
  "message": "No accessible series found"
}
```

---

## 8. 내부 처리 흐름 (구현 가이드)

### 처리 순서

1. Request Body 검증
2. seriesUID 목록 deduplication
3. 각 Series UID에 대해:
   * QIDO-RS로 Series 메타데이터 조회
   * RBAC 권한 검증 (사용자가 속한 프로젝트 확인)
   * project_data_access 테이블 확인
4. QIDO 응답 → Viewer DTO 변환
5. 접근 가능한 Series만 결과에 포함
6. Viewer에 일괄 반환

---

## 9. Study Meta API와의 차이점

| 항목       | Study Meta API          | Series Meta API           |
| -------- | ----------------------- | ------------------------- |
| 엔드포인트    | `/viewer/studies/meta`  | `/viewer/series/meta`     |
| UID 타입   | StudyInstanceUID        | SeriesInstanceUID         |
| 기본 개수 제한 | 20                      | 50                        |
| 주요 용도    | Study 목록 초기화           | Series 상세 정보 조회           |
| 부모 관계    | -                       | studyUID 포함               |

---

## 10. API 사용 예 (Viewer)

```ts
await fetch("/api/v1/viewer/series/meta", {
  method: "POST",
  headers: {
    "Authorization": `Bearer ${token}`,
    "Content-Type": "application/json"
  },
  body: JSON.stringify({
    seriesUIDs: selectedSeriesUIDs
  })
})
```

---

## 11. 확장 예정

* Instance Meta API (`POST /api/v1/viewer/instances/meta`)
* Series Thumbnail API (`POST /api/v1/viewer/series/thumbnails`)

---

### 최종 한 줄 요약

> **`/viewer/series/meta`는
> Viewer에서 선택된 Series들의 메타데이터를 Batch로 조회하는 BFF API이며,
> RBAC 기반 접근 제어를 통해 권한이 있는 Series만 반환한다.**

