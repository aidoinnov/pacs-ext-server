# 🐘 RDS PostgreSQL 구성 가이드

Terraform을 사용하여 AWS RDS PostgreSQL 데이터베이스를 구성하고 관리하는 방법을 학습합니다. PACS 프로젝트의 DICOM 메타데이터 저장을 위한 고가용성 데이터베이스 설정을 중심으로 다룹니다.

## 📋 목차

1. [RDS PostgreSQL이란?](#rds-postgresql이란)
2. [기본 RDS 구성](#기본-rds-구성)
3. [PACS 프로젝트 RDS 설정](#pacs-프로젝트-rds-설정)
4. [고가용성 및 백업](#고가용성-및-백업)
5. [보안 및 모니터링](#보안-및-모니터링)
6. [실습 및 테스트](#실습-및-테스트)

---

## 🎯 RDS PostgreSQL이란?

**Amazon RDS (Relational Database Service)**는 AWS의 관리형 관계형 데이터베이스 서비스입니다.

### 주요 특징
- **관리형 서비스**: 패치, 백업, 모니터링 자동화
- **고가용성**: Multi-AZ 배포로 99.95% 가용성
- **자동 백업**: Point-in-time 복구 지원
- **스케일링**: 읽기 전용 복제본으로 성능 확장
- **보안**: 암호화, VPC 격리, IAM 통합

### PACS 프로젝트에서의 활용
- **DICOM 메타데이터**: 환자 정보, 스터디 데이터 저장
- **사용자 관리**: Keycloak 연동 사용자 정보
- **마스크 정보**: 어노테이션 및 마스크 메타데이터
- **감사 로그**: 데이터 접근 및 변경 이력

---

## 🔧 기본 RDS 구성

### 1. 서브넷 그룹 생성

#### `rds-subnet-group.tf`
```hcl
# RDS 서브넷 그룹
resource "aws_db_subnet_group" "main" {
  name       = "${var.project_name}-db-subnet-group"
  subnet_ids = var.private_subnet_ids

  tags = {
    Name        = "PACS Database Subnet Group"
    Environment = var.environment
    Project     = var.project_name
  }
}

# 가용 영역별 서브넷 확인
data "aws_subnets" "private" {
  filter {
    name   = "vpc-id"
    values = [var.vpc_id]
  }
  
  filter {
    name   = "tag:Type"
    values = ["Private"]
  }
}
```

### 2. 보안 그룹 설정

#### `rds-security-group.tf`
```hcl
# RDS 보안 그룹
resource "aws_security_group" "rds" {
  name_prefix = "${var.project_name}-rds-"
  vpc_id      = var.vpc_id

  # PostgreSQL 포트 (5432)
  ingress {
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = var.allowed_cidr_blocks
  }

  # 애플리케이션 서버에서만 접근
  ingress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [var.app_security_group_id]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name        = "PACS RDS Security Group"
    Environment = var.environment
    Project     = var.project_name
  }
}
```

### 3. 기본 RDS 인스턴스

#### `rds-basic.tf`
```hcl
# RDS 인스턴스
resource "aws_db_instance" "main" {
  identifier = "${var.project_name}-postgres"

  # 엔진 설정
  engine         = "postgres"
  engine_version = var.postgres_version
  instance_class = var.instance_class

  # 스토리지 설정
  allocated_storage     = var.allocated_storage
  max_allocated_storage = var.max_allocated_storage
  storage_type          = "gp3"
  storage_encrypted     = true

  # 데이터베이스 설정
  db_name  = var.database_name
  username = var.master_username
  password = var.master_password

  # 네트워크 설정
  db_subnet_group_name   = aws_db_subnet_group.main.name
  vpc_security_group_ids = [aws_security_group.rds.id]

  # 백업 설정
  backup_retention_period = var.backup_retention_period
  backup_window          = var.backup_window
  maintenance_window     = var.maintenance_window

  # 기타 설정
  skip_final_snapshot = var.environment == "development"
  deletion_protection = var.environment == "production"

  tags = {
    Name        = "PACS PostgreSQL Database"
    Environment = var.environment
    Project     = var.project_name
  }
}
```

---

## 🏥 PACS 프로젝트 RDS 설정

### 1. 환경별 변수 설정

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

# 네트워크 설정
variable "vpc_id" {
  description = "VPC ID for RDS"
  type        = string
}

variable "private_subnet_ids" {
  description = "Private subnet IDs for RDS"
  type        = list(string)
}

variable "app_security_group_id" {
  description = "Application security group ID"
  type        = string
}

variable "allowed_cidr_blocks" {
  description = "CIDR blocks allowed to access RDS"
  type        = list(string)
  default     = ["10.0.0.0/16"]
}

# 데이터베이스 설정
variable "postgres_version" {
  description = "PostgreSQL version"
  type        = string
  default     = "16.1"
}

variable "instance_class" {
  description = "RDS instance class"
  type        = string
  default     = "db.t3.micro"
}

variable "allocated_storage" {
  description = "Initial allocated storage in GB"
  type        = number
  default     = 20
}

variable "max_allocated_storage" {
  description = "Maximum allocated storage in GB"
  type        = number
  default     = 100
}

variable "database_name" {
  description = "Name of the database"
  type        = string
  default     = "pacs_db"
}

variable "master_username" {
  description = "Master username"
  type        = string
  default     = "postgres"
}

variable "master_password" {
  description = "Master password"
  type        = string
  sensitive   = true
}

# 백업 설정
variable "backup_retention_period" {
  description = "Backup retention period in days"
  type        = number
  default     = 7
}

variable "backup_window" {
  description = "Backup window"
  type        = string
  default     = "03:00-04:00"
}

variable "maintenance_window" {
  description = "Maintenance window"
  type        = string
  default     = "sun:04:00-sun:05:00"
}
```

### 2. 환경별 설정 파일

#### `environments/development.tfvars`
```hcl
# Development Environment
environment = "development"
instance_class = "db.t3.micro"
allocated_storage = 20
max_allocated_storage = 50
backup_retention_period = 1
```

#### `environments/staging.tfvars`
```hcl
# Staging Environment
environment = "staging"
instance_class = "db.t3.small"
allocated_storage = 50
max_allocated_storage = 200
backup_retention_period = 7
```

#### `environments/production.tfvars`
```hcl
# Production Environment
environment = "production"
instance_class = "db.r5.large"
allocated_storage = 100
max_allocated_storage = 1000
backup_retention_period = 30
```

### 3. PACS 전용 데이터베이스 설정

#### `pacs-database.tf`
```hcl
# PACS 메인 데이터베이스
resource "aws_db_instance" "pacs_main" {
  identifier = "${var.project_name}-${var.environment}-main"

  # 엔진 설정
  engine         = "postgres"
  engine_version = var.postgres_version
  instance_class = var.instance_class

  # 스토리지 설정
  allocated_storage     = var.allocated_storage
  max_allocated_storage = var.max_allocated_storage
  storage_type          = "gp3"
  storage_encrypted     = true

  # 데이터베이스 설정
  db_name  = "pacs_main"
  username = "pacs_admin"
  password = var.pacs_admin_password

  # 네트워크 설정
  db_subnet_group_name   = aws_db_subnet_group.main.name
  vpc_security_group_ids = [aws_security_group.rds.id]

  # 백업 설정
  backup_retention_period = var.backup_retention_period
  backup_window          = var.backup_window
  maintenance_window     = var.maintenance_window

  # 고가용성 설정
  multi_az = var.environment == "production"

  # 성능 설정
  performance_insights_enabled = true
  monitoring_interval         = 60
  monitoring_role_arn        = aws_iam_role.rds_enhanced_monitoring.arn

  # 기타 설정
  skip_final_snapshot = var.environment == "development"
  deletion_protection = var.environment == "production"

  tags = {
    Name        = "PACS Main Database"
    Environment = var.environment
    Project     = var.project_name
    Purpose     = "DICOM Metadata Storage"
  }
}

# Keycloak 데이터베이스
resource "aws_db_instance" "keycloak" {
  identifier = "${var.project_name}-${var.environment}-keycloak"

  # 엔진 설정
  engine         = "postgres"
  engine_version = var.postgres_version
  instance_class = var.instance_class

  # 스토리지 설정
  allocated_storage     = var.allocated_storage
  max_allocated_storage = var.max_allocated_storage
  storage_type          = "gp3"
  storage_encrypted     = true

  # 데이터베이스 설정
  db_name  = "keycloak"
  username = "keycloak_admin"
  password = var.keycloak_admin_password

  # 네트워크 설정
  db_subnet_group_name   = aws_db_subnet_group.main.name
  vpc_security_group_ids = [aws_security_group.rds.id]

  # 백업 설정
  backup_retention_period = var.backup_retention_period
  backup_window          = var.backup_window
  maintenance_window     = var.maintenance_window

  # 고가용성 설정
  multi_az = var.environment == "production"

  # 기타 설정
  skip_final_snapshot = var.environment == "development"
  deletion_protection = var.environment == "production"

  tags = {
    Name        = "Keycloak Database"
    Environment = var.environment
    Project     = var.project_name
    Purpose     = "User Authentication"
  }
}
```

---

## 🔄 고가용성 및 백업

### 1. Multi-AZ 설정

#### `rds-high-availability.tf`
```hcl
# 읽기 전용 복제본
resource "aws_db_instance" "pacs_read_replica" {
  count = var.environment == "production" ? 1 : 0

  identifier = "${var.project_name}-${var.environment}-read-replica"

  # 복제본 설정
  replicate_source_db = aws_db_instance.pacs_main.identifier
  instance_class      = var.read_replica_instance_class

  # 스토리지 설정
  storage_type      = "gp3"
  storage_encrypted = true

  # 네트워크 설정
  vpc_security_group_ids = [aws_security_group.rds.id]

  # 성능 설정
  performance_insights_enabled = true
  monitoring_interval         = 60
  monitoring_role_arn        = aws_iam_role.rds_enhanced_monitoring.arn

  # 기타 설정
  skip_final_snapshot = true
  deletion_protection = false

  tags = {
    Name        = "PACS Read Replica"
    Environment = var.environment
    Project     = var.project_name
    Purpose     = "Read-Only Replica"
  }
}

# 글로벌 데이터베이스 (선택사항)
resource "aws_rds_global_cluster" "pacs_global" {
  count = var.enable_global_database ? 1 : 0

  global_cluster_identifier = "${var.project_name}-global"
  engine                    = "aurora-postgresql"
  engine_version            = "15.4"
  database_name             = "pacs_global"
  deletion_protection       = false
}
```

### 2. 백업 및 복구 설정

#### `rds-backup.tf`
```hcl
# RDS 스냅샷 스케줄
resource "aws_db_snapshot" "pacs_manual" {
  count = var.environment == "production" ? 1 : 0

  db_instance_identifier = aws_db_instance.pacs_main.identifier
  db_snapshot_identifier = "${var.project_name}-manual-snapshot-${formatdate("YYYY-MM-DD-hhmm", timestamp())}"

  tags = {
    Name        = "PACS Manual Snapshot"
    Environment = var.environment
    Project     = var.project_name
    Type        = "Manual"
  }
}

# 자동 백업 설정
resource "aws_db_instance" "pacs_with_backup" {
  count = var.enable_automated_backups ? 1 : 0

  identifier = "${var.project_name}-${var.environment}-backup"

  # 기본 설정 (위의 pacs_main과 동일)
  engine         = "postgres"
  engine_version = var.postgres_version
  instance_class = var.instance_class

  # 백업 설정
  backup_retention_period = var.backup_retention_period
  backup_window          = var.backup_window
  maintenance_window     = var.maintenance_window

  # 복구 설정
  restore_to_point_in_time {
    source_db_instance_identifier = aws_db_instance.pacs_main.identifier
    use_latest_restorable_time    = true
  }

  tags = {
    Name        = "PACS Backup Instance"
    Environment = var.environment
    Project     = var.project_name
    Purpose     = "Backup and Recovery"
  }
}
```

### 3. 모니터링 및 알림

#### `rds-monitoring.tf`
```hcl
# CloudWatch 로그 그룹
resource "aws_cloudwatch_log_group" "postgresql" {
  name              = "/aws/rds/instance/${aws_db_instance.pacs_main.identifier}/postgresql"
  retention_in_days = var.log_retention_days

  tags = {
    Name        = "PostgreSQL Logs"
    Environment = var.environment
    Project     = var.project_name
  }
}

# RDS 로그 설정
resource "aws_db_parameter_group" "pacs_postgres" {
  family = "postgres16"
  name   = "${var.project_name}-postgres-params"

  parameter {
    name  = "log_statement"
    value = "all"
  }

  parameter {
    name  = "log_min_duration_statement"
    value = "1000"
  }

  parameter {
    name  = "log_connections"
    value = "1"
  }

  parameter {
    name  = "log_disconnections"
    value = "1"
  }

  tags = {
    Name        = "PACS PostgreSQL Parameters"
    Environment = var.environment
    Project     = var.project_name
  }
}

# CloudWatch 알람
resource "aws_cloudwatch_metric_alarm" "database_cpu" {
  alarm_name          = "${var.project_name}-database-cpu"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "CPUUtilization"
  namespace           = "AWS/RDS"
  period              = "300"
  statistic           = "Average"
  threshold           = "80"
  alarm_description   = "This metric monitors database cpu utilization"

  dimensions = {
    DBInstanceIdentifier = aws_db_instance.pacs_main.identifier
  }

  tags = {
    Name        = "Database CPU Alarm"
    Environment = var.environment
    Project     = var.project_name
  }
}

resource "aws_cloudwatch_metric_alarm" "database_connections" {
  alarm_name          = "${var.project_name}-database-connections"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "DatabaseConnections"
  namespace           = "AWS/RDS"
  period              = "300"
  statistic           = "Average"
  threshold           = "80"
  alarm_description   = "This metric monitors database connections"

  dimensions = {
    DBInstanceIdentifier = aws_db_instance.pacs_main.identifier
  }

  tags = {
    Name        = "Database Connections Alarm"
    Environment = var.environment
    Project     = var.project_name
  }
}
```

---

## 🔒 보안 및 모니터링

### 1. 암호화 설정

#### `rds-security.tf`
```hcl
# KMS 키 생성
resource "aws_kms_key" "rds" {
  description             = "KMS key for RDS encryption"
  deletion_window_in_days = 7

  tags = {
    Name        = "PACS RDS KMS Key"
    Environment = var.environment
    Project     = var.project_name
  }
}

resource "aws_kms_alias" "rds" {
  name          = "alias/pacs-rds"
  target_key_id = aws_kms_key.rds.key_id
}

# RDS 암호화 설정
resource "aws_db_instance" "pacs_encrypted" {
  identifier = "${var.project_name}-${var.environment}-encrypted"

  # 기본 설정
  engine         = "postgres"
  engine_version = var.postgres_version
  instance_class = var.instance_class

  # 암호화 설정
  storage_encrypted   = true
  kms_key_id         = aws_kms_key.rds.arn
  storage_type       = "gp3"

  # 네트워크 설정
  db_subnet_group_name   = aws_db_subnet_group.main.name
  vpc_security_group_ids = [aws_security_group.rds.id]

  # 기타 설정
  skip_final_snapshot = var.environment == "development"
  deletion_protection = var.environment == "production"

  tags = {
    Name        = "PACS Encrypted Database"
    Environment = var.environment
    Project     = var.project_name
    Encrypted   = "true"
  }
}
```

### 2. IAM 역할 및 정책

#### `rds-iam.tf`
```hcl
# RDS Enhanced Monitoring 역할
resource "aws_iam_role" "rds_enhanced_monitoring" {
  name = "${var.project_name}-rds-enhanced-monitoring"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "monitoring.rds.amazonaws.com"
        }
      }
    ]
  })

  tags = {
    Name        = "RDS Enhanced Monitoring Role"
    Environment = var.environment
    Project     = var.project_name
  }
}

# RDS Enhanced Monitoring 정책
resource "aws_iam_role_policy_attachment" "rds_enhanced_monitoring" {
  role       = aws_iam_role.rds_enhanced_monitoring.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonRDSEnhancedMonitoringRole"
}

# RDS 로그 내보내기 역할
resource "aws_iam_role" "rds_log_export" {
  name = "${var.project_name}-rds-log-export"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "rds.amazonaws.com"
        }
      }
    ]
  })

  tags = {
    Name        = "RDS Log Export Role"
    Environment = var.environment
    Project     = var.project_name
  }
}

