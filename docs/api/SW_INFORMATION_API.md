# SW Information API 가이드

**작성일**: 2026-02-02
**버전**: 1.0

---

## 📋 개요

SW Information API는 의료영상저장장치 소프트웨어 정보를 조회하는 API입니다.  
화면(SW Information 모달)에 표시되는 품목, 모델명, 제조업자, UDI 등의 정보를 제공합니다.

### 데이터 필드

| 필드(한글) | 영문 필드명 | 타입 | 설명 |
|-----------|-------------|------|------|
| 품목 | product_item | string | 예: 의료영상저장장치소프트웨어 |
| 모델명 | model_name | string | 예: Aid-U |
| SW Ver. | sw_version | string? | nullable |
| 제조업자 | manufacturer | string | 예: (주)아이에이드 |
| 주소 | address | string | 제조업자 주소 |
| 제조허가번호 | manufacturing_permit_number | string | 예: 제6816호 |
| 제조연월 | manufacturing_year_month | string? | nullable |
| 시리얼번호 | serial_number | string? | nullable |
| UDI | udi | string? | Unique Device Identification, 다중 라인 가능 |

---

## 🔑 인증

현재 구현은 인증 없이 조회 가능합니다. (화면 노출용 공개 정보)

> 향후 인증 정책 변경 시 `Authorization: Bearer <jwt_token>` 헤더가 필요할 수 있습니다.

---

## 1. 목록 조회

### GET /api/sw-information

SW Information 전체 목록을 조회합니다.

```http
GET /api/sw-information
```

**응답 (200 OK):**

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
      "created_at": "2026-02-02T12:24:40.325562Z",
      "updated_at": "2026-02-02T12:24:40.325562Z"
    }
  ],
  "total_count": 1
}
```

| 필드 | 타입 | 설명 |
|------|------|------|
| success | boolean | 요청 성공 여부 |
| items | array | SW Information 항목 배열 |
| total_count | number | 전체 항목 수 |

---

## 2. 상세 조회

### GET /api/sw-information/{id}

ID로 SW Information 상세를 조회합니다.

```http
GET /api/sw-information/1
```

**경로 파라미터:**
| 파라미터 | 타입 | 설명 |
|----------|------|------|
| id | integer | SW Information ID |

**응답 (200 OK):**

```json
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
  "created_at": "2026-02-02T12:24:40.325562Z",
  "updated_at": "2026-02-02T12:24:40.325562Z"
}
```

**응답 (404 Not Found):**

```json
{
  "error": "Not Found",
  "message": "SW Information not found"
}
```

---

## 3. 사용 예시

### cURL

```bash
# 목록 조회
curl -X GET "http://localhost:8080/api/sw-information"

# 상세 조회
curl -X GET "http://localhost:8080/api/sw-information/1"
```

### JavaScript (fetch)

```javascript
// 목록 조회
const listRes = await fetch('http://localhost:8080/api/sw-information');
const listData = await listRes.json();
console.log(listData.items);

// 상세 조회
const detailRes = await fetch('http://localhost:8080/api/sw-information/1');
const detail = await detailRes.json();
```

---

## 4. 관련 문서

- **Planning**: `docs/plans/plan_sw_information.md`
- **마이그레이션**: `pacs-server/migrations/20260202_01_create_sw_information.sql`
- **E2E 테스트**: `tests/e2e/test_sw_information.py`
