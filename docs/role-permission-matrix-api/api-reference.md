# 롤별 권한 관리 API 참조

> **참고**: 이 문서는 상세한 API 명세를 제공합니다. 빠른 시작을 원한다면 [README.md](README.md)를 참고하세요.

## 📋 API 엔드포인트 목록

### 1. 매트릭스 조회

| 메서드 | 엔드포인트 | 설명 |
|--------|------------|------|
| `GET` | `/api/roles/global/permissions/matrix` | 글로벌 롤-권한 매트릭스 조회 |

### 2. 권한 할당/제거

| 메서드 | 엔드포인트 | 설명 |
|--------|------------|------|
| `PUT` | `/api/roles/{role_id}/permissions/{permission_id}` | 글로벌 롤에 권한 할당/제거 |

## 🔍 상세 API 명세

### GET /api/roles/global/permissions/matrix

**설명**: 글로벌 범위의 모든 역할과 권한 간의 관계를 매트릭스 형태로 조회합니다.

**요청 헤더**:
```
Authorization: Bearer <jwt-token>
Content-Type: application/json
```

**응답**:
```json
{
  "roles": [
    {
      "id": 1,
      "name": "Admin",
      "description": "시스템 관리자",
      "scope": "GLOBAL"
    }
  ],
  "permissions_by_category": {
    "USER": [
      {
        "id": 1,
        "resource_type": "USER",
        "action": "CREATE",
        "description": "사용자 생성"
      }
    ]
  },
  "assignments": [
    {
      "role_id": 1,
      "permission_id": 1,
      "assigned": true
    }
  ]
}
```

**상태 코드**:
- `200 OK`: 성공
- `401 Unauthorized`: 인증 실패
- `500 Internal Server Error`: 서버 오류

## 📊 데이터 모델 상세

### Role (롤)
```json
{
  "id": 1,
  "name": "Admin",
  "description": "시스템 관리자",
  "scope": "GLOBAL"
}
```

**필드 설명**:
- `id`: 롤 고유 식별자
- `name`: 롤 이름
- `description`: 롤 설명
- `scope`: 롤 범위 (GLOBAL)

### Permission (권한)
```json
{
  "id": 1,
  "resource_type": "USER",
  "action": "CREATE",
  "description": "사용자 생성"
}
```

**필드 설명**:
- `id`: 권한 고유 식별자
- `resource_type`: 리소스 타입 (USER, PROJECT, ANNOTATION 등)
- `action`: 액션 (CREATE, READ, UPDATE, DELETE)
- `description`: 권한 설명

### Assignment (할당)
```json
{
  "role_id": 1,
  "permission_id": 1,
  "assigned": true
}
```

**필드 설명**:
- `role_id`: 롤 ID
- `permission_id`: 권한 ID
- `assigned`: 할당 여부 (true/false)

## 🔐 인증 및 보안

### JWT 토큰 인증
모든 API 엔드포인트는 JWT Bearer Token을 통한 인증이 필요합니다.

**토큰 획득 방법**:
1. 로그인 API를 통해 토큰 획득
2. Authorization 헤더에 `Bearer <token>` 형식으로 포함

**예시**:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### 권한 요구사항
- **매트릭스 조회**: `ROLE_MANAGEMENT` 권한 필요
- **권한 할당/제거**: `ROLE_MANAGEMENT` 권한 필요

## ⚠️ 에러 처리

### 일반적인 에러 응답 형식
```json
{
  "error": "Error message",
  "code": "ERROR_CODE",
  "details": "Additional error details"
}
```

### 주요 에러 코드

| 상태 코드 | 에러 코드 | 설명 |
|-----------|-----------|------|
| 400 | `INVALID_REQUEST` | 잘못된 요청 형식 |
| 401 | `UNAUTHORIZED` | 인증 실패 |
| 403 | `FORBIDDEN` | 권한 부족 |
| 404 | `NOT_FOUND` | 리소스를 찾을 수 없음 |
| 409 | `CONFLICT` | 충돌 (이미 할당된 권한 등) |
| 500 | `INTERNAL_ERROR` | 서버 내부 오류 |

## 🚀 사용 예시

### cURL 예시

```bash
# 1. 매트릭스 조회
curl -X GET "http://localhost:8080/api/roles/global/permissions/matrix" \
  -H "Authorization: Bearer YOUR_TOKEN"

# 2. 권한 켜기
curl -X PUT "http://localhost:8080/api/roles/1/permissions/1" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"assign": true}'

# 3. 권한 끄기
curl -X PUT "http://localhost:8080/api/roles/1/permissions/1" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"assign": false}'
```

### JavaScript 예시

```javascript
// 매트릭스 조회
const response = await fetch('/api/roles/global/permissions/matrix', {
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  }
});
const matrix = await response.json();

// 권한 토글
const toggleResponse = await fetch(`/api/roles/${roleId}/permissions/${permissionId}`, {
  method: 'PUT',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({ assign: newState })
});
```

## 📝 주의사항

1. **권한 확인**: 사용자가 권한 관리 권한을 가지고 있는지 확인
2. **토큰 갱신**: JWT 토큰이 만료되면 새로 발급받아야 함
3. **에러 처리**: 모든 API 호출에 대한 적절한 에러 처리 필요
4. **데이터 동기화**: 권한 변경 후 매트릭스 데이터를 다시 로드
5. **UI 피드백**: 사용자에게 명확한 피드백 제공

## 📁 관련 문서

- [README.md](README.md) - 빠른 시작 가이드
- [사용 예시](api-examples.md) - 다양한 프레임워크 예시
- [사용자 가이드](user-guide.md) - 상세 사용법
- [기술 문서](technical-documentation.md) - API 구현 세부사항

이 API 참조를 통해 롤별 권한 관리 기능을 정확하게 구현할 수 있습니다! 🎉
