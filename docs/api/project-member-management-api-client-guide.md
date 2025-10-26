# 프로젝트 멤버 관리 API 클라이언트 가이드

## 개요

프로젝트 멤버를 추가, 삭제, 확인하는 3개의 API를 제공합니다. 이 API들을 통해 프로젝트 멤버십을 명확하게 관리할 수 있습니다.

## 기본 정보

- **Base URL**: `http://localhost:8080/api`
- **Content-Type**: `application/json`
- **인증**: 현재 미구현 (향후 JWT 토큰 필요)

## API 엔드포인트

### 1. 프로젝트 멤버 추가

프로젝트에 새로운 멤버를 추가합니다.

#### 요청

```http
POST /api/projects/{project_id}/members
Content-Type: application/json

{
  "user_id": 123,
  "role_id": 1632
}
```

#### 파라미터

- **Path Parameters**:
  - `project_id` (integer, required): 프로젝트 ID

- **Request Body**:
  - `user_id` (integer, required): 추가할 사용자 ID
  - `role_id` (integer, optional): 할당할 역할 ID (미제공 시 기본 역할 할당)

#### 응답

**성공 (200 OK)**:
```json
{
  "message": "Member added to project successfully",
  "user_id": 123,
  "project_id": 456,
  "role_id": 1632,
  "role_name": "PROJECT_ADMIN"
}
```

**에러 응답**:
- **400 Bad Request**: 잘못된 요청 데이터
- **404 Not Found**: 프로젝트 또는 사용자를 찾을 수 없음
- **409 Conflict**: 사용자가 이미 프로젝트 멤버임
- **500 Internal Server Error**: 서버 내부 오류

#### 예시

```bash
curl -X POST "http://localhost:8080/api/projects/2/members" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": 6,
    "role_id": 1633
  }'
```

### 2. 프로젝트 멤버 제거

프로젝트에서 멤버를 제거합니다.

#### 요청

```http
DELETE /api/projects/{project_id}/members/{user_id}
```

#### 파라미터

- **Path Parameters**:
  - `project_id` (integer, required): 프로젝트 ID
  - `user_id` (integer, required): 제거할 사용자 ID

#### 응답

**성공 (200 OK)**:
```json
{
  "message": "Member removed from project successfully",
  "user_id": 123,
  "project_id": 456
}
```

**에러 응답**:
- **404 Not Found**: 프로젝트 또는 사용자를 찾을 수 없음
- **500 Internal Server Error**: 서버 내부 오류

#### 예시

```bash
curl -X DELETE "http://localhost:8080/api/projects/2/members/6"
```

### 3. 멤버십 확인

사용자의 프로젝트 멤버십 상태를 확인합니다.

#### 요청

```http
GET /api/projects/{project_id}/members/{user_id}/membership
```

#### 파라미터

- **Path Parameters**:
  - `project_id` (integer, required): 프로젝트 ID
  - `user_id` (integer, required): 확인할 사용자 ID

#### 응답

**성공 (200 OK)**:
```json
{
  "is_member": true,
  "role_id": 1632,
  "role_name": "PROJECT_ADMIN",
  "joined_at": "2025-01-26T04:39:16Z"
}
```

**멤버가 아닌 경우**:
```json
{
  "is_member": false,
  "role_id": null,
  "role_name": null,
  "joined_at": null
}
```

**에러 응답**:
- **404 Not Found**: 프로젝트를 찾을 수 없음
- **500 Internal Server Error**: 서버 내부 오류

#### 예시

```bash
curl -X GET "http://localhost:8080/api/projects/2/members/6/membership"
```

## 역할 정보

### 기본 역할

현재 시스템에서 사용 가능한 역할들:

- **1632**: `ADMIN` - 관리자 역할
- **1633**: `PROJECT_ADMIN` - 프로젝트 관리자 역할
- **1634**: `PROJECT_MEMBER` - 프로젝트 멤버 역할
- **1635**: `VIEWER` - 조회자 역할

### 역할 자동 할당

멤버 추가 시 `role_id`를 제공하지 않으면 기본 역할이 자동으로 할당됩니다. 현재는 기본 역할이 설정되지 않아 명시적으로 `role_id`를 제공해야 합니다.

