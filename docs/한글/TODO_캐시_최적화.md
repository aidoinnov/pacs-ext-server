# TODO: DICOM Gateway API 캐시 최적화

## 📋 현재 상태

**캐시 비활성화됨** (`Cache-Control: no-cache, no-store, must-revalidate`)

- 모든 요청마다 DB에서 전체 데이터 조회
- 네트워크 트래픽: 높음
- 서버 부하: 높음
- 데이터 일관성: ✅ 완벽 (항상 최신 데이터)

## ❌ 이전 ETag 구현의 문제점

```rust
// 1. DB에서 전체 데이터 조회 (느림!)
let final_studies = query_studies(...).await?;

// 2. 데이터를 JSON으로 직렬화해서 해시 계산 (느림!)
let etag = calculate_hash(&final_studies);

// 3. 클라이언트 ETag와 비교
if client_etag == etag {
    return 304;  // 데이터 안 보냄
}
```

**문제**:
- ❌ DB 쿼리는 항상 실행됨 (서버 부하 그대로)
- ❌ JSON 직렬화 + 해시 계산 오버헤드
- ✅ 네트워크 트래픽만 감소 (304 응답)

**결론**: 비효율적! 서버 부하는 그대로인데 복잡도만 증가

## ✅ 개선 방안

### 방안 1: Last-Modified 기반 (권장) ⭐

**개요**: DB에 `updated_at` 타임스탬프를 활용하여 변경 여부만 빠르게 확인

**구현**:
```rust
// 1. 최신 업데이트 시간만 조회 (매우 빠름!)
let last_modified = sqlx::query_scalar!(
    "SELECT MAX(updated_at) FROM studies WHERE ..."
)
.fetch_one(pool)
.await?;

// 2. If-Modified-Since 헤더 확인
if let Some(if_modified_since) = req.headers().get("If-Modified-Since") {
    let client_time = parse_http_date(if_modified_since)?;
    if client_time >= last_modified {
        // 변경 안됨 - DB 쿼리 안함!
        return HttpResponse::NotModified()
            .insert_header(("Last-Modified", format_http_date(last_modified)))
            .finish();
    }
}

// 3. 변경됐을 때만 전체 데이터 조회
let final_studies = query_studies(...).await?;

HttpResponse::Ok()
    .insert_header(("Last-Modified", format_http_date(last_modified)))
    .insert_header(("Cache-Control", "no-cache, must-revalidate"))
    .json(final_studies)
```

**필요 작업**:
1. DB 스키마 확인
   - `studies` 테이블에 `updated_at` 컬럼 있는지 확인
   - 없으면 추가 (migration)
   - TimePoint 할당/해제 시 `updated_at` 업데이트
2. HTTP 날짜 파싱 유틸리티 추가
   - `chrono` crate 사용
   - `parse_http_date()`, `format_http_date()` 함수
3. 컨트롤러 수정
   - Last-Modified 헤더 처리 로직 추가

**장점**:
- ✅ DB 쿼리 최소화 (타임스탬프만 조회)
- ✅ 서버 부하 대폭 감소
- ✅ 네트워크 트래픽 감소
- ✅ HTTP 표준 준수
- ✅ 구현 간단

**단점**:
- ⚠️ DB 스키마 변경 필요 (migration)
- ⚠️ TimePoint 변경 시 `updated_at` 업데이트 로직 필요

### 방안 2: Redis 캐시 + ETag

**개요**: Redis에 ETag를 저장하여 DB 쿼리 없이 변경 여부 확인

