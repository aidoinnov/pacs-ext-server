# 어노테이션 측정값 기능

## 개요

어노테이션 시스템에 `measurement_values` 필드를 추가하여 구조화된 측정 데이터를 저장하고 관리할 수 있습니다. 이 필드는 JSONB 형태로 저장되며, 다양한 측정 타입과 단위를 지원합니다.

## 필드 구조

### 데이터베이스 스키마

```sql
-- annotation_annotation 테이블에 measurement_values 컬럼 추가
ALTER TABLE annotation_annotation 
ADD COLUMN measurement_values JSONB DEFAULT NULL;

-- JSONB 쿼리 성능을 위한 인덱스 추가
CREATE INDEX idx_annotation_measurement_values ON annotation_annotation USING GIN (measurement_values);
```

### JSON 구조

`measurement_values` 필드는 측정 객체의 배열로 구성됩니다:

```json
[
  {
    "id": "m1",
    "type": "raw",
    "values": [42.3, 18.7],
    "unit": "mm"
  },
  {
    "id": "m2", 
    "type": "mean",
    "values": [30.5],
    "unit": "mm"
  }
]
```

### 필드 설명

- **id**: 측정값의 고유 식별자 (문자열)
- **type**: 측정 타입 (문자열)
  - `raw`: 원시 측정값
  - `mean`: 평균값
  - `stddev`: 표준편차
  - `min`: 최솟값
  - `max`: 최댓값
  - `custom`: 사용자 정의 타입
- **values**: 측정값 배열 (숫자 배열)
- **unit**: 측정 단위 (문자열)
  - `mm`: 밀리미터
  - `cm`: 센티미터
  - `px`: 픽셀
  - `HU`: Hounsfield Unit
  - `custom`: 사용자 정의 단위

## API 사용법

### 어노테이션 생성

```http
POST /api/annotations
Content-Type: application/json

{
  "user_id": 1,
  "project_id": 1,
  "study_instance_uid": "1.2.3.4.5",
  "series_instance_uid": "1.2.3.4.6",
  "sop_instance_uid": "1.2.3.4.7",
  "annotation_data": {
    "type": "measurement",
    "points": [[0, 0], [100, 100]]
  },
  "description": "폐 결절 크기 측정",
  "tool_name": "Measurement Tool",
  "tool_version": "2.1.0",
  "viewer_software": "OHIF Viewer",
  "measurement_values": [
    {
      "id": "m1",
      "type": "raw",
      "values": [42.3, 18.7],
      "unit": "mm"
    },
    {
      "id": "m2",
      "type": "mean", 
      "values": [30.5],
      "unit": "mm"
    }
  ]
}
```

### 어노테이션 업데이트

```http
PUT /api/annotations/{id}
Content-Type: application/json

{
  "annotation_data": {
    "type": "measurement",
    "points": [[0, 0], [120, 120]]
  },
  "description": "업데이트된 측정",
  "measurement_values": [
    {
      "id": "m1",
      "type": "raw",
      "values": [42.3, 18.7],
      "unit": "mm"
    },
    {
      "id": "m2",
      "type": "mean",
      "values": [30.5],
      "unit": "mm"
    },
    {
      "id": "m3",
      "type": "stddev",
      "values": [5.2],
      "unit": "mm"
    }
  ]
}
```

### 어노테이션 조회

```http
GET /api/annotations/{id}
```

응답 예시:

