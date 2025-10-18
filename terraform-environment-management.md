# 🌍 환경별 설정 관리 가이드

Terraform을 사용하여 development, staging, production 환경을 분리하고 관리하는 실습 가이드입니다.

## 📋 목차

1. [환경 분리 전략](#환경-분리-전략)
2. [디렉토리 구조 설계](#디렉토리-구조-설계)
3. [환경별 설정 구현](#환경별-설정-구현)
4. [변수 관리 전략](#변수-관리-전략)
5. [상태 관리](#상태-관리)
6. [배포 자동화](#배포-자동화)
7. [실습 및 테스트](#실습-및-테스트)

---

## 🎯 환경 분리 전략

### 환경별 특성

#### Development (개발 환경)
- **목적**: 개발자 로컬 개발 및 테스트
- **특징**: 빠른 배포, 디버깅 용이, 비용 최소화
- **리소스**: 최소한의 리소스, 로컬 Docker
- **보안**: 낮은 보안 수준, 개발용 데이터

#### Staging (스테이징 환경)
- **목적**: 프로덕션 배포 전 최종 테스트
- **특징**: 프로덕션과 유사한 환경, 통합 테스트
- **리소스**: 프로덕션과 유사한 리소스
- **보안**: 중간 보안 수준, 테스트용 데이터

#### Production (프로덕션 환경)
- **목적**: 실제 서비스 운영
- **특징**: 고가용성, 확장성, 모니터링
- **리소스**: 최적화된 리소스, 클라우드 인프라
- **보안**: 최고 보안 수준, 실제 데이터

### 환경 분리 원칙

```
┌─────────────────────────────────────────────────────────────┐
│                    Environment Strategy                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Development          Staging            Production        │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐   │
│  │   Local     │     │   Cloud     │     │   Cloud     │   │
│  │   Docker    │     │   AWS       │     │   AWS       │   │
│  │             │     │   ECS/EKS   │     │   EKS       │   │
│  │  - MinIO    │     │             │     │             │   │
│  │  - Local DB │     │  - RDS      │     │  - RDS      │   │
│  │  - Dev Data │     │  - S3       │     │  - S3       │   │
│  │             │     │  - Test Data│     │  - Prod Data│   │
│  └─────────────┘     └─────────────┘     └─────────────┘   │
│                                                             │
│  Fast Deploy        Integration Test    High Availability   │
│  Low Cost          Pre-prod Testing    Monitoring & Alert  │
│  Easy Debug        Similar to Prod     Backup & Recovery   │
└─────────────────────────────────────────────────────────────┘
```

---

## 🏗️ 디렉토리 구조 설계

### 권장 디렉토리 구조

```
terraform/
├── modules/                    # 재사용 가능한 모듈
│   ├── postgres/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   └── versions.tf
│   ├── redis/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   ├── s3/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   └── monitoring/
│       ├── main.tf
│       ├── variables.tf
│       └── outputs.tf
├── environments/               # 환경별 설정
│   ├── development/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   ├── terraform.tfvars
│   │   └── versions.tf
│   ├── staging/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   ├── terraform.tfvars
│   │   └── versions.tf
│   └── production/
│       ├── main.tf
│       ├── variables.tf
│       ├── outputs.tf
│       ├── terraform.tfvars
│       └── versions.tf
├── shared/                     # 공통 리소스
│   ├── iam/
│   │   ├── main.tf
│   │   └── variables.tf
│   ├── vpc/
│   │   ├── main.tf
│   │   └── variables.tf
│   └── s3-backend/
│       ├── main.tf
│       └── variables.tf
├── scripts/                    # 배포 스크립트
│   ├── deploy.sh
│   ├── destroy.sh
│   └── validate.sh
└── docs/                       # 문서
    ├── README.md
    └── deployment-guide.md
```

---

## 🌍 환경별 설정 구현

### 1. Development 환경

#### `environments/development/main.tf`
```hcl
# Development 환경 메인 설정
terraform {
  required_version = ">= 1.0"
  
  required_providers {
    docker = {
      source  = "kreuzwerker/docker"
      version = "~> 3.0"
    }
  }
  
  # 로컬 상태 저장 (개발 환경)
  backend "local" {
    path = "terraform.tfstate"
  }
}

provider "docker" {
  host = "unix:///var/run/docker.sock"
}

# 로컬 변수
locals {
  environment = "development"
  project_name = "pacs"
  
  # 개발 환경 특화 설정
  common_tags = {
    Environment = local.environment
    Project     = local.project_name
    ManagedBy   = "terraform"
    Owner       = "development-team"
  }
}

# PostgreSQL 모듈
module "postgres" {
  source = "../../modules/postgres"
  
  # 개발 환경 설정
  environment = local.environment
  project_name = local.project_name
  
  # 데이터베이스 설정
  postgres_user     = var.postgres_user
  postgres_password = var.postgres_password
  postgres_db       = "${var.postgres_db}_dev"
  postgres_port     = var.postgres_port
  
  # 개발 환경 특화 설정
  container_restart_policy = "unless-stopped"
  healthcheck_interval    = "30s"  # 느린 헬스체크
  healthcheck_timeout     = "10s"
  healthcheck_retries     = 3
  healthcheck_start_period = "60s"
  
  # 개발용 볼륨 설정
  volume_driver = "local"
  
  tags = local.common_tags
}

# Redis 모듈
module "redis" {
  source = "../../modules/redis"
  
  environment = local.environment
  project_name = local.project_name
  
  redis_port = var.redis_port
  redis_password = var.redis_password
  
  # 개발 환경 특화 설정
  container_restart_policy = "unless-stopped"
  maxmemory = "256mb"  # 작은 메모리
  
  tags = local.common_tags
}

# MinIO 모듈 (개발용 Object Storage)
module "minio" {
  source = "../../modules/minio"
  
  environment = local.environment
  project_name = local.project_name
  
  minio_port = var.minio_port
  minio_console_port = var.minio_console_port
  minio_access_key = var.minio_access_key
  minio_secret_key = var.minio_secret_key
  
  # 개발 환경 특화 설정
  container_restart_policy = "unless-stopped"
  
  tags = local.common_tags
}

# PACS 애플리케이션
module "pacs_server" {
  source = "../../modules/pacs-server"
  
  environment = local.environment
  project_name = local.project_name
  
  # 애플리케이션 설정
  server_port = var.server_port
  log_level = "debug"  # 개발용 디버그 로그
  
  # 데이터베이스 연결
  database_url = "postgresql://${var.postgres_user}:${var.postgres_password}@${module.postgres.host}:${module.postgres.port}/${module.postgres.database}"
  redis_url = "redis://${module.redis.host}:${module.redis.port}"
  
  # Object Storage 설정
  object_storage_provider = "minio"
  object_storage_endpoint = "http://${module.minio.host}:${module.minio.port}"
  object_storage_access_key = var.minio_access_key
  object_storage_secret_key = var.minio_secret_key
  
  # 개발 환경 특화 설정
  container_restart_policy = "unless-stopped"
  enable_debug = true
  enable_hot_reload = true
  
  depends_on = [
    module.postgres,
    module.redis,
    module.minio
  ]
  
  tags = local.common_tags
}
```

#### `environments/development/variables.tf`
```hcl
# 개발 환경 변수
variable "postgres_user" {
  description = "PostgreSQL username for development"
  type        = string
  default     = "admin"
}

variable "postgres_password" {
  description = "PostgreSQL password for development"
  type        = string
  sensitive   = true
  default     = "admin123"
}

variable "postgres_db" {
  description = "PostgreSQL database name for development"
  type        = string
  default     = "pacs_db"
}

variable "postgres_port" {
  description = "PostgreSQL port for development"
  type        = number
  default     = 5432
}

variable "redis_port" {
  description = "Redis port for development"
  type        = number
  default     = 6379
}

variable "redis_password" {
  description = "Redis password for development"
  type        = string
  sensitive   = true
  default     = "redis123"
}

variable "minio_port" {
  description = "MinIO port for development"
  type        = number
  default     = 9000
}

variable "minio_console_port" {
  description = "MinIO console port for development"
  type        = number
  default     = 9001
}

variable "minio_access_key" {
  description = "MinIO access key for development"
  type        = string
  default     = "minioadmin"
}

variable "minio_secret_key" {
  description = "MinIO secret key for development"
  type        = string
  sensitive   = true
  default     = "minioadmin"
}

variable "server_port" {
  description = "PACS server port for development"
  type        = number
  default     = 8080
}
```

#### `environments/development/terraform.tfvars`
```hcl
# Development 환경 설정값
postgres_user     = "admin"
postgres_password = "admin123"
postgres_db       = "pacs_db"
postgres_port     = 5432

redis_port     = 6379
redis_password = "redis123"

minio_port         = 9000
minio_console_port = 9001
minio_access_key   = "minioadmin"
minio_secret_key   = "minioadmin"

server_port = 8080
```

### 2. Staging 환경

#### `environments/staging/main.tf`
```hcl
# Staging 환경 메인 설정
terraform {
  required_version = ">= 1.0"
  
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
  
  # S3 백엔드 사용
  backend "s3" {
    bucket         = "pacs-terraform-state-staging"
    key            = "staging/terraform.tfstate"
    region         = "ap-northeast-2"
    encrypt        = true
    dynamodb_table = "pacs-terraform-locks-staging"
  }
}

provider "aws" {
  region = var.aws_region
  
  default_tags {
    tags = {
      Environment = "staging"
      Project     = "pacs"
      ManagedBy   = "terraform"
      Owner       = "platform-team"
    }
  }
}

# 로컬 변수
locals {
  environment = "staging"
  project_name = "pacs"
  
  common_tags = {
    Environment = local.environment
    Project     = local.project_name
    ManagedBy   = "terraform"
    Owner       = "platform-team"
  }
}

# VPC 모듈
module "vpc" {
  source = "../../modules/vpc"
  
  environment = local.environment
  project_name = local.project_name
  
  vpc_cidr = var.vpc_cidr
  availability_zones = var.availability_zones
  
  tags = local.common_tags
}

# RDS 모듈
module "postgres" {
  source = "../../modules/rds"
  
  environment = local.environment
  project_name = local.project_name
  
  # RDS 설정
  instance_class = var.rds_instance_class
  allocated_storage = var.rds_allocated_storage
  max_allocated_storage = var.rds_max_allocated_storage
  
  # 데이터베이스 설정
  postgres_user     = var.postgres_user
  postgres_password = var.postgres_password
  postgres_db       = "${var.postgres_db}_staging"
  
  # 네트워크 설정
  vpc_id = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnet_ids
  security_group_ids = [module.vpc.database_security_group_id]
  
  # 백업 설정
  backup_retention_period = 7
  backup_window = "03:00-04:00"
  maintenance_window = "sun:04:00-sun:05:00"
  
  # 모니터링 설정
  monitoring_interval = 60
  monitoring_role_arn = module.iam.rds_monitoring_role_arn
  
  tags = local.common_tags
}

# ElastiCache 모듈
module "redis" {
  source = "../../modules/elasticache"
  
  environment = local.environment
  project_name = local.project_name
  
  # Redis 설정
  node_type = var.redis_node_type
  num_cache_nodes = var.redis_num_cache_nodes
  
  # 네트워크 설정
  vpc_id = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnet_ids
  security_group_ids = [module.vpc.cache_security_group_id]
  
  tags = local.common_tags
}

# S3 모듈
module "s3" {
  source = "../../modules/s3"
  
  environment = local.environment
  project_name = local.project_name
  
  # S3 설정
  bucket_name = "${var.project_name}-${local.environment}-data"
  
  # 버전 관리
  versioning_enabled = true
  
  # 암호화
  encryption_enabled = true
  kms_key_id = module.kms.s3_key_id
  
  # 수명 주기 정책
  lifecycle_rules = [
    {
      id = "staging_lifecycle"
      enabled = true
      transitions = [
        {
          days = 30
          storage_class = "STANDARD_IA"
        },
        {
          days = 90
          storage_class = "GLACIER"
        }
      ]
    }
  ]
  
  tags = local.common_tags
}

# ECS 모듈
module "ecs" {
  source = "../../modules/ecs"
  
  environment = local.environment
  project_name = local.project_name
  
  # ECS 설정
  cluster_name = "${var.project_name}-${local.environment}"
  service_name = "${var.project_name}-server"
  
  # 태스크 정의
  cpu = var.ecs_cpu
  memory = var.ecs_memory
  
  # 네트워크 설정
  vpc_id = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnet_ids
  security_group_ids = [module.vpc.application_security_group_id]
  
  # 로드 밸런서 설정
  load_balancer_arn = module.alb.arn
  target_group_arn = module.alb.target_group_arn
  
  # 환경변수
  environment_variables = {
    DATABASE_URL = "postgresql://${var.postgres_user}:${var.postgres_password}@${module.postgres.endpoint}:5432/${module.postgres.database}"
    REDIS_URL = "redis://${module.redis.endpoint}:6379"
    S3_BUCKET = module.s3.bucket_name
    S3_REGION = var.aws_region
    LOG_LEVEL = "info"
  }
  
  tags = local.common_tags
}
```

### 3. Production 환경

#### `environments/production/main.tf`
```hcl
# Production 환경 메인 설정
terraform {
  required_version = ">= 1.0"
  
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
  
  # S3 백엔드 사용
  backend "s3" {
    bucket         = "pacs-terraform-state-production"
    key            = "production/terraform.tfstate"
    region         = "ap-northeast-2"
    encrypt        = true
    dynamodb_table = "pacs-terraform-locks-production"
  }
}

provider "aws" {
  region = var.aws_region
  
  default_tags {
    tags = {
      Environment = "production"
      Project     = "pacs"
      ManagedBy   = "terraform"
      Owner       = "platform-team"
      CostCenter  = "engineering"
    }
  }
}

# 로컬 변수
locals {
  environment = "production"
  project_name = "pacs"
  
  common_tags = {
    Environment = local.environment
    Project     = local.project_name
    ManagedBy   = "terraform"
    Owner       = "platform-team"
    CostCenter  = "engineering"
  }
}

# VPC 모듈 (고가용성)
module "vpc" {
  source = "../../modules/vpc"
  
  environment = local.environment
  project_name = local.project_name
  
  vpc_cidr = var.vpc_cidr
  availability_zones = var.availability_zones
  
  # 프로덕션 특화 설정
  enable_nat_gateway = true
  enable_vpn_gateway = true
  enable_dns_hostnames = true
  enable_dns_support = true
  
  tags = local.common_tags
}

# RDS 모듈 (고가용성)
module "postgres" {
  source = "../../modules/rds"
  
  environment = local.environment
  project_name = local.project_name
  
  # 프로덕션 RDS 설정
  instance_class = var.rds_instance_class
  allocated_storage = var.rds_allocated_storage
  max_allocated_storage = var.rds_max_allocated_storage
  
  # Multi-AZ 설정
  multi_az = true
  
  # 데이터베이스 설정
  postgres_user     = var.postgres_user
  postgres_password = var.postgres_password
  postgres_db       = "${var.postgres_db}_prod"
  
  # 네트워크 설정
  vpc_id = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnet_ids
  security_group_ids = [module.vpc.database_security_group_id]
  
  # 백업 설정
  backup_retention_period = 30
  backup_window = "03:00-04:00"
  maintenance_window = "sun:04:00-sun:05:00"
  
  # 모니터링 설정
  monitoring_interval = 60
  monitoring_role_arn = module.iam.rds_monitoring_role_arn
  
  # 성능 인사이트
  performance_insights_enabled = true
  performance_insights_retention_period = 7
  
  tags = local.common_tags
}

# ElastiCache 모듈 (클러스터 모드)
module "redis" {
  source = "../../modules/elasticache"
  
  environment = local.environment
  project_name = local.project_name
  
  # 프로덕션 Redis 설정
  node_type = var.redis_node_type
  num_cache_nodes = var.redis_num_cache_nodes
  
  # 클러스터 모드
  cluster_mode_enabled = true
  num_node_groups = 2
  replicas_per_node_group = 1
  
  # 네트워크 설정
  vpc_id = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnet_ids
  security_group_ids = [module.vpc.cache_security_group_id]
  
  # 백업 설정
  snapshot_retention_limit = 7
  snapshot_window = "03:00-05:00"
  
  tags = local.common_tags
}

# S3 모듈 (프로덕션)
module "s3" {
  source = "../../modules/s3"
  
  environment = local.environment
  project_name = local.project_name
  
  # S3 설정
  bucket_name = "${var.project_name}-${local.environment}-data"
  
  # 버전 관리
  versioning_enabled = true
  
  # 암호화
  encryption_enabled = true
  kms_key_id = module.kms.s3_key_id
  
  # 수명 주기 정책
  lifecycle_rules = [
    {
      id = "production_lifecycle"
      enabled = true
      transitions = [
        {
          days = 30
          storage_class = "STANDARD_IA"
        },
        {
          days = 90
          storage_class = "GLACIER"
        },
        {
          days = 365
          storage_class = "DEEP_ARCHIVE"
        }
      ]
    }
  ]
  
  # 교차 리전 복제
  replication_configuration = {
    role = module.iam.s3_replication_role_arn
    rules = [
      {
        id = "replication_to_backup_region"
        status = "Enabled"
        destination = {
          bucket = "arn:aws:s3:::${var.project_name}-${local.environment}-data-backup"
          storage_class = "STANDARD"
        }
      }
    ]
  }
  
  tags = local.common_tags
}

# EKS 모듈 (프로덕션)
module "eks" {
  source = "../../modules/eks"
  
  environment = local.environment
  project_name = local.project_name
  
  # EKS 설정
  cluster_name = "${var.project_name}-${local.environment}"
  cluster_version = var.eks_cluster_version
  
  # 노드 그룹 설정
  node_groups = {
    main = {
      instance_types = var.eks_node_instance_types
      min_size = var.eks_node_min_size
      max_size = var.eks_node_max_size
      desired_size = var.eks_node_desired_size
      
      # 스팟 인스턴스 사용
      capacity_type = "SPOT"
      
      # 업데이트 설정
      update_config = {
        max_unavailable_percentage = 25
      }
    }
  }
  
  # 네트워크 설정
  vpc_id = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnet_ids
  
  # 로그 설정
  cluster_enabled_log_types = [
    "api",
    "audit",
    "authenticator",
    "controllerManager",
    "scheduler"
  ]
  
  tags = local.common_tags
}

# 모니터링 모듈
module "monitoring" {
  source = "../../modules/monitoring"
  
  environment = local.environment
  project_name = local.project_name
  
  # CloudWatch 설정
  log_retention_days = 30
  
  # 알람 설정
  alarms = {
    high_cpu = {
      metric_name = "CPUUtilization"
      threshold = 80
      comparison_operator = "GreaterThanThreshold"
    }
    high_memory = {
      metric_name = "MemoryUtilization"
      threshold = 85
      comparison_operator = "GreaterThanThreshold"
    }
    database_connections = {
      metric_name = "DatabaseConnections"
      threshold = 80
      comparison_operator = "GreaterThanThreshold"
    }
  }
  
  # SNS 알림
  notification_topic_arn = module.sns.topic_arn
  
  tags = local.common_tags
}
```

---

## 🔧 변수 관리 전략

### 1. 공통 변수 파일

#### `variables.tf` (공통)
```hcl
# 공통 변수
variable "environment" {
  description = "Environment name"
  type        = string
  
  validation {
    condition     = contains(["development", "staging", "production"], var.environment)
    error_message = "Environment must be one of: development, staging, production."
  }
}

variable "project_name" {
  description = "Project name"
  type        = string
  default     = "pacs"
}

variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "ap-northeast-2"
}

# 네트워크 변수
variable "vpc_cidr" {
  description = "VPC CIDR block"
  type        = string
  default     = "10.0.0.0/16"
}

variable "availability_zones" {
  description = "Availability zones"
  type        = list(string)
  default     = ["ap-northeast-2a", "ap-northeast-2c"]
}

# 데이터베이스 변수
variable "postgres_user" {
  description = "PostgreSQL username"
  type        = string
  sensitive   = true
}

variable "postgres_password" {
  description = "PostgreSQL password"
  type        = string
  sensitive   = true
}

variable "postgres_db" {
  description = "PostgreSQL database name"
  type        = string
  default     = "pacs_db"
}

# RDS 변수
variable "rds_instance_class" {
  description = "RDS instance class"
  type        = string
  default     = "db.t3.micro"
}

variable "rds_allocated_storage" {
  description = "RDS allocated storage"
  type        = number
  default     = 20
}

variable "rds_max_allocated_storage" {
  description = "RDS max allocated storage"
  type        = number
  default     = 100
}

# Redis 변수
variable "redis_node_type" {
  description = "Redis node type"
  type        = string
  default     = "cache.t3.micro"
}

variable "redis_num_cache_nodes" {
  description = "Number of Redis cache nodes"
  type        = number
  default     = 1
}

# EKS 변수
variable "eks_cluster_version" {
  description = "EKS cluster version"
  type        = string
  default     = "1.28"
}

variable "eks_node_instance_types" {
  description = "EKS node instance types"
  type        = list(string)
  default     = ["t3.medium"]
}

variable "eks_node_min_size" {
  description = "EKS node group minimum size"
  type        = number
  default     = 1
}

variable "eks_node_max_size" {
  description = "EKS node group maximum size"
  type        = number
  default     = 10
}

variable "eks_node_desired_size" {
  description = "EKS node group desired size"
  type        = number
  default     = 3
}
```

### 2. 환경별 변수 파일

#### `environments/development/terraform.tfvars`
```hcl
# Development 환경 설정
environment = "development"
project_name = "pacs"
aws_region = "ap-northeast-2"

# 네트워크 설정
vpc_cidr = "10.0.0.0/16"
availability_zones = ["ap-northeast-2a"]

# 데이터베이스 설정
postgres_user = "admin"
postgres_password = "admin123"
postgres_db = "pacs_db"

# RDS 설정 (개발용 작은 인스턴스)
rds_instance_class = "db.t3.micro"
rds_allocated_storage = 20
rds_max_allocated_storage = 50

# Redis 설정
redis_node_type = "cache.t3.micro"
redis_num_cache_nodes = 1

# EKS 설정 (최소 리소스)
eks_cluster_version = "1.28"
eks_node_instance_types = ["t3.small"]
eks_node_min_size = 1
eks_node_max_size = 3
eks_node_desired_size = 1
```

#### `environments/staging/terraform.tfvars`
```hcl
# Staging 환경 설정
environment = "staging"
project_name = "pacs"
aws_region = "ap-northeast-2"

# 네트워크 설정
vpc_cidr = "10.1.0.0/16"
availability_zones = ["ap-northeast-2a", "ap-northeast-2c"]

# 데이터베이스 설정
postgres_user = "pacs_staging"
postgres_password = "staging_password_123"
postgres_db = "pacs_db"

# RDS 설정 (스테이징용 중간 인스턴스)
rds_instance_class = "db.t3.small"
rds_allocated_storage = 50
rds_max_allocated_storage = 200

# Redis 설정
redis_node_type = "cache.t3.small"
redis_num_cache_nodes = 1

# EKS 설정 (스테이징용)
eks_cluster_version = "1.28"
eks_node_instance_types = ["t3.medium"]
eks_node_min_size = 2
eks_node_max_size = 5
eks_node_desired_size = 2
```

#### `environments/production/terraform.tfvars`
```hcl
# Production 환경 설정
environment = "production"
project_name = "pacs"
aws_region = "ap-northeast-2"

# 네트워크 설정
vpc_cidr = "10.2.0.0/16"
availability_zones = ["ap-northeast-2a", "ap-northeast-2c", "ap-northeast-2d"]

# 데이터베이스 설정
postgres_user = "pacs_production"
postgres_password = "production_password_456"
postgres_db = "pacs_db"

# RDS 설정 (프로덕션용 큰 인스턴스)
rds_instance_class = "db.r5.large"
rds_allocated_storage = 100
rds_max_allocated_storage = 1000

# Redis 설정
redis_node_type = "cache.r5.large"
redis_num_cache_nodes = 2

# EKS 설정 (프로덕션용)
eks_cluster_version = "1.28"
eks_node_instance_types = ["r5.large", "r5.xlarge"]
eks_node_min_size = 3
eks_node_max_size = 20
eks_node_desired_size = 5
```

---

## 📊 상태 관리

### 1. S3 백엔드 설정

#### `shared/s3-backend/main.tf`
```hcl
# S3 백엔드 버킷 생성
resource "aws_s3_bucket" "terraform_state" {
  bucket = "pacs-terraform-state-${var.environment}"
  
  tags = {
    Name        = "Terraform State Bucket"
    Environment = var.environment
    Project     = "pacs"
    ManagedBy   = "terraform"
  }
}

# S3 버킷 버전 관리
resource "aws_s3_bucket_versioning" "terraform_state" {
  bucket = aws_s3_bucket.terraform_state.id
  versioning_configuration {
    status = "Enabled"
  }
}

# S3 버킷 암호화
resource "aws_s3_bucket_server_side_encryption_configuration" "terraform_state" {
  bucket = aws_s3_bucket.terraform_state.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# S3 버킷 퍼블릭 액세스 차단
resource "aws_s3_bucket_public_access_block" "terraform_state" {
  bucket = aws_s3_bucket.terraform_state.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# DynamoDB 테이블 (상태 잠금)
resource "aws_dynamodb_table" "terraform_locks" {
  name           = "pacs-terraform-locks-${var.environment}"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "LockID"

  attribute {
    name = "LockID"
    type = "S"
  }

  tags = {
    Name        = "Terraform State Lock Table"
    Environment = var.environment
    Project     = "pacs"
    ManagedBy   = "terraform"
  }
}
```

### 2. 상태 마이그레이션

#### `scripts/migrate-state.sh`
```bash
#!/bin/bash
# 상태 마이그레이션 스크립트

set -e

ENVIRONMENT=$1
if [ -z "$ENVIRONMENT" ]; then
    echo "Usage: $0 <environment>"
    echo "Environments: development, staging, production"
    exit 1
fi

echo "Migrating Terraform state for $ENVIRONMENT environment..."

cd "environments/$ENVIRONMENT"

# 백엔드 초기화
terraform init -backend-config="bucket=pacs-terraform-state-$ENVIRONMENT" \
               -backend-config="key=$ENVIRONMENT/terraform.tfstate" \
               -backend-config="region=ap-northeast-2" \
               -backend-config="dynamodb_table=pacs-terraform-locks-$ENVIRONMENT"

# 상태 마이그레이션 확인
terraform plan

echo "State migration completed for $ENVIRONMENT environment!"
```

---

## 🚀 배포 자동화

### 1. 배포 스크립트

#### `scripts/deploy.sh`
```bash
#!/bin/bash
# 환경별 배포 스크립트

set -e

ENVIRONMENT=$1
ACTION=$2

if [ -z "$ENVIRONMENT" ] || [ -z "$ACTION" ]; then
    echo "Usage: $0 <environment> <action>"
    echo "Environments: development, staging, production"
    echo "Actions: plan, apply, destroy"
    exit 1
fi

echo "Executing $ACTION for $ENVIRONMENT environment..."

cd "environments/$ENVIRONMENT"

# 환경별 설정 로드
source "../../scripts/load-env-$ENVIRONMENT.sh"

# Terraform 초기화
terraform init

# 선택된 액션 실행
case $ACTION in
    "plan")
        echo "Planning $ENVIRONMENT environment..."
        terraform plan -var-file="terraform.tfvars"
        ;;
    "apply")
        echo "Applying $ENVIRONMENT environment..."
        terraform apply -var-file="terraform.tfvars" -auto-approve
        ;;
    "destroy")
        echo "Destroying $ENVIRONMENT environment..."
        terraform destroy -var-file="terraform.tfvars" -auto-approve
        ;;
    *)
        echo "Unknown action: $ACTION"
        exit 1
        ;;
esac

echo "$ACTION completed for $ENVIRONMENT environment!"
```

### 2. 환경별 설정 로드

#### `scripts/load-env-development.sh`
```bash
#!/bin/bash
# Development 환경 설정

export TF_VAR_environment="development"
export TF_VAR_project_name="pacs"
export TF_VAR_aws_region="ap-northeast-2"

# 개발 환경 특화 설정
export TF_VAR_postgres_user="admin"
export TF_VAR_postgres_password="admin123"
export TF_VAR_postgres_db="pacs_db"

# 로컬 Docker 사용
export DOCKER_HOST="unix:///var/run/docker.sock"
```

#### `scripts/load-env-staging.sh`
```bash
#!/bin/bash
# Staging 환경 설정

export TF_VAR_environment="staging"
export TF_VAR_project_name="pacs"
export TF_VAR_aws_region="ap-northeast-2"

# 스테이징 환경 설정
export TF_VAR_postgres_user="pacs_staging"
export TF_VAR_postgres_password="staging_password_123"
export TF_VAR_postgres_db="pacs_db"

# AWS 자격 증명
export AWS_PROFILE="pacs-staging"
```

#### `scripts/load-env-production.sh`
```bash
#!/bin/bash
# Production 환경 설정

export TF_VAR_environment="production"
export TF_VAR_project_name="pacs"
export TF_VAR_aws_region="ap-northeast-2"

# 프로덕션 환경 설정
export TF_VAR_postgres_user="pacs_production"
export TF_VAR_postgres_password="production_password_456"
export TF_VAR_postgres_db="pacs_db"

# AWS 자격 증명
export AWS_PROFILE="pacs-production"

# 프로덕션 보안 설정
export TF_LOG="INFO"
export TF_LOG_PATH="terraform.log"
```

### 3. GitHub Actions 워크플로우

#### `.github/workflows/terraform.yml`
```yaml
name: 'Terraform'

on:
  push:
    branches: [ main, develop ]
    paths: [ 'terraform/**' ]
  pull_request:
    branches: [ main ]
    paths: [ 'terraform/**' ]

env:
  TF_VERSION: '1.6.0'
  AWS_REGION: 'ap-northeast-2'

jobs:
  terraform:
    name: 'Terraform'
    runs-on: ubuntu-latest
    
    strategy:
      matrix:
        environment: [development, staging, production]
    
    steps:
    - name: Checkout
      uses: actions/checkout@v4
      
    - name: Setup Terraform
      uses: hashicorp/setup-terraform@v3
      with:
        terraform_version: ${{ env.TF_VERSION }}
        
    - name: Terraform Format
      run: terraform fmt -check -recursive
      working-directory: ./terraform
      
    - name: Terraform Init
      run: terraform init
      working-directory: ./terraform/environments/${{ matrix.environment }}
      
    - name: Terraform Validate
      run: terraform validate
      working-directory: ./terraform/environments/${{ matrix.environment }}
      
    - name: Terraform Plan
      run: terraform plan
      working-directory: ./terraform/environments/${{ matrix.environment }}
      env:
        TF_VAR_environment: ${{ matrix.environment }}
        
    - name: Terraform Apply (Development)
      if: matrix.environment == 'development' && github.ref == 'refs/heads/develop'
      run: terraform apply -auto-approve
      working-directory: ./terraform/environments/${{ matrix.environment }}
      env:
        TF_VAR_environment: ${{ matrix.environment }}
        
    - name: Terraform Apply (Staging)
      if: matrix.environment == 'staging' && github.ref == 'refs/heads/main'
      run: terraform apply -auto-approve
      working-directory: ./terraform/environments/${{ matrix.environment }}
      env:
        TF_VAR_environment: ${{ matrix.environment }}
        
    - name: Terraform Apply (Production)
      if: matrix.environment == 'production' && github.ref == 'refs/heads/main'
      run: terraform apply -auto-approve
      working-directory: ./terraform/environments/${{ matrix.environment }}
      env:
        TF_VAR_environment: ${{ matrix.environment }}
```

---

## 🧪 실습 및 테스트

### 1. 환경별 배포 테스트

#### `test-deployment.sh`
```bash
#!/bin/bash
# 환경별 배포 테스트 스크립트

set -e

echo "Testing environment-specific deployments..."

# Development 환경 테스트
echo "1. Testing Development environment..."
cd environments/development
terraform init
terraform plan -var-file="terraform.tfvars"
echo "✅ Development environment plan successful"

# Staging 환경 테스트
echo "2. Testing Staging environment..."
cd ../staging
terraform init
terraform plan -var-file="terraform.tfvars"
echo "✅ Staging environment plan successful"

# Production 환경 테스트
echo "3. Testing Production environment..."
cd ../production
terraform init
terraform plan -var-file="terraform.tfvars"
echo "✅ Production environment plan successful"

echo "All environment tests passed! 🎉"
```

### 2. 변수 검증 테스트

#### `test-variables.sh`
```bash
#!/bin/bash
# 변수 검증 테스트 스크립트

echo "Testing variable validation..."

# Development 환경 변수 검증
echo "1. Validating Development variables..."
cd environments/development
terraform validate
echo "✅ Development variables valid"

# Staging 환경 변수 검증
echo "2. Validating Staging variables..."
cd ../staging
terraform validate
echo "✅ Staging variables valid"

# Production 환경 변수 검증
echo "3. Validating Production variables..."
cd ../production
terraform validate
echo "✅ Production variables valid"

echo "All variable validations passed! 🎉"
```

### 3. 상태 동기화 테스트

#### `test-state-sync.sh`
```bash
#!/bin/bash
# 상태 동기화 테스트 스크립트

echo "Testing state synchronization..."

# 각 환경의 상태 확인
for env in development staging production; do
    echo "Checking $env environment state..."
    cd "environments/$env"
    
    # 상태 목록 확인
    terraform state list
    
    # 상태 동기화
    terraform refresh
    
    echo "✅ $env environment state synchronized"
    cd ../..
done

echo "All state synchronizations completed! 🎉"
```

---

## 📚 다음 단계

이제 환경별 설정 관리를 완전히 마스터했으니 다음 문서들을 학습하세요:

1. **AWS Provider 설정 가이드** - AWS 클라우드 인프라 시작
2. **S3 버킷 생성 및 관리** - Object Storage 설정
3. **IAM 정책 및 사용자 생성** - AWS 권한 관리

---

## 📖 참고 자료

- [Terraform 환경 관리 모범 사례](https://developer.hashicorp.com/terraform/language/state/workspaces)
- [AWS Provider 문서](https://registry.terraform.io/providers/hashicorp/aws/latest/docs)
- [Terraform 백엔드 설정](https://developer.hashicorp.com/terraform/language/settings/backends)

이제 PACS 프로젝트를 여러 환경에서 안전하고 효율적으로 관리할 수 있게 되었습니다! 🚀


