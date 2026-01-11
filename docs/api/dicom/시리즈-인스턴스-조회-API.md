# 시리즈 Instance 조회 API

## 개요

`GET /api/me/dicom/studies/{study_uid}/series/{series_uid}/instances` 엔드포인트는 특정 Series에 속한 모든 Instance(이미지)를 조회하는 API입니다.

### 주요 특징

- **RBAC 기반 권한 검증**: 사용자의 역할과 권한에 따라 자동으로 접근 제어
- **정렬 기능**: InstanceNumber 기준 정렬 지원
- **DICOM 메타데이터 제공**: 각 Instance의 DICOM 표준 메타데이터 포함
- **WADO-RS URL 제공**: 이미지 다운로드를 위한 WADO-RS URL 자동 생성

---

## 엔드포인트

```
GET /api/me/dicom/studies/{study_uid}/series/{series_uid}/instances
```

---

## 인증

### 헤더

| 헤더 | 필수 | 설명 |
|------|------|------|
| `Authorization` | ✅ | Bearer 토큰 (JWT) |

### 예시

```http
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

---

## Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `study_uid` | string | ✅ | Study Instance UID (DICOM 표준) |
| `series_uid` | string | ✅ | Series Instance UID (DICOM 표준) |

---

## Query Parameters

| 파라미터 | 타입 | 필수 | 기본값 | 설명 |
|---------|------|------|--------|------|
| `project_id` | integer | ✅ | - | 프로젝트 ID (RBAC 권한 검증에 사용) |
| `orderby` | string | ❌ | - | 정렬 기준 (현재 `InstanceNumber`만 지원) |

---

## 요청 예시

### 1. 기본 조회

```bash
curl -X GET "http://localhost:8080/api/me/dicom/studies/1.2.840.113619.2.55.3.604688433.1234/series/1.2.840.113619.2.55.3.604688433.5678/instances?project_id=2" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 2. InstanceNumber 순서로 정렬

```bash
curl -X GET "http://localhost:8080/api/me/dicom/studies/1.2.840.113619.2.55.3.604688433.1234/series/1.2.840.113619.2.55.3.604688433.5678/instances?project_id=2&orderby=InstanceNumber" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 응답 형식

### 성공 응답 (200 OK)

응답은 DICOM JSON 배열 형식으로 반환되며, 각 Instance는 DICOM 표준 메타데이터를 포함합니다.

```json
[
  {
    "00080005": {
      "vr": "CS",
      "Value": ["ISO_IR 100"]
    },
    "00080016": {
      "vr": "UI",
      "Value": ["1.2.840.10008.5.1.4.1.1.2"]
    },
    "00080018": {
      "vr": "UI",
      "Value": ["1.2.840.113619.2.55.3.604688433.9012"]
    },
    "00080054": {
      "vr": "AE",
      "Value": ["iAID_PACS"]
    },
    "00080056": {
      "vr": "CS",
      "Value": ["ONLINE"]
    },
    "00081190": {
      "vr": "UR",
      "Value": ["http://archive.pacs.ai-do.co.kr/iaid-pacs/aets/iAID_PACS/rs/studies/1.2.840.113619.2.55.3.604688433.1234/series/1.2.840.113619.2.55.3.604688433.5678/instances/1.2.840.113619.2.55.3.604688433.9012"]
    },
    "0020000D": {
      "vr": "UI",
      "Value": ["1.2.840.113619.2.55.3.604688433.1234"]
    },
    "0020000E": {
      "vr": "UI",
      "Value": ["1.2.840.113619.2.55.3.604688433.5678"]
    },
    "00200013": {
      "vr": "IS",
      "Value": ["1"]
    },
    "00280010": {
      "vr": "US",
      "Value": [512]
    },
    "00280011": {
      "vr": "US",
      "Value": [512]
    },
    "00280100": {
      "vr": "US",
      "Value": [16]
    }
  }
]
```

### DICOM 필드 설명

| DICOM 태그 | 이름 | VR | 설명 |
|-----------|------|----|----|
| `00080005` | SpecificCharacterSet | CS | 문자 인코딩 (예: ISO_IR 100) |
| `00080016` | SOPClassUID | UI | SOP Class UID (이미지 타입) |
| `00080018` | SOPInstanceUID | UI | Instance 고유 식별자 |
| `00080054` | RetrieveAETitle | AE | PACS AE Title |
| `00080056` | InstanceAvailability | CS | Instance 가용성 (ONLINE/NEARLINE/OFFLINE) |
| `00081190` | RetrieveURL | UR | WADO-RS 이미지 다운로드 URL |
| `0020000D` | StudyInstanceUID | UI | Study 고유 식별자 |
| `0020000E` | SeriesInstanceUID | UI | Series 고유 식별자 |
| `00200013` | InstanceNumber | IS | Instance 번호 (정렬 기준) |
| `00280010` | Rows | US | 이미지 높이 (픽셀) |
| `00280011` | Columns | US | 이미지 너비 (픽셀) |
| `00280100` | BitsAllocated | US | 픽셀당 비트 수 |

---

## 에러 응답

### 401 Unauthorized

인증 토큰이 없거나 유효하지 않은 경우

```json
{
  "error": "Invalid or missing authorization token"
}
```

### 403 Forbidden

Series에 대한 접근 권한이 없는 경우

```json
{
  "error": "Access denied to this series"
}
```

### 404 Not Found

Series가 존재하지 않는 경우

```json
{
  "error": "Series not found"
}
```

### 400 Bad Request

잘못된 파라미터

```json
{
  "error": "project_id is required"
}
```

### 502 Bad Gateway

QIDO 서버 연결 실패

```json
{
  "error": "Failed to connect to QIDO server: connection timeout"
}
```

---

## 동작 방식

### 1. 권한 검증

먼저 사용자가 해당 Series에 접근할 수 있는지 RBAC 평가를 수행합니다.

```rust
// Series 접근 권한 검증
let rbac_result = evaluator.evaluate_series_uid(
    user_id,
    project_id,
    &study_uid,
    &series_uid
).await;

