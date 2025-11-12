# DICOM API 문서

이 디렉토리는 DICOM 관련 API 문서를 포함합니다.

## 📄 문서 목록

### [frontend-api-spec.md](./frontend-api-spec.md)

**프론트엔드 개발자를 위한 완전한 API 명세**

이 문서는 프론트엔드 개발자가 DICOM API를 사용하는 데 필요한 모든 정보를 포함합니다:

#### 포함된 내용

1. **인증**
   - Keycloak 토큰 획득 (CORS 우회용 백엔드 프록시)
   - Bearer Token 사용법

2. **DICOM 데이터 조회**
   - Studies 조회 (전체 / 프로젝트별)
   - Series 조회 (계층적 경로)
   - Instances 조회 (계층적 경로)
   - 할당 여부 확인 (`check_assignment_for_project`)
   - 필터링 (modality, patient_id, study_date 등)
   - 페이지네이션

3. **프로젝트 데이터 할당**
   - Study 할당
   - Series 할당

4. **에러 처리**
   - HTTP 상태 코드
   - 에러 응답 형식
   - JavaScript 에러 처리 예제

5. **주요 사용 시나리오**
   - 전체 워크플로우
   - 프로젝트별 조회
   - 할당 여부 확인

6. **DICOM 태그 참조**
   - 자주 사용되는 DICOM 태그 목록
   - 태그 값 추출 예제

7. **테스트 계정**
   - 개발/테스트용 계정 정보

8. **참고 사항**
   - 권한 시스템
   - 페이지네이션
   - 필터링
   - 할당 여부 확인

---

## 🚀 빠른 시작

### 1. Keycloak 토큰 획득

```javascript
const response = await axios.post(`${apiUrl}/auth/keycloak-token`, {
  username: 'test_super_admin',
  password: 'TestAdmin123!'
});

const token = response.data.access_token;
```

### 2. Studies 조회

```javascript
// 전체 조회 (SUPER_ADMIN/ADMIN만 가능)
const studies = await axios.get(`${apiUrl}/dicom/studies`, {
  headers: { 'Authorization': `Bearer ${token}` }
});

// 프로젝트별 조회
const studies = await axios.get(`${apiUrl}/dicom/studies?project_id=150`, {
  headers: { 'Authorization': `Bearer ${token}` }
});
```

### 3. Study 할당

```javascript
const response = await axios.post(
  `${apiUrl}/projects/150/studies/assign`,
  {
    study_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
    study_description: "CT Chest",
    patient_id: "P12345"
  },
  { headers: { 'Authorization': `Bearer ${token}` } }
);
```

---

## 📌 주요 특징

### 1. 전역 접근 권한 (Global Access)

- **SUPER_ADMIN**, **ADMIN** 역할은 `DICOM_GLOBAL_ACCESS` 권한을 가짐
- `project_id` 없이 전체 DICOM 데이터 조회 가능
- 모든 프로젝트의 데이터 할당 가능

### 2. 할당 여부 확인

- `check_assignment_for_project` 파라미터로 특정 프로젝트에 할당되었는지 확인
- 응답에 `is_assigned` (boolean), `checked_project_id` (integer) 필드 추가
- `project_id`와 독립적으로 사용 가능

### 3. 계층적 QIDO-RS 경로

- Studies: `/api/dicom/studies`
- Series: `/api/dicom/studies/{study_uid}/series`
- Instances: `/api/dicom/studies/{study_uid}/series/{series_uid}/instances`

### 4. CORS 우회

- 브라우저에서 Keycloak으로 직접 요청하면 CORS 에러 발생
- 백엔드 프록시 엔드포인트 `/api/auth/keycloak-token` 사용

---

## 🔗 관련 문서

- [DICOM Gateway API](../dicom-gateway-api.md) - 백엔드 개발자용 상세 문서
- [Project Data Assignment API](../project-data-assignment-api.md) - 데이터 할당 API 상세 문서
- [Project Study Series Assignment API](../project-study-series-assignment-api.md) - Study/Series 할당 API 상세 문서

---

## 📝 변경 이력

### 2025-11-11

- 초기 문서 작성
- Keycloak 토큰 획득 프록시 엔드포인트 추가
- `check_assignment_for_project` 파라미터 추가
- 전역 접근 권한 (Global Access) 기능 추가
- 계층적 QIDO-RS 경로 수정 (Series, Instances)

---

## 💡 팁

1. **토큰 만료**: Keycloak 토큰은 300초(5분) 후 만료됩니다. 만료 전에 재발급하세요.
2. **DICOM 태그**: QIDO-RS 응답은 DICOM 표준 JSON 형식입니다. 태그 값 추출 방법은 문서를 참고하세요.
3. **에러 처리**: 모든 API 호출에 적절한 에러 처리를 추가하세요.
4. **페이지네이션**: 대량의 데이터를 조회할 때는 `page_size`를 적절히 조정하세요.

---

**문서 버전**: 1.0  
**최종 수정일**: 2025-11-11

