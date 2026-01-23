# QIDO Cache E2E Tests

## Overview

Comprehensive E2E tests for QIDO-RS response caching functionality using Redis.

## Test Coverage

### ✅ Test 1: Series Cache MISS → HIT
- **Purpose**: Verify cache MISS on first request, HIT on subsequent requests
- **Endpoint**: `/api/me/dicom/studies/{study_uid}/series?project_id={project_id}`
- **Expected Behavior**:
  - First request: Cache MISS (QIDO call to Dcm4chee)
  - Second request: Cache HIT (Redis retrieval)
  - Data consistency between requests

### ✅ Test 2: Series Cache Expiry
- **Purpose**: Verify cache expires after TTL (60 seconds)
- **Expected Behavior**:
  - Cache HIT within 60 seconds
  - Cache MISS after 60 seconds
  - Data consistency after expiry
- **Note**: Skipped by default (takes 65+ seconds)

### ✅ Test 3: Different Cache Entries per Project
- **Purpose**: Verify separate cache entries for different project_id parameters
- **Endpoints**:
  - `/api/me/dicom/studies/{study_uid}/series?project_id=2`
  - `/api/me/dicom/studies/{study_uid}/series` (no project_id)
- **Expected Behavior**:
  - Different cache keys for different project_id values
  - Independent cache entries

### ✅ Test 4: Studies Cache
- **Purpose**: Verify caching for studies endpoint
- **Endpoint**: `/api/me/dicom/studies?project_id={project_id}`
- **Expected Behavior**:
  - First request: Cache MISS
  - Second request: Cache HIT
  - Data consistency

### ✅ Test 5: Cache Performance Improvement
- **Purpose**: Measure performance improvement from caching
- **Methodology**:
  - Measure average Cache MISS time (3 samples)
  - Measure average Cache HIT time (5 samples)
  - Calculate improvement percentage
- **Expected**: Cache HIT should be faster or comparable to MISS
- **Note**: Skipped by default (takes ~200 seconds)

### ✅ Test 6: Concurrent Cache Requests
- **Purpose**: Verify cache handles concurrent requests correctly
- **Methodology**:
  - Populate cache with initial request
  - Make 10 concurrent requests
  - Verify all return same data
- **Expected Behavior**:
  - All concurrent requests return identical data
  - No race conditions or data corruption

## Running Tests

### Quick Test (Fast tests only)
```bash
python3 pacs-server/e2e/test_qido_cache_e2e.py
```

**Duration**: ~5 seconds  
**Tests**: 1, 3, 4, 6

### Full Test Suite
```bash
python3 pacs-server/e2e/test_qido_cache_e2e.py --full
```

**Duration**: ~200 seconds  
**Tests**: All tests including performance benchmarks

## Test Results

### Performance Metrics

| Test | Metric | Value |
|------|--------|-------|
| Series Cache MISS | Response Time | ~0.15-0.30s |
| Series Cache HIT | Response Time | ~0.10-0.15s |
| Studies Cache MISS | Response Time | ~0.30-0.50s |
| Studies Cache HIT | Response Time | ~0.30-0.50s |
| Concurrent Requests (10) | Total Time | ~1.0s |
| Concurrent Requests (10) | Avg per Request | ~0.10s |

### Cache Behavior

- **TTL**: 60 seconds (configurable via `QIDO_CACHE_TTL_SEC`)
- **Cache Keys**:
  - Series: `qido:series:{study_uid}:p{project_id}:h{params_hash}`
  - Studies: `qido:studies:p{project_id}:h{params_hash}`
- **Storage**: Redis
- **Serialization**: JSON

## Verification

### Check Cache Logs

```bash
tail -50 backend.log | grep -E "(⚡|🔄)"
```

**Expected Output**:
```
[INFO] ⚡ Cache HIT - study=1.2.410..., project=2
[INFO] 🔄 Cache MISS - study=1.2.410..., project=2
[INFO] ⚡ Cache HIT - studies for project=2
[INFO] 🔄 Cache MISS - studies for project=2
```

### Check Redis Keys

```bash
redis-cli KEYS "qido:*"
```

**Expected Output**:
```
1) "qido:series:1.2.410.200022.500.202205101053010.12252192375:p2:h..."
2) "qido:studies:p2:h..."
```

## Troubleshooting

### Cache Not Working

1. **Check Redis Connection**:
   ```bash
   redis-cli PING
   ```

2. **Check Server Logs**:
   ```bash
   tail -100 backend.log | grep -i cache
   ```

3. **Verify Environment Variables**:
   ```bash
   echo $QIDO_CACHE_TTL_SEC  # Should be 60 or custom value
   ```

### Cache Always MISS

1. **Check TTL**: Ensure cache hasn't expired
2. **Check Parameters**: Different query parameters create different cache keys
3. **Check Project ID**: Different project_id values create different cache keys

## Implementation Details

### Cached Endpoints

1. **Series Endpoint**:
   - `GET /api/me/dicom/studies/{study_uid}/series`
   - `GET /api/dicom/studies/{study_uid}/series`

2. **Studies Endpoint**:
   - `GET /api/me/dicom/studies`

### Cache Service

- **Location**: `pacs-server/src/infrastructure/services/qido_cache_service.rs`
- **Methods**:
  - `get_series()` / `set_series()`
  - `get_studies()` / `set_studies()`
  - `hash_params()` - Generate consistent cache keys

### Cache Strategy

- **Write**: Fire-and-forget async write (doesn't block response)
- **Read**: Synchronous read with fallback to QIDO on cache miss
- **Expiry**: TTL-based automatic expiration
- **Invalidation**: Manual invalidation methods available (not yet implemented in endpoints)

