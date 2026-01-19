# 📊 Annotations by TimePoint API (Y축 API)

## 개요

TimePoint에 속한 모든 Annotation을 조회하는 API입니다.

**주요 기능:**
- TimePoint에 할당된 모든 Study의 Annotation 조회
- Lesion 정보 포함 (`lesion_type`, `lesion_number`)
- Lesion 타입 → Lesion 번호 → 생성일시 순으로 정렬
- RECIST Report 작성 시 Y축 데이터로 사용

---

## 엔드포인트

**GET** `/api/timepoints/{timepoint_id}/annotations`

---

## Request

### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `timepoint_id` | integer | Yes | TimePoint ID |

### Headers

| Header | Value | Required |
|--------|-------|----------|
| `Authorization` | `Bearer {token}` | Yes |

### 예시

```bash
curl -X GET "http://localhost:8080/api/timepoints/1/annotations" \
  -H "Authorization: Bearer $TOKEN"
```

---

## Response

### 성공: `200 OK`

```json
{
  "timepoint_id": 1,
  "timepoint_name": "Baseline",
  "annotations": [
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
      "measurement_values": [
        {
          "id": "m1",
          "type": "raw",
          "values": [25.5],
          "unit": "mm"
        }
      ],
      "created_at": "2026-01-01T10:00:00Z",
      "updated_at": "2026-01-01T10:00:00Z",
      "version": 1
    },
    {
      "id": 124,
      "project_id": 1,
      "user_id": 5,
      "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
      "series_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
      "instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.4",
      "tool_name": "ruler",
      "data": {
        "points": [[150, 150], [250, 250]],
        "length": 141.42
      },
      "lesion_type": "TARGET",
      "lesion_number": 2,
      "description": "Lung lesion #2",
      "measurement_values": {
        "diameter": 18.3,
        "unit": "mm"
      },
      "created_at": "2026-01-01T10:05:00Z",
      "updated_at": "2026-01-01T10:05:00Z",
      "version": 1
    },
    {
      "id": 125,
      "project_id": 1,
      "user_id": 5,
      "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
      "series_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
      "instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.5",
      "tool_name": "freehand",
      "data": {
        "points": [[200, 200], [250, 250], [300, 200]]
      },
      "lesion_type": "NON_TARGET",
      "lesion_number": 1,
      "description": "Bone lesion",
      "created_at": "2026-01-01T10:10:00Z",
      "updated_at": "2026-01-01T10:10:00Z",
      "version": 1
    }
  ],
  "total": 3
}
```

### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `timepoint_id` | integer | TimePoint ID |
| `timepoint_name` | string | TimePoint 이름 (예: "Baseline", "TP1") |
| `annotations` | array | Annotation 목록 |
| `annotations[].id` | integer | Annotation ID |
| `annotations[].lesion_type` | string | Lesion 타입 (TARGET, NON_TARGET, TARGET_NEW, NON_TARGET_NEW) |
| `annotations[].lesion_number` | integer | Lesion 번호 (1, 2, 3, ...) |
| `annotations[].description` | string | Annotation 설명 |
| `annotations[].measurement_values` | object | 측정값 (diameter, unit 등) |
| `total` | integer | 총 Annotation 개수 |

### 정렬 순서

Annotations는 다음 순서로 정렬됩니다:
1. `lesion_type` (TARGET → NON_TARGET → TARGET_NEW → NON_TARGET_NEW)
2. `lesion_number` (1 → 2 → 3 → ...)
3. `created_at` (오래된 것 → 최신)

---

## 에러 응답

### TimePoint 없음: `404 Not Found`

```json
{
  "error": "NOT_FOUND",
  "message": "TimePoint not found"
}
```

### 인증 실패: `401 Unauthorized`

```json
{
  "error": "UNAUTHORIZED",
  "message": "Invalid or missing token"
}
```

### 권한 없음: `403 Forbidden`

```json
{
  "error": "FORBIDDEN",
  "message": "You don't have permission to access this TimePoint"
}
```

---

## 사용 예시

### 예시 1: Baseline TimePoint의 모든 Annotation 조회

```bash
curl -X GET "http://localhost:8080/api/timepoints/1/annotations" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

**응답:**
```json
{
  "timepoint_id": 1,
  "timepoint_name": "Baseline",
  "annotations": [
    {
      "id": 123,
      "lesion_type": "TARGET",
      "lesion_number": 1,
      "description": "Liver lesion #1"
    },
    {
      "id": 124,
      "lesion_type": "TARGET",
      "lesion_number": 2,
      "description": "Lung lesion #2"
    }
  ],
  "total": 2
}
```

---

## RECIST Report 작성 워크플로우

### JavaScript 예시

```javascript
// TimePoint의 모든 Annotations 조회
async function getAnnotationsByTimePoint(timepointId) {
  const response = await fetch(
    `/api/timepoints/${timepointId}/annotations`,
    {
      headers: {
        'Authorization': `Bearer ${token}`
      }
    }
  );
  
  const data = await response.json();
  return data;
}

// Lesion별로 그룹화
function groupByLesionType(annotations) {
  return {
    target: annotations.filter(a => a.lesion_type === 'TARGET'),
    nonTarget: annotations.filter(a => a.lesion_type === 'NON_TARGET'),
    targetNew: annotations.filter(a => a.lesion_type === 'TARGET_NEW'),
    nonTargetNew: annotations.filter(a => a.lesion_type === 'NON_TARGET_NEW')
  };
}

// 사용 예시
const data = await getAnnotationsByTimePoint(1);
const grouped = groupByLesionType(data.annotations);

console.log('Target Lesions:', grouped.target);
console.log('Non-Target Lesions:', grouped.nonTarget);
console.log('New Target Lesions:', grouped.targetNew);
console.log('New Non-Target Lesions:', grouped.nonTargetNew);
```

---

## 관련 문서

- [TimePoints with Studies API (X축 API)](./timepoints-with-studies.md)
- [Lesion Assignment API](../annotation/lesion-assignment.md)
- [Subject API](../subject/subject-crud.md)
- [TimePoint CRUD API](./timepoint-crud.md)