## 에러 처리

### 공통 에러 응답 형식

```json
{
  "error": "에러 메시지 설명"
}
```

### 주요 에러 케이스

1. **사용자 또는 프로젝트가 존재하지 않는 경우**:
   ```json
   {
     "error": "Failed to add member: Not found: User not found"
   }
   ```

2. **이미 멤버인 경우**:
   ```json
   {
     "error": "Failed to add member: Already exists: User is already a member of this project"
   }
   ```

3. **역할이 존재하지 않는 경우**:
   ```json
   {
     "error": "Failed to add member: Not found: Role not found"
   }
   ```

## 사용 시나리오

### 시나리오 1: 새 멤버 추가

1. 사용자 ID와 프로젝트 ID를 확인
2. 적절한 역할 ID 선택
3. 멤버 추가 API 호출
4. 응답에서 할당된 역할 정보 확인

```javascript
// JavaScript 예시
const addMember = async (projectId, userId, roleId) => {
  const response = await fetch(`/api/projects/${projectId}/members`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      user_id: userId,
      role_id: roleId
    })
  });
  
  if (response.ok) {
    const result = await response.json();
    console.log('멤버 추가 성공:', result);
  } else {
    const error = await response.json();
    console.error('멤버 추가 실패:', error);
  }
};
```

### 시나리오 2: 멤버십 상태 확인

1. 사용자와 프로젝트의 멤버십 상태 확인
2. 멤버인 경우 역할 정보 표시
3. 멤버가 아닌 경우 추가 버튼 표시

```javascript
// JavaScript 예시
const checkMembership = async (projectId, userId) => {
  const response = await fetch(`/api/projects/${projectId}/members/${userId}/membership`);
  
  if (response.ok) {
    const membership = await response.json();
    if (membership.is_member) {
      console.log(`멤버입니다. 역할: ${membership.role_name}`);
    } else {
      console.log('멤버가 아닙니다.');
    }
  }
};
```

### 시나리오 3: 멤버 제거

1. 멤버십 확인
2. 멤버인 경우 제거 확인
3. 멤버 제거 API 호출

```javascript
// JavaScript 예시
const removeMember = async (projectId, userId) => {
  const response = await fetch(`/api/projects/${projectId}/members/${userId}`, {
    method: 'DELETE'
  });
  
  if (response.ok) {
    const result = await response.json();
    console.log('멤버 제거 성공:', result);
  } else {
    const error = await response.json();
    console.error('멤버 제거 실패:', error);
  }
};
```

## 주의사항

1. **중복 멤버십**: 같은 사용자를 같은 프로젝트에 중복으로 추가할 수 없습니다.
2. **존재 여부 확인**: 사용자, 프로젝트, 역할이 모두 존재해야 합니다.
3. **역할 ID**: 현재는 명시적으로 역할 ID를 제공해야 합니다.
4. **트랜잭션**: 멤버 추가/제거는 원자적으로 처리됩니다.

## 향후 개선사항

1. **기본 역할 설정**: 멤버 추가 시 기본 역할 자동 할당
2. **배치 작업**: 여러 사용자 동시 추가/제거 API
3. **권한 체크**: 멤버 관리 권한 검증
4. **감사 로그**: 멤버 추가/제거 이벤트 로깅
5. **이메일 알림**: 멤버 추가/제거 시 알림

## 관련 API

- **프로젝트 멤버 목록**: `GET /api/projects/{project_id}/users`
- **사용자 프로젝트 목록**: `GET /api/users/{user_id}/projects`
- **역할 할당**: `PUT /api/projects/{project_id}/users/{user_id}/role`
- **역할 제거**: `DELETE /api/projects/{project_id}/users/{user_id}/role`

## 지원

API 사용 중 문제가 발생하면 다음을 확인하세요:

1. 서버가 실행 중인지 확인 (`http://localhost:8080/health`)
2. 요청 URL과 파라미터가 올바른지 확인
3. 요청 본문의 JSON 형식이 올바른지 확인
4. 사용자, 프로젝트, 역할 ID가 존재하는지 확인