```json
{
  "id": 123,
  "user_id": 1,
  "study_instance_uid": "1.2.3.4.5",
  "series_instance_uid": "1.2.3.4.6",
  "sop_instance_uid": "1.2.3.4.7",
  "annotation_data": {
    "type": "measurement",
    "points": [[0, 0], [100, 100]]
  },
  "tool_name": "Measurement Tool",
  "tool_version": "2.1.0",
  "viewer_software": "OHIF Viewer",
  "description": "폐 결절 크기 측정",
  "measurement_values": [
    {
      "id": "m1",
      "type": "raw",
      "values": [42.3, 18.7],
      "unit": "mm"
    },
    {
      "id": "m2",
      "type": "mean",
      "values": [30.5],
      "unit": "mm"
    }
  ],
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

## 지원되는 측정 타입

### 기본 측정 타입

1. **raw**: 원시 측정값
   - 여러 개의 측정값을 배열로 저장
   - 예: `[42.3, 18.7, 25.1]`

2. **mean**: 평균값
   - 단일 평균값 저장
   - 예: `[30.5]`

3. **stddev**: 표준편차
   - 단일 표준편차 값 저장
   - 예: `[5.2]`

4. **min**: 최솟값
   - 단일 최솟값 저장
   - 예: `[18.7]`

5. **max**: 최댓값
   - 단일 최댓값 저장
   - 예: `[42.3]`

### 사용자 정의 타입

- **custom**: 사용자 정의 측정 타입
- 필요에 따라 추가 타입 정의 가능

## 지원되는 측정 단위

### 기본 단위

1. **mm**: 밀리미터
2. **cm**: 센티미터
3. **px**: 픽셀
4. **HU**: Hounsfield Unit (CT 스캔용)
5. **%**: 퍼센트
6. **ratio**: 비율

### 사용자 정의 단위

- **custom**: 사용자 정의 단위
- 필요에 따라 추가 단위 정의 가능

## 측정 데이터 모범 사례

### 1. 측정값 ID 명명 규칙

- 의미있는 ID 사용: `m1`, `m2` 대신 `length`, `width`, `area` 등
- 일관된 명명 규칙 적용
- 버전 관리 시 ID 유지

### 2. 측정 타입 선택

- **raw**: 원시 데이터가 필요한 경우
- **mean**: 통계적 분석이 필요한 경우
- **stddev**: 데이터의 분산을 나타내는 경우
- **min/max**: 범위를 나타내는 경우

### 3. 단위 표준화

- 프로젝트 내에서 일관된 단위 사용
- 국제 표준 단위 우선 사용
- 변환 공식 문서화

### 4. 데이터 검증

- 측정값의 유효성 검사
- 단위의 일관성 확인
- 필수 필드 누락 방지

## 성능 고려사항

### 1. 인덱싱

- JSONB 필드에 GIN 인덱스 적용
- 자주 쿼리되는 측정 타입별 인덱스 고려

### 2. 쿼리 최적화

```sql
-- 측정 타입으로 필터링
SELECT * FROM annotation_annotation 
WHERE measurement_values @> '[{"type": "raw"}]';

-- 특정 단위로 필터링
SELECT * FROM annotation_annotation 
WHERE measurement_values @> '[{"unit": "mm"}]';

-- 측정값 범위 검색
SELECT * FROM annotation_annotation 
WHERE measurement_values @> '[{"values": [30.0, 50.0]}]';
```

### 3. 저장 공간

- JSONB는 효율적인 압축 사용
- 불필요한 중복 데이터 제거
- 정기적인 데이터 정리

## 마이그레이션

### 기존 데이터 처리

기존 어노테이션의 `measurement_values`는 `NULL`로 초기화됩니다:

```sql
-- 기존 어노테이션 확인
SELECT id, measurement_values FROM annotation_annotation 
WHERE measurement_values IS NOT NULL;

-- 필요시 기존 데이터 업데이트
UPDATE annotation_annotation 
SET measurement_values = '[{"id": "legacy", "type": "raw", "values": [0], "unit": "unknown"}]'
WHERE measurement_values IS NULL AND tool_name LIKE '%measurement%';
```

## 에러 처리

### 1. 잘못된 JSON 형식

```json
{
  "error": "Invalid JSON format in measurement_values",
  "message": "Expected array of measurement objects"
}
```

### 2. 필수 필드 누락

```json
{
  "error": "Missing required field",
  "message": "measurement_values[0].id is required"
}
```

### 3. 잘못된 데이터 타입

```json
{
  "error": "Invalid data type",
  "message": "measurement_values[0].values must be an array of numbers"
}
```

## 테스트

### 단위 테스트

```rust
#[tokio::test]
async fn test_create_annotation_with_measurement_values() {
    let annotation_request = CreateAnnotationRequest {
        // ... 기타 필드들
        measurement_values: Some(json!([
            {"id": "m1", "type": "raw", "values": [42.3, 18.7], "unit": "mm"},
            {"id": "m2", "type": "mean", "values": [30.5], "unit": "mm"}
        ])),
    };
    
    // 테스트 실행 및 검증
}
```

### 통합 테스트

```rust
#[tokio::test]
async fn test_measurement_values_api_integration() {
    // API 엔드포인트 테스트
    let req = test::TestRequest::post()
        .uri("/api/annotations")
        .set_json(&annotation_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["measurement_values"].is_array());
}
```

## 향후 확장 계획

### 1. 측정값 검증

- 스키마 기반 검증 추가
- 측정값 범위 검증
- 단위 변환 기능

### 2. 고급 쿼리

- 측정값 기반 필터링
- 통계적 분석 쿼리
- 측정값 비교 기능

### 3. 시각화 지원

- 측정값 차트 생성
- 히스토그램 분석
- 트렌드 분석

## 참고 자료

- [PostgreSQL JSONB 문서](https://www.postgresql.org/docs/current/datatype-json.html)
- [DICOM 측정 표준](https://dicom.nema.org/medical/dicom/current/output/chtml/part03/sect_C.10.6.html)
- [의료 영상 측정 가이드라인](https://www.rsna.org/practice-tools/data-tools-and-standards)