# RDS 로그 내보내기 정책
resource "aws_iam_policy" "rds_log_export" {
  name        = "${var.project_name}-rds-log-export"
  description = "Policy for RDS log export to CloudWatch"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents",
          "logs:DescribeLogStreams"
        ]
        Resource = "arn:aws:logs:*:*:log-group:/aws/rds/instance/*"
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "rds_log_export" {
  role       = aws_iam_role.rds_log_export.name
  policy_arn = aws_iam_policy.rds_log_export.arn
}
```

---

## 🧪 실습 및 테스트

### 1. RDS 생성 테스트

#### `test-rds-creation.sh`
```bash
#!/bin/bash
# RDS 생성 테스트 스크립트

echo "Testing RDS creation..."

# Terraform 초기화
echo "1. Initializing Terraform..."
terraform init

# Terraform 검증
echo "2. Validating configuration..."
terraform validate

# RDS 서브넷 그룹 생성
echo "3. Creating RDS subnet group..."
terraform apply -target=aws_db_subnet_group.main -auto-approve

# RDS 보안 그룹 생성
echo "4. Creating RDS security group..."
terraform apply -target=aws_security_group.rds -auto-approve

# RDS 인스턴스 생성
echo "5. Creating RDS instance..."
terraform apply -target=aws_db_instance.pacs_main -auto-approve

