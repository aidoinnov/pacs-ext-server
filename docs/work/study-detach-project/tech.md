# Study 정규화 리팩토링 - 기술 설계서

## 설계 개요
- 정규화 목표: Study는 전역 엔터티(`study`)로 관리, 프로젝트 종속성은 `project_study_map`으로 분리
- 접근 제어: 멤버십(`security_user_project`), 명시 권한(`project_data_access`), 규칙(프로젝트/역할 연결)의 조합

## 스키마 변경안
- `study` 테이블
  - 컬럼: id(PK), study_uid(UNIQUE), study_description, patient_id, patient_name, patient_birth_date, study_date, created_at, updated_at
  - 인덱스: (study_uid)
- `project_study_map` 테이블
  - 컬럼: id(PK), project_id(FK), study_id(FK), UNIQUE(project_id, study_id)
  - 인덱스: (project_id, study_id)
- 호환 뷰 `project_data_study_compat` (임시)

## 데이터 이전 전략
1. `study`에 UID 기준 upsert
2. `project_study_map`에 (project_id, study_id) 삽입
3. series/instance는 상위 참조를 통해 `study` 기준으로 정합성 유지
4. 이전 전/후 카운트/무결성 비교, 롤백 스크립트 제공

## 코드 영향
- 엔티티: `Study`, `ProjectStudyMap` 신설/반영
- 레포지토리: UID 기반 조회/프로젝트별 목록 조회 JOIN화
- 평가기: UID→ID 해석에서 프로젝트 필터 제거, 접근성은 별도 검증
- 게이트웨이: RBAC 필터 시 UID→ID 매핑 대상 변경
- 동기화: `study` upsert + 프로젝트 매핑 정책화

## 핵심 쿼리(초안)
```sql
-- UID→study.id
SELECT id FROM study WHERE study_uid = $1;

-- 프로젝트별 study 목록
SELECT s.*
FROM study s
JOIN project_study_map psm ON psm.study_id = s.id
WHERE psm.project_id = $1
ORDER BY s.study_date DESC NULLS LAST, s.created_at DESC
LIMIT $2 OFFSET $3;

-- series 상위 study 확인 (마이그레이션 후 경로 업데이트 필요)
SELECT s.id
FROM project_data_series pds
JOIN project_data_study pdt ON pds.study_id = pdt.id;
```

## 설정/마이그레이션 가드
- 트랜잭션, 타임아웃, 배치 크기(예: 10k), 중단 가능 체크포인트
- `APP_SYNC__DEFAULT_PROJECT_ID`는 매핑 초기화 정책에만 사용(선택)

## 성능
- 인덱스 설계와 EXPLAIN 스냅샷 자동화 유지
- 고빈도 쿼리: UID 매핑, 프로젝트 리스트, 접근성 확인

## 보안/권한
- 프로젝트 멤버십/명시 권한/룰 평가 로직은 변경 없음
- 데이터 정규화로 권한 경로가 단순화되어 감사/추적 용이

## 롤백
- 이전 테이블 보존 또는 역이전 스크립트 제공
- 호환 뷰로 단기 우회 가능

## 오픈 이슈
- series/instance 스키마 상위 참조 일치화 상세안
- 기존 테스트 케이스 마이그레이션 순서/범위
