# 📊 TimePoints with Studies API (X축 API)

## 개요

Subject의 모든 TimePoints와 각 TimePoint에 할당된 Studies를 한 번에 조회하는 API입니다.

**주요 기능:**
- Subject의 모든 TimePoints 조회
- 각 TimePoint에 할당된 Studies 조회
- Unassigned Studies 조회 (선택사항)
- RECIST Report 작성 시 X축 데이터로 사용

---

## 엔드포인트

**GET** `/api/subjects/{subject_id}/timepoints-with-studies`

---

## Request

### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `subject_id` | integer | Yes | Subject ID |

### Query Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `include_unassigned` | boolean | No | `false` | Unassigned Studies 포함 여부 |

### Headers

| Header | Value | Required |
|--------|-------|----------|
| `Authorization` | `Bearer {token}` | Yes |

### 예시

```bash
curl -X GET "http://localhost:8080/api/subjects/1/timepoints-with-studies?include_unassigned=true" \
  -H "Authorization: Bearer $TOKEN"
```

---

## Response

### 성공: `200 OK`

```json
{
  "subject_id": 1,
  "subject_code": "SUBJ-001",
  "timepoints": [
    {
      "id": 1,
      "subject_id": 1,
      "name": "Baseline",
      "timepoint_date": "2026-01-01",
      "description": "Initial evaluation",
      "created_at": "2026-01-01T09:00:00Z",
      "updated_at": "2026-01-01T09:00:00Z",
      "studies": [
        {
          "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
          "study_date": "2026-01-01",
          "study_time": "090000",
          "study_description": "CT Chest/Abdomen/Pelvis",
          "modality": "CT",
          "patient_id": "P001",
          "patient_name": "Patient^One",
          "accession_number": "ACC001"
        }
      ]
    },
    {
      "id": 2,
      "subject_id": 1,
      "name": "TP1",
      "timepoint_date": "2026-02-01",
      "description": "First follow-up",
      "created_at": "2026-02-01T09:00:00Z",
      "updated_at": "2026-02-01T09:00:00Z",
      "studies": [
        {
          "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
          "study_date": "2026-02-01",
          "study_time": "090000",
          "study_description": "CT Chest/Abdomen/Pelvis",
          "modality": "CT",
          "patient_id": "P001",
          "patient_name": "Patient^One",
          "accession_number": "ACC002"
        }
      ]
    }
  ],
  "unassigned_studies": [
    {
      "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
      "study_date": "2026-03-01",
      "study_time": "090000",
      "study_description": "CT Chest",
      "modality": "CT",
      "patient_id": "P001",
      "patient_name": "Patient^One",
      "accession_number": "ACC003"
    }
  ]
}
```

### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `subject_id` | integer | Subject ID |
| `subject_code` | string | Subject 코드 |
| `timepoints` | array | TimePoint 목록 |
| `timepoints[].id` | integer | TimePoint ID |
| `timepoints[].name` | string | TimePoint 이름 (예: "Baseline", "TP1") |
| `timepoints[].timepoint_date` | string | TimePoint 날짜 (YYYY-MM-DD) |
| `timepoints[].studies` | array | 할당된 Study 목록 |
| `timepoints[].studies[].study_uid` | string | Study UID |
| `timepoints[].studies[].study_date` | string | Study 날짜 (YYYYMMDD) |
| `timepoints[].studies[].modality` | string | Modality (CT, MR, PET 등) |
| `unassigned_studies` | array | 할당되지 않은 Study 목록 |

---

## 에러 응답

### Subject 없음: `404 Not Found`

```json
{
  "error": "NOT_FOUND",
  "message": "Subject not found"
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
  "message": "You don't have permission to access this Subject"
}
```

---

## 사용 예시

### 예시 1: Unassigned Studies 포함 조회

```bash
curl -X GET "http://localhost:8080/api/subjects/1/timepoints-with-studies?include_unassigned=true" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### 예시 2: Unassigned Studies 제외 조회

```bash
curl -X GET "http://localhost:8080/api/subjects/1/timepoints-with-studies?include_unassigned=false" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

---

## RECIST Report 작성 워크플로우

### JavaScript 예시

```javascript
// Subject의 모든 TimePoints와 Studies 조회
async function getTimePointsWithStudies(subjectId, includeUnassigned = true) {
  const response = await fetch(
    `/api/subjects/${subjectId}/timepoints-with-studies?include_unassigned=${includeUnassigned}`,
    {
      headers: {
        'Authorization': `Bearer ${token}`
      }
    }
  );

  const data = await response.json();
  return data;
}

// 사용 예시
const data = await getTimePointsWithStudies(1, true);

console.log('Subject:', data.subject_code);
console.log('TimePoints:', data.timepoints.map(tp => tp.name));
console.log('Unassigned Studies:', data.unassigned_studies.length);

// 각 TimePoint의 Studies 확인
data.timepoints.forEach(tp => {
  console.log(`${tp.name}: ${tp.studies.length} studies`);
});
```

---

## 관련 문서

- [Annotations by TimePoint API (Y축 API)](./annotations-by-timepoint.md)
- [Lesion Assignment API](../annotation/lesion-assignment.md)
- [Subject API](../subject/subject-crud.md)
- [TimePoint CRUD API](./timepoint-crud.md)

