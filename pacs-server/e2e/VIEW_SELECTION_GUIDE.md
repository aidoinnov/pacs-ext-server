# View Selection 스크립트 가이드

## 📋 생성된 스크립트

### 1. `create_view_selections.py` - 간단한 생성 스크립트
**용도**: 지정된 Study/Series UID로 View Selection을 빠르게 생성

**실행 방법**:
```bash
cd pacs-server/e2e
python create_view_selections.py
```

**생성되는 Selection**:
1. **첫 번째 데이터** (1개 Series)
   - Study UID: `1.3.6.1.4.1.14519.5.2.1.6655.2359.307959856517080892181338382781`
   - Series UID: `1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345`

2. **두 번째 데이터** (1개 Series)
   - Study UID: `1.3.6.1.4.1.14519.5.2.1.6655.2359.321111757620390201880556376661`
   - Series UID: `1.3.6.1.4.1.14519.5.2.1.6655.2359.260616660471925521837323152953`

3. **두 개 모두 포함** (2개 Series)
   - 위의 두 데이터를 모두 포함

**출력 예시**:
```
🚀 View Selection 생성 시작...

🔐 로그인 중: iaid-pacs-admin
✅ 로그인 성공

======================================================================
View Selection 생성
======================================================================

1. 첫 번째 데이터 생성 중...
   Status: 201
   ✅ 생성 성공! ID: sel_69a9df
   Series 수: 1

2. 두 번째 데이터 생성 중...
   Status: 201
   ✅ 생성 성공! ID: sel_be1325
   Series 수: 1

3. 두 개 모두 포함 생성 중...
   Status: 201
   ✅ 생성 성공! ID: sel_a793ed
   Series 수: 2

======================================================================
생성 요약
======================================================================
✅ 총 3개 View Selection 생성 완료!

생성된 Selection ID 목록:
   1. sel_69a9df
   2. sel_be1325
   3. sel_a793ed
```

---

### 2. `test_view_selection_refactored.py` - 전체 E2E 테스트
**용도**: View Selection의 생성, 조회, 삭제 전체 워크플로우 테스트

**실행 방법**:
```bash
cd pacs-server/e2e
python test_view_selection_refactored.py
```

**테스트 항목**:
1. **테스트 1**: View Selection 생성 (3개)
2. **테스트 2**: View Selection 조회 (각 Selection의 상세 정보 확인)
3. **테스트 3**: View Selection 삭제 (자동 cleanup)

**특징**:
- ✅ 자동 로그인
- ✅ 자동 cleanup (삭제)
- ✅ 상세한 출력
- ✅ 에러 처리

---

## 🔧 커스터마이징

### 다른 Study/Series UID 사용하기

`create_view_selections.py` 파일을 수정:

```python
selections = [
    {
        'name': '내 데이터',
        'series': [
            {
                'study_uid': '여기에_Study_UID',
                'series_uid': '여기에_Series_UID'
            }
        ]
    }
]
```

### 여러 Series 포함하기

```python
selections = [
    {
        'name': '여러 Series',
        'series': [
            {
                'study_uid': 'Study_UID_1',
                'series_uid': 'Series_UID_1'
            },
            {
                'study_uid': 'Study_UID_2',
                'series_uid': 'Series_UID_2'
            },
            {
                'study_uid': 'Study_UID_3',
                'series_uid': 'Series_UID_3'
            }
        ]
    }
]
```

---

## 📚 View Selection API

### 엔드포인트

#### 1. 생성
```http
POST /api/v1/view-selections
Content-Type: application/json
Authorization: Bearer {token}

{
  "series": [
    {
      "study_uid": "1.2.3.4.5...",
      "series_uid": "1.2.3.4.6..."
    }
  ]
}
```

**응답**:
```json
{
  "selection_id": "sel_abc123",
  "user_id": 1,
  "series": [...],
  "created_at": "2024-01-14T12:00:00Z"
}
```

#### 2. 조회
```http
GET /api/v1/view-selections/{selection_id}
Authorization: Bearer {token}
```

**응답**:
```json
{
  "selection_id": "sel_abc123",
  "user_id": 1,
  "series": [
    {
      "study_uid": "1.2.3.4.5...",
      "series_uid": "1.2.3.4.6..."
    }
  ],
  "created_at": "2024-01-14T12:00:00Z"
}
```

#### 3. 삭제
```http
DELETE /api/v1/view-selections/{selection_id}
Authorization: Bearer {token}
```

**응답**: `204 No Content`

---

## 🎯 사용 시나리오

### 시나리오 1: 빠르게 테스트 데이터 생성
```bash
# 3개의 View Selection 생성
python create_view_selections.py

# 생성된 ID 확인
# 출력에서 Selection ID 복사
```

### 시나리오 2: 전체 워크플로우 테스트
```bash
# 생성 → 조회 → 삭제 전체 테스트
python test_view_selection_refactored.py
```

### 시나리오 3: 커스텀 데이터로 생성
```bash
# 1. create_view_selections.py 수정
# 2. selections 배열에 원하는 데이터 추가
# 3. 실행
python create_view_selections.py
```

---

## ✅ 체크리스트

스크립트 실행 전:

- [ ] 서버 실행 중 (`http://localhost:8080`)
- [ ] 데이터베이스 연결 가능
- [ ] Keycloak 연결 가능
- [ ] 테스트 사용자 존재 (`iaid-pacs-admin`)

---

## 🔍 문제 해결

### 로그인 실패 (401)
```
원인: Keycloak 연결 실패
해결: CONNECTION_INFO.md 참고하여 Keycloak 설정 확인
```

### Selection 생성 실패 (400)
```
원인: 잘못된 Study/Series UID
해결: UID 형식 확인 (DICOM UID 형식이어야 함)
```

### Selection 조회 실패 (404)
```
원인: Selection이 존재하지 않음
해결: Selection ID 확인 또는 다시 생성
```

---

## 📖 관련 문서

- [E2E_TEST_RULES.md](./E2E_TEST_RULES.md) - E2E 테스트 작성 규칙
- [CONNECTION_INFO.md](./CONNECTION_INFO.md) - 연결 정보
- [QUICK_START.md](./QUICK_START.md) - 빠른 시작 가이드

