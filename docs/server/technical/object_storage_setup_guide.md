# Object Storage 설정 가이드

## 개요

PACS Extension Server의 마스크 업로드 시스템을 위한 Object Storage 설정 가이드입니다.

## 지원되는 Object Storage

1. **AWS S3** - 프로덕션 환경
2. **MinIO** - 로컬 개발 환경

## 설정 방법

### 1. 환경 변수 설정

`.env` 파일을 생성하고 다음 환경 변수를 설정하세요:

```bash
# Object Storage Configuration
# For AWS S3:
APP_OBJECT_STORAGE__PROVIDER=s3
APP_OBJECT_STORAGE__BUCKET_NAME=your-bucket-name
APP_OBJECT_STORAGE__REGION=us-east-1
APP_OBJECT_STORAGE__ENDPOINT=
APP_OBJECT_STORAGE__ACCESS_KEY=your-access-key
APP_OBJECT_STORAGE__SECRET_KEY=your-secret-key

# For MinIO (local development):
APP_OBJECT_STORAGE__PROVIDER=minio
APP_OBJECT_STORAGE__BUCKET_NAME=pacs-masks-dev
APP_OBJECT_STORAGE__REGION=us-east-1
APP_OBJECT_STORAGE__ENDPOINT=http://localhost:9000
APP_OBJECT_STORAGE__ACCESS_KEY=minioadmin
APP_OBJECT_STORAGE__SECRET_KEY=minioadmin

# Signed URL Configuration
APP_SIGNED_URL__DEFAULT_TTL=600
APP_SIGNED_URL__MAX_TTL=3600
```

### 2. 설정 파일 수정

`config/development.toml` 또는 `config/production.toml`에서 Object Storage 설정을 수정할 수 있습니다:

```toml
[object_storage]
provider = "minio"  # "s3" or "minio"
bucket_name = "pacs-masks-dev"
region = "us-east-1"
endpoint = "http://localhost:9000"  # MinIO endpoint (empty for AWS S3)
access_key = "minioadmin"
secret_key = "minioadmin"

[signed_url]
default_ttl = 600   # 10 minutes
max_ttl = 3600      # 1 hour
```

## MinIO 로컬 개발 설정

### 1. MinIO 설치

#### Docker를 사용한 설치 (권장)

```bash
# MinIO 서버 실행
docker run -p 9000:9000 -p 9001:9001 \
  --name minio \
  -e "MINIO_ROOT_USER=minioadmin" \
  -e "MINIO_ROOT_PASSWORD=minioadmin" \
  -v /data:/data \
  quay.io/minio/minio server /data --console-address ":9001"
```

#### 직접 설치

```bash
# macOS
brew install minio/stable/minio

# Ubuntu/Debian
wget https://dl.min.io/server/minio/release/linux-amd64/minio
chmod +x minio
sudo mv minio /usr/local/bin/

# 실행
minio server /data --console-address ":9001"
```

### 2. MinIO 설정

1. 브라우저에서 `http://localhost:9001` 접속
2. 사용자명: `minioadmin`, 비밀번호: `minioadmin`으로 로그인
3. 버킷 생성: `pacs-masks-dev`
4. 버킷 정책 설정 (선택사항)

### 3. 버킷 정책 설정

MinIO 콘솔에서 다음 정책을 설정하세요:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": "*",
      "Action": [
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject"
      ],
      "Resource": "arn:aws:s3:::pacs-masks-dev/*"
    }
  ]
}
```

## AWS S3 프로덕션 설정

### 1. AWS 계정 설정

1. AWS 계정 생성 및 로그인
2. IAM 사용자 생성
3. S3 권한 정책 할당

### 2. IAM 정책

다음 정책을 IAM 사용자에게 할당하세요:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject",
        "s3:GetObjectVersion"
      ],
      "Resource": "arn:aws:s3:::your-bucket-name/*"
    },
    {
      "Effect": "Allow",
      "Action": [
        "s3:ListBucket"
      ],
      "Resource": "arn:aws:s3:::your-bucket-name"
    }
  ]
}
```

### 3. S3 버킷 생성

1. AWS S3 콘솔에서 새 버킷 생성
2. 버킷 이름: `your-bucket-name`
3. 리전: `us-east-1` (또는 원하는 리전)
4. 버킷 정책 설정

### 4. 환경 변수 설정

```bash
APP_OBJECT_STORAGE__PROVIDER=s3
APP_OBJECT_STORAGE__BUCKET_NAME=your-bucket-name
APP_OBJECT_STORAGE__REGION=us-east-1
APP_OBJECT_STORAGE__ENDPOINT=
APP_OBJECT_STORAGE__ACCESS_KEY=your-access-key
APP_OBJECT_STORAGE__SECRET_KEY=your-secret-key
```

## 서버 시작

Object Storage 설정이 완료되면 서버를 시작하세요:

```bash
# 개발 환경
cargo run

# 프로덕션 환경
RUN_ENV=production cargo run
```

## 확인 방법

### 1. 서버 로그 확인

서버 시작 시 다음과 같은 로그가 출력되어야 합니다:

```
☁️  Initializing Object Storage service... ✅ Done (Provider: minio)
```

### 2. API 테스트

마스크 관련 API 엔드포인트가 활성화되었는지 확인:

```bash
# Swagger UI 접속
http://localhost:8080/swagger-ui/

# 또는 직접 API 호출
curl -X GET http://localhost:8080/api/annotations/1/mask-groups
```

### 3. Object Storage 연결 테스트

서버가 시작되면 Object Storage 연결이 자동으로 테스트됩니다. 연결 실패 시 서버가 시작되지 않습니다.

## 문제 해결

### 1. 연결 실패

**증상**: `Failed to initialize Object Storage` 오류

**해결 방법**:
- MinIO 서버가 실행 중인지 확인
- 네트워크 연결 확인
- 인증 정보 확인
- 버킷이 존재하는지 확인

### 2. 권한 오류

**증상**: `Permission denied` 오류

**해결 방법**:
- IAM 정책 확인
- 버킷 정책 확인
- MinIO 사용자 권한 확인

### 3. 버킷 없음 오류

**증상**: `Bucket not found` 오류

**해결 방법**:
- 버킷이 생성되었는지 확인
- 버킷 이름이 정확한지 확인
- 리전이 올바른지 확인

## 보안 고려사항

### 1. 인증 정보 보호

- `.env` 파일을 `.gitignore`에 추가
- 프로덕션에서는 환경 변수 사용
- IAM 사용자 최소 권한 원칙 적용

### 2. 네트워크 보안

- HTTPS 사용 (프로덕션)
- VPC 내부에서만 접근 가능하도록 설정
- 방화벽 규칙 설정

### 3. 데이터 암호화

- S3 서버 측 암호화 활성화
- 전송 중 암호화 (HTTPS)
- 민감한 데이터는 추가 암호화 고려

## 모니터링

### 1. 로그 모니터링

- Object Storage 작업 로그 확인
- 에러 로그 모니터링
- 성능 메트릭 수집

### 2. 비용 모니터링

- S3 사용량 모니터링
- 요청 수 모니터링
- 비용 알림 설정

## 참고 문서

- [AWS S3 개발자 가이드](https://docs.aws.amazon.com/s3/)
- [MinIO 문서](https://docs.min.io/)
- [Rust AWS SDK 문서](https://docs.rs/aws-sdk-s3/)
- [Object Storage 서비스 구현](./object_storage_integration.md)