**구현**:
```rust
// 1. Redis에서 ETag 조회 (매우 빠름!)
let cache_key = format!("etag:studies:{}:{}", project_id, filters_hash);
let cached_etag: Option<String> = redis.get(&cache_key).await?;

// 2. 클라이언트 ETag와 비교
if let Some(etag) = cached_etag {
    if let Some(if_none_match) = req.headers().get("If-None-Match") {
        if if_none_match.to_str()? == etag {
            // 변경 안됨 - DB 쿼리 안함!
            return HttpResponse::NotModified()
                .insert_header(("ETag", etag))
                .finish();
        }
    }
}

// 3. DB에서 데이터 조회
let final_studies = query_studies(...).await?;

// 4. 새 ETag 생성 및 Redis에 저장
let new_etag = calculate_etag(&final_studies);
redis.set_ex(&cache_key, &new_etag, 300).await?; // 5분 TTL

HttpResponse::Ok()
    .insert_header(("ETag", new_etag))
    .insert_header(("Cache-Control", "no-cache, must-revalidate"))
    .json(final_studies)
```

**필요 작업**:
1. Redis 연결 확인 (이미 있음)
2. ETag 계산 함수 구현
3. TimePoint 변경 시 Redis 캐시 무효화
   - `redis.del(pattern)` 사용
   - 관련된 모든 ETag 삭제

**장점**:
- ✅ DB 쿼리 최소화
- ✅ 서버 부하 감소
- ✅ 네트워크 트래픽 감소
- ✅ DB 스키마 변경 불필요

**단점**:
- ⚠️ Redis 의존성 증가
- ⚠️ 캐시 무효화 로직 복잡
- ⚠️ TTL 관리 필요

### 방안 3: 쿼리 결과 전체 캐싱

**개요**: Redis에 전체 쿼리 결과를 캐싱

**구현**:
```rust
// 1. Redis에서 캐시된 결과 조회
let cache_key = format!("studies:{}:{}", project_id, filters_hash);
if let Some(cached_data) = redis.get::<String>(&cache_key).await? {
    let studies: Vec<Study> = serde_json::from_str(&cached_data)?;
    return HttpResponse::Ok()
        .insert_header(("X-Cache", "HIT"))
        .json(studies);
}

// 2. 캐시 미스 - DB 조회
let final_studies = query_studies(...).await?;

// 3. Redis에 저장
let json = serde_json::to_string(&final_studies)?;
redis.set_ex(&cache_key, &json, 60).await?; // 1분 TTL

HttpResponse::Ok()
    .insert_header(("X-Cache", "MISS"))
    .json(final_studies)
```

**장점**:
- ✅ DB 쿼리 완전 제거 (캐시 히트 시)
- ✅ 서버 부하 최소화
- ✅ 응답 속도 매우 빠름

**단점**:
- ⚠️ 메모리 사용량 증가
- ⚠️ 캐시 무효화 로직 복잡
- ⚠️ 데이터 일관성 문제 (TTL 동안 이전 데이터)

## 🎯 권장 사항

**1단계: Last-Modified 방식 구현** (우선순위: 높음)
- 가장 효율적이고 표준적인 방법
- DB 스키마 변경 필요하지만 장기적으로 유리
- 예상 작업 시간: 2-3시간

**2단계: Redis 캐시 추가** (우선순위: 중간)
- Last-Modified로 충분하지 않을 경우
- 트래픽이 매우 높을 때 고려
- 예상 작업 시간: 3-4시간

## 📊 예상 성능 개선

### 현재 (캐시 없음)
- 요청당 DB 쿼리: 1회
- 평균 응답 시간: 500ms
- 네트워크 트래픽: 높음

### Last-Modified 적용 후
- 요청당 DB 쿼리: 0.1회 (변경 안됐을 때 타임스탬프만)
- 평균 응답 시간: 50ms (304) / 500ms (200)
- 네트워크 트래픽: 낮음 (304 응답 90%)
- **예상 서버 부하 감소: 80-90%**

## 📝 체크리스트

- [ ] DB 스키마 확인 (`studies.updated_at` 컬럼)
- [ ] Migration 작성 (필요 시)
- [ ] TimePoint 할당/해제 시 `updated_at` 업데이트 로직 추가
- [ ] HTTP 날짜 파싱 유틸리티 구현
- [ ] DICOM Gateway 컨트롤러에 Last-Modified 로직 추가
- [ ] E2E 테스트 작성
- [ ] 성능 테스트 (Before/After 비교)
- [ ] 문서 업데이트

