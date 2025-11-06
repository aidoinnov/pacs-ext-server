# DateTime 타입 수정 작업 계획

## 개요
PostgreSQL의 TIMESTAMPTZ 타입과 Rust의 NaiveDateTime 타입 간의 호환성 문제를 해결하기 위해 DateTime<Utc> 타입으로 통일합니다.

## 계획된 작업

### 1. 엔티티 타입 수정
- `Annotation` 엔티티의 `created_at`, `updated_at` 필드를 `DateTime<Utc>`로 변경
- `NewAnnotation` 엔티티의 `created_at`, `updated_at` 필드를 `DateTime<Utc>`로 변경
- 기타 엔티티들의 DateTime 필드 타입 통일

### 2. DTO 타입 수정
- `AnnotationResponse`의 `created_at`, `updated_at` 필드를 `DateTime<Utc>`로 변경
- 기타 응답 DTO들의 DateTime 필드 타입 통일

### 3. 테스트 코드 수정
- 모든 테스트에서 `NaiveDateTime` 대신 `DateTime<Utc>` 사용
- `chrono::TimeZone` 트레이트 import 추가
- `Utc.timestamp_opt` 사용법으로 변경

### 4. 컴파일 에러 수정
- 타입 불일치 에러 해결
- 누락된 필드 초기화 에러 해결
- Move 에러 해결

## 예상 문제점
- 기존 코드에서 `NaiveDateTime` 사용하는 부분들
- 테스트 코드의 DateTime 초기화 방식
- 타입 변환 관련 에러들
