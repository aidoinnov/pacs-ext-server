# View Selection API 사용 가이드

## 개요

View Selection API는 DICOM Viewer에서 여러 Study/Series를 선택하여 세션 상태를 저장하고 공유할 수 있는 기능을 제공합니다.

## 주요 기능

- ✅ **멀티 Study/Series 선택**: 여러 Study의 Series를 하나의 Selection으로 관리
- ✅ **Viewport Layout 설정**: 그리드 기반 레이아웃 (rows × cols)
- ✅ **Initial Views 설정**: 각 Viewport에 표시할 초기 이미지 지정
- ✅ **자동 TTL 연장**: Selection 조회 시 자동으로 만료 시간 연장
- ✅ **URL 공유**: Selection ID를 통한 Viewer 상태 공유
- ✅ **Redis/In-memory 지원**: Redis 미연결 시 자동으로 in-memory 사용

## 인증

모든 API 요청은 JWT 토큰이 필요합니다.

```bash
# 로그인
curl -X POST http://localhost:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{
    "username": "your_username",
    "password": "your_password"
  }'

# 응답
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user_id": 123
}
```

이후 모든 요청에 `Authorization: Bearer <token>` 헤더를 포함해야 합니다.

## API 엔드포인트

### 1. Selection 생성 (POST)

**Endpoint**: `POST /api/v1/view-selections`

#### 기본 사용 (Series만 지정)

```bash
curl -X POST http://localhost:8080/api/v1/view-selections \
  -H 'Authorization: Bearer <token>' \
  -H 'Content-Type: application/json' \
  -d '{
    "series": [
      {
        "study_uid": "1.2.840.113619.2.55.3.604641477.123",
        "series_uid": "1.2.840.113619.2.55.3.604641477.124"
      },
      {
        "study_uid": "1.2.840.113619.2.55.3.604641477.123",
        "series_uid": "1.2.840.113619.2.55.3.604641477.125"
      }
    ]
  }'
```

**응답**:
```json
{
  "selection_id": "sel_a1b2c3"
}
```

#### Layout + Initial Views 사용

```bash
curl -X POST http://localhost:8080/api/v1/view-selections \
  -H 'Authorization: Bearer <token>' \
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
      },
      {
        "row": 0,
        "col": 1,
        "study_uid": "1.2.840.113619.2.55.3.604641477.123",
        "series_uid": "1.2.840.113619.2.55.3.604641477.124",
        "frame_index": 5
      }
    ]
  }'
```

**필드 설명**:
- `series` (필수): 선택된 Series 목록
  - `study_uid`: Study Instance UID
  - `series_uid`: Series Instance UID
- `layout` (선택): Viewport 레이아웃
  - `rows`: 행 개수
  - `cols`: 열 개수
- `initial_views` (선택): 각 Viewport의 초기 이미지
  - `row`, `col`: Viewport 위치 (0-based)
  - `study_uid`, `series_uid`: 표시할 이미지의 Study/Series
  - `sop_uid` (선택): 특정 SOP Instance 지정
  - `frame_index` (선택): Multi-frame 이미지의 프레임 번호

**유효성 검증**:
- ❌ `initial_views`가 있으면 `layout`도 필수
- ❌ Viewport 위치(`row`, `col`)는 layout 범위 내여야 함
- ❌ `series` 리스트는 비어있을 수 없음

**응답 코드**:
- `201`: 생성 성공
- `400`: 유효성 검증 실패
- `401`: 인증 실패
- `500`: 서버 오류

### 2. Selection 조회 (GET)

**Endpoint**: `GET /api/v1/view-selections/{selection_id}`

```bash
curl -X GET http://localhost:8080/api/v1/view-selections/sel_a1b2c3 \
  -H 'Authorization: Bearer <token>'
```

**응답**:
```json
{
  "selection_id": "sel_a1b2c3",
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
  ],
  "created_at": "2026-01-20T12:00:00Z",
  "expires_at": "2026-01-20T12:30:00Z",
  "user_id": 123
}
```

