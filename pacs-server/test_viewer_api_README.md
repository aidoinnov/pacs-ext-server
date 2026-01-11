# Viewer Study Meta API E2E 테스트

## 개요

Viewer Study Meta API (`POST /api/v1/viewer/studies/meta`)의 E2E 테스트 스크립트입니다.

## 사전 요구사항

1. **서버 실행**
   ```bash
   cd pacs-server
   cargo run --bin pacs_server
   ```

2. **Python 의존성**
   ```bash
   pip install requests
   ```

3. **로그인 정보**
   - 테스트 스크립트는 기본적으로 다음 계정을 사용합니다:
     - Username: `iaid-pacs-admin`
     - Password: `Qlalfqjsgh1!`
   - 필요시 `test_viewer_api_e2e.py` 파일의 `get_token()` 함수에서 수정 가능

## 테스트 실행

```bash
cd pacs-server
python3 test_viewer_api_e2e.py
```

## 테스트 시나리오

### 1. 서버 Health Check
- 서버가 정상적으로 실행 중인지 확인

### 2. 로그인 및 토큰 획득
- JWT 토큰 발급 테스트
- 인증 시스템 동작 확인

### 3. 샘플 Study UID 조회
- QIDO-RS를 통해 실제 Study UID 조회
- 실패 시 하드코딩된 테스트 UID 사용

### 4. Viewer Study Meta API - 정상 케이스
- 여러 Study UID에 대한 메타데이터 조회
- 응답 구조 검증
- 필수 필드 확인 (study_uid, patient_id 등)

### 5. 빈 Study UID 목록 테스트
- 빈 배열 전송 시 서버 응답 확인

### 6. 존재하지 않는 Study UID 테스트
- 유효하지 않은 UID 전송 시 처리 확인

### 7. 인증 없는 접근 테스트
- Authorization 헤더 없이 요청 시 401/403 응답 확인

## 예상 결과

```
======================================================================
🔍 Viewer Study Meta API E2E 테스트 시작
======================================================================

📋 Test: 서버 Health Check
✅ 서버 정상: {'service': 'pacs-extension-server', 'status': 'healthy'}

📋 Test: 로그인 및 토큰 획득
✅ 로그인 성공 (token length: 1385)

📋 Test: 샘플 Study UID 조회
✅ 2개 Study UID 조회 성공

📋 Test: Viewer Study Meta API - 정상 케이스
✅ 응답 성공: 2개 Study 메타데이터 조회

...

======================================================================
🔍 테스트 결과 요약
======================================================================
Total Tests: 7
✅ Passed: 7
❌ Failed: 0

🎉 모든 테스트 통과!
```

## API 엔드포인트

- **URL**: `POST /api/v1/viewer/studies/meta`
- **인증**: Bearer Token (JWT)
- **Content-Type**: `application/json`

### Request Body

```json
{
  "study_uids": [
    "1.2.840.113619.2.55.3.604688433.1234",
    "1.2.840.113619.2.55.3.604688433.5678"
  ]
}
```

### Response (200 OK)

```json
{
  "studies": [
    {
      "study_uid": "1.2.840.113619.2.55.3.604688433.1234",
      "study_description": "Chest CT",
      "patient_id": "P123456",
      "patient_name": "DOE^JOHN",
      "study_date": "20240115",
      "modality": "CT"
    }
  ]
}
```

## 트러블슈팅

### 서버 연결 실패
```
❌ 서버 연결 실패: Connection refused
```
- 서버가 실행 중인지 확인: `curl http://localhost:8080/health`
- 포트 8080이 사용 중인지 확인: `lsof -i :8080`

### 로그인 실패
```
❌ 로그인 실패: 401
```
- 로그인 정보 확인
- 데이터베이스 연결 확인
- JWT 서비스 설정 확인

### 404 Not Found
```
❌ API 호출 실패: 404
```
- 서버가 최신 코드로 빌드되었는지 확인
- 라우팅 설정 확인
- 의존성 주입 확인

## 커스터마이징

### 다른 로그인 정보 사용

`test_viewer_api_e2e.py` 파일에서 수정:

```python
def get_token() -> Optional[str]:
    resp = requests.post(f'{BASE_URL}/api/auth/login', json={
        'username': 'your-username',  # 여기 수정
        'password': 'your-password'   # 여기 수정
    }, timeout=10)
```

### 다른 Study UID 사용

```python
hardcoded_uids = [
    "your-study-uid-1",  # 여기 수정
    "your-study-uid-2"   # 여기 수정
]
```

## 참고 문서

- [API 스펙](../docs/api/viewer-api/study-api.md)
- [구현 가이드](../docs/api/viewer-api/)

