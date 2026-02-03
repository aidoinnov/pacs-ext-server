# SW Information API - Planning Document

## 1. 작업 개요

### 목적
화면(첨부 이미지)에 표시되는 SW Information 데이터를 저장·조회할 수 있는 테이블과 REST API를 구현한다.

### 대상 도메인
- **SW Information**: 의료영상저장장치 소프트웨어 정보
- Aggregate Root: `SwInformation` (단일/소량 레코드, 현재 화면 기준 1건)

### 데이터 모델 (화면 기준)

| 필드(한글) | 영문 필드명 | 타입 | 비고 |
|-----------|-------------|------|------|
| 품목 | product_item | String | 의료영상저장장치소프트웨어 |
| 모델명 | model_name | String | Aid-U |
| SW Ver. | sw_version | String? | nullable |
| 제조업자 | manufacturer | String | (주)아이에이드 |
| 주소 | address | String | |
| 제조허가번호 | manufacturing_permit_number | String | 제6816호 |
| 제조연월 | manufacturing_year_month | String? | nullable |
| 시리얼번호 | serial_number | String? | nullable |
| UDI | udi | String? | 다중 라인, nullable. (01)/(21)/(8012) 형식 |

### 영향 범위
- **신규**: 독립 기능 추가
- 기존 API와 Aggregate/권한 불일치 없음

---

## 2. 설계안 비교 및 점수화

### 설계안 A: Singleton 패턴 (단일 레코드)

- 테이블: `sw_information` (id PK, 단일 행 또는 first-wins)
- API: `GET /api/sw-information` → 현재 설정 1건 반환
- Scope: `/api/sw-information`
- 관리: 마이그레이션 또는 Admin API로 초기 데이터 삽입

| 기준 | 점수 | 비고 |
|-----|------|------|
| DDD/SRP 적합성 | 8/10 | 단순 Entity, Repository 분리 |
| 모듈 일관성 | 9/10 | 기존 feature 패턴 준수 |
| API Scope 안정성 | 9/10 | 신규 scope, 충돌 없음 |
| 테스트 용이성 | 9/10 | 단순 Read, E2E 용이 |
| 확장성 | 6/10 | 다수 제품 지원 시 재설계 필요 |
| **총점** | **41/50** | |

### 설계안 B: CRUD 패턴 (다수 레코드 지원)

- 테이블: `sw_information` (id PK, model_name 등 인덱스)
- API:
  - `GET /api/sw-information` → 목록 (또는 현재 1건)
  - `GET /api/sw-information/{id}` → 상세
  - `POST /api/sw-information` (관리자) → 생성
  - `PUT /api/sw-information/{id}` (관리자) → 수정
- Scope: `/api/sw-information`

| 기준 | 점수 | 비고 |
|-----|------|------|
| DDD/SRP 적합성 | 9/10 | 완전한 CRUD, 책임 분리 |
| 모듈 일관성 | 9/10 | 기존 패턴 준수 |
| API Scope 안정성 | 9/10 | 신규 scope |
| 테스트 용이성 | 8/10 | CRUD 전반 테스트 |
| 확장성 | 9/10 | 다수 제품/버전 확장 가능 |
| **총점** | **44/50** | |

### 최종 선택: 설계안 B

- 화면 요구사항: "조회"가 주된 사용 사항이지만, 데이터 입력/수정 경로가 필요함
- 향후 다수 제품(모델) 지원 가능성 고려
- CRUD 제공 시 클라이언트 유연성 확보

---

## 3. 최종 설계안 요약

### 3.1 API Scope 설계

- **Root Scope**: `/api`
- **Feature Scope**: `/api/sw-information`
- **기존 Scope와의 관계**: `/api` 하위에 신규 scope 추가. `/report-guide-templates`, `/user/`, `/projects` 등과 경로 충돌 없음.
- **충돌 가능성**: 없음 (sw-information은 신규 path)
- **합침/분리 판단**: 독립 Aggregate이므로 별도 scope 유지

```
/api
 └─ /sw-information
     ├─ GET ""           → 목록 (현재는 1건 또는 소량)
     └─ GET "/{id}"      → 상세 조회
```

