# 다음 단계 가이드

## 현재 상황
- DB 데이터 확인: ❌ 연결 실패 (인증 문제)
- Dcm4chee 통신: ❌ 502 에러
- API 응답: 0개 Series

## 즉시 확인할 사항

### 1. DB 데이터 확인 (DBeaver 사용)

#### 연결 정보
- **호스트**: localhost (또는 127.0.0.1)
- **포트**: 5456
- **데이터베이스**: pacs_db
- **사용자**: 확인 필요 (admin 또는 다른 사용자)
- **비밀번호**: 확인 필요

#### 실행할 쿼리
`test_get_allowed_series_uids.sql` 파일을 열어서 실행:

```sql
-- 1. 기본 확인
SELECT COUNT(*) FROM project_data WHERE project_id = 2;

-- 2. 허용된 Series UID 조회
SELECT DISTINCT pdser.series_uid
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = 2
  AND pdser.series_uid IS NOT NULL;

-- 3. 조인 문제 확인
SELECT 
    pd.id,
    pd.study_id,
    pds.id as pds_id,
    pdser.id as pdser_id,
    pdser.study_id as pdser_study_id,
    pdser.series_uid
FROM project_data pd
LEFT JOIN project_data_study pds ON pd.study_id = pds.id
LEFT JOIN project_data_series pdser ON pd.series_id = pdser.id
WHERE pd.project_id = 2
LIMIT 10;
```

#### 확인 사항
- `project_data`에 `project_id=2`인 행이 있는지
- `get_allowed_series_uids` 쿼리가 결과를 반환하는지
- 조인이 제대로 되는지

---

### 2. 서버 로그 확인

#### 서버 재시작
로깅이 추가된 코드로 서버를 재시작하세요.

#### API 호출
```bash
python3 check_with_logging.py
```

#### 확인할 로그 메시지
1. `🔍 Gateway /series: Found {} allowed series UIDs for project {}`
   - **의미**: DB에서 조회한 허용된 Series UID 개수
   - **0이면**: DB 쿼리 문제

2. `🔍 Gateway /series: QIDO returned {} series`
   - **의미**: Dcm4chee QIDO에서 반환된 Series 개수
   - **0이면**: Dcm4chee 연결 문제

3. `🔍 Gateway /series: Filtered {} series from {} QIDO results`
   - **의미**: 필터링 후 남은 Series 개수
   - **0이면**: 필터링 로직 문제

---

### 3. Dcm4chee 통신 확인

#### 확인 사항
1. Dcm4chee 서버가 실행 중인지
2. 네트워크 연결이 정상인지
3. 인증 설정이 올바른지

#### 테스트 방법
```bash
# Dcm4chee QIDO 엔드포인트 직접 호출 테스트
curl -X GET "https://archive.pacs.ai-do.co.kr/iaid-pacs/aets/iAID_PACS/rs/series?limit=1" \
  -H "Accept: application/json"
```

---

## 문제별 해결 방법

### 문제 1: DB에서 허용된 Series UID가 0개

**원인:**
- `project_data`에 데이터가 없음
- 조인 실패 (`pd.study_id = pds.id` 또는 `pds.id = pdser.study_id`)
- `series_uid`가 NULL

**해결:**
1. 데이터 할당 재실행
2. 조인 조건 확인
3. 데이터 무결성 확인

### 문제 2: Dcm4chee QIDO가 0개 반환

**원인:**
- Dcm4chee 서버 연결 실패
- 인증 실패
- 실제로 Series가 없음

**해결:**
1. Dcm4chee 서버 상태 확인
2. 네트워크 연결 확인
3. 인증 설정 확인

### 문제 3: 필터링 후 0개

**원인:**
- Series UID 형식 불일치
- `extract_series_uid` 함수 문제
- 필터링 로직 문제

**해결:**
1. QIDO 응답 형식 확인
2. DB의 `series_uid` 형식 확인
3. 필터링 로직 디버깅

---

## 체크리스트

- [ ] DBeaver로 DB 연결 성공
- [ ] `test_get_allowed_series_uids.sql` 실행
- [ ] 허용된 Series UID 개수 확인
- [ ] 서버 재시작
- [ ] `check_with_logging.py` 실행
- [ ] 서버 로그에서 메시지 확인
- [ ] Dcm4chee 서버 상태 확인
- [ ] 문제 원인 파악
- [ ] 해결 방법 적용

---

## 참고 파일
- `CHECK_RESULTS.md` - 검사 결과 상세
- `test_get_allowed_series_uids.sql` - DB 쿼리 테스트
- `check_with_logging.py` - 서버 로그 확인
- `comprehensive_check.py` - 종합 검사
- `VERIFICATION_GUIDE.md` - 검증 가이드

