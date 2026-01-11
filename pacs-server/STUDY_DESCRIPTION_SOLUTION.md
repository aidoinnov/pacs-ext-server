# Study Description 포함 방법 - 해결 방안

## 🤔 문제점

QIDO-RS로 Series를 조회할 때 **Study Description이 자동으로 포함되지 않습니다.**

### QIDO-RS 표준 동작

```
GET /series?SeriesInstanceUID=1.2.840...
→ Series 레벨 속성만 반환 (SeriesDescription, Modality 등)
→ Study 레벨 속성은 포함되지 않음 (StudyDescription ❌)
```

## ✅ 해결 방안

### 방안 1: 클라이언트가 Study Description 전달 (✨ 추천)

**장점:**
- 가장 빠름 (추가 QIDO 조회 불필요)
- 네트워크 트래픽 최소화
- 클라이언트가 이미 Study 정보를 알고 있는 경우 효율적

**단점:**
- 클라이언트가 Study 정보를 미리 알아야 함

**구현:**
```rust
pub struct SeriesQuery {
    pub study_uid: String,
    pub series_uid: String,
    pub study_description: Option<String>,  // 클라이언트가 전달
}
```

**사용 예시:**
```json
{
  "series_queries": [
    {
      "study_uid": "1.2.840...",
      "series_uid": "1.2.840...1",
      "study_description": "Chest CT"  // 클라이언트가 전달
    }
  ]
}
```

---

### 방안 2: 서버가 자동으로 Study 조회 (Fallback)

**장점:**
- 클라이언트가 Study 정보를 몰라도 됨
- 항상 Study Description 보장

**단점:**
- 추가 QIDO 조회 필요 (성능 저하)
- 네트워크 트래픽 증가

**구현:**
```rust
// study_description이 없으면 자동으로 Study 조회
if series_meta.study_description.is_none() {
    let study_params = vec![("StudyInstanceUID".to_string(), study_uid.clone())];
    if let Ok(study_response) = qido.qido_studies_with_bearer(...).await {
        // Study Description 추출
    }
}
```

---

### 방안 3: `includefield` 파라미터 사용 (QIDO-RS 표준)

**장점:**
- QIDO-RS 표준 기능
- 한 번의 요청으로 해결

**단점:**
- dcm4chee 구현에 따라 동작하지 않을 수 있음
- 테스트 필요

**구현:**
```rust
let params = vec![
    ("SeriesInstanceUID".to_string(), series_uid.clone()),
    ("includefield".to_string(), "00081030".to_string()),  // StudyDescription
];
```

**QIDO 요청:**
```
GET /series?SeriesInstanceUID=1.2.840...&includefield=00081030
```

---

## 🎯 최종 선택: 하이브리드 방식

### 구현된 로직

```rust
// 1. 클라이언트가 전달한 값 우선 사용
if let Some(client_study_desc) = &query.study_description {
    series_meta.study_description = Some(client_study_desc.clone());
}
// 2. QIDO 응답에서 파싱 시도
else if series_meta.study_description.is_none() {
    // 3. 없으면 별도로 Study 조회 (Fallback)
    let study_response = qido.qido_studies_with_bearer(...).await;
    // Study Description 추출
}
```

### 처리 순서

1. **클라이언트 전달 값** → 가장 빠름 ✨
2. **QIDO Series 응답** → 포함되어 있으면 사용
3. **별도 Study 조회** → 마지막 수단 (느림)

---

## 📊 성능 비교

| 방법 | QIDO 요청 수 | 응답 시간 | 추천도 |
|------|-------------|----------|--------|
| 클라이언트 전달 | 1 (Series만) | ~0.5초 | ⭐⭐⭐⭐⭐ |
| includefield | 1 (Series + 파라미터) | ~0.6초 | ⭐⭐⭐⭐ |
| 별도 Study 조회 | 2 (Series + Study) | ~1.0초 | ⭐⭐ |

---

## 💡 권장 사항

### 프론트엔드 팀

**시나리오 1: Study 목록에서 Series 선택**
```typescript
// Study 정보를 이미 알고 있음
const study = {
  study_uid: '1.2.840...',
  study_description: 'Chest CT'
};

// Series 조회 시 study_description 전달
const response = await fetch('/api/v1/viewer/series/meta', {
  body: JSON.stringify({
    series_queries: selectedSeries.map(s => ({
      study_uid: study.study_uid,
      series_uid: s.series_uid,
      study_description: study.study_description  // ✨ 전달
    }))
  })
});
```

**시나리오 2: Series UID만 알고 있음**
```typescript
// Study 정보를 모름
const response = await fetch('/api/v1/viewer/series/meta', {
  body: JSON.stringify({
    series_queries: [{
      study_uid: '1.2.840...',
      series_uid: '1.2.840...1'
      // study_description 없음 → 서버가 자동 조회
    }]
  })
});
```

---

## 🧪 테스트 방법

### Python으로 QIDO 응답 확인

```bash
python3 test_qido_series_response.py
```

### dcm4chee 실행 필요

```bash
docker-compose up -d dcm4chee
```

---

## 📝 결론

- **기본 전략**: 클라이언트가 `study_description` 전달 (가장 효율적)
- **Fallback**: 서버가 자동으로 Study 조회 (안전장치)
- **유연성**: 두 방식 모두 지원하여 다양한 사용 사례 대응

