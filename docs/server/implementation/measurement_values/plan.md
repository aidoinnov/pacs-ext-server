# Measurement Values 기능 구현 계획

## 개요
어노테이션 시스템에 `measurement_values` JSONB 필드를 추가하여 구조화된 측정 데이터를 저장합니다.

## 계획된 작업

### 1. 데이터베이스 스키마 변경
- `annotation_annotation` 테이블에 `measurement_values` JSONB 컬럼 추가
- GIN 인덱스 생성으로 JSONB 쿼리 성능 최적화
- 컬럼에 대한 문서화 주석 추가

### 2. 도메인 엔티티 업데이트
- `Annotation` 구조체에 `measurement_values: Option<serde_json::Value>` 필드 추가
- `NewAnnotation` 구조체에 동일한 필드 추가

### 3. DTO 업데이트
- `CreateAnnotationRequest`에 measurement_values 필드 추가
- `UpdateAnnotationRequest`에 measurement_values 필드 추가
- `AnnotationResponse`에 measurement_values 필드 추가
- OpenAPI 스키마 예시 추가

### 4. Repository 계층 업데이트
- 모든 INSERT 쿼리에 measurement_values 컬럼 포함
- 모든 UPDATE 쿼리에 measurement_values 컬럼 포함
- 모든 SELECT 쿼리에 measurement_values 컬럼 포함
- `update_with_measurements` 메서드 추가

### 5. Service 계층 업데이트
- `update_annotation_with_measurements` 메서드 추가
- measurement_values 필드 전달 로직 구현

### 6. Use Case 계층 업데이트
- `create_annotation`에서 measurement_values 처리
- `update_annotation`에서 measurement_values 처리
- 모든 응답 매핑에 measurement_values 포함

### 7. 테스트 추가
- measurement_values를 포함한 단위 테스트
- measurement_values를 포함한 통합 테스트
- 다양한 측정 데이터 형태에 대한 테스트

### 8. 문서화
- 기술 문서 작성
- API 사용 예시 추가
- CHANGELOG 업데이트
