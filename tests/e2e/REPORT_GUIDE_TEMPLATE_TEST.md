# Report Guide Template API E2E 테스트

**작성일**: 2026-02-01  
**테스트 파일**: `test_report_guide_template.py`  
**API 문서**: `docs/api/REPORT_GUIDE_TEMPLATE_API.md`

---

## 📋 개요

이 E2E 테스트는 `docs/api/REPORT_GUIDE_TEMPLATE_API.md`에 문서화된 모든 Report Guide Template API 엔드포인트를 검증합니다.

---

## 🧪 테스트 시나리오

### 1. 원본 템플릿 API (5개 테스트)

| 테스트 | 엔드포인트 | 설명 |
|--------|-----------|------|
| test_01 | `POST /api/report-guide-templates` | 원본 템플릿 생성 (관리자) |
| test_02 | `GET /api/report-guide-templates` | 템플릿 목록 조회 |
| test_03 | `GET /api/report-guide-templates/{id}` | 템플릿 상세 조회 |
| test_04 | `PUT /api/report-guide-templates/{id}` | 템플릿 수정 (관리자) |
| test_23 | `DELETE /api/report-guide-templates/{id}` | 템플릿 삭제 (관리자) |

### 2. 가이드 이미지 업로드 API (5개 테스트)

| 테스트 | 엔드포인트 | 설명 |
|--------|-----------|------|
| test_05 | `POST /api/report-guide-templates/{id}/images/upload-url` | 업로드 URL 생성 |
| test_06 | `POST /api/report-guide-templates/{id}/images/complete` | 업로드 완료 처리 |
| test_07 | `GET /api/report-guide-templates/{id}` | 이미지 목록 조회 |
| test_08 | `PUT /api/report-guide-templates/{id}/images/{image_id}/share` | 이미지 공유 상태 업데이트 |
| test_22 | `DELETE /api/report-guide-templates/{id}/images/{image_id}` | 이미지 삭제 |

### 3. 사용자 커스텀 템플릿 API (9개 테스트)

| 테스트 | 엔드포인트 | 설명 |
|--------|-----------|------|
| test_09 | `POST /api/user/custom-report-templates` | 원본에서 커스텀 생성 |
| test_10 | `POST /api/user/custom-report-templates/new` | 새 커스텀 템플릿 생성 |
| test_11 | `GET /api/user/custom-report-templates` | 커스텀 템플릿 목록 조회 |
| test_02b | `GET /api/user/report-templates` | 유효 템플릿 조회 (커스텀 없을 때) |
| test_11b | `GET /api/user/report-templates` | 유효 템플릿 조회 (커스텀 있을 때) |
| test_12 | `GET /api/user/custom-report-templates/{id}` | 커스텀 템플릿 상세 조회 |
| test_13 | `PUT /api/user/custom-report-templates/{id}` | 커스텀 템플릿 수정 |
| test_14 | `POST /api/user/custom-report-templates/{id}/images` | 커스텀 이미지 추가 |
| test_20 | `DELETE /api/user/custom-report-templates/{id}/images/{image_id}` | 커스텀 이미지 삭제 |
| test_21 | `DELETE /api/user/custom-report-templates/{id}` | 커스텀 템플릿 삭제 |

### 4. Report-가이드 매핑 API (8개 테스트)

| 테스트 | 엔드포인트 | 설명 |
|--------|-----------|------|
| test_15 | `PUT /api/series/{series_uid}/report` | 테스트용 Report 생성 |
| test_15a | `GET /api/reports/{id}/guides` | 템플릿 미적용 시 빈 guides |
| test_15b | `GET /api/series/{series_uid}/report` | 시리즈 리포트 조회 (report_id 포함) |
| test_16 | `POST /api/reports/{id}/guides` | Report에 가이드 추가 |
| test_16b | - | 이미지 스냅샷 유지 검증 (템플릿 변경 후에도 유지) |
| test_17 | `GET /api/reports/{id}/guides` | Report 가이드 목록 조회 |
| test_18 | `POST /api/reports/{id}/guides` | Report에 커스텀 가이드 추가 |
| test_18b | - | Report 가이드 덮어쓰기 검증 |
| test_19 | `DELETE /api/reports/{id}/guides/{guide_id}` | Report에서 가이드 삭제 |

### 5. 권장 워크플로우 통합 (1개 테스트)

| 테스트 | 설명 |
|--------|------|
| TestFullRecommendedWorkflow::test_full_recommended_workflow | guide-images → image_ids → Report → guides 전체 플로우 |

---

## 🚀 테스트 실행 방법

### 방법 1: 쉘 스크립트 사용 (추천)

```bash
cd tests/e2e
./run_report_guide_template.sh
```

### 방법 2: pytest 직접 실행

