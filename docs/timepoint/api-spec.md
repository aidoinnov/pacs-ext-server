# Subject & TimePoint API Specification

## Study 할당 API

### 1. Unassigned Studies 조회
```http
GET /api/subjects/{subject_id}/studies/unassigned
Authorization: Bearer {token}
```

**Response 200 OK:**
```json
{
  "studies": [
    {
      "study_id": 101,
      "study_uid": "1.2.840.113619.2.55.3.1234567890",
      "study_description": "CT Chest",
      "study_date": "2026-01-15",
      "patient_id": "P12345",
      "modality": "CT"
    }
  ]
}
```

### 2. TimePoint별 Studies 조회
```http
GET /api/timepoints/{timepoint_id}/studies
Authorization: Bearer {token}
```

**Response 200 OK:**
```json
{
  "timepoint": {
    "id": 1,
    "name": "Baseline",
    "visit_type": "Baseline"
  },
  "studies": [
    {
      "study_id": 100,
      "study_uid": "1.2.840.113619.2.55.3.0987654321",
      "study_description": "CT Abdomen",
      "study_date": "2026-01-10",
      "assigned_at": "2026-01-18T10:30:00Z",
      "assigned_by": 5
    }
  ]
}
```

### 3. Study를 TimePoint에 할당
```http
POST /api/timepoints/{timepoint_id}/studies
Authorization: Bearer {token}
Content-Type: application/json

{
  "study_ids": [101, 102, 103]
}
```

**Response 200 OK:**
```json
{
  "assigned_count": 3,
  "timepoint_id": 1,
  "study_ids": [101, 102, 103]
}
```

**Note:** 이미 다른 TimePoint에 할당된 Study는 자동으로 재할당됨 (MOVE 동작)

### 4. Study를 Unassigned로 이동
```http
DELETE /api/timepoints/{timepoint_id}/studies
Authorization: Bearer {token}
Content-Type: application/json

{
  "study_ids": [101, 102]
}
```

**Response 200 OK:**
```json
{
  "removed_count": 2,
  "study_ids": [101, 102]
}
```

---

## 보드 뷰 API (통합 조회)

### Subject의 전체 보드 데이터 조회
```http
GET /api/subjects/{subject_id}/board
Authorization: Bearer {token}
```

**Response 200 OK:**
```json
{
  "subject": {
    "id": 1,
    "subject_code": "A001",
    "patient_name": "홍길동"
  },
  "unassigned": {
    "count": 2,
    "studies": [
      {
        "study_id": 103,
        "study_uid": "1.2.840...",
        "study_description": "MRI Brain",
        "study_date": "2026-01-16"
      }
    ]
  },
  "timepoints": [
    {
      "id": 1,
      "name": "Baseline",
      "visit_type": "Baseline",
      "order_index": 0,
      "studies": [
        {
          "study_id": 100,
          "study_uid": "1.2.840...",
          "study_description": "CT Chest",
          "study_date": "2026-01-10"
        }
      ]
    },
    {
      "id": 2,
      "name": "TP1",
      "visit_type": "Visit",
      "order_index": 1,
      "studies": [
        {
          "study_id": 101,
          "study_uid": "1.2.840...",
          "study_description": "CT Abdomen",
          "study_date": "2026-01-12"
        }
      ]
    }
  ]
}
```

---

## 에러 코드 정의

| HTTP Status | Error Code | Description |
|-------------|------------|-------------|
| 400 | INVALID_REQUEST | 잘못된 요청 파라미터 |
| 401 | UNAUTHORIZED | 인증 실패 |
| 403 | FORBIDDEN | 권한 없음 |
| 404 | SUBJECT_NOT_FOUND | Subject를 찾을 수 없음 |
| 404 | TIMEPOINT_NOT_FOUND | TimePoint를 찾을 수 없음 |
| 404 | STUDY_NOT_FOUND | Study를 찾을 수 없음 |
| 409 | SUBJECT_CODE_DUPLICATE | Subject 코드 중복 |
| 409 | BASELINE_ALREADY_EXISTS | Baseline이 이미 존재함 |
| 409 | TIMEPOINT_NAME_DUPLICATE | TimePoint 이름 중복 |
| 500 | INTERNAL_SERVER_ERROR | 서버 내부 오류 |

---

## 권한 정책

| API | Required Permission |
|-----|---------------------|
| Subject 조회 | `project:read` |
| Subject 생성/수정/삭제 | `project:write` |
| TimePoint 조회 | `project:read` |
| TimePoint 생성/수정/삭제 | `project:write` |
| Study 할당/해제 | `project:write` |

---

## 구현 우선순위

### Phase 1 (MVP)
1. ✅ Subject 생성/조회
2. ✅ TimePoint 생성/조회/삭제
3. ✅ Study 할당/해제
4. ✅ 보드 뷰 API

### Phase 2 (Enhancement)
5. ⏳ TimePoint 순서 변경 (Drag & Drop)
6. ⏳ Bulk 작업 (여러 Study 동시 할당)
7. ⏳ 할당 이력 조회

### Phase 3 (CTIMS Integration)
8. ⏳ CTIMS Subject 동기화
9. ⏳ CTIMS TimePoint 동기화
10. ⏳ Read-only 모드 전환

---

## 관련 문서

- [Subject & TimePoint 설계 문서](./erd.md)
- [전체 데이터베이스 ERD](../database/ERD.md)
- [마이그레이션 스크립트](../../migrations/040_create_subject_timepoint.sql)

