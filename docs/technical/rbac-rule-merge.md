# RBAC 규칙 병합 및 우선순위 동작

## 개요
- 본 문서는 DICOM RBAC 평가에서 규칙의 병합 방식, 우선순위, 충돌 해결 규칙을 정의합니다.
- 적용 대상: `DicomRbacEvaluatorImpl`의 평가 로직과 게이트웨이 사후 필터링.

## 용어
- 명시 권한: `project_data_access`에 의해 직접 부여된 접근
- 규칙: `security_project_dicom_condition`, `security_role_dicom_condition`에 연결된 `security_access_condition`
- ConditionType: ALLOW, DENY, LIMIT

## 병합 원칙
1. 프로젝트 멤버십 확인이 선행되고, 미가입자는 거부
2. 명시 권한이 존재하면 해당 리소스 레벨에서 즉시 허용
3. 규칙 병합 순서
   - 역할 규칙과 프로젝트 규칙을 수집
   - 우선순위(priority DESC)로 정렬, 동순위 시 id ASC
4. 충돌 해결
   - DENY는 항상 ALLOW/LIMIT보다 우선
   - LIMIT는 허용 범위를 축소하는 교집합으로 해석
   - ALLOW는 허용 후보를 확장
5. 규칙 부재 시 기본 거부(secure-by-default)

## 연산자/태그 매핑
- EQ/NE/CONTAINS/RANGE 지원
- 대표 태그
  - 00080060 Modality ↔ QIDO `Modality`
  - 00100020 PatientID ↔ QIDO `PatientID`
  - 00080020 StudyDate ↔ QIDO `StudyDate` (YYYYMMDD 또는 RANGE)

## 결정 트리(요약)
- 멤버십? 아니오 → 거부
- 명시 권한? 예 → 허용
- 규칙 목록 평가
  - DENY 매치? → 거부
  - LIMIT 집계 → 최종 허용 후보 축소
  - ALLOW 매치 존재하고 LIMIT 만족? → 허용, 아니면 거부

## 예시
- 역할 규칙: ALLOW Modality=CT (priority 10)
- 프로젝트 규칙: LIMIT StudyDate=20240101-20241231 (priority 9)
- 결과: CT이면서 2024년 기간에 한해 허용

## 트러블슈팅
- 기대보다 결과가 적은 경우: LIMIT 조건 교집합 확인
- 전부 거부되는 경우: 우선순위 높은 DENY 유무, 규칙 부재 확인
- 성능: 인덱스(018_core_indices.sql) 적용 여부와 쿼리 플랜 확인
