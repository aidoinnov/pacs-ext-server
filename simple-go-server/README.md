# Simple Go Server

간단한 Go 기반 HTTP 서버

## 요구사항

- Go 1.21 이상
- gvm (Go Version Manager) - 선택사항

## gvm 설치

```bash
bash < <(curl -s -S -L https://raw.githubusercontent.com/moovweb/gvm/master/binscripts/gvm-installer)
source ~/.gvm/scripts/gvm
```

## Go 설치 (gvm 사용)

```bash
# 사용 가능한 버전 확인
gvm listall

# Go 설치
gvm install go1.21.13 -B

# 기본 버전으로 설정
gvm use go1.21.13 --default
```

## 실행

```bash
go run main.go
```

서버는 `http://localhost:8080`에서 실행됩니다.

## 엔드포인트

- `GET /` - "Hello, World!" 응답
- `GET /health` - 헬스체크 (OK 응답)

## 테스트

```bash
curl http://localhost:8080
curl http://localhost:8080/health
```
