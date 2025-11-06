# Docker 배포 가이드

## 개요

이 문서는 PACS Extension Server를 Docker 컨테이너로 배포하는 방법을 설명합니다.

## 사전 요구사항

- Docker 20.10 이상
- Docker Compose 2.0 이상
- 최소 4GB RAM
- 최소 10GB 디스크 공간

## 빠른 시작

### 1. 개발 환경에서 실행

```bash
# 저장소 클론
git clone <repository-url>
cd pacs-ext-server/pacs-server

# 개발 환경으로 실행
./scripts/docker-compose-up.sh development
```

### 2. 프로덕션 환경에서 실행

```bash
# 프로덕션 환경 설정 파일 수정
cp env.production .env
# .env 파일에서 실제 값들로 수정

# 프로덕션 환경으로 실행
./scripts/docker-compose-up.sh production
```

## 환경별 설정

### Development 환경

- **파일**: `env.development`
- **용도**: 로컬 개발 및 테스트
- **특징**: 
  - 디버그 로깅 활성화
  - 로컬 데이터베이스 사용
  - MinIO 객체 저장소 사용

### Production 환경

- **파일**: `env.production`
- **용도**: 실제 서비스 운영
- **특징**:
  - 최적화된 로깅
  - AWS S3 객체 저장소 사용
  - 보안 강화 설정

### Test 환경

- **파일**: `env.test`
- **용도**: CI/CD 및 테스트
- **특징**:
  - 격리된 테스트 데이터베이스
  - 빠른 테스트 실행

## 서비스 구성

### 1. PACS Server (메인 애플리케이션)

- **포트**: 8080
- **이미지**: `pacs-server:latest`
- **의존성**: PostgreSQL, Redis, MinIO

### 2. PostgreSQL (데이터베이스)

- **포트**: 5432
- **이미지**: `postgres:15-alpine`
- **볼륨**: `pacs-postgres-data`

### 3. Redis (캐시)

- **포트**: 6379
- **이미지**: `redis:7-alpine`
- **볼륨**: `pacs-redis-data`

### 4. MinIO (객체 저장소)

- **포트**: 9000 (API), 9001 (Console)
- **이미지**: `minio/minio:latest`
- **볼륨**: `pacs-minio-data`

### 5. Nginx (리버스 프록시)

- **포트**: 80, 443
- **이미지**: `nginx:alpine`
- **역할**: 로드 밸런싱, SSL 종료

## 스크립트 사용법

### 빌드 스크립트

```bash
# 개발 환경 이미지 빌드
./scripts/docker-build.sh development

# 프로덕션 환경 이미지 빌드
./scripts/docker-build.sh production
```

### 실행 스크립트

```bash
# 개발 환경 실행
./scripts/docker-compose-up.sh development

# 프로덕션 환경 실행
./scripts/docker-compose-up.sh production

# 기존 컨테이너 정리 후 실행
./scripts/docker-compose-up.sh development --clean
```

### 중지 스크립트

```bash
# 서비스만 중지
./scripts/docker-compose-down.sh development

# 볼륨도 함께 제거
./scripts/docker-compose-down.sh development --volumes

# 모든 리소스 제거
./scripts/docker-compose-down.sh development --all
```

### 테스트 스크립트

```bash
# 테스트 환경에서 테스트
./scripts/docker-test.sh test

# 개발 환경에서 테스트
./scripts/docker-test.sh development
```

## 환경 변수 설정

### 필수 환경 변수

```bash
# 데이터베이스
POSTGRES_DB=pacs_db
POSTGRES_USER=admin
POSTGRES_PASSWORD=your_password
APP_DATABASE_URL=postgres://admin:your_password@postgres:5432/pacs_db

# JWT
APP_JWT_SECRET=your_jwt_secret_key

# 객체 저장소
APP_OBJECT_STORAGE_PROVIDER=minio
APP_OBJECT_STORAGE_ENDPOINT=http://minio:9000
APP_OBJECT_STORAGE_ACCESS_KEY=minioadmin
APP_OBJECT_STORAGE_SECRET_KEY=minioadmin123
APP_OBJECT_STORAGE_BUCKET=pacs-masks
```

### 선택적 환경 변수

