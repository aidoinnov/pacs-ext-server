# S3 Signed URL 생성 오류 수정

## 문제 상황
- S3 signed URL 생성 시 "액세스키가 없다" 오류 발생
- `AuthorizationQueryParametersError: Error parsing the X-Amz-Credential parameter; a non-empty Access Key (AKID) must be provided in the credential.`

## 원인 분석
1. **환경 변수 로딩 문제**: `.env` 파일에 중복된 키가 있어서 환경 변수가 제대로 로드되지 않음
2. **Config 파일 하드코딩**: TOML 파일에 하드코딩된 값이 환경 변수를 덮어쓰고 있었음

## 해결 과정

### 1. .env 파일 정리
**문제**: 중복된 키와 주석 처리된 키가 혼재
```bash
# 기존 .env 파일 (중복된 키와 주석 처리된 키 혼재)
# APP_OBJECT_STORAGE__ACCESS_KEY_ID=AKIA...
APP_OBJECT_STORAGE__ACCESS_KEY_ID=AKIA...
# APP_OBJECT_STORAGE__SECRET_ACCESS_KEY=ViC4...
APP_OBJECT_STORAGE__SECRET_ACCESS_KEY=VUEI...
```

**해결**: 중복 제거 및 정리
```bash
# 정리된 .env 파일
APP_OBJECT_STORAGE__PROVIDER=s3
APP_OBJECT_STORAGE__BUCKET_NAME=pacs-masks
APP_OBJECT_STORAGE__REGION=ap-northeast-2
APP_OBJECT_STORAGE__ACCESS_KEY_ID=AKIA...
APP_OBJECT_STORAGE__SECRET_ACCESS_KEY=VUEI...
```

### 2. Config 파일 수정
**문제**: TOML 파일에 하드코딩된 값이 환경 변수를 덮어쓰고 있었음

**수정된 파일들**:
- `config/default.toml`
- `config/development.toml` 
- `config/production.toml`

**변경 내용**:
```toml
# 기존 (하드코딩된 값)
[object_storage]
provider = "s3"
bucket_name = "pacs-masks"
region = "ap-northeast-2"
access_key_id = "AKIA..."
secret_access_key = "VUEI..."

# 수정 후 (환경 변수 사용)
[object_storage]
# All object storage settings should be configured via environment variables:
# APP_OBJECT_STORAGE__PROVIDER
# APP_OBJECT_STORAGE__BUCKET_NAME
# APP_OBJECT_STORAGE__REGION
# APP_OBJECT_STORAGE__ENDPOINT (optional)
# APP_OBJECT_STORAGE__ACCESS_KEY_ID
# APP_OBJECT_STORAGE__SECRET_ACCESS_KEY
```

### 3. 디버깅 코드 추가
환경 변수 로딩 상태를 확인하기 위해 디버깅 코드 추가:

```rust
// main.rs
println!("🔍 환경 변수 로딩 확인:");
println!("   APP_OBJECT_STORAGE__ACCESS_KEY_ID: {}", 
    std::env::var("APP_OBJECT_STORAGE__ACCESS_KEY_ID").unwrap_or_else(|_| "NOT_FOUND".to_string()));
println!("   APP_OBJECT_STORAGE__SECRET_ACCESS_KEY: {}", 
    std::env::var("APP_OBJECT_STORAGE__SECRET_ACCESS_KEY").unwrap_or_else(|_| "NOT_FOUND".to_string()));

// settings.rs
println!("🔧 Object Storage 설정 로드:");
println!("   Access Key: {} (길이: {})", 
    if access_key.is_empty() { "EMPTY".to_string() } else { format!("{}...{}", &access_key[..access_key.len().min(8)], &access_key[access_key.len().saturating_sub(4)..]) },
    access_key.len()
);
```

## 결과
✅ **S3 자격 증명이 정상적으로 로드됨**:
- Access Key Length: 20
- Secret Key Length: 40
- Bucket: pacs-masks
- Region: ap-northeast-2

✅ **S3 Signed URL 생성이 정상적으로 작동함**

## 환경 변수 우선순위
1. **환경 변수** (APP_ 접두사) - 최고 우선순위
2. **.env 파일**
3. **config/{environment}.toml**
4. **config/default.toml** - 최저 우선순위

## 주의사항
- TOML 파일에 민감한 정보(API 키, 비밀번호 등)를 하드코딩하지 말 것
- 모든 민감한 정보는 환경 변수로 관리할 것
- .env 파일에 중복된 키가 있지 않도록 주의할 것

## 관련 파일
- `.env`
- `config/default.toml`
- `config/development.toml`
- `config/production.toml`
- `src/main.rs`
- `src/infrastructure/config/settings.rs`
- `src/infrastructure/external/s3_service.rs`
