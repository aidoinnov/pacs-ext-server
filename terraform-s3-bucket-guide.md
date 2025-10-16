# 🪣 S3 버킷 생성 및 관리 가이드

Terraform을 사용하여 AWS S3 버킷을 생성하고 관리하는 방법을 학습합니다. PACS 프로젝트의 마스크 이미지 저장을 위한 Object Storage 설정을 중심으로 다룹니다.

## 📋 목차

1. [S3 버킷이란?](#s3-버킷이란)
2. [기본 S3 버킷 생성](#기본-s3-버킷-생성)
3. [고급 S3 설정](#고급-s3-설정)
4. [PACS 프로젝트 S3 구성](#pacs-프로젝트-s3-구성)
5. [CORS 및 보안 설정](#cors-및-보안-설정)
6. [실습 및 테스트](#실습-및-테스트)

---

## 🎯 S3 버킷이란?

**Amazon S3 (Simple Storage Service)**는 AWS의 객체 스토리지 서비스입니다.

### 주요 특징
- **무제한 스토리지**: 확장 가능한 객체 스토리지
- **고가용성**: 99.999999999% (11개 9) 내구성
- **다양한 스토리지 클래스**: 비용 최적화를 위한 선택
- **버전 관리**: 파일 변경 이력 추적
- **암호화**: 저장 및 전송 중 암호화

### PACS 프로젝트에서의 활용
- **마스크 이미지 저장**: DICOM 마스크 파일 저장
- **Signed URL**: 안전한 업로드/다운로드
- **CORS 설정**: 웹 브라우저에서 직접 업로드
- **생명주기 정책**: 오래된 파일 자동 아카이브

---

## 🔧 기본 S3 버킷 생성

### 1. 최소 구성

#### `s3-basic.tf`
```hcl
# 기본 S3 버킷
resource "aws_s3_bucket" "basic_bucket" {
  bucket = "my-basic-bucket-${random_id.bucket_suffix.hex}"
  
  tags = {
    Name        = "Basic S3 Bucket"
    Environment = "development"
  }
}

# 버킷 이름 충돌 방지를 위한 랜덤 ID
resource "random_id" "bucket_suffix" {
  byte_length = 4
}

# 버킷 공개 액세스 차단
resource "aws_s3_bucket_public_access_block" "basic_bucket" {
  bucket = aws_s3_bucket.basic_bucket.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}
```

### 2. 버킷 정책 설정

#### `s3-policy.tf`
```hcl
# S3 버킷 정책
resource "aws_s3_bucket_policy" "basic_bucket_policy" {
  bucket = aws_s3_bucket.basic_bucket.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid       = "DenyInsecureConnections"
        Effect    = "Deny"
        Principal = "*"
        Action    = "s3:*"
        Resource = [
          aws_s3_bucket.basic_bucket.arn,
          "${aws_s3_bucket.basic_bucket.arn}/*"
        ]
        Condition = {
          Bool = {
            "aws:SecureTransport" = "false"
          }
        }
      }
    ]
  })
}
```

### 3. 버킷 생성 및 확인

```bash
# Terraform 실행
terraform init
terraform plan
terraform apply

# S3 버킷 확인
aws s3 ls
aws s3 ls s3://my-basic-bucket-abc123def
```

---

## ⚙️ 고급 S3 설정

### 1. 버전 관리 및 암호화

#### `s3-advanced.tf`
```hcl
# 고급 S3 버킷
resource "aws_s3_bucket" "advanced_bucket" {
  bucket = "my-advanced-bucket-${random_id.bucket_suffix.hex}"
  
  tags = {
    Name        = "Advanced S3 Bucket"
    Environment = "production"
    Purpose     = "PACS Data Storage"
  }
}

# 버전 관리 활성화
resource "aws_s3_bucket_versioning" "advanced_bucket" {
  bucket = aws_s3_bucket.advanced_bucket.id
  
  versioning_configuration {
    status = "Enabled"
  }
}

# 서버 측 암호화 설정
resource "aws_s3_bucket_server_side_encryption_configuration" "advanced_bucket" {
  bucket = aws_s3_bucket.advanced_bucket.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
    bucket_key_enabled = true
  }
}

# 생명주기 정책
resource "aws_s3_bucket_lifecycle_configuration" "advanced_bucket" {
  bucket = aws_s3_bucket.advanced_bucket.id

  rule {
    id     = "transition_to_ia"
    status = "Enabled"

    transition {
      days          = 30
      storage_class = "STANDARD_IA"
    }

    transition {
      days          = 90
      storage_class = "GLACIER"
    }

    transition {
      days          = 365
      storage_class = "DEEP_ARCHIVE"
    }
  }

  rule {
    id     = "delete_old_versions"
    status = "Enabled"

    noncurrent_version_transition {
      days          = 30
      storage_class = "STANDARD_IA"
    }

    noncurrent_version_transition {
      days          = 90
      storage_class = "GLACIER"
    }

    noncurrent_version_expiration {
      days = 2555  # 7년
    }
  }
}
```

### 2. 로깅 및 모니터링

#### `s3-logging.tf`
```hcl
# 로그 버킷
resource "aws_s3_bucket" "logs_bucket" {
  bucket = "my-logs-bucket-${random_id.bucket_suffix.hex}"
  
  tags = {
    Name        = "S3 Logs Bucket"
    Environment = "production"
  }
}

# 로그 버킷 공개 액세스 차단
resource "aws_s3_bucket_public_access_block" "logs_bucket" {
  bucket = aws_s3_bucket.logs_bucket.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# 액세스 로깅 설정
resource "aws_s3_bucket_logging" "advanced_bucket" {
  bucket = aws_s3_bucket.advanced_bucket.id

  target_bucket = aws_s3_bucket.logs_bucket.id
  target_prefix = "logs/"
}

# CloudWatch 메트릭 설정
resource "aws_s3_bucket_metric" "advanced_bucket" {
  bucket = aws_s3_bucket.advanced_bucket.id
  name   = "EntireBucket"
}
```

### 3. 알림 설정

#### `s3-notifications.tf`
```hcl
# SNS 토픽 생성
resource "aws_sns_topic" "s3_notifications" {
  name = "s3-bucket-notifications"
  
  tags = {
    Name = "S3 Notifications"
  }
}

# S3 버킷 알림 설정
resource "aws_s3_bucket_notification" "advanced_bucket" {
  bucket = aws_s3_bucket.advanced_bucket.id

  topic {
    topic_arn     = aws_sns_topic.s3_notifications.arn
    events        = ["s3:ObjectCreated:*", "s3:ObjectRemoved:*"]
    filter_prefix = "uploads/"
    filter_suffix = ".png"
  }
}
```

---

## 🏥 PACS 프로젝트 S3 구성

### 1. 마스크 이미지 전용 버킷

#### `pacs-s3.tf`
```hcl
# PACS 마스크 이미지 버킷
resource "aws_s3_bucket" "pacs_masks" {
  bucket = "${var.project_name}-${var.environment}-masks"
  
  tags = {
    Name        = "PACS Masks Storage"
    Project     = var.project_name
    Environment = var.environment
    Purpose     = "DICOM Mask Images"
    ManagedBy   = "terraform"
  }
}

# 버킷 버전 관리
resource "aws_s3_bucket_versioning" "pacs_masks" {
  bucket = aws_s3_bucket.pacs_masks.id
  
  versioning_configuration {
    status = "Enabled"
  }
}

# 서버 측 암호화
resource "aws_s3_bucket_server_side_encryption_configuration" "pacs_masks" {
  bucket = aws_s3_bucket.pacs_masks.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
    bucket_key_enabled = true
  }
}

# 퍼블릭 액세스 차단
resource "aws_s3_bucket_public_access_block" "pacs_masks" {
  bucket = aws_s3_bucket.pacs_masks.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}
```

### 2. CORS 설정

#### `pacs-cors.tf`
```hcl
# CORS 설정
resource "aws_s3_bucket_cors_configuration" "pacs_masks" {
  bucket = aws_s3_bucket.pacs_masks.id

  cors_rule {
    allowed_headers = [
      "Authorization",
      "Content-Length",
      "Content-Type",
      "X-Amz-Date",
      "X-Amz-Content-Sha256",
      "X-Amz-User-Agent"
    ]
    allowed_methods = ["GET", "PUT", "POST", "DELETE", "HEAD"]
    allowed_origins = var.allowed_origins
    expose_headers  = ["ETag", "x-amz-request-id"]
    max_age_seconds = 3000
  }
}
```

### 3. IAM 정책 및 역할

#### `pacs-iam.tf`
```hcl
# S3 액세스 정책
resource "aws_iam_policy" "pacs_s3_policy" {
  name        = "${var.project_name}-s3-policy"
  description = "Policy for PACS S3 access"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject",
          "s3:ListBucket"
        ]
        Resource = [
          aws_s3_bucket.pacs_masks.arn,
          "${aws_s3_bucket.pacs_masks.arn}/*"
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "s3:GetObjectVersion"
        ]
        Resource = "${aws_s3_bucket.pacs_masks.arn}/*"
      }
    ]
  })
}

# IAM 역할
resource "aws_iam_role" "pacs_s3_role" {
  name = "${var.project_name}-s3-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ec2.amazonaws.com"
        }
      }
    ]
  })
}

# 정책 연결
resource "aws_iam_role_policy_attachment" "pacs_s3_policy" {
  role       = aws_iam_role.pacs_s3_role.name
  policy_arn = aws_iam_policy.pacs_s3_policy.arn
}
```

### 4. Signed URL 생성을 위한 Lambda 함수

#### `pacs-lambda.tf`
```hcl
# Lambda 함수용 IAM 역할
resource "aws_iam_role" "lambda_s3_role" {
  name = "${var.project_name}-lambda-s3-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      }
    ]
  })
}

# Lambda 함수 정책
resource "aws_iam_policy" "lambda_s3_policy" {
  name        = "${var.project_name}-lambda-s3-policy"
  description = "Policy for Lambda S3 operations"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject"
        ]
        Resource = "${aws_s3_bucket.pacs_masks.arn}/*"
      },
      {
        Effect = "Allow"
        Action = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
        Resource = "arn:aws:logs:*:*:*"
      }
    ]
  })
}

# 정책 연결
resource "aws_iam_role_policy_attachment" "lambda_s3_policy" {
  role       = aws_iam_role.lambda_s3_role.name
  policy_arn = aws_iam_policy.lambda_s3_policy.arn
}

# Lambda 함수
resource "aws_lambda_function" "signed_url_generator" {
  filename         = "signed_url_generator.zip"
  function_name    = "${var.project_name}-signed-url-generator"
  role            = aws_iam_role.lambda_s3_role.arn
  handler         = "index.handler"
  runtime         = "python3.9"
  timeout         = 30

  environment {
    variables = {
      S3_BUCKET = aws_s3_bucket.pacs_masks.bucket
    }
  }

  tags = {
    Name = "Signed URL Generator"
  }
}
```

---

## 🔒 CORS 및 보안 설정

### 1. 환경별 CORS 설정

#### `cors-config.tf`
```hcl
# 환경별 CORS 설정
locals {
  cors_config = {
    development = [
      "http://localhost:3000",
      "http://localhost:8080",
      "http://127.0.0.1:3000",
      "http://127.0.0.1:8080"
    ]
    staging = [
      "https://staging.pacs.example.com",
      "https://staging-viewer.pacs.example.com"
    ]
    production = [
      "https://pacs.example.com",
      "https://viewer.pacs.example.com"
    ]
  }
}

# 동적 CORS 설정
resource "aws_s3_bucket_cors_configuration" "pacs_masks_dynamic" {
  bucket = aws_s3_bucket.pacs_masks.id

  cors_rule {
    allowed_headers = ["*"]
    allowed_methods = ["GET", "PUT", "POST", "DELETE", "HEAD"]
    allowed_origins = local.cors_config[var.environment]
    expose_headers  = ["ETag", "x-amz-request-id", "x-amz-id-2"]
    max_age_seconds = 3600
  }
}
```

### 2. 버킷 정책 보안 강화

#### `security-policy.tf`
```hcl
# 강화된 버킷 정책
resource "aws_s3_bucket_policy" "pacs_masks_secure" {
  bucket = aws_s3_bucket.pacs_masks.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "DenyInsecureConnections"
        Effect = "Deny"
        Principal = "*"
        Action = "s3:*"
        Resource = [
          aws_s3_bucket.pacs_masks.arn,
          "${aws_s3_bucket.pacs_masks.arn}/*"
        ]
        Condition = {
          Bool = {
            "aws:SecureTransport" = "false"
          }
        }
      },
      {
        Sid    = "DenyUnencryptedObjectUploads"
        Effect = "Deny"
        Principal = "*"
        Action = "s3:PutObject"
        Resource = "${aws_s3_bucket.pacs_masks.arn}/*"
        Condition = {
          StringNotEquals = {
            "s3:x-amz-server-side-encryption" = "AES256"
          }
        }
      },
      {
        Sid    = "AllowPACSApplicationAccess"
        Effect = "Allow"
        Principal = {
          AWS = aws_iam_role.pacs_s3_role.arn
        }
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject"
        ]
        Resource = "${aws_s3_bucket.pacs_masks.arn}/*"
      }
    ]
  })
}
```

---

## 🧪 실습 및 테스트

### 1. S3 버킷 생성 테스트

#### `test-s3-creation.sh`
```bash
#!/bin/bash
# S3 버킷 생성 테스트 스크립트

echo "Testing S3 bucket creation..."

# Terraform 초기화
echo "1. Initializing Terraform..."
terraform init

# Terraform 검증
echo "2. Validating configuration..."
terraform validate

# Terraform 계획
echo "3. Planning S3 bucket creation..."
terraform plan -target=aws_s3_bucket.pacs_masks

# S3 버킷 생성
echo "4. Creating S3 bucket..."
terraform apply -target=aws_s3_bucket.pacs_masks -auto-approve

# 버킷 확인
echo "5. Verifying bucket creation..."
aws s3 ls s3://pacs-development-masks

echo "S3 bucket creation test completed! 🎉"
```

### 2. CORS 설정 테스트

#### `test-cors.sh`
```bash
#!/bin/bash
# CORS 설정 테스트 스크립트

echo "Testing CORS configuration..."

# CORS 설정 확인
echo "1. Checking CORS configuration..."
aws s3api get-bucket-cors --bucket pacs-development-masks

# 테스트 파일 업로드
echo "2. Testing file upload..."
echo "test content" > test-file.txt
aws s3 cp test-file.txt s3://pacs-development-masks/test/

# 테스트 파일 다운로드
echo "3. Testing file download..."
aws s3 cp s3://pacs-development-masks/test/test-file.txt downloaded-test-file.txt

# 정리
echo "4. Cleaning up test files..."
rm test-file.txt downloaded-test-file.txt
aws s3 rm s3://pacs-development-masks/test/test-file.txt

echo "CORS test completed! 🎉"
```

### 3. 보안 설정 테스트

#### `test-security.sh`
```bash
#!/bin/bash
# 보안 설정 테스트 스크립트

echo "Testing security configuration..."

# 버킷 정책 확인
echo "1. Checking bucket policy..."
aws s3api get-bucket-policy --bucket pacs-development-masks

# 암호화 설정 확인
echo "2. Checking encryption configuration..."
aws s3api get-bucket-encryption --bucket pacs-development-masks

# 퍼블릭 액세스 차단 확인
echo "3. Checking public access block..."
aws s3api get-public-access-block --bucket pacs-development-masks

# 버전 관리 확인
echo "4. Checking versioning..."
aws s3api get-bucket-versioning --bucket pacs-development-masks

echo "Security test completed! 🎉"
```

### 4. Lambda 함수 테스트

#### `test-lambda.sh`
```bash
#!/bin/bash
# Lambda 함수 테스트 스크립트

echo "Testing Lambda function..."

# Lambda 함수 호출
echo "1. Invoking Lambda function..."
aws lambda invoke \
  --function-name pacs-signed-url-generator \
  --payload '{"action": "upload", "filename": "test-mask.png"}' \
  response.json

# 응답 확인
echo "2. Checking response..."
cat response.json

# 정리
rm response.json

echo "Lambda test completed! 🎉"
```

---

## 🔧 문제 해결

### 1. 버킷 이름 충돌

**증상**: 버킷 이름이 이미 존재함
```
Error: BucketAlreadyExists: The requested bucket name is not available
```

**해결 방법**:
```hcl
# 랜덤 ID 사용
resource "random_id" "bucket_suffix" {
  byte_length = 4
}

resource "aws_s3_bucket" "pacs_masks" {
  bucket = "${var.project_name}-${var.environment}-masks-${random_id.bucket_suffix.hex}"
}
```

### 2. CORS 오류

**증상**: 브라우저에서 CORS 오류 발생
```
Access to fetch at 'https://s3.amazonaws.com/...' from origin 'http://localhost:3000' has been blocked by CORS policy
```

**해결 방법**:
```hcl
# CORS 설정 확인
resource "aws_s3_bucket_cors_configuration" "pacs_masks" {
  bucket = aws_s3_bucket.pacs_masks.id

  cors_rule {
    allowed_headers = ["*"]
    allowed_methods = ["GET", "PUT", "POST", "DELETE", "HEAD"]
    allowed_origins = ["http://localhost:3000"]  # 정확한 origin 지정
    max_age_seconds = 3000
  }
}
```

### 3. 권한 오류

**증상**: S3 액세스 권한 부족
```
Error: AccessDenied: Access Denied
```

**해결 방법**:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject"
      ],
      "Resource": "arn:aws:s3:::bucket-name/*"
    }
  ]
}
```

---

## 📚 다음 단계

이제 S3 버킷을 성공적으로 설정했으니 다음 문서들을 학습하세요:

1. **IAM 정책 및 사용자 생성** - AWS 권한 관리
2. **RDS PostgreSQL 구성** - 데이터베이스 설정
3. **VPC 및 네트워킹** - 네트워크 보안 구성

---

## 📖 참고 자료

- [AWS S3 공식 문서](https://docs.aws.amazon.com/s3/)
- [S3 버킷 정책 예제](https://docs.aws.amazon.com/s3/latest/userguide/example-bucket-policies.html)
- [CORS 설정 가이드](https://docs.aws.amazon.com/AmazonS3/latest/userguide/cors.html)

이제 PACS 프로젝트의 마스크 이미지를 안전하게 저장할 S3 버킷이 준비되었습니다! 🚀