```bash
# 서버 설정
APP_SERVER_HOST=0.0.0.0
APP_SERVER_PORT=8080

# CORS 설정
APP_CORS_ALLOWED_ORIGINS=http://localhost:3000,http://localhost:8080

# 로깅 설정
RUST_LOG=info
RUST_BACKTRACE=1
```

## 모니터링 및 로그

### 로그 확인

```bash
# 모든 서비스 로그
docker-compose --env-file env.development logs -f

# 특정 서비스 로그
docker-compose --env-file env.development logs -f pacs-server

# 최근 100줄 로그
docker-compose --env-file env.development logs --tail=100 pacs-server
```

### 헬스체크

```bash
# 서비스 상태 확인
docker-compose --env-file env.development ps

# 헬스체크 실행
./scripts/docker-test.sh development
```

## 보안 고려사항

### 1. 환경 변수 보안

- 프로덕션에서는 강력한 비밀번호 사용
- JWT 시크릿 키는 충분히 복잡하게 설정
- 환경 변수 파일은 `.gitignore`에 추가

### 2. 네트워크 보안

- 필요한 포트만 노출
- 내부 네트워크 사용
- SSL/TLS 인증서 설정

### 3. 컨테이너 보안

- 비root 사용자로 실행
- 최신 이미지 사용
- 불필요한 패키지 제거

## 트러블슈팅

### 일반적인 문제

1. **포트 충돌**
   ```bash
   # 사용 중인 포트 확인
   lsof -i :8080
   
   # 포트 변경
   # env.development 파일에서 APP_SERVER_PORT 수정
   ```

2. **메모리 부족**
   ```bash
   # Docker 메모리 제한 확인
   docker stats
   
   # Docker 메모리 제한 설정
   # docker-compose.yaml에서 mem_limit 추가
   ```

3. **데이터베이스 연결 실패**
   ```bash
   # PostgreSQL 로그 확인
   docker-compose --env-file env.development logs postgres
   
   # 데이터베이스 연결 테스트
   docker-compose --env-file env.development exec postgres pg_isready -U admin -d pacs_db
   ```

### 로그 분석

```bash
# 에러 로그만 필터링
docker-compose --env-file env.development logs pacs-server | grep ERROR

# 특정 시간대 로그
docker-compose --env-file env.development logs --since="2024-01-01T00:00:00" pacs-server
```

## 성능 최적화

### 1. 리소스 할당

```yaml
# docker-compose.yaml에서 리소스 제한 설정
services:
  pacs-server:
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: '1.0'
        reservations:
          memory: 1G
          cpus: '0.5'
```

### 2. 데이터베이스 최적화

```bash
# PostgreSQL 설정 최적화
# postgresql.conf에서 다음 설정 조정
shared_buffers = 256MB
effective_cache_size = 1GB
max_connections = 100
```

### 3. 캐싱 전략

```bash
# Redis 메모리 설정
# redis.conf에서 다음 설정 조정
maxmemory 512mb
maxmemory-policy allkeys-lru
```

## 백업 및 복구

### 데이터베이스 백업

```bash
# 백업 생성
docker-compose --env-file env.production exec postgres pg_dump -U admin pacs_db > backup.sql

# 백업 복원
docker-compose --env-file env.production exec -T postgres psql -U admin pacs_db < backup.sql
```

### 볼륨 백업

```bash
# 볼륨 백업
docker run --rm -v pacs-postgres-data:/data -v $(pwd):/backup alpine tar czf /backup/postgres-backup.tar.gz -C /data .

# 볼륨 복원
docker run --rm -v pacs-postgres-data:/data -v $(pwd):/backup alpine tar xzf /backup/postgres-backup.tar.gz -C /data
```

## 업그레이드

### 1. 이미지 업데이트

```bash
# 최신 이미지 다운로드
docker-compose --env-file env.production pull

# 서비스 재시작
docker-compose --env-file env.production up -d
```

### 2. 데이터베이스 마이그레이션

```bash
# 마이그레이션 실행
docker-compose --env-file env.production exec pacs-server ./pacs-server migrate
```

## 지원 및 문의

문제가 발생하거나 도움이 필요한 경우:

1. 로그 파일 확인
2. GitHub Issues에 문제 보고
3. 문서 검토
4. 커뮤니티 포럼 참여
