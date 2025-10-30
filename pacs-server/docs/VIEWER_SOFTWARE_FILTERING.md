# Viewer Software Filtering Feature

## 개요

Annotation 목록 조회 시 `viewer_software` 필드로 필터링할 수 있는 기능을 추가했습니다. 이를 통해 특정 뷰어 소프트웨어에서 생성된 어노테이션만 조회할 수 있습니다.

## 기능 설명

### 1. API 엔드포인트

**GET /api/annotations**

#### 쿼리 파라미터
- `viewer_software` (선택사항): 뷰어 소프트웨어 이름으로 필터링
- `user_id` (선택사항): 사용자 ID로 필터링
- `project_id` (선택사항): 프로젝트 ID로 필터링
- `study_instance_uid` (선택사항): Study Instance UID로 필터링

#### 사용 예시
```bash
# OHIF Viewer로 생성된 어노테이션만 조회
GET /api/annotations?viewer_software=OHIF%20Viewer

# 특정 사용자의 DICOM Viewer 어노테이션 조회
GET /api/annotations?user_id=123&viewer_software=DICOM%20Viewer

# 특정 프로젝트의 모든 어노테이션 조회
GET /api/annotations?project_id=456

# 필터 없이 모든 어노테이션 조회
GET /api/annotations
```

### 2. 지원되는 뷰어 소프트웨어

- OHIF Viewer
- DICOM Viewer
- 기타 사용자 정의 뷰어 소프트웨어

### 3. 데이터베이스 스키마

`annotation_annotation` 테이블에 `viewer_software` 컬럼이 추가되었습니다:

```sql
ALTER TABLE annotation_annotation 
ADD COLUMN viewer_software VARCHAR(255);
```

## 아키텍처

### 1. 계층별 구현

#### Repository Layer
- `AnnotationRepository` 트레이트에 새로운 메서드 추가:
  - `find_by_user_id_with_viewer`
  - `find_by_project_id_with_viewer`
  - `find_by_study_uid_with_viewer`

#### Service Layer
- `AnnotationService` 트레이트에 새로운 메서드 추가:
  - `get_annotations_by_user_with_viewer`
  - `get_annotations_by_project_with_viewer`
  - `get_annotations_by_study_with_viewer`

#### Use Case Layer
- `AnnotationUseCase`에 새로운 메서드 추가:
  - `get_annotations_by_user_with_viewer`
  - `get_annotations_by_project_with_viewer`
  - `get_annotations_by_study_with_viewer`

#### Controller Layer
- `list_annotations` 함수에서 `viewer_software` 쿼리 파라미터 처리

### 2. SQL 쿼리 예시

```sql
-- OHIF Viewer로 필터링된 어노테이션 조회
SELECT id, project_id, user_id, study_uid, series_uid, instance_uid,
       tool_name, tool_version, data, is_shared, created_at, updated_at,
       viewer_software, description
FROM annotation_annotation
WHERE user_id = $1 AND viewer_software = $2
ORDER BY created_at DESC;

-- 필터 없이 모든 어노테이션 조회
SELECT id, project_id, user_id, study_uid, series_uid, instance_uid,
       tool_name, tool_version, data, is_shared, created_at, updated_at,
       viewer_software, description
FROM annotation_annotation
WHERE user_id = $1
ORDER BY created_at DESC;
```

## 테스트

### 1. 단위 테스트
- `AnnotationUseCase`의 `viewer_software` 필터링 로직 테스트
- `AnnotationRepository`의 데이터베이스 쿼리 테스트

### 2. 통합 테스트
- API 엔드포인트를 통한 전체 워크플로우 테스트
- 다양한 필터 조합 테스트

### 3. 성능 테스트
- 대용량 데이터에서의 필터링 성능 테스트
- 인덱스 최적화 검증

## 마이그레이션

### 1. 데이터베이스 마이그레이션
```sql
-- 004_add_viewer_software_column.sql
ALTER TABLE annotation_annotation 
ADD COLUMN viewer_software VARCHAR(255);

-- 인덱스 추가 (성능 최적화)
CREATE INDEX idx_annotation_viewer_software ON annotation_annotation(viewer_software);
```

### 2. 기존 데이터 처리
기존 어노테이션의 `viewer_software` 필드는 `NULL`로 설정되며, 필터링 시 `NULL` 값은 제외됩니다.

## 성능 고려사항

### 1. 인덱스
- `viewer_software` 컬럼에 인덱스 추가
- 복합 인덱스 고려 (`user_id`, `viewer_software`)

### 2. 쿼리 최적화
- 조건부 WHERE 절 사용으로 불필요한 필터링 방지
- 적절한 ORDER BY 절 사용

## 보안 고려사항

### 1. 입력 검증
- `viewer_software` 파라미터 길이 제한 (255자)
- SQL 인젝션 방지를 위한 파라미터화된 쿼리 사용

### 2. 권한 검증
- 사용자별 어노테이션 접근 권한 확인
- 프로젝트 멤버십 검증

## 향후 개선사항

### 1. 기능 확장
- 다중 뷰어 소프트웨어 필터링
- 뷰어 소프트웨어별 통계 제공
- 뷰어 소프트웨어 자동 감지

### 2. 성능 최적화
- 캐싱 전략 도입
- 페이지네이션 개선
- 쿼리 최적화

## 문제 해결

### 1. 일반적인 문제
- **404 에러**: API 라우팅 설정 확인
- **빈 결과**: 필터 조건과 데이터 일치 여부 확인
- **성능 이슈**: 인덱스 설정 확인

### 2. 디버깅
- 로그 레벨을 `debug`로 설정하여 쿼리 파라미터 확인
- 데이터베이스에서 직접 쿼리 실행하여 결과 검증

## 참고사항

- 이 기능은 기존 API와 완전히 호환됩니다
- `viewer_software` 파라미터는 선택사항이므로 기존 클라이언트는 수정 없이 사용 가능
- OpenAPI 문서가 자동으로 업데이트됩니다
