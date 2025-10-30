# DB Sync Engine 구현 내용

- 설정: `ServerMode` 추가(full/api-only/sync-only), `SyncConfig`, `Dcm4cheeDbConfig`
- 워커: 실제 dcm4chee 스키마 반영 조인/열 타입 맞춤, 델타 기준 select, RBAC DB upsert
- 스케줄러: interval 루프, run/pause/resume, 상태 유지(Arc<RwLock<SyncState>>)
- API: `/api/sync` status/run/pause/resume/schedule
- 보강: PgConnectOptions 사용으로 특수문자 비밀번호 안전 처리

주의: 포트/접속 오류 시 기능 비활성화 경고 후 서버 계속 동작
