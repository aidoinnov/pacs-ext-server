# Viewer Series Meta API 업데이트

## 📋 변경 사항 요약

### 1. 요청 구조 변경

#### 이전 (Series UID만)
```json
{
  "series_uids": [
    "1.2.840.113619.2.55.3.604688433.1234.1",
    "1.2.840.113619.2.55.3.604688433.1234.2"
  ],
  "max_count": 50
}
```

#### 현재 (Study-Series 쌍 + Study Description)
```json
{
  "series_queries": [
    {
      "study_uid": "1.2.840.113619.2.55.3.604688433.1234",
      "series_uid": "1.2.840.113619.2.55.3.604688433.1234.1",
      "study_description": "Chest CT"  // 선택사항: 클라이언트가 이미 알고 있다면 전달
    },
    {
      "study_uid": "1.2.840.113619.2.55.3.604688433.1234",
      "series_uid": "1.2.840.113619.2.55.3.604688433.1234.2",
      "study_description": "Chest CT"  // 선택사항
    }
  ],
  "max_count": 50
}
```

**💡 Study Description 처리 로직:**
1. 클라이언트가 `study_description`을 전달하면 → 그대로 사용 (가장 빠름)
2. 전달하지 않으면 → QIDO Series 응답에서 파싱 시도
3. QIDO 응답에도 없으면 → 별도로 Study 조회하여 가져옴

### 2. 응답에 Study Description 추가

#### 이전
```json
{
  "series": [
    {
      "series_uid": "1.2.840.113619.2.55.3.604688433.1234.1",
      "study_uid": "1.2.840.113619.2.55.3.604688433.1234",
      "series_number": 1,
      "series_description": "Axial",
      "modality": "CT"
    }
  ]
}
```

#### 현재
```json
{
  "series": [
    {
      "series_uid": "1.2.840.113619.2.55.3.604688433.1234.1",
      "study_uid": "1.2.840.113619.2.55.3.604688433.1234",
      "study_description": "Chest CT",  // ✨ 추가됨
      "series_number": 1,
      "series_description": "Axial",
      "modality": "CT"
    }
  ]
}
```

## 🔧 구현 세부사항

### DTO 변경

#### SeriesQuery 추가
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SeriesQuery {
    pub study_uid: String,
    pub series_uid: String,
    pub study_description: Option<String>,  // 선택사항
}
```

**💡 왜 클라이언트가 study_description을 전달하나요?**

QIDO-RS 표준에서는 Series 조회 시 Study 레벨 속성(StudyDescription)이 포함되지 않을 수 있습니다.
- `/series` 엔드포인트: Series 레벨 속성만 반환
- `/studies/{uid}/series` 엔드포인트: Study 속성 포함 여부는 구현에 따라 다름

따라서:
1. **클라이언트가 이미 Study 정보를 알고 있다면** → `study_description` 전달 (추가 조회 불필요)
2. **모르면** → 서버가 별도로 Study 조회 (성능 저하 가능)

#### ViewerSeriesMetaRequest 변경
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ViewerSeriesMetaRequest {
    pub series_queries: Vec<SeriesQuery>,  // 변경됨
    pub max_count: Option<usize>,
}
```

#### ViewerSeriesMeta에 study_description 추가
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ViewerSeriesMeta {
    pub series_uid: String,
    pub study_uid: Option<String>,
    pub study_description: Option<String>,  // ✨ 추가됨
    pub series_number: Option<i32>,
    pub series_description: Option<String>,
    pub modality: Option<String>,
    // ...
}
```

### 컨트롤러 로직 변경

1. **요청 검증**: `series_queries` 배열 검증
2. **중복 제거**: Study UID + Series UID 조합으로 중복 제거
3. **QIDO 조회**: Study UID와 Series UID를 모두 파라미터로 전달
4. **Study Description 파싱**: DICOMweb JSON에서 `00081030` 태그 추출

## 🧪 테스트

### 단위 테스트 업데이트

```rust
#[test]
fn test_viewer_series_meta_from_dicomweb_json() {
    let dicomweb_json = json!({
        "0020000E": {"vr": "UI", "Value": ["1.2.840...1"]},
        "0020000D": {"vr": "UI", "Value": ["1.2.840..."]},
        "00081030": {"vr": "LO", "Value": ["Brain MRI Study"]},  // ✨ 추가
        // ...
    });

    let series_meta = ViewerSeriesMeta::from_dicomweb_json(&dicomweb_json);
    
    assert_eq!(series_meta.study_description, Some("Brain MRI Study".to_string()));
}
```

### 테스트 실행 결과

```
running 4 tests
test test_viewer_series_meta_request_serialization ... ok
test test_viewer_series_meta_from_dicomweb_json ... ok
test test_viewer_study_meta_request_serialization ... ok
test test_viewer_study_meta_from_dicomweb_json ... ok

test result: ok. 4 passed; 0 failed; 0 ignored
```

## 📝 마이그레이션 가이드

### 클라이언트 코드 변경 필요

#### JavaScript/TypeScript 예시

**이전:**
```typescript
const response = await fetch('/api/v1/viewer/series/meta', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    series_uids: ['1.2.840...1', '1.2.840...2'],
    max_count: 50
  })
});
```

**현재 (옵션 1: Study Description 전달 - 추천):**
```typescript
// 클라이언트가 이미 Study 정보를 알고 있는 경우
const studyInfo = {
  study_uid: '1.2.840...',
  study_description: 'Chest CT'
};

const response = await fetch('/api/v1/viewer/series/meta', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    series_queries: [
      {
        study_uid: studyInfo.study_uid,
        series_uid: '1.2.840...1',
        study_description: studyInfo.study_description  // 전달
      },
      {
        study_uid: studyInfo.study_uid,
        series_uid: '1.2.840...2',
        study_description: studyInfo.study_description  // 전달
      }
    ],
    max_count: 50
  })
});

const data = await response.json();
// data.series[0].study_description === 'Chest CT'
```

**현재 (옵션 2: Study Description 미전달):**
```typescript
// 클라이언트가 Study 정보를 모르는 경우
// 서버가 자동으로 Study 조회하여 Description 가져옴 (느림)
const response = await fetch('/api/v1/viewer/series/meta', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    series_queries: [
      { study_uid: '1.2.840...', series_uid: '1.2.840...1' },  // study_description 없음
      { study_uid: '1.2.840...', series_uid: '1.2.840...2' }
    ],
    max_count: 50
  })
});

const data = await response.json();
// data.series[0].study_description 자동으로 채워짐 (서버가 조회)
```

## ✅ 체크리스트

- [x] DTO 구조 변경 (`SeriesQuery` 추가)
- [x] 요청 검증 로직 업데이트
- [x] QIDO 조회 파라미터 변경
- [x] Study Description 파싱 추가
- [x] 단위 테스트 업데이트
- [x] 문서 업데이트
- [ ] 클라이언트 코드 업데이트 (프론트엔드 팀)
- [ ] E2E 테스트 (선택사항)

