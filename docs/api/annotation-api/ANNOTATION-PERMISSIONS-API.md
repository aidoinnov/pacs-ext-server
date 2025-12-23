# 📋 Annotation 권한 조회 API

## 🎯 개요

사용자의 어노테이션 권한을 조회하는 API입니다. 특정 프로젝트에서 사용자가 가진 어노테이션 관련 권한(읽기, 쓰기, 삭제, 공유 등)을 확인할 수 있습니다.

---

## 📡 API 엔드포인트

### 권한 조회

```http
GET /api/annotations/permissions
```

---

## 📥 요청

### 쿼리 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | number | ✅ | 프로젝트 ID (필수) |
| `user_id` | number | ❌ | 조회할 사용자 ID (선택사항, 기본값: 요청한 사용자) |

### 요청 예제

#### 본인 권한 조회

```http
GET /api/annotations/permissions?project_id=299
Authorization: Bearer {token}
```

#### 다른 사용자 권한 조회 (프로젝트 멤버만 가능)

```http
GET /api/annotations/permissions?project_id=299&user_id=336
Authorization: Bearer {token}
```

---

## 📤 응답

### 성공 응답 (200 OK)

```json
{
  "can_read_own": true,
  "can_read_all": false,
  "can_write": true,
  "can_delete": true,
  "can_share": false
}
```

### 응답 필드 설명

| 필드 | 타입 | 설명 |
|------|------|------|
| `can_read_own` | boolean | 본인이 생성한 어노테이션 읽기 권한 |
| `can_read_all` | boolean | 프로젝트 내 모든 어노테이션 읽기 권한 |
| `can_write` | boolean | 어노테이션 생성/수정 권한 |
| `can_delete` | boolean | 어노테이션 삭제 권한 |
| `can_share` | boolean | 어노테이션 공유 권한 (선택적 권한) |

---

## ❌ 에러 응답

### 400 Bad Request

**project_id 누락 또는 잘못된 형식**

```json
{
  "error": "Bad Request",
  "message": "project_id is required and must be greater than 0"
}
```

### 401 Unauthorized

**인증 토큰 누락 또는 유효하지 않음**

```json
{
  "error": "Unauthorized",
  "message": "User ID is required"
}
```

### 403 Forbidden

**다른 사용자의 권한을 조회하려고 하지만 프로젝트 멤버가 아님**

```json
{
  "error": "Forbidden",
  "message": "You must be a member of this project to view other user's permissions"
}
```

### 404 Not Found

**사용자 또는 프로젝트를 찾을 수 없음**

```json
{
  "error": "Not Found",
  "message": "User or Project not found"
}
```

### 500 Internal Server Error

**서버 내부 오류**

```json
{
  "error": "Internal Server Error",
  "message": "An unexpected error occurred"
}
```

---

## 🔐 권한 체크 로직

### 권한 종류

1. **READ_OWN**: 본인이 생성한 어노테이션만 읽을 수 있는 권한
2. **READ_ALL**: 프로젝트 내 모든 사용자의 어노테이션을 읽을 수 있는 권한
3. **WRITE**: 어노테이션을 생성하고 수정할 수 있는 권한
4. **DELETE**: 어노테이션을 삭제할 수 있는 권한
5. **SHARE**: 어노테이션을 공유할 수 있는 권한 (선택적 권한)

### 권한 조회 규칙

1. **본인 권한 조회**: `user_id` 파라미터 없이 요청하면 자동으로 요청한 사용자의 권한을 조회합니다.
2. **다른 사용자 권한 조회**: `user_id` 파라미터를 제공하면 해당 사용자의 권한을 조회할 수 있습니다.
   - 단, 요청한 사용자가 해당 프로젝트의 멤버여야 합니다.
   - 프로젝트 멤버가 아닌 경우 403 Forbidden 응답을 받습니다.

---

## 💡 사용 예제

### 예제 1: 본인 권한 확인

프로젝트에서 본인이 가진 어노테이션 권한을 확인합니다.

```javascript
// JavaScript 예제
async function getMyPermissions(projectId) {
  const response = await fetch(
    `/api/annotations/permissions?project_id=${projectId}`,
    {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      }
    }
  );

  if (response.ok) {
    const permissions = await response.json();
    console.log('읽기 권한:', permissions.can_read_own);
    console.log('쓰기 권한:', permissions.can_write);
    console.log('삭제 권한:', permissions.can_delete);
    
    // UI 업데이트
    updateUI(permissions);
  } else {
    const error = await response.json();
    console.error('권한 조회 실패:', error.message);
  }
}
```

