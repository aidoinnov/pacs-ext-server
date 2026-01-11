# 동기화 시스템 사용 가이드

## 🚀 빠른 시작

### 1. 동기화 실행

```bash
# 수동으로 동기화 실행
curl -X POST http://localhost:8080/api/sync/run
```

**응답 예시**:
```json
{
  "success": true,
  "processed": 1089,
  "duration_ms": 32407,
  "error": null
}
```

### 2. 동기화 상태 확인

```bash
# 현재 상태 조회
curl http://localhost:8080/api/sync/status
```

**응답 예시**:
```json
{
  "is_running": false,
  "last_run": "2025-12-25T05:13:08.929248+00:00",
  "next_run": "2025-12-25T05:18:08.929261+00:00",
  "interval_sec": 300
}
```

---

## 📋 주요 사용 시나리오

### 시나리오 1: 수동 동기화 실행

데이터를 즉시 동기화해야 할 때:

```bash
curl -X POST http://localhost:8080/api/sync/run
```

### 시나리오 2: 동기화 일시 중지

유지보수나 문제 해결을 위해 동기화를 일시 중지:

```bash
# 일시 중지
curl -X POST http://localhost:8080/api/sync/pause

# 상태 확인
curl http://localhost:8080/api/sync/status
# "is_running": false, "paused": true

# 재개
curl -X POST http://localhost:8080/api/sync/resume
```

### 시나리오 3: 동기화 간격 변경

동기화 빈도를 조정:

```bash
# 현재 간격 확인
curl http://localhost:8080/api/sync/schedule

# 간격 변경 (10분 = 600초)
curl -X PUT http://localhost:8080/api/sync/schedule \
  -H "Content-Type: application/json" \
  -d '{"interval_sec": 600}'
```

### 시나리오 4: 동기화 모니터링

동기화 상태를 지속적으로 모니터링:

```bash
# 상태 확인 스크립트
watch -n 5 'curl -s http://localhost:8080/api/sync/status | jq'
```

---

## 🔍 동기화 프로세스 이해

### 동기화 단계

1. **Study 동기화**
   - Dcm4chee에서 Study 조회
   - RBAC DB에 INSERT/UPDATE
   - 프로젝트에 할당

2. **Series 동기화**
   - Dcm4chee에서 Series 조회
   - RBAC DB에 INSERT/UPDATE

3. **Instance 동기화**
   - Dcm4chee에서 Instance 조회
   - RBAC DB에 INSERT/UPDATE

4. **정리 작업 (Cleanup)**
   - PACS에 없는 Study 삭제
   - PACS에 없는 Series 삭제
   - PACS에 없는 Instance 삭제

### 델타 동기화

- 첫 실행: 모든 데이터 동기화
- 이후 실행: `last_run` 이후 변경된 데이터만 동기화
- 효율적인 증분 동기화로 성능 최적화

---

## ⚠️ 주의사항

### 1. 데이터 삭제

동기화는 **PACS에 없는 데이터를 자동으로 삭제**합니다.

- Study 삭제 시 → 관련 Series, Instance, project_data도 함께 삭제 (CASCADE)
- Series 삭제 시 → 관련 Instance, project_data도 함께 삭제 (CASCADE)
- Instance 삭제 시 → 관련 project_data도 함께 삭제 (CASCADE)

**주의**: 수동으로 할당한 데이터도 PACS에 없으면 삭제될 수 있습니다.

### 2. 동시 실행

동기화는 **한 번에 하나만 실행**됩니다.

- 이미 실행 중인 동기화가 있으면 새로운 요청은 대기하거나 실패할 수 있습니다.
- `is_running: true`일 때는 추가 실행을 피하세요.

### 3. 타임아웃

동기화 API는 **60초 타임아웃**이 있습니다.

- 대량의 데이터가 있으면 타임아웃이 발생할 수 있습니다.
- 타임아웃 발생 시 백그라운드에서 계속 실행될 수 있습니다.

### 4. 성능 영향

동기화는 **DB 리소스를 많이 사용**합니다.

- 동기화 중에는 API 응답이 느려질 수 있습니다.
- 운영 시간대에 동기화를 실행할 때는 주의가 필요합니다.

