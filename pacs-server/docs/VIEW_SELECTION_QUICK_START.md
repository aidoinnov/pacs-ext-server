# View Selection API - Quick Start

## 5분 안에 시작하기

### 1. 로그인

```bash
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username": "your_username", "password": "your_password"}' \
  | jq -r '.token')
```

### 2. Selection 생성

```bash
SELECTION_ID=$(curl -s -X POST http://localhost:8080/api/v1/view-selections \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{
    "series": [
      {
        "study_uid": "1.2.840.113619.2.55.3.604641477.123",
        "series_uid": "1.2.840.113619.2.55.3.604641477.124"
      }
    ]
  }' | jq -r '.selection_id')

echo "Selection ID: $SELECTION_ID"
```

### 3. Selection 조회

```bash
curl -s -X GET "http://localhost:8080/api/v1/view-selections/$SELECTION_ID" \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### 4. Selection 삭제

```bash
curl -s -X DELETE "http://localhost:8080/api/v1/view-selections/$SELECTION_ID" \
  -H "Authorization: Bearer $TOKEN" | jq .
```

## Layout + Initial Views 예제

```bash
curl -X POST http://localhost:8080/api/v1/view-selections \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{
    "series": [
      {
        "study_uid": "1.2.840.113619.2.55.3.604641477.123",
        "series_uid": "1.2.840.113619.2.55.3.604641477.124"
      }
    ],
    "layout": {
      "rows": 2,
      "cols": 2
    },
    "initial_views": [
      {
        "row": 0,
        "col": 0,
        "study_uid": "1.2.840.113619.2.55.3.604641477.123",
        "series_uid": "1.2.840.113619.2.55.3.604641477.124",
        "sop_uid": "1.2.840.113619.2.55.3.604641477.126"
      }
    ]
  }' | jq .
```

## JavaScript 예제

```javascript
// 1. 로그인
const loginResponse = await fetch('/api/auth/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    username: 'your_username',
    password: 'your_password'
  })
});
const { token } = await loginResponse.json();

// 2. Selection 생성
const createResponse = await fetch('/api/v1/view-selections', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    series: [
      {
        study_uid: "1.2.840.113619.2.55.3.604641477.123",
        series_uid: "1.2.840.113619.2.55.3.604641477.124"
      }
    ]
  })
});
const { selection_id } = await createResponse.json();

// 3. Selection 조회
const getResponse = await fetch(`/api/v1/view-selections/${selection_id}`, {
  headers: { 'Authorization': `Bearer ${token}` }
});
const selection = await getResponse.json();
console.log(selection);

// 4. Selection 삭제
await fetch(`/api/v1/view-selections/${selection_id}`, {
  method: 'DELETE',
  headers: { 'Authorization': `Bearer ${token}` }
});
```

## Python 예제

```python
import requests

# 1. 로그인
login_response = requests.post(
    'http://localhost:8080/api/auth/login',
    json={'username': 'your_username', 'password': 'your_password'}
)
token = login_response.json()['token']

headers = {'Authorization': f'Bearer {token}'}

# 2. Selection 생성
create_response = requests.post(
    'http://localhost:8080/api/v1/view-selections',
    headers=headers,
    json={
        'series': [
            {
                'study_uid': '1.2.840.113619.2.55.3.604641477.123',
                'series_uid': '1.2.840.113619.2.55.3.604641477.124'
            }
        ]
    }
)
selection_id = create_response.json()['selection_id']

# 3. Selection 조회
get_response = requests.get(
    f'http://localhost:8080/api/v1/view-selections/{selection_id}',
    headers=headers
)
selection = get_response.json()
print(selection)

# 4. Selection 삭제
requests.delete(
    f'http://localhost:8080/api/v1/view-selections/{selection_id}',
    headers=headers
)
```

## 다음 단계

- 📖 [전체 API 가이드](./VIEW_SELECTION_API_GUIDE.md)
- 📖 [한국어 가이드](./VIEW_SELECTION_API_GUIDE_KR.md)
- 🧪 [E2E 테스트 예제](../e2e/test_view_selection_e2e.py)
- 📚 [Swagger UI](http://localhost:8080/swagger-ui/)

