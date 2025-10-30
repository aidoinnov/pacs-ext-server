# ☁️ AWS Provider 설정 가이드

Terraform에서 AWS 클라우드 인프라를 관리하기 위한 AWS Provider 설정 및 사용법을 학습합니다.

## 📋 목차

1. [AWS Provider란?](#aws-provider란)
2. [설치 및 설정](#설치-및-설정)
3. [인증 방법](#인증-방법)
4. [기본 리소스 구성](#기본-리소스-구성)
5. [PACS 프로젝트 AWS 설정](#pacs-프로젝트-aws-설정)
6. [실습 및 테스트](#실습-및-테스트)

---

## 🎯 AWS Provider란?

**AWS Provider**는 Terraform에서 AWS 서비스를 관리할 수 있게 해주는 공식 플러그인입니다.

### 주요 특징
- **AWS 서비스 지원**: 200+ AWS 서비스 지원
- **자동 업데이트**: 새로운 AWS 서비스 자동 추가
- **상태 관리**: AWS 리소스 상태 추적
- **의존성 관리**: 리소스 간 의존성 자동 해결

### 지원하는 주요 서비스
- **컴퓨팅**: EC2, ECS, EKS, Lambda
- **스토리지**: S3, EBS, EFS
- **데이터베이스**: RDS, DynamoDB, ElastiCache
- **네트워킹**: VPC, ALB, NLB, CloudFront
- **보안**: IAM, KMS, Secrets Manager

---

## 🔧 설치 및 설정

### 1. 사전 요구사항

```bash
# AWS CLI 설치 확인
aws --version
# aws-cli/2.13.0 Python/3.11.0 Darwin/22.5.0

# AWS 자격 증명 확인
aws sts get-caller-identity
# {
#     "UserId": "AIDACKCEVSQ6C2EXAMPLE",
#     "Account": "123456789012",
#     "Arn": "arn:aws:iam::123456789012:user/terraform-user"
# }
```

### 2. Terraform 프로젝트 초기화

```bash
# 프로젝트 디렉토리 생성
mkdir terraform-aws-guide
cd terraform-aws-guide

# Terraform 초기화
terraform init
```

### 3. Provider 설정

#### `providers.tf`
```hcl
terraform {
  required_version = ">= 1.0"
  
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
  
  # 기본 태그 설정
  default_tags {
    tags = {
      Project     = var.project_name
      Environment = var.environment
      ManagedBy   = "terraform"
      Owner       = "platform-team"
    }
  }
}
```

### 4. Provider 다운로드

```bash
terraform init
```

**출력 예시:**
```
Initializing the backend...

Initializing provider plugins...
- Finding hashicorp/aws versions matching "~> 5.0"...
- Installing hashicorp/aws v5.0.0...
- Installed hashicorp/aws v5.0.0 (signed by HashiCorp)

Terraform has been successfully initialized!
```

---

## 🔐 인증 방법

### 1. AWS 자격 증명 설정

#### 방법 1: AWS CLI 설정
```bash
# AWS 자격 증명 설정
aws configure

# AWS Access Key ID: AKIA...
# AWS Secret Access Key: ...
# Default region name: ap-northeast-2
# Default output format: json
```

#### 방법 2: 환경변수 설정
```bash
# 환경변수로 설정
export AWS_ACCESS_KEY_ID="AKIA..."
export AWS_SECRET_ACCESS_KEY="..."
export AWS_DEFAULT_REGION="ap-northeast-2"
```

#### 방법 3: IAM 역할 사용 (EC2/ECS/EKS)
```hcl
provider "aws" {
  region = var.aws_region
  
  # IAM 역할 사용 (EC2 인스턴스에서)
  # assume_role {
  #   role_arn = "arn:aws:iam::123456789012:role/terraform-role"
  # }
}
```

### 2. IAM 권한 설정

#### 최소 권한 정책
```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "ec2:*",
                "s3:*",
                "rds:*",
                "iam:*",
                "vpc:*",
                "elasticache:*",
                "eks:*",
                "cloudwatch:*",
                "logs:*"
            ],
            "Resource": "*"
        }
    ]
}
```

#### PACS 프로젝트 전용 정책
```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "ec2:Describe*",
                "ec2:Create*",
                "ec2:Modify*",
                "ec2:Delete*",
                "s3:Get*",
                "s3:Put*",
                "s3:Delete*",
                "s3:List*",
                "rds:Describe*",
                "rds:Create*",
                "rds:Modify*",
                "rds:Delete*",
                "iam:Get*",
                "iam:Create*",
                "iam:Attach*",
                "iam:Detach*",
                "vpc:Describe*",
                "vpc:Create*",
                "vpc:Modify*",
                "vpc:Delete*"
            ],
            "Resource": "*"
        }
    ]
}
```

---

## 🏗️ 기본 리소스 구성

### 1. VPC 구성

#### `vpc.tf`
```hcl
# VPC 생성
resource "aws_vpc" "pacs_vpc" {
  cidr_block           = var.vpc_cidr
  enable_dns_hostnames = true
  enable_dns_support   = true
  
  tags = {
    Name = "${var.project_name}-vpc"
  }
}

# 인터넷 게이트웨이
resource "aws_internet_gateway" "pacs_igw" {
  vpc_id = aws_vpc.pacs_vpc.id
  
  tags = {
    Name = "${var.project_name}-igw"
  }
}

# 퍼블릭 서브넷
resource "aws_subnet" "public" {
  count = length(var.availability_zones)
  
  vpc_id                  = aws_vpc.pacs_vpc.id
  cidr_block              = var.public_subnet_cidrs[count.index]
  availability_zone       = var.availability_zones[count.index]
  map_public_ip_on_launch = true
  
  tags = {
    Name = "${var.project_name}-public-subnet-${count.index + 1}"
    Type = "Public"
  }
}

# 프라이빗 서브넷
resource "aws_subnet" "private" {
  count = length(var.availability_zones)
  
  vpc_id            = aws_vpc.pacs_vpc.id
  cidr_block        = var.private_subnet_cidrs[count.index]
  availability_zone = var.availability_zones[count.index]
  
  tags = {
    Name = "${var.project_name}-private-subnet-${count.index + 1}"
    Type = "Private"
  }
}

# NAT 게이트웨이
resource "aws_eip" "nat" {
  count = length(var.availability_zones)
  
  domain = "vpc"
  
  tags = {
    Name = "${var.project_name}-nat-eip-${count.index + 1}"
  }
}

resource "aws_nat_gateway" "pacs_nat" {
  count = length(var.availability_zones)
  
  allocation_id = aws_eip.nat[count.index].id
  subnet_id     = aws_subnet.public[count.index].id
  
  tags = {
    Name = "${var.project_name}-nat-gateway-${count.index + 1}"
  }
  
  depends_on = [aws_internet_gateway.pacs_igw]
}
```

### 2. 보안 그룹 구성

#### `security_groups.tf`
```hcl
# 웹 서버 보안 그룹
resource "aws_security_group" "web" {
  name_prefix = "${var.project_name}-web-"
  vpc_id      = aws_vpc.pacs_vpc.id
  
  # HTTP
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  
  # HTTPS
  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  
  # SSH (관리자만)
  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = [var.admin_cidr]
  }
  
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
  
  tags = {
    Name = "${var.project_name}-web-sg"
  }
}

# 애플리케이션 보안 그룹
resource "aws_security_group" "app" {
  name_prefix = "${var.project_name}-app-"
  vpc_id      = aws_vpc.pacs_vpc.id
  
  # 웹 서버에서만 접근
  ingress {
    from_port       = 8080
    to_port         = 8080
    protocol        = "tcp"
    security_groups = [aws_security_group.web.id]
  }
  
  # 데이터베이스 접근
  egress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.database.id]
  }
  
  # Redis 접근
  egress {
    from_port       = 6379
    to_port         = 6379
    protocol        = "tcp"
    security_groups = [aws_security_group.cache.id]
  }
  
  tags = {
    Name = "${var.project_name}-app-sg"
  }
}

# 데이터베이스 보안 그룹
resource "aws_security_group" "database" {
  name_prefix = "${var.project_name}-database-"
  vpc_id      = aws_vpc.pacs_vpc.id
  
  # 애플리케이션에서만 접근
  ingress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.app.id]
  }
  
  tags = {
    Name = "${var.project_name}-database-sg"
  }
}

# 캐시 보안 그룹
resource "aws_security_group" "cache" {
  name_prefix = "${var.project_name}-cache-"
  vpc_id      = aws_vpc.pacs_vpc.id
  
  # 애플리케이션에서만 접근
  ingress {
    from_port       = 6379
    to_port         = 6379
    protocol        = "tcp"
    security_groups = [aws_security_group.app.id]
  }
  
  tags = {
    Name = "${var.project_name}-cache-sg"
  }
}
```

### 3. S3 버킷 구성

#### `s3.tf`
```hcl
# S3 버킷 생성
resource "aws_s3_bucket" "pacs_data" {
  bucket = "${var.project_name}-${var.environment}-data"
  
  tags = {
    Name = "${var.project_name}-data-bucket"
  }
}

# S3 버킷 버전 관리
resource "aws_s3_bucket_versioning" "pacs_data" {
  bucket = aws_s3_bucket.pacs_data.id
  versioning_configuration {
    status = "Enabled"
  }
}

# S3 버킷 암호화
resource "aws_s3_bucket_server_side_encryption_configuration" "pacs_data" {
  bucket = aws_s3_bucket.pacs_data.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# S3 버킷 퍼블릭 액세스 차단
resource "aws_s3_bucket_public_access_block" "pacs_data" {
  bucket = aws_s3_bucket.pacs_data.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# S3 버킷 CORS 설정
resource "aws_s3_bucket_cors_configuration" "pacs_data" {
  bucket = aws_s3_bucket.pacs_data.id

  cors_rule {
    allowed_headers = ["*"]
    allowed_methods = ["GET", "PUT", "POST", "DELETE"]
    allowed_origins = var.allowed_origins
    expose_headers  = ["ETag"]
    max_age_seconds = 3000
  }
}
```

---

## 🏥 PACS 프로젝트 AWS 설정

### 1. 변수 정의

#### `variables.tf`
```hcl
# 프로젝트 설정
variable "project_name" {
  description = "Name of the project"
  type        = string
  default     = "pacs"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "development"
}

variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "ap-northeast-2"
}

# 네트워크 설정
variable "vpc_cidr" {
  description = "CIDR block for VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "availability_zones" {
  description = "Availability zones"
  type        = list(string)
  default     = ["ap-northeast-2a", "ap-northeast-2c"]
}

variable "public_subnet_cidrs" {
  description = "CIDR blocks for public subnets"
  type        = list(string)
  default     = ["10.0.1.0/24", "10.0.2.0/24"]
}

variable "private_subnet_cidrs" {
  description = "CIDR blocks for private subnets"
  type        = list(string)
  default     = ["10.0.10.0/24", "10.0.20.0/24"]
}

# 보안 설정
variable "admin_cidr" {
  description = "CIDR block for admin access"
  type        = string
  default     = "0.0.0.0/0"
}

# S3 설정
variable "allowed_origins" {
  description = "Allowed origins for CORS"
  type        = list(string)
  default     = ["http://localhost:3000", "http://localhost:8080"]
}
```

### 2. RDS 구성

#### `rds.tf`
```hcl
# RDS 서브넷 그룹
resource "aws_db_subnet_group" "pacs" {
  name       = "${var.project_name}-db-subnet-group"
  subnet_ids = aws_subnet.private[*].id
  
  tags = {
    Name = "${var.project_name}-db-subnet-group"
  }
}

# RDS 인스턴스
resource "aws_db_instance" "pacs_postgres" {
  identifier = "${var.project_name}-postgres"
  
  engine         = "postgres"
  engine_version = "16.1"
  instance_class = var.rds_instance_class
  
  allocated_storage     = var.rds_allocated_storage
  max_allocated_storage = var.rds_max_allocated_storage
  storage_type          = "gp3"
  storage_encrypted     = true
  
  db_name  = var.postgres_db
  username = var.postgres_user
  password = var.postgres_password
  
  vpc_security_group_ids = [aws_security_group.database.id]
  db_subnet_group_name   = aws_db_subnet_group.pacs.name
  
  backup_retention_period = var.backup_retention_period
  backup_window          = "03:00-04:00"
  maintenance_window     = "sun:04:00-sun:05:00"
  
  skip_final_snapshot = var.environment == "development"
  
  tags = {
    Name = "${var.project_name}-postgres"
  }
}
```

### 3. ElastiCache 구성

#### `elasticache.tf`
```hcl
# ElastiCache 서브넷 그룹
resource "aws_elasticache_subnet_group" "pacs" {
  name       = "${var.project_name}-cache-subnet-group"
  subnet_ids = aws_subnet.private[*].id
}

# ElastiCache 클러스터
resource "aws_elasticache_replication_group" "pacs_redis" {
  replication_group_id       = "${var.project_name}-redis"
  description                = "Redis cluster for PACS"
  
  node_type                  = var.redis_node_type
  port                       = 6379
  parameter_group_name       = "default.redis7"
  
  num_cache_clusters         = var.redis_num_cache_nodes
  
  subnet_group_name          = aws_elasticache_subnet_group.pacs.name
  security_group_ids         = [aws_security_group.cache.id]
  
  at_rest_encryption_enabled = true
  transit_encryption_enabled = true
  
  tags = {
    Name = "${var.project_name}-redis"
  }
}
```

---

## 🧪 실습 및 테스트

### 1. 기본 구성 테스트

#### `test-aws-setup.sh`
```bash
#!/bin/bash
# AWS 설정 테스트 스크립트

echo "Testing AWS Provider setup..."

# AWS 자격 증명 확인
echo "1. Checking AWS credentials..."
aws sts get-caller-identity

# Terraform 초기화
echo "2. Initializing Terraform..."
terraform init

# Terraform 검증
echo "3. Validating Terraform configuration..."
terraform validate

# Terraform 계획
echo "4. Planning Terraform deployment..."
terraform plan

echo "AWS Provider setup test completed! 🎉"
```

### 2. 리소스 생성 테스트

#### `test-resource-creation.sh`
```bash
#!/bin/bash
# AWS 리소스 생성 테스트 스크립트

echo "Testing AWS resource creation..."

# VPC 생성
echo "1. Creating VPC..."
terraform apply -target=aws_vpc.pacs_vpc -auto-approve

# 서브넷 생성
echo "2. Creating subnets..."
terraform apply -target=aws_subnet.public -auto-approve
terraform apply -target=aws_subnet.private -auto-approve

# 보안 그룹 생성
echo "3. Creating security groups..."
terraform apply -target=aws_security_group.web -auto-approve
terraform apply -target=aws_security_group.app -auto-approve

# S3 버킷 생성
echo "4. Creating S3 bucket..."
terraform apply -target=aws_s3_bucket.pacs_data -auto-approve

echo "Resource creation test completed! 🎉"
```

### 3. 연결 테스트

#### `test-connectivity.sh`
```bash
#!/bin/bash
# AWS 리소스 연결 테스트 스크립트

echo "Testing AWS resource connectivity..."

# VPC 상태 확인
echo "1. Checking VPC status..."
aws ec2 describe-vpcs --filters "Name=tag:Name,Values=pacs-vpc"

# 서브넷 상태 확인
echo "2. Checking subnet status..."
aws ec2 describe-subnets --filters "Name=tag:Name,Values=pacs-public-subnet-1"

# 보안 그룹 상태 확인
echo "3. Checking security group status..."
aws ec2 describe-security-groups --filters "Name=tag:Name,Values=pacs-web-sg"

# S3 버킷 상태 확인
echo "4. Checking S3 bucket status..."
aws s3 ls s3://pacs-development-data

echo "Connectivity test completed! 🎉"
```

---

## 🔧 문제 해결

### 1. 인증 오류

**증상**: AWS 자격 증명 오류
```
Error: No valid credential sources found for AWS Provider
```

**해결 방법**:
```bash
# AWS 자격 증명 확인
aws sts get-caller-identity

# AWS CLI 재설정
aws configure

# 환경변수 확인
echo $AWS_ACCESS_KEY_ID
echo $AWS_SECRET_ACCESS_KEY
```

### 2. 권한 오류

**증상**: IAM 권한 부족
```
Error: AccessDenied: User is not authorized to perform: ec2:CreateVpc
```

**해결 방법**:
```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "ec2:*",
                "s3:*",
                "rds:*",
                "iam:*"
            ],
            "Resource": "*"
        }
    ]
}
```

### 3. 리전 오류

**증상**: 리전 불일치
```
Error: InvalidParameterValue: The parameter availabilityZone is not valid
```

**해결 방법**:
```hcl
# 올바른 리전 설정
variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "ap-northeast-2"  # 서울 리전
}

# 가용 영역 확인
variable "availability_zones" {
  description = "Availability zones"
  type        = list(string)
  default     = ["ap-northeast-2a", "ap-northeast-2c"]
}
```

---

## 📚 다음 단계

이제 AWS Provider를 성공적으로 설정했으니 다음 문서들을 학습하세요:

1. **S3 버킷 생성 및 관리** - Object Storage 설정
2. **IAM 정책 및 사용자 생성** - AWS 권한 관리
3. **RDS PostgreSQL 구성** - 데이터베이스 설정

---

## 📖 참고 자료

- [AWS Provider 공식 문서](https://registry.terraform.io/providers/hashicorp/aws/latest/docs)
- [AWS 서비스 문서](https://docs.aws.amazon.com/)
- [Terraform AWS 예제](https://github.com/hashicorp/terraform-provider-aws/tree/main/examples)

이제 PACS 프로젝트를 AWS 클라우드에서 관리할 준비가 완료되었습니다! 🚀