# RDS 확인
echo "6. Verifying RDS creation..."
aws rds describe-db-instances --db-instance-identifier pacs-development-main

echo "RDS creation test completed! 🎉"
```

### 2. 데이터베이스 연결 테스트

#### `test-database-connection.sh`
```bash
#!/bin/bash
# 데이터베이스 연결 테스트 스크립트

echo "Testing database connection..."

# RDS 엔드포인트 가져오기
RDS_ENDPOINT=$(aws rds describe-db-instances \
  --db-instance-identifier pacs-development-main \
  --query 'DBInstances[0].Endpoint.Address' \
  --output text)

echo "RDS Endpoint: $RDS_ENDPOINT"

# PostgreSQL 클라이언트 설치 확인
if ! command -v psql &> /dev/null; then
    echo "Installing PostgreSQL client..."
    # macOS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install postgresql
    # Ubuntu/Debian
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        sudo apt-get update
        sudo apt-get install -y postgresql-client
    fi
fi

# 데이터베이스 연결 테스트
echo "Testing connection to $RDS_ENDPOINT..."
PGPASSWORD=$MASTER_PASSWORD psql \
  -h $RDS_ENDPOINT \
  -U pacs_admin \
  -d pacs_main \
  -c "SELECT version();"

echo "Database connection test completed! 🎉"
```

### 3. 백업 및 복구 테스트

#### `test-backup-restore.sh`
```bash
#!/bin/bash
# 백업 및 복구 테스트 스크립트

