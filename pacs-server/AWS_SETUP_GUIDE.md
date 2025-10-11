# 🚀 AWS S3 설정 가이드

## 📋 필요한 AWS 정보

### 1. **AWS 계정 정보**
- **AWS Access Key ID**: 20자리 영문+숫자 조합
- **AWS Secret Access Key**: 40자리 영문+숫자+기호 조합
- **AWS Region**: 예) `us-east-1`, `ap-northeast-2` (서울)

### 2. **S3 버킷 설정**
- **버킷 이름**: `pacs-masks` (또는 원하는 이름)
- **리전**: Access Key와 동일한 리전
- **버킷 정책**: 마스크 업로드용 권한 설정

### 3. **IAM 정책 설정**
```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "s3:PutObject",
                "s3:GetObject",
                "s3:DeleteObject"
            ],
            "Resource": "arn:aws:s3:::pacs-masks/mask/*"
        },
        {
            "Effect": "Allow",
            "Action": "s3:ListBucket",
            "Resource": "arn:aws:s3:::pacs-masks"
        }
    ]
}
```

## 🔧 설정 방법

### 1. **AWS 콘솔에서 IAM 사용자 생성**

1. **AWS IAM 콘솔** 접속
2. **사용자** → **사용자 추가**
3. **사용자 이름**: `pacs-mask-uploader`
4. **액세스 유형**: **프로그래밍 방식 액세스** 선택
5. **권한**: **기존 정책 직접 연결** → 위의 정책 JSON 붙여넣기
6. **태그**: 선택사항 (환경 구분용)
7. **검토** → **사용자 만들기**
8. **Access Key ID**와 **Secret Access Key** 복사 (한 번만 표시됨!)

### 2. **S3 버킷 생성**

1. **AWS S3 콘솔** 접속
2. **버킷 만들기**
3. **버킷 이름**: `pacs-masks`
4. **리전**: IAM 사용자와 동일한 리전 선택
5. **퍼블릭 액세스 차단**: **모든 퍼블릭 액세스 차단** (보안)
6. **버킷 만들기**

### 3. **CORS 설정 (선택사항)**

버킷 → **권한** → **CORS** → 다음 설정 추가:

```json
[
    {
        "AllowedHeaders": ["*"],
        "AllowedMethods": ["GET", "PUT", "POST", "DELETE"],
        "AllowedOrigins": ["http://localhost:3000", "http://localhost:8080"],
        "ExposeHeaders": ["ETag"],
        "MaxAgeSeconds": 3000
    }
]
```

## ⚙️ 환경변수 설정

### 1. **.env 파일에 추가**
```bash
# AWS S3 설정
APP_OBJECT_STORAGE__PROVIDER=s3
APP_OBJECT_STORAGE__BUCKET_NAME=pacs-masks
APP_OBJECT_STORAGE__REGION=us-east-1
APP_OBJECT_STORAGE__ENDPOINT=
APP_OBJECT_STORAGE__ACCESS_KEY=AKIA...  # 실제 Access Key
APP_OBJECT_STORAGE__SECRET_KEY=...      # 실제 Secret Key

# Signed URL 설정
APP_SIGNED_URL__DEFAULT_TTL=600
APP_SIGNED_URL__MAX_TTL=3600
```

### 2. **config/production.toml에 추가**
```toml
[object_storage]
provider = "s3"
bucket_name = "pacs-masks"
region = "us-east-1"
endpoint = ""
access_key = "AKIA..."  # 실제 값으로 교체
secret_key = "..."      # 실제 값으로 교체

[signed_url]
default_ttl = 600
max_ttl = 3600
```

## 🧪 테스트 방법

### 1. **AWS CLI로 테스트**
```bash
# AWS CLI 설치 후
aws configure
# Access Key ID, Secret Key, Region 입력

# 버킷 접근 테스트
aws s3 ls s3://pacs-masks

# 파일 업로드 테스트
echo "test" > test.txt
aws s3 cp test.txt s3://pacs-masks/test.txt
aws s3 rm s3://pacs-masks/test.txt
```

### 2. **Rust 코드로 테스트**
```rust
// 간단한 연결 테스트
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client as S3Client;

#[tokio::main]
async fn main() {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = S3Client::new(&config);
    
    // 버킷 리스트 조회
    let response = client.list_buckets().send().await.unwrap();
    println!("Buckets: {:?}", response.buckets);
}
```

## 🔒 보안 고려사항

### 1. **최소 권한 원칙**
- S3 버킷의 특정 경로(`mask/*`)에만 접근 가능
- 다른 AWS 서비스 접근 불가
- 버킷 생성/삭제 권한 없음

### 2. **환경별 분리**
- **개발**: `pacs-masks-dev`
- **스테이징**: `pacs-masks-staging`  
- **프로덕션**: `pacs-masks-prod`

### 3. **비용 최적화**
- **스토리지 클래스**: Standard (자주 접근)
- **수명 주기 정책**: 90일 후 IA, 1년 후 Glacier
- **버전 관리**: 비활성화 (마스크는 덮어쓰기)

## 📊 모니터링 설정

### 1. **CloudWatch 메트릭**
- S3 요청 수
- 데이터 전송량
- 에러율

### 2. **알람 설정**
- 4xx 에러율 > 5%
- 5xx 에러율 > 1%
- 요청 수 급증

## 🚨 문제 해결

### 1. **접근 거부 오류**
```bash
# 권한 확인
aws s3api get-bucket-policy --bucket pacs-masks
aws iam get-user-policy --user-name pacs-mask-uploader --policy-name S3MaskPolicy
```

### 2. **리전 불일치**
- Access Key와 버킷이 같은 리전에 있는지 확인
- `us-east-1`이 기본값이므로 주의

### 3. **CORS 오류**
- 브라우저에서 직접 업로드 시 CORS 설정 확인
- AllowedOrigins에 클라이언트 도메인 포함

## 📝 체크리스트

- [ ] AWS 계정 생성/로그인
- [ ] IAM 사용자 생성 (`pacs-mask-uploader`)
- [ ] IAM 정책 적용 (S3 마스크 업로드 권한)
- [ ] Access Key ID 및 Secret Key 복사
- [ ] S3 버킷 생성 (`pacs-masks`)
- [ ] CORS 설정 (필요시)
- [ ] 환경변수 설정 (`.env` 또는 `production.toml`)
- [ ] AWS CLI 테스트
- [ ] Rust 코드 테스트
- [ ] 모니터링 설정

---

**⚠️ 주의사항**: 
- Secret Key는 절대 코드에 하드코딩하지 마세요
- 프로덕션에서는 환경변수나 AWS Secrets Manager 사용
- 정기적으로 Access Key 로테이션 권장
