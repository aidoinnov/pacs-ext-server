# Annotation 목록 조회 API (Lesion 정보 포함)

## 개요

Annotation 목록을 조회할 때 **Lesion 정보**(`lesion_type`, `lesion_number`)가 자동으로 포함됩니다.

---

## API 엔드포인트

### 1. 프로젝트별 Annotation 목록 조회

**GET** `/api/projects/{project_id}/annotations`

**Query Parameters:**
- `lesion_type` (optional): Lesion 타입 필터
  - `TARGET` - Target Lesion
  - `NON_TARGET` - Non-target Lesion
  - `TARGET_NEW` - New Target Lesion
  - `NON_TARGET_NEW` - New Non-target Lesion
- `lesion_number` (optional): Lesion 번호 (1, 2, 3, ...)

**Response:** `200 OK`
```json
{
  "annotations": [
    {
      "id": 123,
      "project_id": 1,
      "user_id": 10,
      "study_uid": "1.2.840.113619...",
      "series_uid": "1.2.840.113619...",
      "instance_uid": "1.2.840.113619...",
      "lesion_type": "TARGET",
      "lesion_number": 1,
      "label": "Liver lesion #1",
      "description": "Target lesion in liver",
      "measurement_values": [
        {
          "id": "m1",
          "type": "diameter",
          "values": [42.3, 18.7],
          "unit": "mm"
        }
      ],
      "created_at": "2026-01-19T12:00:00Z",
      "updated_at": "2026-01-19T12:00:00Z"
    },
    {
      "id": 124,
      "project_id": 1,
      "user_id": 10,
      "study_uid": "1.2.840.113619...",
      "series_uid": "1.2.840.113619...",
      "instance_uid": "1.2.840.113619...",
      "lesion_type": "TARGET",
      "lesion_number": 2,
      "label": "Lung lesion #2",
      "description": "Target lesion in lung",
      "measurement_values": [
        {
          "id": "m1",
          "type": "diameter",
          "values": [35.2, 22.1],
          "unit": "mm"
        }
      ],
      "created_at": "2026-01-19T12:01:00Z",
      "updated_at": "2026-01-19T12:01:00Z"
    }
  ],
  "total": 2
}
```

---

### 2. 사용자별 Annotation 목록 조회

**GET** `/api/annotations?user_id={user_id}`

**Response:** `200 OK`
```json
{
  "annotations": [
    {
      "id": 125,
      "lesion_type": "NON_TARGET",
      "lesion_number": 1,
      "label": "Bone metastasis",
      "description": "Non-target lesion in bone",
      "created_at": "2026-01-19T12:02:00Z"
    }
  ],
  "total": 1
}
```

---

### 3. Study별 Annotation 목록 조회

**GET** `/api/annotations?study_instance_uid={study_uid}`

**Response:** `200 OK`
```json
{
  "annotations": [
    {
      "id": 126,
      "lesion_type": "TARGET_NEW",
      "lesion_number": 3,
      "label": "New liver lesion",
      "description": "New target lesion discovered during follow-up",
      "created_at": "2026-01-19T12:03:00Z"
    }
  ],
  "total": 1
}
```

---

## Lesion 정보 필드

모든 Annotation 목록 조회 API에서 다음 필드가 자동으로 포함됩니다:

| 필드 | 타입 | 설명 | 예시 |
|------|------|------|------|
| `lesion_type` | string | Lesion 타입 (기본값: `"UNSPECIFIED"`) | `"TARGET"`, `"NON_TARGET"`, `"TARGET_NEW"`, `"NON_TARGET_NEW"`, `"UNSPECIFIED"` |
| `lesion_number` | integer (nullable) | Lesion 번호 (Subject 내에서 타입별로 순차 번호) | `1`, `2`, `3`, ..., `null` |

**참고:**
- Lesion 타입이 지정되지 않은 일반 Annotation의 경우 `lesion_type`은 `"UNSPECIFIED"`입니다 (기본값).
- Lesion 번호는 **Subject 내에서 타입별로** 자동 할당됩니다.
  - 예: Subject A의 TARGET Lesion은 1, 2, 3, ...
  - 예: Subject A의 NON_TARGET Lesion은 1, 2, 3, ...

---

## 사용 예시

### Target Lesion만 조회
```bash
curl -X GET "http://localhost:8080/api/projects/1/annotations?lesion_type=TARGET" \
  -H "Authorization: Bearer $TOKEN"
```

### 특정 Lesion 번호 조회
```bash
curl -X GET "http://localhost:8080/api/projects/1/annotations?lesion_type=TARGET&lesion_number=1" \
  -H "Authorization: Bearer $TOKEN"
```

### 모든 Annotation 조회 (Lesion 정보 포함)
```bash
curl -X GET "http://localhost:8080/api/projects/1/annotations" \
  -H "Authorization: Bearer $TOKEN"
```

---

## 관련 API

- [Lesion Assignment API](./lesion-assignment.md) - Lesion 정보 생성/수정
- [TimePoint Annotations API](../timepoint/annotations-by-timepoint.md) - TimePoint별 Annotation 조회