echo "Testing backup and restore..."

# 수동 스냅샷 생성
echo "1. Creating manual snapshot..."
aws rds create-db-snapshot \
  --db-instance-identifier pacs-development-main \
  --db-snapshot-identifier pacs-test-snapshot-$(date +%Y%m%d%H%M%S)

# 스냅샷 상태 확인
echo "2. Checking snapshot status..."
aws rds describe-db-snapshots \
  --db-snapshot-identifier pacs-test-snapshot-$(date +%Y%m%d%H%M%S)

# 자동 백업 확인
echo "3. Checking automated backups..."
aws rds describe-db-instances \
  --db-instance-identifier pacs-development-main \
  --query 'DBInstances[0].BackupRetentionPeriod'

echo "Backup and restore test completed! 🎉"
```

### 4. 모니터링 테스트

#### `test-monitoring.sh`
```bash
#!/bin/bash
# 모니터링 테스트 스크립트

echo "Testing monitoring configuration..."

# CloudWatch 메트릭 확인
echo "1. Checking CloudWatch metrics..."
aws cloudwatch list-metrics \
  --namespace "AWS/RDS" \
  --metric-name "CPUUtilization"

# 알람 상태 확인
echo "2. Checking CloudWatch alarms..."
aws cloudwatch describe-alarms \
  --alarm-names "pacs-database-cpu"

