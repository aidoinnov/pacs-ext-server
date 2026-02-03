# Report 가이드 이미지 영속화 설계

**작성일**: 2026-02-01  
**목적**: 리포트에 템플릿 적용 시 이미지를 스냅샷하여, 템플릿이 변경되어도 리포트의 이미지는 유지

---

## 1. 배경

### 현재 구조의 한계

```
Report → series_user_report_guide (N개 가이드, 각각 template 참조)
         → 이미지는 항상 템플릿에서 "실시간" 조회
```

- 리포트 1개에 가이드(템플릿) N개가 연결 가능한 구조였으나, **실제 요구사항은 리포트 1개 = 템플릿 1개**
- 템플릿을 수정하면 기존 리포트에 적용된 이미지도 같이 바뀜
- "리포트 작성 시점의 이미지는 유지" 요구사항 불가

### 요구사항

- **리포트 1개 : 템플릿 1개** (시리즈당 리포트 1개, 리포트당 템플릿 1개)
- 이미지는 여러 개 가능
- 템플릿 적용 시점의 이미지를 **스냅샷**하여, 이후 템플릿이 변경되어도 리포트 이미지는 유지

---

## 2. 설계 개요

### 핵심 원칙

- **리포트 1개 = 템플릿 1개** — `series_user_report`에 `template_id`, `custom_template_id` 직접 저장
- **이미지 스냅샷** — `report_image` 테이블로 리포트 단의 이미지 보관 (report_id 기준)
- `series_user_report_guide` 테이블 **폐기** (1:1 구조로 단순화)

### 데이터 흐름

```
[적용 시]
템플릿 선택 → series_user_report.template_id (또는 custom_template_id) 설정
            → 템플릿 이미지 목록을 report_image에 복사 (image_id 참조)

[조회 시]
report_image + guide_image 조인 → 이미지 URL 등 반환
(템플릿 실시간 조회 안 함)
```

### ER 관계

```
series_user_report
    ├── template_id ──→ report_guide_template (출처, nullable)
    ├── custom_template_id ──→ user_custom_report_template (출처, nullable)
    └── report_image (1:N) ──→ guide_image

CHECK: (template_id IS NOT NULL AND custom_template_id IS NULL) OR
       (template_id IS NULL AND custom_template_id IS NOT NULL) OR
       (template_id IS NULL AND custom_template_id IS NULL)  -- 템플릿 미적용
```

---

## 3. 데이터베이스 스키마

### 3.1 series_user_report 컬럼 추가

```sql
ALTER TABLE series_user_report
    ADD COLUMN IF NOT EXISTS template_id INTEGER NULL REFERENCES report_guide_template(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS custom_template_id INTEGER NULL REFERENCES user_custom_report_template(id) ON DELETE SET NULL;

ALTER TABLE series_user_report
    ADD CONSTRAINT chk_report_template_exclusive CHECK (
        (template_id IS NOT NULL AND custom_template_id IS NULL) OR
        (template_id IS NULL AND custom_template_id IS NOT NULL) OR
        (template_id IS NULL AND custom_template_id IS NULL)
    );
```

### 3.2 새 테이블: report_image

```sql
CREATE TABLE report_image (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    report_id INTEGER NOT NULL REFERENCES series_user_report(id) ON DELETE CASCADE,
    image_id INTEGER NOT NULL REFERENCES guide_image(id) ON DELETE RESTRICT,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (report_id, image_id)
);

CREATE INDEX idx_report_image_report ON report_image(report_id);
CREATE INDEX idx_report_image_order ON report_image(report_id, display_order);

COMMENT ON TABLE report_image IS '리포트의 가이드 이미지 스냅샷 (템플릿 변경과 무관하게 유지)';
```

### 3.3 series_user_report_guide 폐기

- 기존 데이터를 `series_user_report` + `report_image`로 마이그레이션
- 마이그레이션 완료 후 `series_user_report_guide` 테이블 삭제 (또는 향후 별도 마이그레이션)

---

## 4. API 변경

### 4.1 템플릿 적용 (기존 POST /api/reports/{report_id}/guides)

**요청**: 동일

```json
{
  "template_id": 1,
  "custom_template_id": null,
  "display_order": 0
}
```

**로직 변경**:

1. `series_user_report.template_id` 또는 `custom_template_id` 설정 (기존 값 덮어쓰기)
2. 기존 `report_image` 행 삭제
3. 해당 템플릿의 이미지 목록 조회 → `report_image`에 삽입
4. `display_order`는 가이드가 1개이므로 0 고정 (또는 무시)

