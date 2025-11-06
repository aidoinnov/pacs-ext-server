# 프로젝트 종속성 제거(Study) 리팩토링 - 계획서

## 배경/문제
- `project_data_study`가 `project_id`에 종속되어 동일 Study가 프로젝트별로 중복 저장됨
- Study는 프로젝트 독립적으로 관리하고, 프로젝트별 접근은 `project_data_access`/멤버십/룰로 판정하는 것이 정규화와 운영에 적합

## 목표
- `project_data_study`의 프로젝트 종속성 제거
- UID 단일화: `study_uid`를 전역 고유로 관리
- 프로젝트별 접근 제어는 접근 테이블과 규칙으로만 결정

## 범위
- DB 스키마/마이그레이션, 동기화 엔진, 레포지토리/서비스, RBAC 평가기, 게이트웨이 컨트롤러, 테스트, 문서/체인지로그

## 작업 항목
1. DB 마이그레이션
   - 신규 테이블 `study(id, study_uid UNIQUE, description, patient_id, patient_name, patient_birth_date, study_date, created_at, updated_at)`
   - `project_data_study` → `project_study_map(project_id, study_id)`로 매핑 테이블화, UNIQUE(project_id, study_id)
   - 데이터 이전: 기존 `project_data_study`를 `study`로 upsert 후 `project_study_map`에 (project_id, study_id) 삽입
   - 인덱스: `study(study_uid)`, `project_study_map(project_id, study_id)`
   - 호환 뷰(읽기 전용) `project_data_study_compat` 임시 제공

2. 엔티티/도메인
   - `ProjectDataStudy` → `Study` (project_id 제거)
   - `ProjectStudyMap` 엔티티 신설

3. 레포지토리/서비스
   - `find_study_by_uid(study_uid)`로 시그니처 변경
   - `find_studies_by_project_id`는 `JOIN project_study_map`으로 구현

4. RBAC 평가기
   - UID→ID 해석에서 `project_id` 의존 제거; 프로젝트 접근성은 멤버십/explicit/룰로 별도 판정

5. 게이트웨이 컨트롤러
   - QIDO 결과 RBAC 필터에서 UID→ID 매핑은 `study` 기준으로 수행

6. 동기화 엔진
   - Study upsert를 `study`에 수행, 프로젝트 매핑은 정책(자동/수동)으로 처리
   - Series/Instance 외래키 경로 업데이트

7. 데이터 이전/검증 스크립트
   - 트랜잭션 기반 이전, 카운트/무결성 검증, 롤백 가이드
   - EXPLAIN 스냅샷 갱신

8. 테스트
   - UID 매핑/시나리오/평가기/게이트웨이 테스트 전면 수정
   - DB 필요 테스트는 `APP_DATABASE_URL` 게이트

9. 문서/체인지로그
   - 스키마/아키텍처/API 문서 갱신, CHANGELOG 추가

## 위험/대응
- 대규모 스키마 변경 → 호환 뷰로 점진 이행, 백업/롤백 절차 문서화
- 성능 리스크 → 핵심 쿼리 인덱스 설계와 EXPLAIN 검증

## 완료 기준
- 마이그레이션/데이터 이전 성공 및 무결성 검증 통과
- 테스트 스위트 통과(무시 테스트 제외), 게이트웨이 기능 정상
- 문서/체인지로그 반영 완료


