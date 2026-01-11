# DICOM 데이터 동기화 시스템

## 📋 개요

PACS Extension Server의 DICOM 데이터 동기화 시스템은 Dcm4chee 데이터베이스와 RBAC 데이터베이스 간의 데이터를 동기화하는 기능을 제공합니다.

### 주요 기능

1. **델타 동기화**: 마지막 실행 시간 이후 변경된 데이터만 동기화
2. **자동 정리**: PACS에 없는 데이터 자동 삭제
3. **API 제어**: REST API를 통한 동기화 제어
4. **스케줄링**: 주기적 자동 동기화

---

## 🏗️ 아키텍처

### 현재 구조 (통합 서버)

```
┌─────────────────────────────────────────────────┐
│         PACS Extension Server                   │
│  ┌──────────────────────────────────────────┐  │
│  │  API Layer                                │  │
│  │  - /api/sync/*                            │  │
│  └──────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────┐  │
│  │  Sync Service                             │  │
│  │  - SyncWorker (동기화 로직)                │  │
│  │  - SyncScheduler (스케줄러)                │  │
│  │  - SyncState (상태 관리)                   │  │
│  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
         │                    │
         ▼                    ▼
┌──────────────┐    ┌──────────────┐
│  RBAC DB     │    │ Dcm4chee DB  │
│ (PostgreSQL) │    │ (PostgreSQL) │
└──────────────┘    └──────────────┘
```

### 동기화 흐름

1. **Study 동기화**
   - Dcm4chee의 `study` 테이블에서 데이터 조회
   - `project_data_study` 테이블에 INSERT/UPDATE
   - `project_data` 테이블에 프로젝트 할당

2. **Series 동기화**
   - Dcm4chee의 `series` 테이블에서 데이터 조회
   - `project_data_series` 테이블에 INSERT/UPDATE

3. **Instance 동기화**
   - Dcm4chee의 `instance` 테이블에서 데이터 조회
   - `project_data_instance` 테이블에 INSERT/UPDATE

4. **정리 작업 (Cleanup)**
   - PACS에 없는 Study/Series/Instance 삭제
   - CASCADE DELETE로 관련 데이터 자동 삭제

---

## 📡 API 엔드포인트

### 1. 동기화 실행

```http
POST /api/sync/run
```

**설명**: 수동으로 동기화를 즉시 실행합니다.

**응답 예시**:
```json
{
  "success": true,
  "processed": 1089,
  "duration_ms": 32407,
  "error": null
}
```

**필드 설명**:
- `success`: 동기화 성공 여부
- `processed`: 처리된 항목 수 (Study + Series + Instance)
- `duration_ms`: 소요 시간 (밀리초)
- `error`: 에러 메시지 (있을 경우)

### 2. 동기화 상태 조회

```http
GET /api/sync/status
```

**설명**: 현재 동기화 상태를 조회합니다.

**응답 예시**:
```json
{
  "is_running": false,
  "last_run": "2025-12-25T05:13:08.929248+00:00",
  "next_run": "2025-12-25T05:18:08.929261+00:00",
  "interval_sec": 300
}
```

**필드 설명**:
- `is_running`: 현재 동기화 실행 중 여부
- `last_run`: 마지막 실행 시간
- `next_run`: 다음 예정 실행 시간
- `interval_sec`: 자동 실행 간격 (초)

### 3. 동기화 일시 중지

```http
POST /api/sync/pause
```

**설명**: 자동 동기화를 일시 중지합니다.

### 4. 동기화 재개

```http
POST /api/sync/resume
```

**설명**: 일시 중지된 자동 동기화를 재개합니다.

### 5. 스케줄 조회

```http
GET /api/sync/schedule
```

**응답 예시**:
```json
{
  "interval_sec": 300
}
```

### 6. 스케줄 변경

```http
PUT /api/sync/schedule
Content-Type: application/json

{
  "interval_sec": 600
}
```

**설명**: 자동 동기화 간격을 변경합니다 (초 단위).

---

## 🔧 구현 상세

### 동기화 로직

#### 1. Study 동기화 (`sync_studies`)

```rust
// 델타 동기화: last_run 이후 변경된 Study만 조회
SELECT st.study_iuid, st.study_desc, NULL::text AS patient_id, 
       st.study_date, st.updated_time
FROM study st
LEFT JOIN patient pt ON st.patient_fk = pt.pk
WHERE st.updated_time > $1
ORDER BY st.updated_time ASC
LIMIT 500

// RBAC DB에 INSERT/UPDATE
INSERT INTO project_data_study (study_uid, study_description, patient_id, study_date)
VALUES ($1, $2, $3, to_date($4, 'YYYYMMDD'))
ON CONFLICT (study_uid)
DO UPDATE SET study_description = EXCLUDED.study_description,
              patient_id = EXCLUDED.patient_id,
              study_date = EXCLUDED.study_date
```

#### 2. Series 동기화 (`sync_series`)

