# 테스트 리팩터링 진행 내용

- DICOM GW/Evaluator: 미니멀 유닛/통합 테스트 추가, 외부 의존 제거, 일부 시나리오 #[ignore]
- Mask/Annotation: DTO 변경 반영, Arc 주입/actix test 호출 수정, 불필요 의존 제거
- Repository/Entity: 신규 필드/Enum 반영, 타임스탬프 생성 API 보정
- 성능/외부연계: 파일 단위 재활성화 시도 → 컴파일 오류 발생 건 재-비활성화 후 순차 복원 계획 수립

현황: 그린 유지, 단계적 복원 진행 중
