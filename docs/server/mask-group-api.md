# Mask Group API Documentation

## 개요

Mask Group API는 의료 영상 어노테이션에 대한 마스크 그룹을 관리하는 RESTful API입니다. 이 API를 통해 AI 모델이 생성한 세그멘테이션 마스크를 그룹화하고 관리할 수 있습니다.

## 아키텍처

### 계층 구조
```
Presentation Layer (Controller)
    ↓
Application Layer (Use Case)
    ↓
Domain Layer (Service)
    ↓
Infrastructure Layer (Repository)
    ↓
Database (PostgreSQL)
```

### 주요 컴포넌트

1. **MaskGroupController**: HTTP 요청/응답 처리
2. **MaskGroupUseCase**: 비즈니스 로직 오케스트레이션
3. **MaskGroupService**: 도메인 서비스 로직
4. **MaskGroupRepository**: 데이터 접근 계층

## API 엔드포인트

### 1. 마스크 그룹 생성

**POST** `/api/annotations/{annotation_id}/mask-groups`

새로운 마스크 그룹을 생성합니다.

#### 요청 파라미터
- `annotation_id` (path): 어노테이션 ID

#### 요청 본문
```json
{
  "group_name": "Liver_Segmentation_v2",
  "model_name": "monai_unet",
  "version": "2.1.0",
  "modality": "CT",
  "slice_count": 120,
  "mask_type": "segmentation",
  "description": "간 세그멘테이션 결과"
}
```

#### 응답
```json
{
  "id": 1,
  "annotation_id": 112,
  "group_name": "Liver_Segmentation_v2",
  "model_name": "monai_unet",
  "version": "2.1.0",
  "modality": "CT",
  "slice_count": 120,
  "mask_type": "segmentation",
  "description": "간 세그멘테이션 결과",
  "created_by": 1,
  "created_at": "2025-01-23T04:10:35.989242Z",
  "updated_at": "2025-01-23T04:10:35.989242Z"
}
```

### 2. 마스크 그룹 목록 조회

**GET** `/api/annotations/{annotation_id}/mask-groups`

특정 어노테이션의 마스크 그룹 목록을 조회합니다.

#### 쿼리 파라미터
- `offset` (optional): 페이지 오프셋 (기본값: 0)
- `limit` (optional): 페이지 크기 (기본값: 50)
- `modality` (optional): 모달리티 필터
- `mask_type` (optional): 마스크 타입 필터

#### 응답
```json
{
  "mask_groups": [
    {
      "id": 1,
      "annotation_id": 112,
      "group_name": "Liver_Segmentation_v2",
      "model_name": "monai_unet",
      "version": "2.1.0",
      "modality": "CT",
      "slice_count": 120,
      "mask_type": "segmentation",
      "description": "간 세그멘테이션 결과",
      "created_by": 1,
      "created_at": "2025-01-23T04:10:35.989242Z",
      "updated_at": "2025-01-23T04:10:35.989242Z"
    }
  ],
  "total_count": 1,
  "offset": 0,
  "limit": 50
}
```

### 3. 마스크 그룹 상세 조회

**GET** `/api/annotations/{annotation_id}/mask-groups/{group_id}`

특정 마스크 그룹의 상세 정보를 조회합니다.

#### 응답
```json
{
  "id": 1,
  "annotation_id": 112,
  "group_name": "Liver_Segmentation_v2",
  "model_name": "monai_unet",
  "version": "2.1.0",
  "modality": "CT",
  "slice_count": 120,
  "mask_type": "segmentation",
  "description": "간 세그멘테이션 결과",
  "created_by": 1,
  "created_at": "2025-01-23T04:10:35.989242Z",
  "updated_at": "2025-01-23T04:10:35.989242Z",
  "masks": [],
  "stats": {
    "total_groups": 1,
    "modality_counts": {
      "CT": 1
    },
    "mask_type_counts": {
      "segmentation": 1
    }
  }
}
```

### 4. 마스크 그룹 수정

**PUT** `/api/annotations/{annotation_id}/mask-groups/{group_id}`

마스크 그룹 정보를 수정합니다.

#### 요청 본문
```json
{
  "group_name": "Liver_Segmentation_v3",
  "description": "업데이트된 간 세그멘테이션 결과"
}
```

### 5. 마스크 그룹 삭제

**DELETE** `/api/annotations/{annotation_id}/mask-groups/{group_id}`

마스크 그룹을 삭제합니다.

#### 응답
- `204 No Content`: 성공적으로 삭제됨

