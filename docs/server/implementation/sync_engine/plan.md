# DB Sync Engine 구현 계획

목표: dcm4chee DB → RBAC DB로 Study/Series/Instance 동기화(델타 기준 `updated_time`).

작업:
- 설정: SERVER_MODE(full|api-only|sync-only), dcm4chee DB 접속 설정 추가
- 워커: study/series/instance select 및 upsert 구현(to_date/to_timestamp 변환 포함)
- 스케줄러: interval 기반 run/pause/resume/status
- API: /api/sync 상태/실행/일시정지/재개/스케줄 조회·변경

완료 기준: run_once 성공, 상태 조회 정상, 에러 우아한 처리