```bash
cd tests/e2e
source venv/bin/activate
pytest test_report_guide_template.py -v -s
```

### 방법 3: 특정 테스트만 실행

```bash
# 원본 템플릿 테스트만
pytest test_report_guide_template.py::TestReportGuideTemplateAPI::test_01_create_template -v -s

# 이미지 업로드 테스트만
pytest test_report_guide_template.py::TestReportGuideTemplateAPI::test_05_generate_image_upload_url -v -s
```

---

## 📊 테스트 커버리지

### API 엔드포인트 커버리지

| 카테고리 | 엔드포인트 수 | 테스트 수 | 커버리지 |
|---------|-------------|----------|---------|
| 원본 템플릿 | 5 | 5 | 100% |
| 이미지 업로드 | 5 | 5 | 100% |
| 커스텀 템플릿 | 8 | 10 | 100% |
| 유효 템플릿 통합 | 1 | 2 | 100% |
| Report-가이드 매핑 | 3 | 4 | 133% |
| **전체** | **22** | **25** | **100%** |

### 테스트 시나리오 커버리지

- ✅ CRUD 작업 (Create, Read, Update, Delete)
- ✅ 이미지 업로드 3단계 워크플로우
- ✅ 권한 검증 (관리자 vs 일반 사용자)
- ✅ 데이터 무결성 검증
- ✅ 에러 처리 (404, 401 등)
- ✅ **Signed URL 검증**: 템플릿/가이드 이미지 image_url이 presigned URL로 반환
- ✅ **시리즈 리포트 조회**: GET /api/series/{series_uid}/report → report_id 포함
- ✅ **빈 guides**: 템플릿 미적용 시 guides 빈 배열 반환
- ✅ **이미지 스냅샷 유지**: 템플릿 변경 후에도 Report guides 이미지 유지
- ✅ **가이드 덮어쓰기**: 원본 → 커스텀 적용 시 덮어쓰기 검증
- ✅ **권장 워크플로우**: guide-images + image_ids 전체 플로우 통합

---

## 🔍 테스트 세부 사항

### 테스트 데이터

- **템플릿 이름**: `E2E Test Template {UUID}`
- **Bodypart**: `chest`, `brain`, `abdomen`
- **Modalities**: `["CT", "MR"]`
- **이미지 크기**: 1MB (1024000 bytes)
- **이미지 형식**: PNG

### 검증 항목

각 테스트는 다음을 검증합니다:

1. **HTTP 상태 코드**: 200, 201, 404, 401 등
2. **응답 데이터 구조**: 필수 필드 존재 여부
3. **데이터 일관성**: 생성/수정된 데이터가 조회 시 일치
4. **관계 무결성**: 템플릿-이미지, Report-가이드 관계
5. **권한 검증**: 관리자/사용자 권한 분리
6. **Signed URL 검증**: 이미지 `image_url`이 S3/MinIO presigned URL 형식인지 (접근 가능해야 함)

---

## ⚠️ 주의사항

### 테스트 환경 요구사항

1. **서버 실행**: PACS 서버가 `http://localhost:8080`에서 실행 중이어야 함
2. **최신 서버 빌드**: `cargo build` 후 재시작 필요. Signed URL 반환 기능이 적용된 서버여야 함
3. **Object Storage**: S3/MinIO 등이 설정되어 presigned URL 생성 가능해야 함
4. **데이터베이스**: PostgreSQL 데이터베이스가 준비되어 있어야 함
5. **인증**: 테스트 계정이 설정되어 있어야 함
   - Admin: `reader1_user` / `Qlalfqjsgh1!`
   - User: `reader1_user` / `Qlalfqjsgh1!`
6. **Report-가이드 테스트 (test_15~19)**:
   - `report_series_uid` 픽스처가 **프로젝트 → series assign** 으로 테스트용 시리즈를 자동 생성
   - 리포트는 **사용자+시리즈당 1개** (project와 무관)

### 테스트 순서

테스트는 **순서대로 실행**되어야 합니다:
1. 템플릿 생성 → 이미지 업로드 → 커스텀 생성 → Report 매핑 → 삭제

각 테스트는 이전 테스트에서 생성된 데이터를 사용합니다.

---

## 📝 예상 출력

```
test_report_guide_template.py::TestReportGuideTemplateAPI::test_01_create_template PASSED
test_report_guide_template.py::TestReportGuideTemplateAPI::test_02_list_templates PASSED
test_report_guide_template.py::TestReportGuideTemplateAPI::test_03_get_template_detail PASSED
...
test_report_guide_template.py::TestReportGuideTemplateAPI::test_23_delete_template PASSED

======================== 25 passed in 15.23s ========================
```

---

**테스트 문서 작성 완료** ✅  
**마지막 업데이트**: 2026-02-02 (Signed URL 검증, test_15b 시리즈 리포트 조회 추가)

