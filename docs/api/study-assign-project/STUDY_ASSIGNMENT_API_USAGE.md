# Study Assignment API 사용법

## 📋 목차
1. [API 엔드포인트](#api-엔드포인트)
2. [Study 할당](#study-할당)
3. [Study 할당 해제](#study-할당-해제)
4. [응답 코드](#응답-코드)
5. [사용 예시](#사용-예시)

---

## 🔗 API 엔드포인트

### 1. Study 할당
```
POST /api/projects/{project_id}/studies/assign
```

### 2. Study 할당 해제
```
DELETE /api/projects/{project_id}/studies/{study_id}/unassign
```

---

## ✅ Study 할당

### **요청**

**URL**
```
POST http://localhost:8080/api/projects/{project_id}/studies/assign
```

**Headers**
```
Authorization: Bearer {access_token}
Content-Type: application/json
```

**Request Body**
```json
{
  "study_uid": "1.2.840.113619.2.1.1.TEST.123",
  "subject_code": "SUB-001"  // 선택 사항
}
```

**파라미터 설명**
- `study_uid` (필수): DICOM Study Instance UID
- `subject_code` (선택): Subject Code 지정 (없으면 Patient ID로 자동 생성)

---

### **응답**

**성공 (200 OK)**
```json
{
  "success": true,
  "message": "Study assigned successfully",
  "study_id": 123,
  "subject_id": 456,
  "subject_code": "SUB-001"
}
```

**Study 없음 (404 Not Found)**
```json
{
  "error": "Study not found in QIDO-RS"
}
```

**중복 할당 (409 Conflict)**
```json
{
  "error": "Study already assigned to this project"
}
```

**프로젝트 없음 (404 Not Found)**
```json
{
  "error": "Project not found"
}
```

---

## ❌ Study 할당 해제

### **요청**

**URL**
```
DELETE http://localhost:8080/api/projects/{project_id}/studies/{study_id}/unassign
```

**Headers**
```
Authorization: Bearer {access_token}
```

**파라미터 설명**
- `project_id`: 프로젝트 ID
- `study_id`: Study ID (Study UID 아님!)

---

### **응답**

**성공 (200 OK)**
```json
{
  "success": true,
  "message": "Study unassigned successfully"
}
```

**Study 없음 (404 Not Found)**
```json
{
  "error": "Study not found or not assigned to this project"
}
```

---

## 📊 응답 코드

| 코드 | 의미 | 설명 |
|------|------|------|
| 200 | OK | 성공 |
| 400 | Bad Request | 잘못된 요청 (Study UID 형식 오류 등) |
| 404 | Not Found | 프로젝트 또는 Study 없음 |
| 409 | Conflict | 중복 할당 |
| 500 | Internal Server Error | 서버 오류 |

---

## 💡 사용 예시

### **예시 1: 기본 Study 할당**

```bash
curl -X POST http://localhost:8080/api/projects/1/studies/assign \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "1.2.840.113619.2.1.1.TEST.123"
  }'
```

**응답**
```json
{
  "success": true,
  "message": "Study assigned successfully",
  "study_id": 123,
  "subject_id": 456,
  "subject_code": "PATIENT-001"
}
```

---

### **예시 2: Subject Code 지정하여 Study 할당**

```bash
curl -X POST http://localhost:8080/api/projects/1/studies/assign \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "1.2.840.113619.2.1.1.TEST.123",
    "subject_code": "SUB-001"
  }'
```

**응답**
```json
{
  "success": true,
  "message": "Study assigned successfully",
  "study_id": 123,
  "subject_id": 456,
  "subject_code": "SUB-001"
}
```

---

### **예시 3: Study 할당 해제**

```bash
curl -X DELETE http://localhost:8080/api/projects/1/studies/123/unassign \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**응답**
```json
{
  "success": true,
  "message": "Study unassigned successfully"
}
```

---

## 🔍 주요 특징

### ✅ **자동 기능**
1. **QIDO-RS 메타데이터 자동 가져오기**
   - Patient ID
   - Patient Name
   - Study Description

2. **Subject 자동 생성**
   - Patient ID 기반 자동 생성
   - 또는 사용자 지정 Subject Code 사용

3. **중복 방지**
   - 같은 Study를 같은 프로젝트에 중복 할당 방지
   - 409 Conflict 반환

---

## ⚠️ 주의사항

1. **Study UID vs Study ID**
   - 할당 시: `study_uid` 사용 (DICOM UID)
   - 할당 해제 시: `study_id` 사용 (데이터베이스 ID)

2. **QIDO-RS 의존성**
   - Study 메타데이터는 QIDO-RS에서 가져옴
   - QIDO-RS에 Study가 없으면 404 반환

3. **권한**
   - 프로젝트 멤버만 할당/해제 가능
   - 적절한 권한 필요

---

## 📚 관련 문서

- **E2E 테스트**: `pacs-server/e2e/test_study_assignment_e2e.py`
- **테스트 결과**: `pacs-server/e2e/TEST_STUDY_ASSIGNMENT_SUMMARY.md`
- **커버리지 분석**: `pacs-server/e2e/STUDY_ASSIGNMENT_TEST_COVERAGE_ANALYSIS.md`

