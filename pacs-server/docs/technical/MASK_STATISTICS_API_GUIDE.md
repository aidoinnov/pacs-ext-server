# 마스크 통계 API 가이드

## 개요

마스크 통계 API는 마스크 그룹의 통계 정보를 조회하는 기능을 제공합니다. 이 API를 통해 마스크의 개수, 파일 크기, MIME 타입 분포, 라벨 이름 분포 등의 통계 정보를 확인할 수 있습니다.

## API 엔드포인트

### 마스크 통계 조회

**GET** `/api/annotations/{annotation_id}/mask-groups/{group_id}/masks/stats`

#### 경로 매개변수

- `annotation_id` (integer): 어노테이션 ID
- `group_id` (integer): 마스크 그룹 ID

#### 헤더

- `X-User-ID` (string): 사용자 ID (테스트용)
- `Authorization` (string): JWT 토큰 (프로덕션용)

#### 응답

**성공 (200 OK)**
```json
{
  "total_masks": 150,
  "total_size_bytes": 52428800,
  "average_file_size": 349525.33,
  "masks_by_label": {
    "lung_nodule": 75,
    "liver_lesion": 45,
    "brain_tumor": 30
  },
  "mime_type_distribution": {
    "image/png": 100,
    "image/jpeg": 50
  }
}
```

**오류 응답**

- `401 Unauthorized`: 인증 실패
- `403 Forbidden`: 권한 없음
- `404 Not Found`: 마스크 그룹을 찾을 수 없음
- `500 Internal Server Error`: 서버 오류

## 구현 세부사항

### 데이터베이스 쿼리

마스크 통계는 다음 3개의 쿼리로 구성됩니다:

1. **기본 통계 조회**
```sql
SELECT 
    COUNT(*) as total_masks,
    COALESCE(SUM(file_size), 0) as total_size_bytes,
    COALESCE(AVG(file_size), 0) as average_file_size,
    COALESCE(MAX(file_size), 0) as largest_file_size,
    COALESCE(MIN(file_size), 0) as smallest_file_size
FROM annotation_mask
WHERE ($1::INTEGER IS NULL OR mask_group_id = $1)
```

2. **MIME 타입 분포 조회**
```sql
SELECT mime_type, COUNT(*) as count
FROM annotation_mask
WHERE ($1::INTEGER IS NULL OR mask_group_id = $1)
AND mime_type IS NOT NULL
GROUP BY mime_type
```

3. **라벨 이름 분포 조회**
```sql
SELECT label_name, COUNT(*) as count
FROM annotation_mask
WHERE ($1::INTEGER IS NULL OR mask_group_id = $1)
AND label_name IS NOT NULL
GROUP BY label_name
```

### 권한 확인

마스크 통계 조회 시 다음 권한 확인이 수행됩니다:

1. **마스크 그룹 존재 확인**: 지정된 마스크 그룹이 존재하는지 확인
2. **어노테이션 접근 권한**: 사용자가 해당 어노테이션에 접근할 수 있는지 확인
3. **마스크 그룹 생성자 권한**: 사용자가 마스크 그룹의 생성자인지 확인

권한 확인 로직:
```rust
// 어노테이션 소유자이거나 마스크 그룹 생성자인 경우 접근 허용
Ok(annotation.user_id == user_id || mask_group.created_by == Some(user_id))
```

## 사용 예시

### cURL 예시

```bash
# 마스크 통계 조회
curl -X GET \
  "http://localhost:8080/api/annotations/123/mask-groups/456/masks/stats" \
  -H "X-User-ID: 789" \
  -H "Content-Type: application/json"
```

### JavaScript 예시

```javascript
async function getMaskStats(annotationId, groupId, userId) {
  const response = await fetch(
    `/api/annotations/${annotationId}/mask-groups/${groupId}/masks/stats`,
    {
      method: 'GET',
      headers: {
        'X-User-ID': userId.toString(),
        'Content-Type': 'application/json'
      }
    }
  );
  
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  
  return await response.json();
}

// 사용 예시
const stats = await getMaskStats(123, 456, 789);
console.log(`총 마스크 개수: ${stats.total_masks}`);
console.log(`총 파일 크기: ${stats.total_size_bytes} bytes`);
```

## 성능 고려사항

### 인덱스 최적화

마스크 통계 조회 성능을 위해 다음 인덱스가 권장됩니다:

```sql
-- 마스크 그룹 ID 인덱스
CREATE INDEX idx_annotation_mask_mask_group_id ON annotation_mask(mask_group_id);

-- MIME 타입 인덱스
CREATE INDEX idx_annotation_mask_mime_type ON annotation_mask(mime_type);

-- 라벨 이름 인덱스
CREATE INDEX idx_annotation_mask_label_name ON annotation_mask(label_name);
```

### 캐싱 전략

자주 조회되는 통계 정보는 캐싱을 고려할 수 있습니다:

- **Redis 캐싱**: 마스크 그룹별 통계 정보를 5-10분간 캐싱
- **캐시 무효화**: 마스크 생성/수정/삭제 시 해당 그룹의 캐시 무효화

## 테스트

### 단위 테스트

```rust
#[actix_web::test]
async fn test_get_mask_stats_success() {
    // 테스트 데이터 생성
    let (annotation_id, mask_group_id, user_id) = create_test_data(&pool).await;
    
    // 마스크 생성
    let create_req = CreateMaskRequest { /* ... */ };
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    
    // 통계 조회
    let req = test::TestRequest::get()
        .uri(&format!("/api/annotations/{}/mask-groups/{}/masks/stats", 
                      annotation_id, mask_group_id))
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}
```

### 통합 테스트

마스크 통계 API는 다음 시나리오를 테스트합니다:

1. **정상 조회**: 유효한 마스크 그룹의 통계 조회
2. **권한 없음**: 접근 권한이 없는 사용자의 요청
3. **존재하지 않는 그룹**: 존재하지 않는 마스크 그룹 조회
4. **빈 그룹**: 마스크가 없는 그룹의 통계 조회

## 문제 해결

### 일반적인 문제

1. **404 Not Found**
   - 마스크 그룹이 존재하지 않음
   - 라우팅 순서 문제 (Actix-web에서 구체적인 라우트가 먼저 정의되어야 함)

2. **401 Unauthorized**
   - 사용자 인증 실패
   - X-User-ID 헤더 누락 (테스트 환경)

3. **403 Forbidden**
   - 마스크 그룹에 대한 접근 권한 없음
   - 어노테이션 소유자가 아니고 마스크 그룹 생성자도 아님

### 디버깅 팁

1. **권한 확인 로그**: `can_access_mask_group` 함수의 디버그 출력 확인
2. **라우팅 확인**: Actix-web 라우팅 순서 확인
3. **데이터베이스 확인**: 마스크 그룹과 어노테이션 데이터 존재 여부 확인

## 향후 개선사항

1. **실시간 통계**: WebSocket을 통한 실시간 통계 업데이트
2. **고급 필터링**: 날짜 범위, 파일 크기 범위 등으로 필터링
3. **통계 내보내기**: CSV, Excel 형태로 통계 데이터 내보내기
4. **시각화**: 차트와 그래프를 통한 통계 시각화
