# 기술 업데이트 (RBAC 규칙 병합)

## 반영 사항
- evaluator: DENY 우선/ LIMIT 교집합/ ALLOW 확장 로직 정리
- UID 단위 접근 평가(evaluate_*_uid) 경로 안정화
- QIDO 파라미터 매핑(StudyDate/Modality/PatientID)

## 문서 링크
- `pacs-server/docs/api/dicom-gateway-api.md` (규칙 병합 섹션 보강)
- `docs/technical/rbac-rule-merge.md` (결정 트리/우선순위/예시)

## 보류 항목
- 시드 데이터 기반 통합 시나리오 전 범위 검증
- EXPLAIN 스냅샷 수집/첨부
- doctest no_run/실행가능 전환
- 경고 정리