**특징**:
- ✅ 조회 시 자동으로 TTL 연장 (기본 30분)
- ✅ `expires_at`이 현재 시간 + TTL로 업데이트됨

**응답 코드**:
- `200`: 조회 성공
- `404`: Selection을 찾을 수 없음
- `401`: 인증 실패

### 3. Selection 삭제 (DELETE)

**Endpoint**: `DELETE /api/v1/view-selections/{selection_id}`

```bash
curl -X DELETE http://localhost:8080/api/v1/view-selections/sel_a1b2c3 \
  -H 'Authorization: Bearer <token>'
```

**응답**:
```json
{
  "message": "Selection deleted successfully"
}
```

**응답 코드**:
- `200`: 삭제 성공
- `404`: Selection을 찾을 수 없음
- `401`: 인증 실패

## 실제 사용 시나리오

### 시나리오 1: 기본 Viewer 세션

```javascript
// 1. PACS UI에서 사용자가 Series 선택
const selectedSeries = [
  { study_uid: "1.2.3.4", series_uid: "1.2.3.4.5" },
  { study_uid: "1.2.3.4", series_uid: "1.2.3.4.6" }
];

// 2. Selection 생성
const response = await fetch('/api/v1/view-selections', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({ series: selectedSeries })
});

const { selection_id } = await response.json();

// 3. Viewer 오픈 (URL에 Selection ID 포함)
window.open(`/viewer/selections/${selection_id}`, '_blank');
```

### 시나리오 2: 그리드 레이아웃 Viewer

```javascript
// 2x2 그리드 레이아웃으로 Viewer 오픈
const response = await fetch('/api/v1/view-selections', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    series: [
      { study_uid: "1.2.3.4", series_uid: "1.2.3.4.5" },
      { study_uid: "1.2.3.4", series_uid: "1.2.3.4.6" }
    ],
    layout: { rows: 2, cols: 2 },
    initial_views: [
      {
        row: 0, col: 0,
        study_uid: "1.2.3.4",
        series_uid: "1.2.3.4.5",
        sop_uid: "1.2.3.4.5.1"  // 첫 번째 이미지
      },
      {
        row: 0, col: 1,
        study_uid: "1.2.3.4",
        series_uid: "1.2.3.4.6",
        frame_index: 10  // 11번째 프레임
      }
    ]
  })
});
```

### 시나리오 3: Viewer에서 Selection 로드

```javascript
// Viewer 컴포넌트 초기화
async function initViewer(selectionId) {
  // Selection 조회
  const response = await fetch(`/api/v1/view-selections/${selectionId}`, {
    headers: { 'Authorization': `Bearer ${token}` }
  });

  const selection = await response.json();

  // Layout 설정
  if (selection.layout) {
    setupViewportGrid(selection.layout.rows, selection.layout.cols);
  }

  // Initial Views 로드
  if (selection.initial_views) {
    for (const view of selection.initial_views) {
      await loadImageToViewport(
        view.row, view.col,
        view.study_uid, view.series_uid,
        view.sop_uid, view.frame_index
      );
    }
  }

  // 나머지 Series는 Progressive Loading
  for (const series of selection.series) {
    await loadSeriesMetadata(series.study_uid, series.series_uid);
  }
}
```

### 시나리오 4: URL 공유

```javascript
// 사용자 A: Selection 생성 후 URL 공유
const { selection_id } = await createSelection(selectedSeries);
const shareableUrl = `https://pacs.example.com/viewer/selections/${selection_id}`;

// 이메일/메신저로 URL 공유
sendMessage(shareableUrl);