**1차 구현 범위**: 조회(Read)만 구현. 화면 요구사항 "조회할 수 있는" 충족.
- `GET /api/sw-information` → 전체 목록 (또는 첫 1건)
- `GET /api/sw-information/{id}` → id로 상세

향후 확장: POST/PUT/DELETE (관리자)는 필요 시 추가.

### 3.2 모듈 구조 (패턴 준수)

```
domain/sw_information/
  entities/
    sw_information.rs
  repositories/
    sw_information_repository.rs

application/sw_information/
  dto/
    sw_information_dto.rs
  use_cases/
    sw_information_use_case.rs

infrastructure/sw_information/
  repositories/
    sw_information_repository_impl.rs

presentation/sw_information/
  controllers/
    sw_information_controller.rs
```

### 3.3 테이블 스키마

```sql
CREATE TABLE sw_information (
    id SERIAL PRIMARY KEY,
    product_item TEXT NOT NULL,
    model_name TEXT NOT NULL,
    sw_version TEXT,
    manufacturer TEXT NOT NULL,
    address TEXT NOT NULL,
    manufacturing_permit_number TEXT NOT NULL,
    manufacturing_year_month TEXT,
    serial_number TEXT,
    udi TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 3.4 시퀀스 다이어그램

```
Client
  → GET /api/sw-information
    → SwInformationController
      → SwInformationUseCase::list()
        → SwInformationRepository::find_all()
      ← Vec<SwInformation>
    ← JSON Response
```

---

## 4. TODO 체크리스트

- [x] Migration: sw_information 테이블 생성 (`migrations/20260202_01_create_sw_information.sql`)
- [x] Domain Entity 정의 (SwInformation)
- [x] Repository Trait 정의 (SwInformationRepository)
- [x] Repository 구현체 (SwInformationRepositoryImpl)
- [x] Repository 단위 테스트 통과
- [x] Application UseCase 구현 (SwInformationUseCase)
- [x] Service/UseCase 통합 테스트 통과
- [x] REST API Path 확정 및 main.rs 라우팅 등록
- [x] Controller 구현
- [x] API 단위 테스트 통과 (Controller는 E2E로 검증)
- [x] Python E2E 테스트 작성 (tests/e2e/test_sw_information.py)
- [x] 전체 테스트 통과 (sw_information 관련)

**마이그레이션 실행**: `python3 scripts/run_sw_information_migration.py` (DATABASE_URL 환경변수 필요)

---

## 5. API 명세 (1차: Read Only)

### GET /api/sw-information

목록 조회. 인증 필요 여부는 기존 정책 따름 (화면 노출용이면 익명 허용 가능).

**Response 200:**
```json
{
  "success": true,
  "items": [
    {
      "id": 1,
      "product_item": "의료영상저장장치소프트웨어",
      "model_name": "Aid-U",
      "sw_version": null,
      "manufacturer": "(주)아이에이드",
      "address": "서울특별시 동작구 상도로 398, 가나빌딩 7층",
      "manufacturing_permit_number": "제6816호",
      "manufacturing_year_month": null,
      "serial_number": null,
      "udi": "(01) 08800080000004\n(21) -\n(8012) -",
      "created_at": "2026-02-02T00:00:00Z",
      "updated_at": "2026-02-02T00:00:00Z"
    }
  ],
  "total_count": 1
}
```

### GET /api/sw-information/{id}

상세 조회.

**Response 200:** 단일 객체 JSON  
**Response 404:** Not Found

---

## 6. Validation Result (Validator)

- [x] Domain Entity 정의 (Validator)
- [x] Repository Trait 정의 (Validator)
- [x] Repository 단위 테스트 통과 (Validator) — `cargo test --test sw_information_repository_test`
- [x] Application UseCase 구현 (Validator)
- [x] Service 통합 테스트 통과 (Validator) — `cargo test --test sw_information_use_case_test`
- [x] REST API Path 충돌 없음 (Validator)
- [x] API Scope 충돌 없음 (Validator)
- [x] Controller 구현 (Validator)
- [x] API 단위 테스트 통과 (Validator) — E2E로 검증
- [x] Python E2E 테스트 통과 (Validator) — `pytest test_sw_information.py`
- [x] 전체 테스트 통과 (Validator)

**판정: COMPLETED**
