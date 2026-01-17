# 🚀 빠른 시작 가이드

## 1️⃣ 서버 실행 확인

```bash
curl http://localhost:8080/health
```

서버가 실행 중이 아니면:
```bash
cd ../../pacs-server
cargo run --release --bin pacs_server
```

---

## 2️⃣ 테스트 실행

### 전체 테스트 한 번에 실행
```bash
cd tests/e2e
./run_all_tests.sh
```

### 개별 테스트 실행 (추천! 👍)

#### 🎬 데모 테스트 (빠른 확인용)
```bash
./run_demo.sh
```
- 서버 헬스 체크
- 로그인
- 사용자 정보 조회
- 프로젝트 목록 조회
- 어노테이션 조회

#### 🔐 인증 테스트
```bash
./run_auth.sh
```
- 로그인 성공/실패
- 토큰 검증
- 권한 체크

#### 📸 스냅샷 URL 테스트
```bash
./run_snapshot.sh
```
- 어노테이션 리스트 조회 시 스냅샷 URL 확인
- 개별 어노테이션 조회 시 스냅샷 URL 확인
- S3 Signed URL 생성 확인

#### ⚡ 성능 테스트
```bash
./run_performance.sh
```
- 동시 로그인 (10명)
- 동시 어노테이션 조회 (100회)
- 동시 프로젝트 조회 (100회)
- 응답 시간 메트릭 (평균, P95, P99)

---

## 3️⃣ 테스트 결과 확인

성공 시:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎉 모든 테스트 통과!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

실패 시:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
❌ 1 개의 테스트 실패
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 📁 파일 구조

```
tests/e2e/
├── run_all_tests.sh       # 전체 테스트 실행
├── run_demo.sh            # 데모 테스트
├── run_auth.sh            # 인증 테스트
├── run_snapshot.sh        # 스냅샷 URL 테스트
├── run_performance.sh     # 성능 테스트
└── README.md              # 상세 문서
```

---

## 🔧 문제 해결

### 가상환경이 없다는 에러
```bash
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

### 서버가 실행 중이지 않다는 에러
```bash
cd ../../pacs-server
cargo run --release --bin pacs_server
```

### 권한 에러
```bash
chmod +x run_*.sh
```

---

## 💡 팁

- **빠른 확인**: `./run_demo.sh` 먼저 실행
- **스냅샷 기능 확인**: `./run_snapshot.sh` 실행
- **성능 확인**: `./run_performance.sh` 실행
- **전체 확인**: `./run_all_tests.sh` 실행

---

## 📞 도움말

더 자세한 내용은 `README.md`를 참고하세요.

