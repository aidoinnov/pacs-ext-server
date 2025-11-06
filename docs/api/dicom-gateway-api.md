# DICOM Gateway API (RBAC 적용)

## 개요
Keycloak 인증 후, RBAC 정책을 적용하여 Dcm4chee QIDO-RS에 대한 프록시 조회를 제공합니다. 응답은 사용자/프로젝트 RBAC에 의해 필터링됩니다.

## 인증
- Authorization: Bearer <JWT>

## 엔드포인트

### GET /api/dicom/studies
- 설명: QIDO-RS Studies 조회를 프록시합니다. 규칙(access_condition) 기반 파라미터와 사용자 입력 필터를 병합하여 QIDO로 전달하고, 응답 배열은 evaluator로 사후 필터링합니다.
- 쿼리 파라미터:
  - project_id: int (필수)
  - 필터: `modality`(→ Modality), `patient_id`(→ PatientID), `study_date`(→ StudyDate, 형식 `YYYYMMDD[-YYYYMMDD]`)
  - 선택 필터: `accession_number`(→ AccessionNumber, EQ/CONTAINS), `patient_name`(→ PatientName, CONTAINS)
  - 페이지네이션: `page`(1-base, 기본 1), `page_size`(기본 50, 최대 200) → QIDO: `offset`, `limit`
- 응답: QIDO-RS JSON 배열(필터링 후)

예시 요청:
```
GET /api/dicom/studies?project_id=1&modality=CT&study_date=20240101-20241231&page=1&page_size=50
Authorization: Bearer <JWT>
```

### GET /api/dicom/studies/{study_uid}/series
- 설명: QIDO-RS Series 조회를 프록시합니다. 규칙 병합 + 사용자 필터 + evaluator 사후 필터링 적용.
- 경로 파라미터: study_uid (StudyInstanceUID)
- 쿼리 파라미터: 위와 동일 필터/페이지네이션 지원
- 응답: QIDO-RS JSON 배열(필터링 후)

예시 요청:
```
GET /api/dicom/studies/1.2.3.4/series?project_id=1&patient_id=PAT001&page=2&page_size=50
Authorization: Bearer <JWT>
```

### GET /api/dicom/studies/{study_uid}/series/{series_uid}/instances
- 설명: QIDO-RS Instances 조회를 프록시합니다. 규칙 병합 + 사용자 필터 + evaluator 사후 필터링 적용.
- 경로 파라미터: study_uid (StudyInstanceUID), series_uid (SeriesInstanceUID)
- 쿼리 파라미터: 위와 동일 필터/페이지네이션 지원
- 응답: QIDO-RS JSON 배열(필터링 후)

예시 요청:
```
GET /api/dicom/studies/1.2.3.4/series/1.2.3.4.5/instances?project_id=1&accession_number=ACC-123&page=1&page_size=10
Authorization: Bearer <JWT>
```

## RBAC 적용 방식
1) 규칙 기반 사전 필터(QIDO 파라미터 병합)
- security_access_condition → QIDO 파라미터 매핑
  - 00080060(Modality) EQ → Modality=...
  - 00100020(PatientID) EQ → PatientID=...
  - 00080020(StudyDate) RANGE → StudyDate=YYYYMMDD-YYYYMMDD
  - 00080050(AccessionNumber) EQ/CONTAINS → AccessionNumber=...
  - 00100010(PatientName) CONTAINS → PatientName=...
- 병합 규칙: "사용자 입력 우선" (동일 키 충돌 시 사용자 파라미터가 규칙 값을 덮어씀)
- 표현 곤란한 연산자(NE 등)는 사후 필터에 위임

2) evaluator 사후 필터(JSON 결과 필터)
- Study: 각 item의 0020000D(StudyInstanceUID)로 DB 매핑 후 접근 허용 여부 평가
- Series: 각 item의 0020000E(SeriesInstanceUID)로 DB 매핑 후 평가
- Instance: 각 item의 00080018(SOPInstanceUID)로 DB 매핑 후 평가

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

## 페이지네이션 동작
- 입력: `page`(>=1), `page_size`(1~200)
- 변환: `offset=(page-1)*page_size`, `limit=page_size`
- 응답 포맷은 QIDO 표준 배열이며, total count는 QIDO upstream 설정에 의존(미보장)

## 에러
- 400 Bad Request: `study_date` 포맷 오류 등 검증 실패
- 502 Bad Gateway: Dcm4chee QIDO 요청 실패 시
- 401 Unauthorized: JWT 검증 실패 시
- 200 OK + 빈 배열: 규칙/권한에 의해 모두 필터링된 경우

## 토큰 릴레이 및 업스트림 인증
- 게이트웨이는 수신한 Authorization Bearer 토큰을 Dcm4chee로 그대로 릴레이합니다.
- Dcm4chee가 Keycloak 보호 중인 경우, 토큰의 aud(클라이언트)가 Dcm4chee 리소스 서버와 일치해야 합니다.
- 401/403 발생 시: 토큰 만료/스코프/오디언스 불일치 여부를 우선 확인하세요.

## 트러블슈팅
- 404 Not Found: QIDO 경로 확인. 예) `aets/<AET>/rs/studies` 형태여야 하며, 운영 경로는 `aets/iAID_PACS/rs/...` 등 환경에 따라 다릅니다.
- 401 Unauthorized(업스트림): Bearer 토큰 릴레이 여부, aud 클레임, Dcm4chee Keycloak 설정 확인.
- 빈 결과: 프로젝트 멤버십/명시 권한 부재 또는 DENY/LIMIT 규칙에 의한 필터링 가능.

## 시나리오 요약(테스트 기준)
- 프로젝트 필터링: 비멤버 프로젝트 데이터는 제외, 멤버 프로젝트 내 데이터만 반환
- 규칙 기반 필터링: QIDO 파라미터 병합으로 CT/PatientID/StudyDate RANGE 등 사전 제한
- RBAC 명시 권한: 해당 레벨(Study/Series/Instance)에서 명시 권한이 있으면 우선 허용
- 계층 상속: Study 접근 허용 시 하위 Series/Instance 접근도 일관되게 허용
- 충돌 처리: DENY > LIMIT > ALLOW, LIMIT는 교집합으로 축소
