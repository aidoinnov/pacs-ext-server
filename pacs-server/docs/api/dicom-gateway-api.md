# DICOM Gateway API (RBAC 적용)

## 개요
Keycloak 인증 후, RBAC 정책을 적용하여 Dcm4chee QIDO-RS에 대한 프록시 조회를 제공합니다. 응답은 사용자/프로젝트 RBAC에 의해 필터링됩니다.

## 인증
- Authorization: Bearer <JWT>

## 엔드포인트

### GET /api/dicom/studies
- 설명: QIDO-RS Studies 조회를 프록시합니다. 질의 파라미터는 그대로 전달되며, 프로젝트 기반 규칙(access_condition)이 QIDO 쿼리에 병합됩니다. 응답 배열은 evaluator로 사후 필터링됩니다.
- 쿼리 파라미터:
  - project_id: int (필수에 가까움; 미지정 시 서버 기본값 사용)
  - QIDO-RS 호환 파라미터 (예: StudyDate, Modality, PatientID 등)
- 응답: QIDO-RS JSON 배열(필터링 후)

예시 요청:
```
GET /api/dicom/studies?project_id=1&StudyDate=20200101-20201231
Authorization: Bearer <JWT>
```

### GET /api/dicom/studies/{study_uid}/series
- 설명: QIDO-RS Series 조회를 프록시합니다. 프로젝트 규칙 병합 + evaluator 사후 필터링 적용.
- 경로 파라미터: study_uid (StudyInstanceUID)
- 쿼리 파라미터:
  - project_id: int
  - QIDO-RS 호환 파라미터
- 응답: QIDO-RS JSON 배열(필터링 후)

예시 요청:
```
GET /api/dicom/studies/1.2.3.4/series?project_id=1
Authorization: Bearer <JWT>
```

### GET /api/dicom/studies/{study_uid}/instances
- 설명: QIDO-RS Instances 조회를 프록시합니다. 프로젝트 규칙 병합 + evaluator 사후 필터링 적용.
- 경로 파라미터: study_uid (StudyInstanceUID)
- 쿼리 파라미터:
  - project_id: int
  - QIDO-RS 호환 파라미터 (예: SOPClassUID, InstanceNumber 등)
- 응답: QIDO-RS JSON 배열(필터링 후)

예시 요청:
```
GET /api/dicom/studies/1.2.3.4/instances?project_id=1&limit=1
Authorization: Bearer <JWT>
```

## RBAC 적용 방식
1) 규칙 기반 사전 필터(QIDO 파라미터 병합)
- security_access_condition → QIDO 파라미터 매핑
  - 00080060(Modality) EQ → Modality=...
  - 00100020(PatientID) EQ → PatientID=...
  - 00080020(StudyDate) RANGE → StudyDate=YYYYMMDD-YYYYMMDD

2) evaluator 사후 필터(JSON 결과 필터)
- Study: 각 item의 0020000D(StudyInstanceUID)로 DB 매핑 후 접근 허용 여부 평가
- Series: 각 item의 0020000E(SeriesInstanceUID)로 DB 매핑 후 평가

### 규칙 병합/우선순위/충돌 처리
- 병합 순서: 역할 규칙 + 프로젝트 규칙 수집 → priority DESC, 동순위 id ASC 정렬
- 충돌 우선순위: DENY > LIMIT > ALLOW
- LIMIT는 허용 후보를 축소(교집합)로 해석, ALLOW는 허용 후보를 확장
- 규칙이 없으면 기본 거부(secure-by-default)
- 명시 권한(`project_data_access`)이 해당 레벨에서 존재하면 우선 허용

예시
```
역할 규칙: ALLOW Modality=CT (priority 10)
프로젝트 규칙: LIMIT StudyDate=20240101-20241231 (priority 9)
결과: CT 이면서 2024년 기간에 한해 허용
```

## 에러
- 502 Bad Gateway: Dcm4chee QIDO 요청 실패 시
- 401 Unauthorized: JWT 검증 실패 시
- 200 OK + 빈 배열: 규칙/권한에 의해 모두 필터링된 경우


## 시나리오 요약(테스트 기준)
- 프로젝트 필터링: 비멤버 프로젝트 데이터는 제외, 멤버 프로젝트 내 데이터만 반환
- 규칙 기반 필터링: QIDO 파라미터 병합으로 CT/PatientID/StudyDate RANGE 등 사전 제한
- RBAC 명시 권한: 해당 레벨(Study/Series/Instance)에서 명시 권한이 있으면 우선 허용
- 계층 상속: Study 접근 허용 시 하위 Series/Instance 접근도 일관되게 허용
- 충돌 처리: DENY > LIMIT > ALLOW, LIMIT는 교집합으로 축소