### 6. 업로드 URL 생성

**POST** `/api/annotations/{annotation_id}/mask-groups/{group_id}/upload-url`

마스크 파일 업로드를 위한 서명된 URL을 생성합니다.

#### 요청 본문
```json
{
  "file_name": "mask_slice_001.png",
  "content_type": "image/png",
  "expires_in": 3600
}
```

#### 응답
```json
{
  "upload_url": "https://s3.amazonaws.com/bucket/masks/...",
  "expires_at": "2025-01-23T05:10:35Z",
  "file_key": "masks/group_1/slice_001.png"
}
```

### 7. 업로드 완료 확인

**POST** `/api/annotations/{annotation_id}/mask-groups/{group_id}/complete-upload`

마스크 파일 업로드 완료를 확인합니다.

#### 요청 본문
```json
{
  "file_key": "masks/group_1/slice_001.png",
  "file_size": 1024000,
  "checksum": "sha256:abc123..."
}
```

## 데이터 모델

### MaskGroup 엔티티

```rust
pub struct MaskGroup {
    pub id: i32,
    pub annotation_id: i32,
    pub group_name: Option<String>,
    pub model_name: Option<String>,
    pub version: Option<String>,
    pub modality: Option<String>,
    pub slice_count: Option<i32>,
    pub mask_type: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### CreateMaskGroupRequest DTO

```rust
pub struct CreateMaskGroupRequest {
    pub group_name: Option<String>,
    pub model_name: Option<String>,
    pub version: Option<String>,
    pub modality: Option<String>,
    pub slice_count: i32,
    pub mask_type: String,
    pub description: Option<String>,
}
```

## 에러 처리

### HTTP 상태 코드

- `200 OK`: 성공적인 조회
- `201 Created`: 성공적인 생성
- `204 No Content`: 성공적인 삭제
- `400 Bad Request`: 잘못된 요청
- `401 Unauthorized`: 인증 실패
- `403 Forbidden`: 권한 부족
- `404 Not Found`: 리소스 없음
- `500 Internal Server Error`: 서버 오류

### 에러 응답 형식

```json
{
  "error": "Validation Error",
  "message": "Invalid mask_type value"
}
```

## 인증 및 권한

모든 API 엔드포인트는 JWT 토큰 기반 인증이 필요합니다.

### 헤더
```
Authorization: Bearer <jwt_token>
```

### 권한 요구사항
- 마스크 그룹 생성/수정/삭제: 해당 어노테이션에 대한 쓰기 권한
- 마스크 그룹 조회: 해당 어노테이션에 대한 읽기 권한

## 사용 예제

### Python 클라이언트 예제

```python
import requests

# 마스크 그룹 생성
def create_mask_group(annotation_id, token):
    url = f"http://localhost:8080/api/annotations/{annotation_id}/mask-groups"
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    data = {
        "group_name": "Liver_Segmentation_v2",
        "model_name": "monai_unet",
        "version": "2.1.0",
        "modality": "CT",
        "slice_count": 120,
        "mask_type": "segmentation",
        "description": "간 세그멘테이션 결과"
    }
    
    response = requests.post(url, json=data, headers=headers)
    return response.json()

# 마스크 그룹 목록 조회
def list_mask_groups(annotation_id, token):
    url = f"http://localhost:8080/api/annotations/{annotation_id}/mask-groups"
    headers = {"Authorization": f"Bearer {token}"}
    
    response = requests.get(url, headers=headers)
    return response.json()
```

## 성능 고려사항

1. **페이지네이션**: 대량의 마스크 그룹 조회 시 페이지네이션 사용
2. **인덱싱**: `annotation_id`, `modality`, `mask_type` 필드에 인덱스 적용
3. **캐싱**: 자주 조회되는 마스크 그룹 정보 캐싱 고려
4. **비동기 처리**: 대용량 파일 업로드 시 비동기 처리

## 보안 고려사항

1. **입력 검증**: 모든 입력 데이터에 대한 검증
2. **SQL 인젝션 방지**: 준비된 문(Prepared Statement) 사용
3. **권한 검사**: 각 요청에 대한 적절한 권한 검사
4. **파일 업로드 보안**: 업로드 파일의 타입 및 크기 제한

## 모니터링 및 로깅

1. **요청 로깅**: 모든 API 요청에 대한 로그 기록
2. **에러 추적**: 에러 발생 시 상세한 스택 트레이스 기록
3. **성능 메트릭**: 응답 시간 및 처리량 모니터링
4. **사용량 통계**: API 사용 패턴 분석
