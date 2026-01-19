# 📋 Annotation Lesion Assignment API

## 개요

RECIST 1.1 기준에 따라 Annotation에 Lesion 정보를 할당하는 API입니다.

**방안 2 (하이브리드 접근법):**
- 사용자는 간단한 데이터만 입력 (`lesion_type` + `lesion_number`)
- 서버가 자동으로 복잡한 추적 관리 (Subject별 Lesion 번호 관리)

---

## Lesion 타입

### TARGET
측정 가능한 병변 (최대 5개/Subject)

### NON_TARGET
측정 불가능한 병변

### TARGET_NEW
Follow-up에서 발견된 새로운 측정 가능 병변

### NON_TARGET_NEW
Follow-up에서 발견된 새로운 측정 불가능 병변

### UNSPECIFIED
Lesion 타입이 지정되지 않은 일반 Annotation (기본값)

---

## API 엔드포인트

### 1. Annotation 생성 시 Lesion 할당

**POST** `/api/projects/{project_id}/annotations`

**Request Body:**
```json
{
  "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
  "series_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
  "instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
  "tool_name": "ruler",
  "data": {
    "points": [[100, 100], [200, 200]],
    "length": 141.42
  },
  "lesion_type": "TARGET",
  "lesion_number": 1,
  "description": "Liver lesion #1"
}
```

**Response:** `201 Created`
```json
{
  "id": 123,
  "project_id": 1,
  "user_id": 5,
  "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
  "series_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
  "instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
  "tool_name": "ruler",
  "data": {
    "points": [[100, 100], [200, 200]],
    "length": 141.42
  },
  "lesion_type": "TARGET",
  "lesion_number": 1,
  "description": "Liver lesion #1",
  "created_at": "2026-01-19T12:00:00Z",
  "updated_at": "2026-01-19T12:00:00Z",
  "version": 1
}
```

---

### 2. Annotation 업데이트로 Lesion 변경

**PUT** `/api/annotations/{id}`

**Request Body:**
```json
{
  "lesion_type": "NON_TARGET",
  "lesion_number": 2,
  "description": "Changed to non-target lesion #2",
  "version": 1
}
```

**Response:** `200 OK`
```json
{
  "id": 123,
  "lesion_type": "NON_TARGET",
  "lesion_number": 2,
  "description": "Changed to non-target lesion #2",
  "version": 2,
  "updated_at": "2026-01-19T12:05:00Z"
}
```

---

### 3. Lesion 할당 제거

**PUT** `/api/annotations/{id}`

**Request Body:**
```json
{
  "lesion_type": null,
  "lesion_number": null,
  "version": 2
}
```

**Response:** `200 OK`
```json
{
  "id": 123,
  "lesion_type": null,
  "lesion_number": null,
  "version": 3,
  "updated_at": "2026-01-19T12:10:00Z"
}
```

---

### 4. Lesion 타입별 Annotation 조회

**GET** `/api/projects/{project_id}/annotations?lesion_type=TARGET`

**Query Parameters:**
- `lesion_type` (optional): `TARGET`, `NON_TARGET`, `TARGET_NEW`, `NON_TARGET_NEW`
- `lesion_number` (optional): Lesion 번호 (1, 2, 3, ...)

**Response:** `200 OK`
```json
{
  "annotations": [
    {
      "id": 123,
      "lesion_type": "TARGET",
      "lesion_number": 1,
      "description": "Liver lesion #1",
      "study_uid": "1.2.840...",
      "created_at": "2026-01-19T12:00:00Z"
    },
    {
      "id": 124,
      "lesion_type": "TARGET",
      "lesion_number": 2,
      "description": "Lung lesion #2",
      "study_uid": "1.2.840...",
      "created_at": "2026-01-19T12:01:00Z"
    }
  ],
  "total": 2
}
```

---

## 사용 예시

### 예시 1: Baseline에서 Target Lesion 3개 생성

```bash
# Target Lesion #1 (Liver)
curl -X POST http://localhost:8080/api/projects/1/annotations \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "1.2.840...",
    "series_uid": "1.2.840...",
    "instance_uid": "1.2.840...",
    "tool_name": "ruler",
    "data": {"length": 25.5},
    "lesion_type": "TARGET",
    "lesion_number": 1,
    "description": "Liver lesion"
  }'

# Target Lesion #2 (Lung)
curl -X POST http://localhost:8080/api/projects/1/annotations \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "1.2.840...",
    "series_uid": "1.2.840...",
    "instance_uid": "1.2.840...",
    "tool_name": "ruler",
    "data": {"length": 18.3},
    "lesion_type": "TARGET",
    "lesion_number": 2,
    "description": "Lung lesion"
  }'
```

### 예시 2: Follow-up에서 New Lesion 발견

```bash
curl -X POST http://localhost:8080/api/projects/1/annotations \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "1.2.840...",
    "series_uid": "1.2.840...",
    "instance_uid": "1.2.840...",
    "tool_name": "ruler",
    "data": {"length": 12.7},
    "lesion_type": "TARGET_NEW",
    "lesion_number": 1,
    "description": "New liver lesion"
  }'
```

---

## 관련 문서

- [TimePoint API](../timepoint/timepoints-with-studies.md)
- [Annotation API](./annotation-crud.md)
- [RECIST 1.1 가이드](../../guides/recist-1.1.md)