if !rbac_result.allowed {
    return Err(ServiceError::Forbidden("Access denied".to_string()));
}
```

### 2. QIDO-RS 요청

권한이 확인되면 QIDO-RS를 통해 Instance 목록을 조회합니다.

```rust
// QIDO-RS Instances 조회
let instances = qido_client.qido_instances(
    &study_uid,
    &series_uid,
    bearer_token
).await?;
```

### 3. 정렬 적용

`orderby` 파라미터가 제공된 경우 Instance를 정렬합니다.

```rust
// InstanceNumber로 정렬
if orderby == "InstanceNumber" {
    instances.sort_by(|a, b| {
        let a_num = extract_instance_number(a).unwrap_or(0);
        let b_num = extract_instance_number(b).unwrap_or(0);
        a_num.cmp(&b_num)
    });
}
```

### 4. 응답 반환

정렬된 Instance 목록을 DICOM JSON 형식으로 반환합니다.

---

## 이미지 다운로드

응답에 포함된 `RetrieveURL` (태그 `00081190`)을 사용하여 이미지를 다운로드할 수 있습니다.

### WADO-RS URL 형식

```
http://archive.pacs.ai-do.co.kr/iaid-pacs/aets/iAID_PACS/rs/studies/{study_uid}/series/{series_uid}/instances/{instance_uid}
```

### 이미지 다운로드 예시

```bash
# DICOM 파일 다운로드
curl -X GET "http://archive.pacs.ai-do.co.kr/iaid-pacs/aets/iAID_PACS/rs/studies/1.2.840.113619.2.55.3.604688433.1234/series/1.2.840.113619.2.55.3.604688433.5678/instances/1.2.840.113619.2.55.3.604688433.9012" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Accept: application/dicom" \
  -o image.dcm

# JPEG 이미지 다운로드
curl -X GET "http://archive.pacs.ai-do.co.kr/iaid-pacs/aets/iAID_PACS/rs/studies/1.2.840.113619.2.55.3.604688433.1234/series/1.2.840.113619.2.55.3.604688433.5678/instances/1.2.840.113619.2.55.3.604688433.9012/frames/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Accept: image/jpeg" \
  -o image.jpg
```

---

## 사용 사례

### 사례 1: DICOM Viewer에서 시리즈 로드

```javascript
// 1. Instance 목록 조회
const response = await fetch(
  `http://localhost:8080/api/me/dicom/studies/${studyUid}/series/${seriesUid}/instances?project_id=2&orderby=InstanceNumber`,
  {
    headers: {
      'Authorization': `Bearer ${token}`
    }
  }
);

const instances = await response.json();

// 2. 각 Instance의 이미지 URL 추출
const imageUrls = instances.map(instance => {
  return instance['00081190'].Value[0]; // RetrieveURL
});

// 3. Viewer에 로드
viewer.loadImages(imageUrls);
```

### 사례 2: 썸네일 생성

```javascript
// 1. 첫 번째 Instance만 조회 (썸네일용)
const response = await fetch(
  `http://localhost:8080/api/me/dicom/studies/${studyUid}/series/${seriesUid}/instances?project_id=2&orderby=InstanceNumber`,
  {
    headers: {
      'Authorization': `Bearer ${token}`
    }
  }
);

const instances = await response.json();