---

## 🛠️ 문제 해결

### 문제 1: 동기화가 실행되지 않음

**증상**: `POST /api/sync/run` 호출 시 에러 또는 응답 없음

**해결 방법**:
1. 서버 모드 확인: `ServerMode::Full` 또는 `ServerMode::SyncOnly`인지 확인
2. Dcm4chee DB 연결 확인: 설정 파일 확인
3. 로그 확인: `backend.log`에서 에러 메시지 확인

```bash
# 로그 확인
tail -f backend.log | grep -i sync
```

### 문제 2: 타임아웃 발생

**증상**: `"error": "Sync operation timed out"`

**해결 방법**:
1. 대량 데이터 처리: 처리량 제한 조정
2. DB 성능 확인: Dcm4chee DB와 RBAC DB 성능 확인
3. 타임아웃 증가: 필요 시 타임아웃 시간 증가

### 문제 3: 데이터가 삭제되지 않음

**증상**: PACS에 없는 데이터가 여전히 존재

**해결 방법**:
1. PACS 연결 확인: Dcm4chee DB 연결 상태 확인
2. 쿼리 확인: PACS에 실제로 데이터가 있는지 확인
3. 로그 확인: cleanup 작업의 로그 확인

```bash
# cleanup 로그 확인
tail -f backend.log | grep -i cleanup
```

### 문제 4: 동기화가 너무 느림

**증상**: 동기화에 시간이 오래 걸림

**해결 방법**:
1. 처리량 조정: `LIMIT` 값 조정
2. 델타 동기화 활용: `last_run` 이후 데이터만 동기화
3. DB 인덱스 확인: `updated_time` 컬럼에 인덱스 확인

---

## 📊 모니터링

### 주요 메트릭

1. **처리된 항목 수** (`processed`)
   - Study, Series, Instance의 총 처리 수
   - 정상 범위: 데이터 양에 따라 다름

2. **소요 시간** (`duration_ms`)
   - 동기화에 걸린 시간
   - 정상 범위: 데이터 양에 따라 다름 (수십 초 ~ 수 분)

3. **삭제된 항목 수** (로그에서 확인)
   - PACS에 없어서 삭제된 항목 수
   - 정상 범위: 데이터 변경량에 따라 다름

### 로그 모니터링

```bash
# 동기화 관련 로그만 필터링
tail -f backend.log | grep -E "\[Sync\]|cleanup|deleted"

# 에러만 확인
tail -f backend.log | grep -E "\[Sync\].*error|❌|Failed"
```

---

## 🔄 자동 동기화

### 스케줄러

서버 시작 시 자동으로 스케줄러가 시작됩니다.

- **기본 간격**: 300초 (5분)
- **설정 변경**: `PUT /api/sync/schedule` 또는 설정 파일

### 스케줄러 동작

1. 설정된 간격마다 자동 실행
2. `paused` 상태면 실행하지 않음
3. 이미 실행 중이면 다음 주기까지 대기

---

## 📝 예제 스크립트

### 동기화 상태 모니터링

```bash
#!/bin/bash
# sync-monitor.sh

while true; do
  echo "=== $(date) ==="
  curl -s http://localhost:8080/api/sync/status | jq
  sleep 10
done
```

### 동기화 실행 및 결과 확인

```bash
#!/bin/bash
# sync-run.sh

echo "Starting sync..."
RESULT=$(curl -s -X POST http://localhost:8080/api/sync/run)

SUCCESS=$(echo $RESULT | jq -r '.success')
PROCESSED=$(echo $RESULT | jq -r '.processed')
DURATION=$(echo $RESULT | jq -r '.duration_ms')

if [ "$SUCCESS" = "true" ]; then
  echo "✅ Sync completed successfully"
  echo "   Processed: $PROCESSED items"
  echo "   Duration: ${DURATION}ms"
else
  echo "❌ Sync failed"
  echo $RESULT | jq -r '.error'
fi
```

---

## 🔗 관련 문서

- [구현 상세](./README.md#구현-상세)
- [아키텍처 결정](./architecture-decision.md)
- [API 문서](../../api/dicom-gateway-api.md)





