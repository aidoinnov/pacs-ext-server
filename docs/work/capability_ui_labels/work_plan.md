# Capability UI 레이블 필드 추가 작업 계획

## 📋 작업 개요

**작업명**: Capability 테이블에 UI 레이블 필드 추가  
**작업일**: 2025-10-25  
**작업자**: AI Assistant  
**상태**: ✅ 완료  

## 🎯 작업 목표

`security_capability` 테이블에 `display_label`과 `category_label` 필드를 추가하여 UI 표에서 사용할 짧은 레이블을 제공합니다.

## 📝 작업 상세 내용

### 1. 데이터베이스 마이그레이션
- **파일**: `pacs-server/migrations/014_add_capability_ui_labels.sql`
- **내용**: 
  - `display_label` VARCHAR(50) 필드 추가
  - `category_label` VARCHAR(50) 필드 추가
  - 기존 데이터에 적절한 레이블 값 설정
  - 인덱스 추가

### 2. Domain Entity 업데이트
- **파일**: `pacs-server/src/domain/entities/capability.rs`
- **내용**:
  - `Capability` 구조체에 새 필드 추가
  - `NewCapability` 구조체에 새 필드 추가
  - `UpdateCapability` 구조체에 새 필드 추가

### 3. DTO 업데이트
- **파일**: `pacs-server/src/application/dto/role_capability_matrix_dto.rs`
- **내용**: `CapabilityInfo` 구조체에 새 필드 추가

### 4. Repository 업데이트
- **파일**: `pacs-server/src/infrastructure/repositories/capability_repository_impl.rs`
- **내용**: 모든 SQL 쿼리에 새 필드 포함

### 5. Use Case 업데이트
- **파일**: `pacs-server/src/application/use_cases/role_capability_matrix_use_case.rs`
- **내용**: `CapabilityInfo` 생성 시 새 필드 매핑

## 🎨 UI 레이블 매핑

### MANAGE 카테고리
- `SYSTEM_ADMIN` → `display_label: "Admin"`, `category_label: "MANAGE"`
- `USER_MANAGEMENT` → `display_label: "Users"`, `category_label: "MANAGE"`
- `ROLE_MANAGEMENT` → `display_label: "Roles"`, `category_label: "MANAGE"`
- `PROJECT_MANAGEMENT` → `display_label: "Projects"`, `category_label: "MANAGE"`

### PROJECT 카테고리
- `PROJECT_CREATE` → `display_label: "CREATE"`, `category_label: "PROJECT"`
- `PROJECT_ASSIGN` → `display_label: "ASSIGN"`, `category_label: "PROJECT"`
- `PROJECT_EDIT` → `display_label: "EDIT"`, `category_label: "PROJECT"`

### DICOM 카테고리
- `DICOM_READ_ACCESS` → `display_label: "READ"`, `category_label: "DICOM"`
- `DICOM_WRITE_ACCESS` → `display_label: "WRITE"`, `category_label: "DICOM"`
- `DICOM_DELETE_ACCESS` → `display_label: "DELETE"`, `category_label: "DICOM"`
- `DICOM_SHARE_ACCESS` → `display_label: "SHARE"`, `category_label: "DICOM"`

### ANNOTATION 카테고리
- `ANNOTATION_READ_OWN` → `display_label: "READ OWN"`, `category_label: "ANNOTATION"`
- `ANNOTATION_READ_ALL` → `display_label: "READ ALL"`, `category_label: "ANNOTATION"`
- `ANNOTATION_WRITE` → `display_label: "WRITE"`, `category_label: "ANNOTATION"`
- `ANNOTATION_DELETE` → `display_label: "DELETE"`, `category_label: "ANNOTATION"`
- `ANNOTATION_SHARE` → `display_label: "SHARE"`, `category_label: "ANNOTATION"`

### MASK 카테고리
- `MASK_READ` → `display_label: "READ"`, `category_label: "MASK"`
- `MASK_WRITE` → `display_label: "WRITE"`, `category_label: "MASK"`
- `MASK_DELETE` → `display_label: "DELETE"`, `category_label: "MASK"`

### HANGING_PROTOCOL 카테고리
- `HANGING_PROTOCOL_MANAGEMENT` → `display_label: "MANAGE"`, `category_label: "HANGING_PROTOCOL"`

## ✅ 완료 체크리스트

- [x] 데이터베이스 마이그레이션 파일 생성
- [x] Domain Entity 업데이트
- [x] DTO 업데이트
- [x] Repository 업데이트
- [x] Use Case 업데이트
- [x] 데이터베이스 마이그레이션 실행
- [x] API 테스트 및 검증
- [x] 문서 업데이트

## 🎯 기대 효과

1. **UI 개선**: 프론트엔드에서 표 헤더와 셀에 적절한 짧은 레이블 사용 가능
2. **사용자 경험 향상**: 더 직관적이고 깔끔한 UI 제공
3. **국제화 지원**: 향후 다국어 지원 시 레이블 기반 매핑 가능
4. **유지보수성**: UI 레이블과 내부 로직 분리로 유지보수 용이
