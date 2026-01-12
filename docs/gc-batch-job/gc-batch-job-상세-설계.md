# 📄 GC Batch Job 상세 설계

## Snapshot / Mask 공통 자산 정리 배치

---

## 1. 목적과 범위

### 1.1 목적

* S3(또는 외부 저장소)에 누적되는 **orphan / stale / failed** 자산을 안전하게 삭제하여

  * 스토리지 비용을 제어하고
  * 데이터 정합성을 유지하며
  * 운영/감사 가능한 형태로 로그를 남긴다.

### 1.2 범위

* **Snapshot**: `annotations/{studyUid}/{annotationId}/v{annotationVersion}.*`
* **Mask**: `masks/{studyUid}/{annotationId}/{groupId}/v{groupVersion}.*` (또는 현행 규칙)

---

## 2. 전제 데이터 모델 (DB 기준)

### 2.1 Annotations 테이블 최소 컬럼

* `id`
* `version`
* `snapshot_status` (`NONE|PENDING|READY|FAILED`)
* `snapshot_image_key` (nullable)
* `snapshot_version` (nullable)
* `updated_at` / `deleted_at`(있으면)

### 2.2 Mask 그룹 테이블(예시)

* `annotation_id`
* `group_id`
* `group_version`
* `mask_key`
* `status` (`READY|FAILED` 등)
* `updated_at`

> 실제 mask 테이블명이 다르면 “동일한 의미의 컬럼”으로 매핑해서 적용.

---

## 3. 삭제 정책 파라미터 (운영 설정)

### 3.1 공통 파라미터 (Config)

* `DRY_RUN` (기본 true → 운영 검증 후 false)
* `BATCH_SIZE_DB` (예: 1000)
* `BATCH_SIZE_S3_DELETE` (예: 1000, S3 DeleteObjects 최대 1000)
* `MAX_CONCURRENCY_S3` (예: 10~50)
* `GRACE_SNAPSHOT_STALE_DAYS` (예: 7)
* `GRACE_SNAPSHOT_FAILED_DAYS` (예: 3)
* `GRACE_MASK_STALE_DAYS` (예: 30)
* `GRACE_MASK_FAILED_DAYS` (예: 7)
* `PENDING_TIMEOUT_DAYS` (예: 1~3)

  * PENDING이 지나치게 오래면 FAILED로 전환(선택)

### 3.2 실행 주기

* 기본: **매일 1회 새벽** (예: 03:00)
* 대규모/비용 민감: 하루 2~4회도 가능

---

## 4. 배치 설계 개요: “DB 중심 GC”가 기본

### 왜 DB 중심인가?

* **정합성 판단(존재/버전/상태)은 DB가 유일하게 안다.**
* S3 Lifecycle만으로는 “이 오브젝트가 유효한지”를 알 수 없음.

따라서 기본 전략은:

> **DB에서 삭제 대상 key를 선정 → S3 삭제 → 결과 기록**

(= “DB-driven deletion”)

---

## 5. 잡 구성: 3개의 서브잡으로 분리

1. **DB 상태 정리(Job A)**: 오래된 `PENDING`을 `FAILED`로 정리(선택)
2. **Snapshot GC(Job B)**: snapshot 대상 선정/삭제/기록
3. **Mask GC(Job C)**: mask 대상 선정/삭제/기록

---

## 6. Job A: PENDING 타임아웃 정리 (선택)

### 6.1 목적

* 업로드 큐가 영원히 남아 “PENDING 폭증”하는 것을 방지
* GC/운영 리포트에서 실패 상태를 명확히

### 6.2 SQL (예시)

```sql
UPDATE annotations
SET snapshot_status = 'FAILED',
    updated_at = NOW()
WHERE snapshot_status = 'PENDING'
  AND updated_at < NOW() - INTERVAL ':PENDING_TIMEOUT_DAYS days';
```

> `updated_at` 대신 `snapshot_last_attempt_at` 같은 컬럼이 있으면 더 정확함(없으면 updated_at로 근사).

---

## 7. Job B: Snapshot GC 상세

### 7.1 삭제 대상 분류

#### B-1) Orphan Snapshot (즉시 삭제)

* S3에 오브젝트가 있는데 DB에 annotation이 없음
  → DB-driven 방식에서는 “annotation 없음”을 직접 찾기 어렵기 때문에 **S3 스캔 보조 모드**가 필요(뒤에서 설명).

#### B-2) Stale Snapshot (지연 삭제)

* `snapshot_version < annotations.version` 인 과거 버전 이미지
* “히스토리 유지 정책”이 없다면 삭제 대상

#### B-3) FAILED 장기 잔여물 (지연 삭제)

* `snapshot_status = FAILED`이고 grace 기간 초과

### 7.2 DB-driven 대상 선정 SQL

#### B-2: “현재 annotation 기준으로 stale 스냅샷 키”를 DB에서 알 수 있나?

* **DB에 과거 snapshot key를 저장하지 않으면**(= 최신 1개만 저장하는 모델) DB만으로는 “과거 버전 파일 목록”을 만들 수 없어.
* 그래서 Snapshot stale GC는 2가지 옵션 중 하나를 선택해야 함:

---

### ✅ 옵션 1 (권장): **S3 prefix 스캔 + DB 검증** (히스토리 테이블 없이도 가능)

* S3에서 `annotations/{studyUid}/{annotationId}/` prefix로 오브젝트 목록을 가져온 뒤
* 각 오브젝트에서 `{annotationId, version}`을 파싱하여 DB와 대조
* grace 적용 후 삭제

장점: 테이블 추가 불필요, 기존 구조 유지
단점: S3 LIST 비용/시간 증가 (prefix가 많으면 최적화 필요)

---

### ✅ 옵션 2: **snapshot 히스토리 테이블 도입** (대규모 운영에 유리)

* `annotation_snapshots(annotation_id, annotation_version, key, status, created_at)` 유지
* stale/failed 목록을 DB에서 정확히 선정 가능
* S3 LIST 최소화

장점: 비용/속도/정확도 최고
단점: 스키마 추가/저장 로직 필요

> 너희는 이미 “mask-group 업로드”처럼 자산을 다루고 있어서, 장기적으로는 옵션 2가 운영 난이도가 낮아짐.

---

### 7.3 (옵션 1) Snapshot S3 스캔 기반 GC 설계

#### B-Scan-1: 스캔 단위

* Prefix 규칙: `annotations/{studyUid}/{annotationId}/`
* 대규모면 “studyUid 단위 prefix”로 1차 분할:

  * `annotations/{studyUid}/` → 그 하위에서 annotationId 분기

#### B-Scan-2: 오브젝트 key 파싱

* key에서 추출:

  * `annotationId`
  * `annotationVersion` (v{n} 파싱)
* 파싱 실패(규칙 위반)는 “즉시 GC 후보”로 분류 가능(보수적으로는 quarantine)

#### B-Validate: DB 검증

* `SELECT version, snapshot_status, updated_at FROM annotations WHERE id=:annotationId`

  * row 없음 → orphan 후보
  * row 있음:

    * `objectVersion == row.version` AND row.snapshot_status == READY → keep
    * `objectVersion < row.version` → stale 후보(그리고 updated_at 기준 grace 체크)
    * `objectVersion == row.version` BUT row.snapshot_status != READY → 보수적으로 keep(또는 별도 정책)
    * `objectVersion > row.version` → 비정상(삭제 금지 + 알림)

#### B-Grace: 유예 기준

* stale: annotation이 업데이트 된 시점 기준으로 `GRACE_SNAPSHOT_STALE_DAYS`

  * 예: `annotations.updated_at < now - GRACE_STALE`
* orphan: 즉시 삭제 가능하지만, 안전을 위해 1~3일 유예 추천

---

### 7.4 (옵션 2) Snapshot 히스토리 테이블 기반 GC 설계

#### 대상 선정 SQL 예시

* **구버전 삭제 후보**

```sql
SELECT s.snapshot_image_key AS key
FROM annotation_snapshots s
JOIN annotations a ON a.id = s.annotation_id
WHERE s.annotation_version < a.version
  AND s.created_at < NOW() - INTERVAL ':GRACE_SNAPSHOT_STALE_DAYS days'
LIMIT :BATCH_SIZE_DB;
```

* **FAILED 장기 후보**

```sql
SELECT s.snapshot_image_key AS key
FROM annotation_snapshots s
WHERE s.status = 'FAILED'
  AND s.created_at < NOW() - INTERVAL ':GRACE_SNAPSHOT_FAILED_DAYS days'
LIMIT :BATCH_SIZE_DB;
```

* 삭제 후 `annotation_snapshots`에서 row 제거 or `deleted_at` 마킹(감사 요건에 따라 결정)

---

## 8. Job C: Mask GC 상세

Mask는 “그룹/버전”이 있고, 대체로 DB에 key가 존재하는 편이라 **DB-driven이 더 쉬움**.

### 8.1 대상 선정 규칙

* annotation 삭제됨 → 해당 annotation_id의 mask 전부 삭제
* mask group 삭제됨 → 해당 group 키 삭제
* FAILED 장기 → grace 후 삭제
* (선택) 구버전 mask 유지 정책에 따라 삭제

### 8.2 DB 대상 선정 SQL 예시

* **Annotation 삭제 연계(soft delete일 때)**

```sql
SELECT mg.mask_key AS key
FROM mask_groups mg
JOIN annotations a ON a.id = mg.annotation_id
WHERE a.deleted_at IS NOT NULL
LIMIT :BATCH_SIZE_DB;
```

* **FAILED 장기**

```sql
SELECT mg.mask_key AS key
FROM mask_groups mg
WHERE mg.status = 'FAILED'
  AND mg.updated_at < NOW() - INTERVAL ':GRACE_MASK_FAILED_DAYS days'
LIMIT :BATCH_SIZE_DB;
```

* **구버전**

```sql
SELECT mg.mask_key AS key
FROM mask_groups mg
WHERE mg.group_version < mg.current_version_ref  -- 구조에 맞게 매핑
  AND mg.updated_at < NOW() - INTERVAL ':GRACE_MASK_STALE_DAYS days'
LIMIT :BATCH_SIZE_DB;
```