// 사용자 B: URL 접속 시 동일한 Viewer 상태 재현
// → Selection이 만료되지 않았다면 동일한 Series 목록과 Layout 표시
```

### 시나리오 5: Progressive Loading with TTL 연장

```javascript
async function progressiveLoad(selectionId) {
  // 1. 초기 로드
  let selection = await getSelection(selectionId);
  console.log('Expires at:', selection.expires_at);

  // 2. 메타데이터 로드 (5초 소요)
  await loadMetadata(selection.series);

  // 3. 다시 조회 → TTL 자동 연장
  selection = await getSelection(selectionId);
  console.log('Extended expires_at:', selection.expires_at);

  // 4. 이미지 로드 (10초 소요)
  await loadImages(selection.series);

  // 5. 다시 조회 → TTL 자동 연장
  selection = await getSelection(selectionId);
  console.log('Extended again:', selection.expires_at);

  // → 로딩 중에도 Selection이 만료되지 않음
}
```

## 에러 처리

### 유효성 검증 에러 (400)

```json
{
  "error": "Validation Error",
  "message": "initial_views requires layout to be specified"
}
```

```json
{
  "error": "Validation Error",
  "message": "Viewport position (2, 3) is out of layout bounds (2, 2)"
}
```

```json
{
  "error": "Validation Error",
  "message": "Series list cannot be empty"
}
```

### Not Found 에러 (404)

```json
{
  "error": "Not Found",
  "message": "Selection not found"
}
```

### 인증 에러 (401)

```json
{
  "error": "Unauthorized",
  "message": "Invalid or missing authorization token"
}
```

## 설정

### Redis 설정 (권장)

```toml
# config/default.toml
[redis]
url = "redis://localhost:6379"
view_selection_ttl_sec = 1800  # 30분
```

### In-memory Fallback

Redis가 연결되지 않으면 자동으로 in-memory 저장소를 사용합니다.

**경고 메시지**:
```
⚠️  ========================================
⚠️  WARNING: Using in-memory ViewSelection storage
⚠️  - Data will be lost on server restart
⚠️  - NOT suitable for multi-server deployments
⚠️  - For production, configure Redis connection
⚠️  ========================================
```

**제한사항**:
- ❌ 서버 재시작 시 모든 Selection 삭제
- ❌ 여러 서버 인스턴스 간 공유 불가
- ✅ 단일 서버 개발 환경에서는 사용 가능

## 모범 사례

### 1. TTL 관리

```javascript
// ✅ Good: 조회를 통한 자동 TTL 연장
async function keepSelectionAlive(selectionId) {
  // 주기적으로 조회하여 TTL 연장
  setInterval(async () => {
    await getSelection(selectionId);
  }, 10 * 60 * 1000); // 10분마다
}

// ❌ Bad: TTL 만료 후 재생성
// → Selection ID가 변경되어 URL 공유가 깨짐
```

### 2. Layout 설정

```javascript
// ✅ Good: Layout과 Initial Views를 함께 지정
{
  layout: { rows: 2, cols: 2 },
  initial_views: [
    { row: 0, col: 0, ... },
    { row: 0, col: 1, ... }
  ]
}

// ❌ Bad: Initial Views만 지정
{
  initial_views: [...]  // → 400 에러
}
```

### 3. 에러 처리

```javascript
// ✅ Good: 적절한 에러 처리
try {
  const selection = await getSelection(selectionId);
  return selection;
} catch (error) {
  if (error.status === 404) {
    // Selection 만료 → 새로 생성
    return await createNewSelection();
  } else if (error.status === 401) {
    // 인증 만료 → 재로그인
    await relogin();
  }
  throw error;
}
```

## 제한사항

- **Selection ID 형식**: `sel_` + 6자리 랜덤 문자열 (예: `sel_a1b2c3`)
- **기본 TTL**: 30분 (조회 시마다 연장)
- **최대 Series 개수**: 제한 없음 (테스트: 10개 이상 정상 동작)
- **Layout 크기**: 제한 없음 (권장: 4x4 이하)

## 참고

- **Swagger UI**: `http://localhost:8080/swagger-ui/`
- **Health Check**: `http://localhost:8080/health`
- **E2E 테스트**: `pacs-server/e2e/test_view_selection_e2e.py`


