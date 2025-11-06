# 테스트 리팩터링/복원 계획

목표: 그린 베이스라인 확보 후 레거시 테스트를 순차 복원.

작업 순서:
1) DICOM GW/Evaluator 테스트 정합성 보정(미니멀, DB-free)
2) Mask/Annotation 테스트 DTO/DI/actix 패턴 정리
3) Repository/Entity 테스트 최신 스키마 반영
4) 성능/외부연계 테스트는 파일별 단계적 복원 + #[ignore]

기준: 컴파일 에러 우선 제거, 시그니처/DTO 최신화, CI 안정 통과