```rust
// 델타 동기화: last_run 이후 변경된 Series만 조회
SELECT se.series_iuid, se.series_desc, se.modality, 
       st.study_iuid, se.updated_time
FROM series se
JOIN study st ON se.study_fk = st.pk
WHERE se.updated_time > $1
ORDER BY se.updated_time ASC
LIMIT 1000

// RBAC DB에 INSERT/UPDATE
INSERT INTO project_data_series (study_id, series_uid, series_description, modality)
VALUES ($1, $2, $3, $4)
ON CONFLICT (study_id, series_uid)
DO UPDATE SET series_description = EXCLUDED.series_description,
              modality = EXCLUDED.modality
```

#### 3. Instance 동기화 (`sync_instances`)

```rust
// 델타 동기화: last_run 이후 변경된 Instance만 조회
SELECT i.sop_iuid, i.sop_cuid, i.inst_no, i.content_date, 
       i.content_time, se.series_iuid, i.updated_time
FROM instance i
JOIN series se ON i.series_fk = se.pk
WHERE i.updated_time > $1
ORDER BY i.updated_time ASC
LIMIT 2000

// RBAC DB에 INSERT/UPDATE
INSERT INTO project_data_instance (series_id, instance_uid, sop_class_uid, 
                                   instance_number, content_date, content_time)
VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (series_id, instance_uid)
DO UPDATE SET sop_class_uid = EXCLUDED.sop_class_uid,
              instance_number = EXCLUDED.instance_number,
              content_date = EXCLUDED.content_date,
              content_time = EXCLUDED.content_time
```

### 정리 작업 (Cleanup)

#### 1. Study 정리 (`cleanup_missing_studies`)

```rust
// PACS에 있는 모든 Study UID 조회
SELECT DISTINCT study_iuid FROM study

// RBAC DB에서 PACS에 없는 Study 삭제
DELETE FROM project_data_study 
WHERE study_uid NOT IN (SELECT unnest($1::text[]))
```

**CASCADE DELETE**: Study 삭제 시 관련 Series, Instance, project_data도 자동 삭제

#### 2. Series 정리 (`cleanup_missing_series`)

```rust
// PACS에 있는 모든 Series UID 조회
SELECT DISTINCT series_iuid FROM series

// RBAC DB에서 PACS에 없는 Series 삭제
DELETE FROM project_data_series 
WHERE series_uid NOT IN (SELECT unnest($1::text[]))
```

**CASCADE DELETE**: Series 삭제 시 관련 Instance, project_data도 자동 삭제

#### 3. Instance 정리 (`cleanup_missing_instances`)

```rust
// PACS에 있는 모든 Instance UID 조회
SELECT DISTINCT sop_iuid FROM instance

// RBAC DB에서 PACS에 없는 Instance 삭제
DELETE FROM project_data_instance 
WHERE instance_uid NOT IN (SELECT unnest($1::text[]))
```

**CASCADE DELETE**: Instance 삭제 시 관련 project_data도 자동 삭제

---

## ⚙️ 설정

### 환경 변수

동기화 기능은 다음 설정을 사용합니다:

```toml
[sync]
interval_sec = 300  # 자동 동기화 간격 (초)
default_project_id = 1  # 기본 프로젝트 ID

[dcm4chee.db]
host = "localhost"
port = 5432
username = "dcm4chee"
password = "password"
database = "dcm4chee"
```

### 서버 모드

동기화 기능은 서버 모드에 따라 동작이 달라집니다:

- **Full**: 모든 기능 활성화 (API + 동기화)
- **SyncOnly**: 동기화 기능만 활성화
- **ApiOnly**: API만 활성화 (동기화 비활성화)

---

## 📊 성능 및 제한사항

### 처리량

- **Study**: 최대 500개/실행
- **Series**: 최대 1000개/실행
- **Instance**: 최대 2000개/실행

### 타임아웃

- **API 타임아웃**: 60초
- **DB 연결 타임아웃**: 설정에 따라 다름

### 주의사항

1. **대량 데이터**: 대량의 데이터가 있는 경우 동기화에 시간이 오래 걸릴 수 있습니다.
2. **CASCADE DELETE**: Study/Series 삭제 시 관련 데이터가 모두 삭제되므로 주의가 필요합니다.
3. **동시 실행**: 동기화는 한 번에 하나만 실행됩니다 (중복 실행 방지).

---

## 🐛 문제 해결

### 동기화가 실행되지 않음

1. 서버 모드 확인: `ServerMode::Full` 또는 `ServerMode::SyncOnly`인지 확인
2. Dcm4chee DB 연결 확인: 설정 파일의 `dcm4chee.db` 설정 확인
3. 로그 확인: `backend.log`에서 에러 메시지 확인

### 타임아웃 발생

1. 타임아웃 시간 증가: `sync_controller.rs`의 타임아웃 설정 확인
2. 처리량 조정: `LIMIT` 값 조정 (더 작게 설정)
3. DB 성능 확인: Dcm4chee DB와 RBAC DB의 성능 확인

### 데이터가 삭제되지 않음

1. PACS 연결 확인: Dcm4chee DB 연결 상태 확인
2. 쿼리 확인: PACS에 실제로 데이터가 있는지 확인
3. 로그 확인: cleanup 작업의 로그 확인

---

## 📝 변경 이력

### 2025-12-25
- PACS에 없는 데이터 삭제 기능 추가
- 타임아웃 시간 증가 (5초 → 60초)
- 테스트 모드 제거





