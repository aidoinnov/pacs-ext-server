# 문제 검증 가이드

## 현재 상황

1. **데이터 할당**: ✅ 성공 (28개 Series 할당 완료)
2. **API 호출**: ✅ 200 OK 반환
3. **결과**: ❌ 0개 Series 반환

## 검증 방법

### 1. 서버 로그 확인 (가장 중요)

서버를 재시작한 후 다음 명령으로 API를 호출:

```bash
python3 check_with_logging.py
```

서버 로그에서 다음 메시지를 확인:

- `🔍 Gateway /series: Found {} allowed series UIDs for project {}`
  - **의미**: DB에서 조회한 허용된 Series UID 개수
  - **0이면**: `get_allowed_series_uids` 쿼리가 빈 결과를 반환
  
- `🔍 Gateway /series: QIDO returned {} series`
  - **의미**: Dcm4chee QIDO에서 반환된 Series 개수
  - **0이면**: Dcm4chee 연결 실패 또는 실제로 Series가 없음
  
- `🔍 Gateway /series: Filtered {} series from {} QIDO results`
  - **의미**: 필터링 후 남은 Series 개수
  - **0이면**: 필터링 로직 문제 (Series UID 매칭 실패)

### 2. DB 직접 확인

DBeaver나 다른 DB 클라이언트로 연결 후 다음 쿼리 실행:

```sql
-- 1. project_data 기본 확인
SELECT COUNT(*) FROM project_data WHERE project_id = 2;

-- 2. get_allowed_series_uids 쿼리 직접 실행
SELECT DISTINCT pdser.series_uid
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = 2
  AND pdser.series_uid IS NOT NULL;

-- 3. 조인 문제 확인
SELECT 
    pd.id,
    pd.project_id,
    pd.resource_level,
    pd.study_id,
    pd.series_id,
    pds.study_uid,
    pdser.series_uid,
    pdser.study_id as pdser_study_id
FROM project_data pd
LEFT JOIN project_data_study pds ON pd.study_id = pds.id
LEFT JOIN project_data_series pdser ON pd.series_id = pdser.id
WHERE pd.project_id = 2
LIMIT 10;
```

### 3. 문제 원인별 해결 방법

#### 원인 1: `get_allowed_series_uids`가 0개 반환

**확인 사항:**
- `project_data`에 `project_id=2`인 행이 있는지
- `pd.study_id = pds.id` 조인이 성공하는지
- `pds.id = pdser.study_id` 조인이 성공하는지
- `pdser.series_uid`가 NULL이 아닌지

**해결 방법:**
- 조인 실패 시: `project_data_study`와 `project_data_series`에 데이터가 있는지 확인
- `series_uid`가 NULL인 경우: 할당 시 `series_uid`가 제대로 저장되었는지 확인

#### 원인 2: Dcm4chee QIDO가 0개 반환

**확인 사항:**
- Dcm4chee 연결 상태
- 인증 토큰 유효성
- Dcm4chee에 실제로 Series가 있는지

**해결 방법:**
- Dcm4chee 연결 확인
- 인증 설정 확인
- QIDO 엔드포인트 직접 호출 테스트

#### 원인 3: 필터링 실패

**확인 사항:**
- QIDO 응답 형식: `{"0020000E": {"Value": ["series_uid"]}}`
- DB의 `series_uid` 형식과 일치하는지
- `extract_series_uid` 함수가 제대로 작동하는지

**해결 방법:**
- QIDO 응답 형식 확인
- Series UID 형식 일치 확인
- 필터링 로직 디버깅

## 다음 단계

1. 서버 재시작
2. `check_with_logging.py` 실행
3. 서버 로그 확인
4. 문제 원인 파악
5. 해결 방법 적용

