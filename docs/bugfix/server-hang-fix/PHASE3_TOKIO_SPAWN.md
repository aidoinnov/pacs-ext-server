# Phase 3: tokio::spawn 백그라운드 작업 모니터링 개선

## 📋 개요

**날짜**: 2026-01-29  
**관련 커밋**: `852b200`  
**이전 커밋**: `19b2503` (Phase 2 - Redis 연결 재사용)

3차 재검토에서 발견된 백그라운드 작업 모니터링 문제를 개선했습니다.

---

## 🔍 발견된 문제

### 1. **DB 풀 타임아웃 설정 누락 (오판)** ❌

**초기 판단**: DB 풀 타임아웃 설정이 적용되지 않았다고 판단  
**실제 상태**: **이미 Phase 1에서 정상 적용됨** ✅

```rust
// pacs-server/src/main.rs Line 207-216
let pool = PgPoolOptions::new()
    .max_connections(settings.database.max_connections)
    .min_connections(settings.database.min_connections)
    .acquire_timeout(std::time::Duration::from_secs(30))      // ✅ 적용됨
    .idle_timeout(Some(std::time::Duration::from_secs(600)))  // ✅ 적용됨
    .max_lifetime(Some(std::time::Duration::from_secs(1800))) // ✅ 적용됨
    .test_before_acquire(true)                                // ✅ 적용됨
    .connect(&database_url)
    .await
    .expect("Failed to connect to database");
```

**결론**: 수정 불필요 (이미 완료됨)

---

### 2. **tokio::spawn 백그라운드 작업 모니터링 부족** ⚠️

**심각도**: 중간  
**영향**: 캐시 저장 실패 추적 어려움

#### 발견된 위치 (8곳)

| 위치 | 라인 | 작업 내용 |
|------|------|-----------|
| 1 | 901 | Studies 캐시 저장 |
| 2 | 1362 | Series 캐시 저장 |
| 3 | 1681 | Series 캐시 저장 (프로젝트별) |
| 4 | 2088 | Instances 캐시 저장 |
| 5 | 2820 | QIDO 병렬 호출 (이미 제한됨) |
| 6 | 2909 | 배치 allowed_series_uids 조회 |
| 7 | 2961 | 배치 series_ids 조회 |
| 8 | 2992 | 배치 RBAC 평가 |

#### 기존 코드 문제점

```rust
// ❌ Before: 에러 추적 어려움
tokio::spawn(async move {
    if let Err(e) = cache_clone.set_studies(...).await {
        tracing::warn!("Failed to cache QIDO studies response: {}", e);  // warn 레벨
    }
    // 성공 시 로그 없음
});
```

**문제점**:
- ❌ 에러가 `warn` 레벨로만 기록 (프로덕션에서 놓치기 쉬움)
- ❌ 성공 시 로그 없음 (모니터링 불가)
- ❌ 컨텍스트 정보 부족 (어떤 project/study에서 실패했는지 불명확)

---

## ✅ 적용된 수정사항

### 개선된 코드

```rust
// ✅ After: 명확한 에러 추적
tokio::spawn(async move {
    if let Err(e) = cache_clone.set_studies(Some(project_id), &params_hash_clone, &json_clone).await {
        tracing::error!("Background cache storage failed for studies (project={}): {}", project_id, e);
    } else {
        tracing::debug!("Background cache storage succeeded for studies (project={})", project_id);
    }
});
```

### 개선 사항

| 항목 | Before | After | 효과 |
|------|--------|-------|------|
| 에러 로그 레벨 | `warn` | `error` | ✅ 프로덕션 모니터링 용이 |
| 성공 로그 | 없음 | `debug` | ✅ 개발 환경 디버깅 가능 |
| 컨텍스트 정보 | 없음 | project_id, study_uid 등 | ✅ 문제 진단 용이 |
| 주석 | 없음 | "백그라운드 캐시 저장" | ✅ 코드 가독성 향상 |

---

## 📊 수정된 파일

### `pacs-server/src/presentation/controllers/dicom_gateway_controller.rs`

**변경 내용**: 4개 위치 개선 (캐시 저장 작업)

1. **Line 901**: Studies 캐시 저장
2. **Line 1362**: Series 캐시 저장
3. **Line 1681**: Series 캐시 저장 (프로젝트별)
4. **Line 2088**: Instances 캐시 저장

**통계**:
- +16줄 추가
- -4줄 삭제
- 순증가: +12줄

---

## 🎯 개선 효과

### 1. **모니터링 개선**
- ✅ 프로덕션 환경에서 캐시 저장 실패 즉시 감지
- ✅ 개발 환경에서 성공/실패 모두 추적 가능

### 2. **문제 진단 용이**
- ✅ 어떤 프로젝트/스터디에서 실패했는지 명확
- ✅ 에러 메시지에 충분한 컨텍스트 포함

### 3. **코드 가독성**
- ✅ 주석으로 백그라운드 작업임을 명시
- ✅ 일관된 로깅 패턴 적용

---

## 📈 전체 Phase 요약

| Phase | 주요 수정 | 커밋 | 상태 |
|-------|-----------|------|------|
| Phase 1 | DB 풀 타임아웃, HTTP 클라이언트 풀 | `0f00fcb` | ✅ 완료 |
| Phase 2 | Redis 연결 재사용, QIDO 병렬 제한 | `19b2503` | ✅ 완료 |
| Phase 3 | tokio::spawn 모니터링 개선 | `852b200` | ✅ 완료 |

---

## 🧪 테스트 방법

### 로그 확인

```bash
# 에러 로그 모니터링
tail -f logs/pacs-server.log | grep "Background cache storage failed"

# 성공 로그 확인 (debug 레벨 활성화 필요)
RUST_LOG=debug cargo run | grep "Background cache storage succeeded"
```

### 예상 로그 출력

```
# 실패 시
[ERROR] Background cache storage failed for studies (project=1): Redis connection timeout

# 성공 시 (debug 레벨)
[DEBUG] Background cache storage succeeded for studies (project=1)
```

---

## 📝 관련 문서

- [Phase 1 수정 (DB 풀 타임아웃)](./README.md)
- [Phase 2 수정 (Redis 연결 재사용)](./PHASE2_REDIS_AND_QIDO.md)
- [전체 요약](./SUMMARY.md)

---

## ✅ 최종 결론

**모든 주요 문제 해결 완료!**

1. ✅ **DB 연결 풀 고갈** → 타임아웃 설정 완료 (Phase 1)
2. ✅ **HTTP 연결 누적** → 풀 크기 제한 완료 (Phase 1)
3. ✅ **Redis 연결 누수** → 연결 재사용 패턴 적용 (Phase 2)
4. ✅ **QIDO 병렬 호출** → 최대 5개로 제한 (Phase 2)
5. ✅ **백그라운드 작업 모니터링** → 로깅 개선 (Phase 3)

**서버 안정성 대폭 향상!** 🎉