### 4.2 가이드(템플릿+이미지) 조회 (기존 GET /api/reports/{report_id}/guides)

**응답**: 기존 구조 유지, 단 0개 또는 1개 항목만 반환

```json
{
  "success": true,
  "guides": [
    {
      "id": 0,
      "report_id": 123,
      "template_id": 5,
      "custom_template_id": null,
      "display_order": 0,
      "images": [
        {
          "id": 101,
          "image_path": "...",
          "image_url": "...",
          "display_order": 0
        }
      ],
      "created_at": "..."
    }
  ]
}
```

- `id`: 가이드가 1개이므로 0 또는 report_id 등 고정값 (하위 호환용)
- `images`: `report_image` + `guide_image` 조인 결과

### 4.3 템플릿 제거 (기존 DELETE /api/reports/{report_id}/guides/{guide_id})

- `series_user_report`의 `template_id`, `custom_template_id`를 NULL로 설정
- `report_image` 해당 report_id 행 전부 삭제
- `guide_id`는 무시 (리포트당 가이드 1개이므로)

---

## 5. 기존 데이터 마이그레이션

### 5.1 series_user_report_guide → series_user_report + report_image

```sql
-- 1. template_id, custom_template_id 설정 (첫 번째 가이드 기준)
UPDATE series_user_report r
SET
    template_id = g.template_id,
    custom_template_id = g.custom_template_id
FROM (
    SELECT DISTINCT ON (report_id) report_id, template_id, custom_template_id
    FROM series_user_report_guide
    ORDER BY report_id, display_order, created_at
) g
WHERE r.id = g.report_id;

-- 2. report_image 삽입 (guide_image 기반 매핑 사용)
-- 원본 템플릿
INSERT INTO report_image (report_id, image_id, display_order, created_at)
SELECT rg.report_id, m.image_id, m.display_order, NOW()
FROM series_user_report_guide rg
JOIN report_guide_template_image_mapping m ON m.template_id = rg.template_id
WHERE rg.template_id IS NOT NULL
ON CONFLICT (report_id, image_id) DO NOTHING;

-- 커스텀 템플릿
INSERT INTO report_image (report_id, image_id, display_order, created_at)
SELECT rg.report_id, m.image_id, m.display_order, NOW()
FROM series_user_report_guide rg
JOIN user_custom_template_image_mapping m ON m.custom_template_id = rg.custom_template_id
WHERE rg.custom_template_id IS NOT NULL
ON CONFLICT (report_id, image_id) DO NOTHING;

-- 3. (선택) series_user_report_guide 테이블 삭제
-- DROP TABLE IF EXISTS series_user_report_guide CASCADE;
```

**주의**: `report_guide_template_image_mapping`, `user_custom_template_image_mapping`가 없는 레거시 환경에서는 `report_guide_template_image`, `user_custom_template_image` 매핑 필요.

---

## 6. 구현 작업 목록

| 순서 | 작업 | 설명 |
|------|------|------|
| 1 | 마이그레이션 | `series_user_report` 컬럼 추가, `report_image` 테이블 생성 |
| 2 | 도메인 | `SeriesUserReport` 엔티티에 template_id, custom_template_id 추가 |
| 3 | Repository | `report_image` CRUD, `series_user_report` template 업데이트 |
| 4 | Service/UseCase | 가이드 추가/조회/삭제를 리포트 단 템플릿+report_image 기반으로 변경 |
| 5 | Controller | 기존 guides API 유지, 내부 로직 변경 |
| 6 | 마이그레이션 | 기존 `series_user_report_guide` 데이터 백필 |
| 7 | API 문서 | REPORT_GUIDE_TEMPLATE_API.md 업데이트 |
| 8 | 테스트 | E2E 테스트 수정 및 검증 |

---

## 7. 주의사항

### guide_image 삭제 시

- `report_image`는 `guide_image(id)` 참조, `ON DELETE RESTRICT` 권장
- 리포트에서 사용 중인 이미지는 삭제 불가

### series_user_report UNIQUE 제약

- 현재 `UNIQUE (series_id, user_id, project_id)` — template 컬럼 추가해도 유지

---

## 8. 요약

| 항목 | 내용 |
|------|------|
| 리포트:템플릿 | 1:1 (`series_user_report`에 template_id/custom_template_id) |
| 새 테이블 | `report_image` (report_id, image_id, display_order) |
| 적용 시점 | 템플릿 설정 시 이미지를 `report_image`에 복사 |
| 조회 시점 | `report_image` + `guide_image` 조인 |
| 폐기 | `series_user_report_guide` |
