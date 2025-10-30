# Viewer Software 필터링 기능 구현 계획

## 개요
어노테이션 목록 조회 시 `viewer_software` 파라미터로 필터링할 수 있는 기능을 추가합니다.

## 계획된 작업

### 1. 데이터베이스 스키마 변경
- `annotation_annotation` 테이블에 `viewer_software` VARCHAR(100) 컬럼 추가
- viewer_software 컬럼에 대한 인덱스 추가

### 2. 도메인 엔티티 업데이트
- `Annotation` 구조체에 `viewer_software: Option<String>` 필드 추가
- `NewAnnotation` 구조체에 동일한 필드 추가

### 3. DTO 업데이트
- `CreateAnnotationRequest`에 viewer_software 필드 추가
- `UpdateAnnotationRequest`에 viewer_software 필드 추가
- `AnnotationResponse`에 viewer_software 필드 추가

### 4. Repository 계층 업데이트
- `find_by_user_id_with_viewer` 메서드 추가
- `find_by_project_id_with_viewer` 메서드 추가
- `find_by_study_uid_with_viewer` 메서드 추가
- 모든 INSERT/UPDATE 쿼리에 viewer_software 컬럼 포함

### 5. Service 계층 업데이트
- `get_annotations_by_user_with_viewer` 메서드 추가
- `get_annotations_by_project_with_viewer` 메서드 추가
- `get_annotations_by_study_with_viewer` 메서드 추가

### 6. Use Case 계층 업데이트
- `get_annotations_by_user_with_viewer` 메서드 추가
- `get_annotations_by_project_with_viewer` 메서드 추가
- `get_annotations_by_study_with_viewer` 메서드 추가

### 7. Controller 계층 업데이트
- `list_annotations` 엔드포인트에 viewer_software 쿼리 파라미터 추가
- OpenAPI 문서화 업데이트

### 8. 테스트 추가
- viewer_software 필터링에 대한 단위 테스트
- viewer_software 필터링에 대한 통합 테스트
- 다양한 필터링 조합에 대한 테스트
