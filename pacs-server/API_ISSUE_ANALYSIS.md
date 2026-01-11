# API 응답 문제 분석

## 현재 상황
- **API**: `GET /api/me/dicom/series?project_id=2&page=1&page_size=200`
- **Status**: 200 OK
- **응답**: 빈 배열 `[]` (content-length: 2)

## 문제 원인 분석

### 가능한 원인 1: DB에서 허용된 Series UID가 0개
**확인 방법:**
```sql
SELECT DISTINCT pdser.series_uid
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = 2
  AND pdser.series_uid IS NOT NULL;
```

**원인:**
- `project_data`에 `project_id=2`인 데이터가 없음
- 조인 실패 (`pd.study_id = pds.id` 또는 `pds.id = pdser.study_id`)
- `series_uid`가 NULL

### 가능한 원인 2: Dcm4chee QIDO가 빈 결과 반환
**확인 방법:**
- 서버 로그에서 `🔍 Gateway /series: QIDO returned {} series` 확인
- Dcm4chee QIDO 엔드포인트 직접 호출

**원인:**
- Dcm4chee 연결 실패 (502 에러)
- 실제로 Series가 없음
- 인증 실패

### 가능한 원인 3: 필터링 후 결과가 0개
**확인 방법:**
- 서버 로그에서 `🔍 Gateway /series: Filtered {} series from {} QIDO results` 확인
- Series UID 형식 비교

**원인:**
- QIDO 응답의 Series UID 형식과 DB의 `series_uid` 형식이 다름
- `extract_series_uid` 함수가 제대로 작동하지 않음
- 필터링 로직 문제

## 로직 흐름

```
1. Dcm4chee QIDO 호출
   ↓
2. get_allowed_series_uids(project_id) 호출
   ↓
3. QIDO 결과를 허용 목록으로 필터링
   ↓
4. 빈 배열 반환
```

## 확인 필요 사항

### 1. 서버 로그 확인 (가장 중요)
서버 로그에서 다음 메시지를 확인하세요:

```
🔍 Gateway /series: Found {} allowed series UIDs for project {}
🔍 Gateway /series: QIDO returned {} series
🔍 Gateway /series: Filtered {} series from {} QIDO results
```

**해석:**
- `Found 0 allowed series UIDs` → DB 문제
- `QIDO returned 0 series` → Dcm4chee 문제
- `Filtered 0 series from X QIDO results` → 필터링 문제

### 2. DB 직접 확인
DBeaver나 다른 DB 클라이언트로:
1. `test_get_allowed_series_uids.sql` 실행
2. `get_allowed_series_uids` 쿼리 결과 확인
3. 조인 문제 확인

### 3. Dcm4chee 직접 확인
```bash
# Dcm4chee QIDO 엔드포인트 직접 호출
curl -X GET "https://archive.pacs.ai-do.co.kr/iaid-pacs/aets/iAID_PACS/rs/series?limit=10" \
  -H "Accept: application/json" \
  -H "Authorization: Bearer {토큰}"
```

## 해결 방법

### DB 문제인 경우
1. 데이터 할당 재실행
2. 조인 문제 해결
3. 데이터 무결성 확인

### Dcm4chee 문제인 경우
1. Dcm4chee 서버 상태 확인
2. 네트워크 연결 확인
3. 인증 설정 확인

### 필터링 문제인 경우
1. Series UID 형식 확인
2. `extract_series_uid` 함수 수정
3. 필터링 로직 디버깅

## 다음 단계

1. **서버 로그 확인** (우선순위 1)
   - 로그에서 위의 메시지 확인
   - 각 단계별 개수 확인

2. **DB 직접 확인** (우선순위 2)
   - DBeaver로 연결
   - `test_get_allowed_series_uids.sql` 실행

3. **Dcm4chee 직접 확인** (우선순위 3)
   - QIDO 엔드포인트 직접 호출
   - 응답 확인

