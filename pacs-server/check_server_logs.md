# 서버 로그 확인 가이드

## 확인할 로그 메시지

서버를 재시작한 후 `/api/me/dicom/series?project_id=2`를 호출하면 다음 로그 메시지들이 출력됩니다:

### 1. DB 조회 결과
```
🔍 Gateway /series: Found X allowed series UIDs for project 2
   Allowed Series UIDs: [...]
```
- **예상**: `Found 5 allowed series UIDs` (DB에 5개가 있으므로)
- **문제**: `Found 0 allowed series UIDs` → 쿼리 문제

### 2. QIDO 응답
```
🔍 Gateway /series: QIDO returned X series
   QIDO Series UIDs: [...]
```
- **예상**: QIDO에서 Series를 반환함
- **문제**: `QIDO returned 0 series` → QIDO 연결 문제

### 3. 매칭 결과
```
   Matched Series UIDs: X/Y
```
- **예상**: `Matched 5/5` 또는 `Matched 5/10` (일부만 매칭)
- **문제**: `Matched 0/10` → UID 형식 불일치

### 4. 매칭 실패 시 경고
```
⚠️  No Series UIDs matched! This might indicate a UID format mismatch.
   Allowed UIDs (first 3): [...]
   QIDO UIDs (first 3): [...]
```
- 이 메시지가 나오면 UID 형식이 다를 수 있음

### 5. 필터링 결과
```
🔍 Gateway /series: Filtered X series from Y QIDO results
```
- **예상**: `Filtered 5 from 10 QIDO results`
- **문제**: `Filtered 0 from 10 QIDO results` → 매칭 실패

## 문제 진단

### 케이스 1: DB 조회 실패
```
Found 0 allowed series UIDs for project 2
⚠️  No allowed series UIDs found for project 2!
```
→ 쿼리 문제 또는 project_data가 비어있음

### 케이스 2: QIDO 응답 없음
```
QIDO returned 0 series
```
→ QIDO 연결 문제 또는 해당 Series가 QIDO에 없음

### 케이스 3: UID 형식 불일치
```
Matched 0/10
⚠️  No Series UIDs matched!
   Allowed UIDs (first 3): ["1.2.840.113619.2.311..."]
   QIDO UIDs (first 3): ["1.2.840.113619.2.311..."]
```
→ UID 형식이 다름 (공백, 대소문자, 특수문자 등)

## 해결 방법

1. **DB 조회 실패**: 쿼리 확인, project_data 확인
2. **QIDO 응답 없음**: QIDO 연결 확인, Dcm4chee에서 해당 Series 존재 확인
3. **UID 형식 불일치**: UID 정규화 (trim, lowercase 등) 적용