### 예제 2: 다른 사용자 권한 확인 (프로젝트 관리자)

프로젝트 관리자가 팀원의 권한을 확인합니다.

```javascript
// JavaScript 예제
async function getUserPermissions(projectId, userId) {
  const response = await fetch(
    `/api/annotations/permissions?project_id=${projectId}&user_id=${userId}`,
    {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      }
    }
  );

  if (response.ok) {
    const permissions = await response.json();
    return permissions;
  } else if (response.status === 403) {
    console.error('프로젝트 멤버가 아니어서 권한 조회 불가');
  } else {
    const error = await response.json();
    console.error('권한 조회 실패:', error.message);
  }
}
```

### 예제 3: 권한 기반 UI 제어

권한에 따라 UI 요소를 표시/숨김 처리합니다.

```javascript
// JavaScript 예제
async function loadAnnotationUI(projectId) {
  const permissions = await getMyPermissions(projectId);
  
  // 권한에 따라 UI 업데이트
  if (permissions.can_write) {
    document.getElementById('create-annotation-btn').style.display = 'block';
  } else {
    document.getElementById('create-annotation-btn').style.display = 'none';
  }
  
  if (permissions.can_delete) {
    document.getElementById('delete-annotation-btn').style.display = 'block';
  } else {
    document.getElementById('delete-annotation-btn').style.display = 'none';
  }
  
  if (permissions.can_read_all) {
    // 모든 사용자의 어노테이션 표시
    loadAllAnnotations(projectId);
  } else {
    // 본인 어노테이션만 표시
    loadMyAnnotations(projectId);
  }
}
```

### 예제 4: TypeScript 타입 정의

```typescript
// TypeScript 예제
interface AnnotationPermissions {
  can_read_own: boolean;
  can_read_all: boolean;
  can_write: boolean;
  can_delete: boolean;
  can_share: boolean;
}

interface ErrorResponse {
  error: string;
  message: string;
}

async function getAnnotationPermissions(
  projectId: number,
  userId?: number
): Promise<AnnotationPermissions> {
  const params = new URLSearchParams({
    project_id: projectId.toString(),
  });
  
  if (userId) {
    params.append('user_id', userId.toString());
  }
  
  const response = await fetch(
    `/api/annotations/permissions?${params.toString()}`,
    {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      }
    }
  );
  
  if (!response.ok) {
    const error: ErrorResponse = await response.json();
    throw new Error(error.message);
  }
  
  return await response.json();
}
```

---

## 🔄 권한 조합 예시

### 일반 사용자

```json
{
  "can_read_own": true,
  "can_read_all": false,
  "can_write": true,
  "can_delete": true,
  "can_share": false
}
```

**의미**: 본인 어노테이션만 읽고, 생성/수정/삭제 가능

### 리뷰어 (Reviewer)

```json
{
  "can_read_own": true,
  "can_read_all": true,
  "can_write": false,
  "can_delete": false,
  "can_share": false
}
```

**의미**: 모든 어노테이션 읽기 가능, 생성/수정/삭제 불가

### 프로젝트 관리자

```json
{
  "can_read_own": true,
  "can_read_all": true,
  "can_write": true,
  "can_delete": true,
  "can_share": true
}
```

**의미**: 모든 권한 보유

---

## 📝 주의사항

1. **인증 필수**: 모든 요청에 유효한 인증 토큰이 필요합니다.
2. **프로젝트 멤버십**: 다른 사용자의 권한을 조회하려면 요청한 사용자가 해당 프로젝트의 멤버여야 합니다.
3. **권한 캐싱**: 권한은 자주 변경되지 않으므로 클라이언트에서 적절히 캐싱하는 것을 권장합니다.
4. **에러 처리**: 403 Forbidden 응답은 프로젝트 멤버가 아니라는 의미이므로, 사용자에게 적절한 메시지를 표시해야 합니다.

---

## 🔗 관련 API

- [Annotation 목록 조회](./FRONTEND-API-SPEC.md#1%EF%B8%8F⃣-study-series-레벨-annotation-조회)
- [Annotation 생성](./FRONTEND-API-SPEC.md#5%EF%B8%8F⃣-annotation-생성)
- [Annotation 수정](./FRONTEND-API-SPEC.md#4%EF%B8%8F⃣-annotation-수정)
- [Annotation 삭제](./FRONTEND-API-SPEC.md#6%EF%B8%8F⃣-annotation-삭제)

---

## 📅 변경 이력

- **2024-01-XX**: 초기 문서 작성


