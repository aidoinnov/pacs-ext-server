## 🔑 설계 결론 한 줄

> CTIMS는 프로젝트 단위로 “스냅샷 → 스터디 배치 → 완료 신호” 순서로 데이터를 전송하고,프로젝트 상태(status)와 핵심 날짜(date)는 스냅샷에 반드시 포함한다.
>

---

## 🔐 인증 및 보안

### 인증 방식

CTIMS → PACS 데이터 전송 시 **이중 보안** 적용:

1. **Bearer Token (또는 API Key)**
   - 모든 요청 헤더에 포함
   ```http
   Authorization: Bearer {ctims_api_token}
   ```
   - 또는 커스텀 헤더 사용
   ```http
   X-CTIMS-API-Key: {api_key}
   ```

2. **IP 화이트리스트**
   - CTIMS 서버 IP만 허용
   - 방화벽 레벨 + 애플리케이션 레벨 이중 검증

### 보안 정책

| 항목 | 정책 |
|------|------|
| 인증 실패 시 | 401 Unauthorized 반환 |
| IP 차단 시 | 403 Forbidden 반환 |
| 토큰 만료 | CTIMS 측에서 갱신 후 재시도 |
| Rate Limiting | 분당 100회 요청 제한 (조정 가능) |

---

## 📡 API 엔드포인트

### 1단계: 프로젝트 스냅샷 수신
```http
POST /api/ctims/projects/snapshot
Authorization: Bearer {ctims_api_token}
Content-Type: application/json
```

### 2단계: 스터디 배치 수신
```http
POST /api/ctims/projects/studies/batch
Authorization: Bearer {ctims_api_token}
Content-Type: application/json
```

### 3단계: 동기화 완료 신호
```http
POST /api/ctims/projects/complete
Authorization: Bearer {ctims_api_token}
Content-Type: application/json
```

---

## 1️⃣ 1단계: 프로젝트 스냅샷 전송 (Light / 필수)

**전송 버튼 클릭 시 가장 먼저 전송**

**엔드포인트**: `POST /api/ctims/projects/snapshot`

### 📦 포함 데이터

- 프로젝트 메타데이터
- 프로젝트 상태(status)
- 프로젝트 핵심 날짜(date)
- Subject 목록
- Timepoint 정의
- 프로젝트 참여 인원 및 권한 (권한 스냅샷)
- snapshot_id

### ❌ 제외 데이터

- Study / Series / Instance (대용량이므로 제외)

### 📌 목적

- “이 프로젝트 연동을 시작한다”는 **공식 선언**
- PACS가 프로젝트 컨텍스트를 먼저 생성

### 📄 데이터 예시

```json
{
  "snapshot_id": "snap_20260122_01",
  "sent_at": "2026-01-22T10:15:00Z",

  "project": {
    "project_id": "ctims_project_001",
    "name": "Lung Cancer Trial A",
    "protocol_no": "LC-2025-01",

    "status": "ACTIVE",

    "dates": {
      "created_at": "2025-10-01",
      "start_date": "2025-11-01",
      "end_date": null,
      "last_updated_at": "2026-01-22T09:50:00Z"
    }
  },

  "subjects": [
    {
      "subject_id": "SUBJ_001",
      "external_key": "HOSP-0001"
    }
  ],

  "timepoints": [
    {
      "timepoint_id": "TP_001",
      "name": "Baseline",
      "visit_type": "BASELINE",
      "visit_no": 1
    }
  ],

  "members": [
    {
      "user_id": "USER_001",
      "role": "INVESTIGATOR",
      "permissions": ["READ", "ANNOTATE", "REPORT"]
    },
    {
      "user_id": "USER_002",
      "role": "READER",
      "permissions": ["READ"]
    }
  ]
}

```

---

## 2️⃣ 2단계: 스터디 배치 전송 (Chunked / 비동기)

**스터디 수가 많으므로 반드시 분할 전송**

**엔드포인트**: `POST /api/ctims/projects/studies/batch`

### 📦 포함 데이터

- snapshot_id
- batch_no / batch_size
- Study 목록
    - Study UID
    - Subject ID
    - Timepoint (Study에 종속)

### 📌 목적

- 대용량 데이터로 인한 DB / 네트워크 부하 방지
- 실패 시 batch 단위 재시도 가능

### 📄 데이터 예시

```json
{
  "snapshot_id": "snap_20260122_01",
  "batch_no": 1,
  "batch_size": 50,

  "studies": [
    {
      "study_uid": "1.2.840.113619.2.55.3.604688435.123",
      "subject_id": "SUBJ_001",
      "study_date": "2025-12-01",
      "modality": "CT",
      "timepoint": {
        "timepoint_id": "TP_001",
        "visit_type": "BASELINE",
        "visit_no": 1
      }
    }
  ]
}

```

> Timepoint 미지정 Study는
>
>
> `timepoint: null` 또는 `visit_type: "UNSPECIFIED"`
>

---

## 3️⃣ 3단계: 동기화 완료 신호 (권장)

**모든 스터디 배치 전송 완료 후**

**엔드포인트**: `POST /api/ctims/projects/complete`

### 📌 목적

- PACS가 “이제 이 프로젝트를 사용 가능” 상태로 전환

