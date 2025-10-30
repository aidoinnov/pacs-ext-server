# 테스트 보강 TODO (2025-10-30)

## 목적
- RBAC 시나리오 전 범위 보강 및 게이트웨이·평가자 동작 신뢰도 향상

## 항목
- [ ] Series-level explicit access overrides rules
- [ ] Instance-level explicit access overrides rules
- [ ] Integration: DENY over ALLOW across study/series/instance
- [ ] Matrix: non-member denied at study/series/instance
- [ ] E2E: Keycloak login → gateway → Dcm4chee token relay
- [ ] QIDO param merge from rules (CT+date+patient)

## 비고
- DB 기반 시나리오 테스트는 `APP_DATABASE_URL` 설정 시에만 실행
- 외부 연동(E2E)은 운영 Keycloak/Dcm4chee 환경 변수 필요

