# Role-Capability Matrix API 참고문서

## 개요

Role-Capability Matrix API는 역할(Role)과 역량(Capability) 간의 매트릭스를 관리하는 API입니다. 이 API를 통해 역할별로 할당된 역량을 조회하고, 페이지네이션 및 검색 기능을 제공합니다.

## 기본 정보

- **Base URL**: `http://localhost:8080/api`
- **Content-Type**: `application/json`
- **인증**: JWT 토큰 필요 (Authorization 헤더)

## API 엔드포인트

### 1. 전역 Role-Capability 매트릭스 조회 (페이지네이션 및 검색)

#### 요청
```http
GET /api/roles/global/capabilities/matrix
```

#### 쿼리 파라미터
| 파라미터 | 타입 | 필수 | 기본값 | 설명 |
|---------|------|------|--------|------|
| `page` | integer | 아니오 | 1 | 페이지 번호 (1부터 시작) |
| `size` | integer | 아니오 | 10 | 페이지 크기 (최대 100) |
| `search` | string | 아니오 | - | 역할 이름 또는 설명 검색 |
| `scope` | string | 아니오 | - | 역할 범위 필터 (GLOBAL, PROJECT) |

#### 응답 예시
```json
{
  "roles": [
    {
      "id": 1,
      "name": "ADMIN",
      "description": "시스템 관리자",
      "scope": "GLOBAL"
    },
    {
      "id": 2,
      "name": "USER",
      "description": "일반 사용자",
      "scope": "GLOBAL"
    }
  ],
  "capabilities_by_category": {
    "관리": [
      {
        "id": 1,
        "name": "사용자 관리",
        "description": "사용자 계정 관리",
        "category": "관리",
        "permissions": [
          {
            "id": 1,
            "name": "사용자 생성",
            "description": "새 사용자 계정 생성",
            "resource_type": "USER",
            "action": "CREATE",
            "category": "관리"
          }
        ]
      }
    ],
    "DICOM": [
      {
        "id": 5,
        "name": "DICOM 읽기",
        "description": "DICOM 이미지 조회",
        "category": "DICOM",
        "permissions": [
          {
            "id": 10,
            "name": "Study 조회",
            "description": "DICOM Study 조회",
            "resource_type": "STUDY",
            "action": "READ",
            "category": "DICOM"
          }
        ]
      }
    ]
  },
  "assignments": [
    {
      "role_id": 1,
      "capability_id": 1
    },
    {
      "role_id": 1,
      "capability_id": 5
    }
  ],
  "pagination": {
    "current_page": 1,
    "page_size": 10,
    "total_pages": 1,
    "total_items": 4,
    "has_next": false,
    "has_previous": false
  }
}
```

#### HTTP 상태 코드
- `200 OK`: 성공
- `400 Bad Request`: 잘못된 요청 파라미터
- `500 Internal Server Error`: 서버 내부 오류

### 2. 전역 Role-Capability 매트릭스 조회 (전체 데이터)

#### 요청
```http
GET /api/roles/global/capabilities/matrix/all
```

이 엔드포인트는 페이지네이션 없이 모든 데이터를 반환합니다. 하위 호환성을 위해 유지됩니다.

### 3. 프로젝트별 Role-Capability 매트릭스 조회

#### 요청
```http
GET /api/projects/{project_id}/roles/capabilities/matrix
```

#### 경로 파라미터
| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | 예 | 프로젝트 ID |

### 4. 모든 Capability 조회

#### 요청
```http
GET /api/capabilities
```

#### 응답 예시
```json
[
  {
    "id": 1,
    "name": "사용자 관리",
    "description": "사용자 계정 관리",
    "category": "관리",
    "permissions": [
      {
        "id": 1,
        "name": "사용자 생성",
        "description": "새 사용자 계정 생성",
        "resource_type": "USER",
        "action": "CREATE",
        "category": "관리"
      }
    ]
  }
]
```

### 5. 특정 Capability 상세 조회

#### 요청
```http
GET /api/capabilities/{capability_id}
```

#### 경로 파라미터
| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `capability_id` | integer | 예 | Capability ID |

### 6. 카테고리별 Capability 조회

#### 요청
```http
GET /api/capabilities/category/{category}
```

#### 경로 파라미터
| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `category` | string | 예 | 카테고리명 (예: "관리", "DICOM") |

### 7. Role-Capability 할당 업데이트

#### 요청
```http
PUT /api/roles/{role_id}/capabilities/{capability_id}
```

#### 경로 파라미터
| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `role_id` | integer | 예 | Role ID |
| `capability_id` | integer | 예 | Capability ID |

#### 요청 본문
```json
{
  "assigned": true
}
```

#### 응답 예시
```json
{
  "success": true,
  "message": "Role-Capability 할당이 업데이트되었습니다"
}
```

## 사용 예시

### 1. 페이지네이션을 사용한 역할 목록 조회
```bash
curl -X GET "http://localhost:8080/api/roles/global/capabilities/matrix?page=1&size=5" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 2. 역할 이름으로 검색
```bash
curl -X GET "http://localhost:8080/api/roles/global/capabilities/matrix?search=admin" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 3. 특정 범위의 역할만 조회
```bash
curl -X GET "http://localhost:8080/api/roles/global/capabilities/matrix?scope=GLOBAL" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 4. 복합 검색 (페이지네이션 + 검색 + 범위)
```bash
curl -X GET "http://localhost:8080/api/roles/global/capabilities/matrix?page=1&size=10&search=user&scope=GLOBAL" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## 데이터 모델

### Role (역할)
```json
{
  "id": 1,
  "name": "ADMIN",
  "description": "시스템 관리자",
  "scope": "GLOBAL"
}
```

### Capability (역량)
```json
{
  "id": 1,
  "name": "사용자 관리",
  "description": "사용자 계정 관리",
  "category": "관리",
  "permissions": [...]
}
```

### Permission (권한)
```json
{
  "id": 1,
  "name": "사용자 생성",
  "description": "새 사용자 계정 생성",
  "resource_type": "USER",
  "action": "CREATE",
  "category": "관리"
}
```

### PaginationInfo (페이지네이션 정보)
```json
{
  "current_page": 1,
  "page_size": 10,
  "total_pages": 1,
  "total_items": 4,
  "has_next": false,
  "has_previous": false
}
```

## 에러 처리

### 일반적인 에러 응답
```json
{
  "error": "에러 타입",
  "message": "에러 메시지"
}
```

### 에러 코드
- `400 Bad Request`: 잘못된 요청 파라미터
- `401 Unauthorized`: 인증 실패
- `403 Forbidden`: 권한 부족
- `404 Not Found`: 리소스를 찾을 수 없음
- `500 Internal Server Error`: 서버 내부 오류

## 주의사항

1. **페이지 크기 제한**: `size` 파라미터는 최대 100까지 설정 가능합니다.
2. **검색 기능**: `search` 파라미터는 역할 이름과 설명에서 대소문자 구분 없이 검색합니다.
3. **범위 필터**: `scope` 파라미터는 "GLOBAL" 또는 "PROJECT" 값만 허용됩니다.
4. **인증**: 모든 API 호출에는 유효한 JWT 토큰이 필요합니다.

## 버전 정보

- **API 버전**: v1.0
- **최종 업데이트**: 2024년 1월
- **호환성**: 하위 호환성 유지 (기존 `/all` 엔드포인트 지원)
