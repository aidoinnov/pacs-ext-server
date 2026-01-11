# Series User Report 기능 구현 상태 점검

## ✅ 구현 완료된 기능

### 1. Report CRUD API
- ✅ 프로젝트 종속 Report 생성/조회/수정/삭제
- ✅ 전역 Report 생성/조회/수정/삭제
- ✅ Report 목록 조회 (프로젝트별, 전역)
- ✅ Report Status 관리 (unread, approval, unapproval)
- ✅ Report UPSERT 동작

### 2. Report Status 필터링
- ✅ DICOM Gateway Series 조회 시 `report_status` 필터링
- ✅ 다중 Status 필터링 지원 (예: `report_status=approved,unread`)
- ✅ 프로젝트 종속/전역 Report 우선순위 처리

### 3. Dictate 파일 업로드
- ✅ Signed URL 생성 API (`POST /api/reports/{report_id}/dictate/upload-url`)
- ✅ 업로드 완료 처리 API (`POST /api/reports/{report_id}/dictate/complete`)
- ✅ Dictate 파일 메타데이터 저장 (path, size, mime_type)

### 4. Guide Template 관리
- ✅ 원본 템플릿 CRUD
  - 생성, 조회, 수정, 삭제
  - 템플릿 목록 조회
- ✅ 커스텀 템플릿 CRUD
  - 원본 템플릿 복사하여 생성
  - 원본 없이 직접 생성
  - 조회, 수정, 삭제
- ✅ 템플릿 Modality 관리
- ✅ 템플릿 이미지 관리
  - 이미지 추가 (이미 업로드된 이미지 선택)
  - 이미지 삭제
  - 이미지 공유 상태 변경

### 5. Template 적용
- ✅ 템플릿을 Report에 적용 (`POST /api/reports/{report_id}/apply-template`)
- ✅ 원본 템플릿 또는 커스텀 템플릿 적용 지원

### 6. 권한 검증
- ✅ 프로젝트 멤버십 검증 (프로젝트 종속 Report)
- ✅ 사용자 인증 검증

### 7. 테스트
- ✅ 단위 테스트 (Repository, Service, UseCase)
- ✅ 통합 테스트 (API 엔드포인트)
- ✅ E2E 시나리오 테스트 (Python)

---

## ❌ 누락된 기능

### 1. Report Guide Image 관리 API

#### 1.1 Report의 Guide Image 목록 조회
- **현재 상태**: Repository에 `find_report_guides` 메서드는 있으나 API 엔드포인트 없음
- **필요한 API**: `GET /api/reports/{report_id}/guides`
- **설명**: Report에 연결된 Guide Image 목록 조회

#### 1.2 개별 Guide Image 추가
- **현재 상태**: 템플릿 전체를 적용하는 `apply_template_to_report`만 있음
- **필요한 API**: `POST /api/reports/{report_id}/guides`
- **설명**: 템플릿에서 개별 Guide Image를 선택하여 Report에 추가
- **요청 본문 예시**:
  ```json
  {
    "template_id": 1,
    "template_image_id": 5,
    "display_order": 0
  }
  ```

#### 1.3 개별 Guide Image 삭제
- **현재 상태**: Repository에 `delete_report_guide` 메서드는 있으나 API 엔드포인트 없음
- **필요한 API**: `DELETE /api/reports/{report_id}/guides/{guide_id}`
- **설명**: Report에서 특정 Guide Image 제거

#### 1.4 Guide Image 순서 변경
- **현재 상태**: `display_order` 필드는 있으나 업데이트 API 없음
- **필요한 API**: `PUT /api/reports/{report_id}/guides/{guide_id}/order`
- **설명**: Guide Image의 표시 순서 변경

### 2. Template 이미지 업로드 API

#### 2.1 Template 이미지 업로드 Signed URL 생성
- **현재 상태**: Template 이미지 추가 API는 있으나, 이미 업로드된 이미지를 선택하는 방식
- **필요한 API**: `POST /api/report-guide-templates/{template_id}/images/upload-url`
- **설명**: Template에 이미지를 업로드하기 위한 Signed URL 생성
- **요청 본문 예시**:
  ```json
  {
    "file_name": "guide_image.png",
    "mime_type": "image/png",
    "file_size": 1024000
  }
  ```

#### 2.2 Template 이미지 업로드 완료 처리
- **필요한 API**: `POST /api/report-guide-templates/{template_id}/images/complete`
- **설명**: 이미지 업로드 완료 후 메타데이터 저장

### 3. Report 응답에 Guide Image 포함

#### 3.1 Report 조회 시 Guide Image 목록 포함
- **현재 상태**: Report 응답에 Guide Image 정보가 포함되지 않음
- **필요한 수정**: `SeriesReportResponse` DTO에 `guides` 필드 추가
- **설명**: Report 조회 시 연결된 Guide Image 목록도 함께 반환

---

## 📋 구현 우선순위

### 높은 우선순위 (P1)
1. **Report의 Guide Image 목록 조회 API**
   - Repository 메서드는 이미 구현됨
   - UseCase와 Controller만 추가하면 됨
   - 프론트엔드에서 Report의 Guide Image를 표시하기 위해 필요

2. **Report 응답에 Guide Image 포함**
   - Report 조회 시 Guide Image 정보도 함께 반환
   - 프론트엔드에서 별도 API 호출 없이 Guide Image 표시 가능

### 중간 우선순위 (P2)
3. **개별 Guide Image 추가/삭제 API**
   - 템플릿 전체 적용 외에 개별 이미지 선택 기능
   - 사용자가 원하는 Guide Image만 선택적으로 추가 가능

4. **Template 이미지 업로드 API**
   - 현재는 이미 업로드된 이미지를 선택하는 방식
   - Template 생성 시 직접 이미지를 업로드할 수 있는 기능

### 낮은 우선순위 (P3)
5. **Guide Image 순서 변경 API**
   - `display_order` 필드 업데이트 기능
   - Guide Image 표시 순서 조정

---

## 🔍 상세 분석

### Report Guide Image 관리

현재 `series_user_report_guide` 테이블은 템플릿 전체를 Report에 연결하는 방식입니다. 개별 Guide Image를 추가하려면:

1. **옵션 A**: 템플릿 이미지 ID를 직접 참조하는 방식
   - `series_user_report_guide` 테이블에 `template_image_id` 필드 추가
   - 템플릿 전체가 아닌 개별 이미지만 연결

2. **옵션 B**: 현재 구조 유지 + 템플릿 이미지 필터링
   - 템플릿 전체를 연결하되, 특정 이미지만 표시하는 플래그 추가
   - `series_user_report_guide` 테이블에 `selected_image_ids` JSON 필드 추가

### Template 이미지 업로드

현재 `AddTemplateImageRequest`는 이미 업로드된 이미지의 경로와 URL을 받습니다. Signed URL 방식으로 변경하려면:

1. Signed URL 생성 API 추가
2. 업로드 완료 후 메타데이터 저장 API 추가
3. `AddTemplateImageRequest`를 업로드 완료 요청으로 변경

---

## 📝 다음 단계

1. **즉시 구현 가능** (Repository 메서드 존재):
   - Report Guide Image 목록 조회 API
   - Report Guide Image 삭제 API

2. **설계 후 구현 필요**:
   - 개별 Guide Image 추가 API (템플릿 이미지 ID 참조 방식 결정)
   - Template 이미지 업로드 API (Signed URL 방식)

3. **DTO 수정 필요**:
   - `SeriesReportResponse`에 `guides` 필드 추가
   - Guide Image 응답 DTO 정의





