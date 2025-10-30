# Annotation API 완전 가이드

## 🎯 API 개요
PACS Extension Server의 DICOM 이미지 어노테이션 관리 API입니다. 의료 영상에 대한 어노테이션(원형, 사각형, 점, 다각형 등)을 생성, 조회, 수정, 삭제할 수 있습니다.

## 📍 Base URL
```
http://localhost:8080/api
```

## 🔗 API 엔드포인트

### 1. 어노테이션 생성
```http
POST /api/annotations
Content-Type: application/json
```

**요청 본문:**
```json
{
  "study_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
  "series_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
  "sop_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
  "annotation_data": {
    "type": "circle",
    "x": 100,
    "y": 200,
    "radius": 50,
    "color": "#FF0000",
    "label": "Test Annotation"
  },
  "description": "This is a test annotation for DICOM image analysis"
}
```

**응답 (201 Created):**
```json
{
  "id": 44,
  "user_id": 336,
  "study_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
  "series_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
  "sop_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
  "annotation_data": {
    "color": "#FF0000",
    "label": "Test Annotation",
    "radius": 50,
    "type": "circle",
    "x": 100,
    "y": 200
  },
  "description": null,
  "created_at": "2025-10-07T10:16:40.072803",
  "updated_at": "2025-10-07T10:16:40.072803"
}
```

### 2. 어노테이션 목록 조회
```http
GET /api/annotations?user_id=336
GET /api/annotations?project_id=302
GET /api/annotations?study_instance_uid=1.2.840.113619.2.55.3.604688119.868.1234567890.1
```

**응답 (200 OK):**
```json
{
  "annotations": [
    {
      "id": 44,
      "user_id": 336,
      "study_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
      "series_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
      "sop_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
      "annotation_data": {
        "color": "#FF0000",
        "label": "Test Annotation",
        "radius": 50,
        "type": "circle",
        "x": 100,
        "y": 200
      },
      "description": null,
      "created_at": "2025-10-07T10:16:40.072803",
      "updated_at": "2025-10-07T10:16:40.072803"
    }
  ],
  "total": 1
}
```

### 3. 특정 어노테이션 조회
```http
GET /api/annotations/{id}
```

**응답 (200 OK):**
```json
{
  "id": 44,
  "user_id": 336,
  "study_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
  "series_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
  "sop_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
  "annotation_data": {
    "color": "#FF0000",
    "label": "Test Annotation",
    "radius": 50,
    "type": "circle",
    "x": 100,
    "y": 200
  },
  "description": null,
  "created_at": "2025-10-07T10:16:40.072803",
  "updated_at": "2025-10-07T10:16:40.072803"
}
```

### 4. 어노테이션 수정
```http
PUT /api/annotations/{id}
Content-Type: application/json
```

**요청 본문:**
```json
{
  "annotation_data": {
    "type": "rectangle",
    "x": 50,
    "y": 50,
    "width": 200,
    "height": 100,
    "color": "#0000FF",
    "label": "Updated Annotation"
  },
  "description": "Updated description"
}
```

### 5. 어노테이션 삭제
```http
DELETE /api/annotations/{id}
```

**응답 (200 OK):**
```json
{
  "message": "Annotation deleted successfully"
}
```

## 📝 어노테이션 데이터 타입

### 원형 어노테이션
```json
{
  "type": "circle",
  "x": 150,
  "y": 150,
  "radius": 75,
  "color": "#00FF00",
  "label": "Circle Annotation"
}
```

### 사각형 어노테이션
```json
{
  "type": "rectangle",
  "x": 50,
  "y": 50,
  "width": 200,
  "height": 100,
  "color": "#0000FF",
  "label": "Rectangle Annotation"
}
```

### 점 어노테이션
```json
{
  "type": "point",
  "x": 300,
  "y": 250,
  "color": "#FFFF00",
  "label": "Point Annotation"
}
```

### 다각형 어노테이션
```json
{
  "type": "polygon",
  "points": [
    {"x": 100, "y": 100},
    {"x": 200, "y": 100},
    {"x": 200, "y": 200},
    {"x": 100, "y": 200}
  ],
  "color": "#FF00FF",
  "label": "Polygon Annotation"
}
```

## 🔍 쿼리 파라미터

| 파라미터 | 타입 | 설명 | 예시 |
|---------|------|------|------|
| `user_id` | integer | 사용자 ID로 필터링 | `?user_id=336` |
| `project_id` | integer | 프로젝트 ID로 필터링 | `?project_id=302` |
| `study_instance_uid` | string | Study Instance UID로 필터링 | `?study_instance_uid=1.2.840.113619.2.55.3.604688119.868.1234567890.1` |

## 📊 HTTP 상태 코드

| 코드 | 설명 | 상황 |
|------|------|------|
| 200 | OK | 조회 성공 |
| 201 | Created | 생성 성공 |
| 400 | Bad Request | 잘못된 요청 |
| 401 | Unauthorized | 인증 실패 |
| 404 | Not Found | 리소스 없음 |
| 500 | Internal Server Error | 서버 오류 |

## 🧪 테스트 예시

### cURL 명령어
```bash
# 어노테이션 생성
curl -X POST http://localhost:8080/api/annotations \
  -H "Content-Type: application/json" \
  -d '{
    "study_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
    "series_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
    "sop_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
    "annotation_data": {
      "type": "circle",
      "x": 100,
      "y": 200,
      "radius": 50,
      "color": "#FF0000",
      "label": "Test Annotation"
    },
    "description": "This is a test annotation"
  }'

# 어노테이션 목록 조회
curl -X GET "http://localhost:8080/api/annotations?user_id=336"

# 특정 어노테이션 조회
curl -X GET http://localhost:8080/api/annotations/44

# 어노테이션 수정
curl -X PUT http://localhost:8080/api/annotations/44 \
  -H "Content-Type: application/json" \
  -d '{
    "annotation_data": {
      "type": "rectangle",
      "x": 50,
      "y": 50,
      "width": 200,
      "height": 100,
      "color": "#0000FF",
      "label": "Updated Annotation"
    },
    "description": "Updated description"
  }'

# 어노테이션 삭제
curl -X DELETE http://localhost:8080/api/annotations/44
```

## 📚 추가 정보

- **Swagger UI**: http://localhost:8080/swagger-ui/
- **OpenAPI JSON**: http://localhost:8080/api-docs/openapi.json
- **헬스 체크**: http://localhost:8080/health

## 🏗️ 아키텍처 정보

이 API는 Clean Architecture 패턴을 따르며, 다음과 같은 계층으로 구성되어 있습니다:

- **Presentation Layer**: HTTP 컨트롤러 및 라우팅
- **Application Layer**: 유스케이스 및 DTO
- **Domain Layer**: 비즈니스 로직 및 엔티티
- **Infrastructure Layer**: 데이터베이스 및 외부 서비스

## 🔒 보안 고려사항

- 현재는 테스트용으로 하드코딩된 사용자 ID를 사용
- 실제 운영 환경에서는 JWT 토큰 기반 인증 필요
- DICOM UID 검증 및 입력 데이터 검증 필요

## 📈 성능 최적화

- HTTP 캐싱 헤더 적용 (GET 요청)
- 데이터베이스 인덱스 최적화
- 비동기 처리로 높은 처리량 지원

이 API는 DICOM 표준을 따르며, 의료 영상 분석을 위한 다양한 어노테이션 타입을 지원합니다.