// 2. 첫 번째 Instance의 JPEG 썸네일 URL 생성
const firstInstance = instances[0];
const instanceUid = firstInstance['00080018'].Value[0];
const thumbnailUrl = `${baseUrl}/studies/${studyUid}/series/${seriesUid}/instances/${instanceUid}/frames/1`;

// 3. 썸네일 표시
document.getElementById('thumbnail').src = thumbnailUrl;
```

### 사례 3: 전체 시리즈 다운로드

```bash
#!/bin/bash

# 1. Instance 목록 조회
INSTANCES=$(curl -s -X GET \
  "http://localhost:8080/api/me/dicom/studies/${STUDY_UID}/series/${SERIES_UID}/instances?project_id=2&orderby=InstanceNumber" \
  -H "Authorization: Bearer ${TOKEN}")

# 2. 각 Instance 다운로드
echo "$INSTANCES" | jq -r '.[].["00081190"].Value[0]' | while read url; do
  filename=$(basename "$url")
  curl -X GET "$url" \
    -H "Authorization: Bearer ${TOKEN}" \
    -H "Accept: application/dicom" \
    -o "${filename}.dcm"
  echo "Downloaded: ${filename}.dcm"
done
```

---

## 권한 요구사항

### 필수 권한

- **인증된 사용자**: 로그인한 사용자만 접근 가능
- **프로젝트 멤버**: 조회하려는 프로젝트의 멤버여야 함
- **Series 접근 권한**: RBAC 평가를 통과해야 함

### 권한별 동작

| 역할 | 동작 |
|------|------|
| **일반 사용자** | 할당된 Series만 조회 가능 |
| **프로젝트 관리자** | 해당 프로젝트의 모든 Series 조회 가능 |
| **SUPER_ADMIN** | 모든 Series 조회 가능 |

---

## 관련 API

- `GET /api/me/dicom/studies` - 사용자의 모든 Study 조회
- `GET /api/me/dicom/studies/{study_uid}/series` - 특정 Study의 Series 조회
- `GET /api/me/dicom/series` - 사용자의 모든 Series 조회

---

## 성능 고려사항

### Instance 개수

일반적인 CT 시리즈는 수백 개의 Instance를 포함할 수 있습니다.

| Modality | 평균 Instance 수 | 예상 응답 크기 |
|----------|-----------------|---------------|
| CT | 100-500 | 50-250 KB |
| MR | 50-200 | 25-100 KB |
| US | 10-50 | 5-25 KB |
| CR/DX | 1-5 | 0.5-2.5 KB |

### 최적화 팁

1. **필요한 경우에만 정렬**: `orderby` 파라미터는 필요한 경우에만 사용
2. **페이지네이션 고려**: Instance가 많은 경우 클라이언트에서 페이지네이션 구현
3. **캐싱 활용**: 동일한 Series를 반복 조회하는 경우 클라이언트 캐싱 활용

---

## 문제 해결

### Q: Instance가 정렬되지 않습니다

**A**: `orderby=InstanceNumber` 파라미터를 추가하세요.

```bash
# 정렬 없음
GET /api/me/dicom/studies/{study_uid}/series/{series_uid}/instances?project_id=2

# InstanceNumber로 정렬
GET /api/me/dicom/studies/{study_uid}/series/{series_uid}/instances?project_id=2&orderby=InstanceNumber
```

### Q: 403 Forbidden 에러가 발생합니다

**A**: 다음을 확인하세요:
- `project_id`가 올바른지 확인
- 해당 프로젝트의 멤버인지 확인
- `project_data_access` 테이블에 Series가 할당되어 있는지 확인
- RBAC 권한 설정 확인

### Q: 이미지를 다운로드할 수 없습니다

**A**: 다음을 확인하세요:
- `RetrieveURL` (태그 `00081190`)이 응답에 포함되어 있는지 확인
- WADO-RS 서버가 정상 작동하는지 확인
- Bearer 토큰이 유효한지 확인

---

## 참고 사항

### InstanceNumber

- DICOM 표준 태그 (0020,0013)
- 시리즈 내에서 Instance의 순서를 나타냄
- 일반적으로 1부터 시작하여 순차적으로 증가
- CT의 경우 슬라이스 위치에 따라 정렬됨

### WADO-RS

- Web Access to DICOM Objects - RESTful Services
- DICOM 표준 (PS3.18)
- HTTP를 통한 DICOM 객체 조회
- 다양한 형식 지원 (DICOM, JPEG, PNG 등)

---

## 변경 이력

| 날짜 | 버전 | 변경 내용 |
|------|------|----------|
| 2026-01-09 | 1.0.0 | 초기 문서 작성 |
| 2026-01-09 | 1.0.0 | InstanceNumber 정렬 기능 추가 |