# 로그 그룹 확인
echo "3. Checking log groups..."
aws logs describe-log-groups \
  --log-group-name-prefix "/aws/rds/instance/pacs"

echo "Monitoring test completed! 🎉"
```

---

## 🔧 문제 해결

### 1. 서브넷 그룹 오류

**증상**: 서브넷 그룹 생성 실패
```
Error: InvalidParameterValue: The specified subnet group does not exist
```

**해결 방법**:
```hcl
# 서브넷 ID 확인
data "aws_subnets" "private" {
  filter {
    name   = "vpc-id"
    values = [var.vpc_id]
  }
  
  filter {
    name   = "tag:Type"
    values = ["Private"]
  }
}

# 서브넷 그룹 생성
resource "aws_db_subnet_group" "main" {
  name       = "${var.project_name}-db-subnet-group"
  subnet_ids = data.aws_subnets.private.ids
}
```

### 2. 보안 그룹 오류

**증상**: RDS 연결 실패
```
Error: connection refused
```

**해결 방법**:
```hcl
# 보안 그룹 규칙 확인
resource "aws_security_group" "rds" {
  name_prefix = "${var.project_name}-rds-"
  vpc_id      = var.vpc_id

  # PostgreSQL 포트 허용
  ingress {
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/16"]  # VPC CIDR
  }
}
```

### 3. 암호화 오류

**증상**: 암호화 설정 실패
```
Error: InvalidParameterValue: The specified KMS key does not exist
```

**해결 방법**:
```hcl
# KMS 키 생성
resource "aws_kms_key" "rds" {
  description = "KMS key for RDS encryption"
}

# RDS에 KMS 키 적용
resource "aws_db_instance" "main" {
  # ... 기타 설정 ...
  
  storage_encrypted = true
  kms_key_id       = aws_kms_key.rds.arn
}
```

---

## 📚 다음 단계

이제 RDS PostgreSQL을 성공적으로 설정했으니 다음 문서들을 학습하세요:

1. **VPC 및 네트워킹** - 네트워크 보안 구성
2. **EKS 클러스터 구성** - 컨테이너 오케스트레이션
3. **Application Load Balancer** - 로드 밸런싱 설정

---

## 📖 참고 자료

- [AWS RDS 공식 문서](https://docs.aws.amazon.com/rds/)
- [PostgreSQL on RDS 가이드](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/CHAP_PostgreSQL.html)
- [RDS 백업 및 복구](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/BackupRestore.html)

이제 PACS 프로젝트의 DICOM 메타데이터를 안전하게 저장할 PostgreSQL 데이터베이스가 준비되었습니다! 🚀