> 실제 mask 스키마에 “current_version_ref”가 없다면, 구버전 GC는 히스토리 테이블/키 규칙 기반 스캔으로 처리.

---

## 9. 삭제 실행부(S3) 설계

### 9.1 DeleteObjects 배치 삭제

* S3 DeleteObjects는 최대 1000개 단위
* 실패 key는 재시도 큐로 분리

### 9.2 재시도 정책

* 네트워크/5xx: 지수 백오프 3회
* 403/AccessDenied: 즉시 실패 + 알림(권한 문제)
* NoSuchKey: “이미 없음”으로 성공 처리 가능(멱등)

---

## 10. 안전장치

### 10.1 Dry-run 모드

* 실제 삭제 대신:

  * 삭제 후보 key 로그만 남김
  * 샘플 100개 정도를 운영자가 검증 가능하게 리포트

### 10.2 Quarantine(선택)

* 곧바로 delete 대신 prefix 이동:

  * `quarantine/annotations/...`
* S3에서 “Move”는 Copy+Delete라 비용 증가 → 의료/감사 요구가 강하면 고려

### 10.3 삭제 전 최종 검증(선택)

* 대량 삭제 전, 일부 key에 대해 HEAD 확인
* 비용 vs 안전 트레이드오프

---

## 11. 감사/로그/메트릭

### 11.1 GC 로그 테이블(권장)

운영/감사/장애 대응을 위해 최소 테이블 하나 두는 게 좋음.

```text
gc_deletion_log
---------------
id
resource_type   (snapshot|mask)
s3_key
annotation_id   (nullable)
version         (nullable)
reason          (orphan|stale|failed|annotation_deleted|policy_violation)
dry_run         boolean
status          (success|failed)
error_code      (nullable)
created_at
```

### 11.2 메트릭

* `gc.candidates.count`
* `gc.deleted.count`
* `gc.failed.count`
* `gc.s3.delete.latency`
* `gc.db.query.latency`

### 11.3 알림(필수)

* 실패율 급증
* 파싱 실패 key 증가(규칙 위반)
* “objectVersion > annotation.version” 감지 (데이터 오염 신호)

---

## 12. 권한(IAM) 최소화

GC 워커 IAM:

* `s3:ListBucket` (필요 시, prefix 제한)
* `s3:DeleteObject` (prefix 제한)
* `s3:GetObject` (HEAD/검증 필요 시)

가능하면 bucket/prefix로 scope 제한:

* `arn:aws:s3:::pacs-assets/annotations/*`
* `arn:aws:s3:::pacs-assets/masks/*`

---

## 13. 구현 워크플로우(의사코드)

### 13.1 DB-driven deletion 공통

```pseudo
for each resourceType in [snapshot, mask]:
  while true:
    keys = queryDeletionCandidates(resourceType, limit=BATCH_SIZE_DB)
    if keys empty: break

    if DRY_RUN:
      logCandidates(keys)
      continue

    results = s3DeleteObjects(keys, chunk=1000, concurrency=MAX_CONCURRENCY_S3)

    writeDeletionLog(results)
    markDbIfNeeded(results)  // history table 있으면 delete/soft delete
```

### 13.2 S3-scan + DB-validate (옵션 1)

```pseudo
for each prefix in scanPrefixes():
  objects = s3List(prefix)
  for obj in objects:
    parsed = parseKey(obj.key)
    if parseFail:
      candidate(policy_violation)
      continue

    a = dbGetAnnotation(parsed.annotationId)
    if not a:
      candidate(orphan)
    else:
      if parsed.version < a.version and a.updated_at < now - GRACE_STALE:
        candidate(stale)
      else:
        keep
delete(candidates)
```

---

## 14. 운영 런북(최소)

* 첫 1주일:

  * DRY_RUN=true로 매일 리포트 확인
  * 삭제 후보 샘플링 검증
* 이후:

  * DRY_RUN=false 전환
  * 실패율/삭제량 모니터링
* 사고 대응:

  * 특정 prefix 삭제 중단(필터 룰)
  * 롤백은 불가(삭제이므로) → quarantine 옵션이 있으면 복구 가능

---

## 15. 추천 결론

* **초기(빠르게)**: 옵션 1(S3 스캔 + DB 검증)으로 시작 가능
* **장기(운영 최적)**: 옵션 2(snapshot 히스토리 테이블)로 전환 추천

  * 비용/시간/정확도/감사 대응이 훨씬 좋아짐

---

원하면 다음 중 하나를 바로 더 구체화해줄게(질문 안 하고 내가 정해서도 가능하지만, 선택하면 더 딱 맞아짐):

1. **너희 실제 테이블명/키 규칙**에 맞춰 SQL/파서 규칙을 “실제값”으로 확정
2. GC 워커를 **Go/Rust/Python** 중 하나로 구현할 때의 구체 코드 구조
3. Kubernetes CronJob spec (리소스/스케줄/재시도/알림 포함)