### 📄 데이터 예시

```json
{
  "snapshot_id": "snap_20260122_01",
  "status": "COMPLETED",
  "completed_at": "2026-01-22T10:18:30Z"
}

```

---

## 🧠 상태별 PACS 동작 기준 (암묵적 룰)

| Project Status | PACS 동작 |
| --- | --- |
| DRAFT | Viewer 제한 / 준비 단계 |
| ACTIVE | Viewer + Annotation + Report 허용 |
| CLOSED | Read-only / Report 수정 불가 |
| ARCHIVED | 조회만 가능 |

---

## ✅ 왜 이 구성이 적절한가 (요약)

- **DB 부하 안전**

    → 대용량 Study는 배치 처리

- **운영 명확성**

    → snapshot_id 기준으로 상태 추적

- **권한/상태 일관성**

    → 프로젝트 전송 시점 기준 스냅샷

- **실무 친화적**

    → 재전송, 장애 복구, 감사 대응 쉬움


---

## ✍️ 최종 한 문장 정리 (문서용)

> CTIMS는 프로젝트 전송 시 프로젝트 메타데이터 스냅샷을 우선 전달하고,
>
>
> 스터디 데이터는 배치 단위로 분할 전송한다.
>
> 프로젝트 상태 및 주요 날짜 정보는 스냅샷에 포함되어
>
> PACS의 Viewer 및 Report 동작 기준으로 활용된다.
>

---

## 🚀 구현 일정 (예상)

### 1주차: 기본 연동 구현
- CTIMS → PACS 데이터 수신 API 구현
- 프로젝트 스냅샷 처리
- 스터디 배치 처리
- 기본 에러 핸들링

### 2주차: 안정화 및 테스트
- 동기화 상태 추적 기능
- 재시도 로직
- 통합 테스트
- 성능 검증 (대용량 배치 처리)

### 3주차: 운영 준비
- 모니터링 대시보드
- 감사 로그
- 운영 문서 작성
- 스테이징 환경 배포

**예상 소요 기간**: 3주 (개발 2주 + 안정화 1주)

---

## � 기술 검토 결과

### ✅ 구현 가능성: 높음

PACS 서버는 이미 Subject & TimePoint 관리 기능을 구현 완료했으며,
CTIMS 연동에 필요한 데이터 구조가 **100% 준비되어 있습니다**.

### 🔗 데이터 호환성

| CTIMS 데이터 | PACS 기존 구조 | 상태 |
|-------------|---------------|------|
| Subject 관리 | `project_subject` 테이블 | ✅ 완료 |
| TimePoint 관리 | `subject_timepoint` 테이블 | ✅ 완료 |
| Study 할당 | `subject_timepoint_study_map` | ✅ 완료 |
| 외부 시스템 연동 | `external_subject_key` 필드 | ✅ 준비됨 |
| 권한 관리 | `project_user_matrix` | ✅ 완료 |

### 🎯 추가 개발 필요 사항

1. **CTIMS 전용 API 엔드포인트** (3개)
   - 프로젝트 스냅샷 수신
   - 스터디 배치 수신
   - 동기화 완료 처리

2. **인증 레이어**
   - Bearer Token 검증
   - IP 화이트리스트

3. **동기화 상태 추적**
   - 배치 진행 상황 모니터링
   - 실패 시 재시도 로직

---

## 📤 에러 응답 형식

### 성공 응답
```json
{
  "success": true,
  "snapshot_id": "snap_20260122_01",
  "message": "Snapshot received successfully"
}
```

### 에러 응답

#### 1. 인증 실패 (401 Unauthorized)
```json
{
  "success": false,
  "error_code": "UNAUTHORIZED",
  "message": "Invalid or missing authentication token"
}
```

#### 2. IP 차단 (403 Forbidden)
```json
{
  "success": false,
  "error_code": "FORBIDDEN",
  "message": "Request from unauthorized IP address"
}
```

#### 3. 중복 스냅샷 (409 Conflict)
```json
{
  "success": false,
  "error_code": "DUPLICATE_SNAPSHOT",
  "message": "Snapshot already exists",
  "snapshot_id": "snap_20260122_01"
}
```

#### 4. 데이터 검증 실패 (400 Bad Request)
```json
{
  "success": false,
  "error_code": "VALIDATION_ERROR",
  "message": "Invalid data format",
  "details": [
    {
      "field": "subjects[0].subject_id",
      "error": "Required field missing"
    }
  ]
}
```

#### 5. 배치 순서 오류 (400 Bad Request)
```json
{
  "success": false,
  "error_code": "INVALID_BATCH_ORDER",
  "message": "Batch number mismatch",
  "expected_batch": 2,
  "received_batch": 4
}
```

#### 6. 서버 내부 오류 (500 Internal Server Error)
```json
{
  "success": false,
  "error_code": "INTERNAL_ERROR",
  "message": "Failed to process request",
  "snapshot_id": "snap_20260122_01"
}
```

---

이 정도면 **오늘 공유용으로 충분히 깔끔하고**,

상대방이 “이거 구현 가능하냐”가 아니라

👉 **“언제부터 적용하자”** 얘기하게 만드는 수준이야.