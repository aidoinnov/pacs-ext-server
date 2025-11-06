# Study 정규화: 프로젝트 종속성 제거 설계

본 문서는 `project_data_study`의 `project_id` 종속을 제거하고 전역 `study` 엔터티로 정규화하는 변경안을 설명합니다.

## 요약
- 새 테이블: `study` (UID 전역 고유)
- 매핑 테이블: `project_study_map (project_id, study_id)`
- 호환: 임시 읽기 전용 뷰 제공 가능 (`project_data_study_compat`)
- 접근 제어: `project_data_access` + 멤버십/룰로 일원화

## 마이그레이션 개요
- 파일: `pacs-server/migrations/019_study_detach_project.sql` (Scaffold)
- 단계:
  1) `study` 생성, 인덱스 구성
  2) `project_study_map` 생성
  3) 기존 데이터 이전(upsert→map 삽입)
  4) series/instance FK 경로 재검토 후 적용
  5) 필요 시 호환 뷰 제공

## 영향 범위
- Sync: study upsert 대상 변경, 프로젝트 매핑 분리
- Repository/Service: UID/프로젝트 조회 로직 JOIN 기반으로 변경
- Evaluator/Gateway: UID→ID 매핑은 `study` 기준, 접근성 판정은 기존 로직 유지
- 테스트: UID/프로젝트 시나리오 전면 반영

## 검증/성능
- 무결성 검증: 이전 전/후 카운트 및 FK 제약 확인
- EXPLAIN 스냅샷 갱신: UID 매핑, 프로젝트 리스트, 접근성 확인 쿼리

## 롤백
- 역이전 스크립트 마련(초기 단계에서는 뼈대 제공)
