# 🔒 보안 및 컴플라이언스 가이드

Terraform을 사용하여 AWS 보안 서비스와 컴플라이언스 요구사항을 구성하고 관리하는 방법을 학습합니다. PACS 프로젝트의 의료 데이터 보안과 HIPAA 컴플라이언스를 위한 보안 설정을 중심으로 다룹니다.

## 📋 목차

1. [보안 및 컴플라이언스란?](#보안-및-컴플라이언스란)
2. [AWS 보안 서비스 기본 설정](#aws-보안-서비스-기본-설정)
3. [PACS 프로젝트 보안 구성](#pacs-프로젝트-보안-구성)
4. [HIPAA 컴플라이언스 설정](#hipaa-컴플라이언스-설정)
5. [고급 보안 기능](#고급-보안-기능)
6. [실습 및 테스트](#실습-및-테스트)

---

## 🎯 보안 및 컴플라이언스란?

**보안 및 컴플라이언스**는 시스템과 데이터를 보호하고 규제 요구사항을 준수하는 프로세스입니다.

### 주요 특징
- **데이터 보호**: 민감한 의료 데이터 암호화 및 접근 제어
- **규제 준수**: HIPAA, GDPR 등 의료 데이터 보호 규정 준수
- **위협 감지**: 보안 위협 실시간 모니터링 및 대응
- **감사 추적**: 보안 이벤트 로깅 및 감사 로그 관리
- **접근 제어**: 최소 권한 원칙 기반 사용자 및 서비스 접근 관리

### PACS 프로젝트에서의 활용
- **의료 데이터 보호**: DICOM 이미지 및 환자 정보 암호화
- **HIPAA 컴플라이언스**: 의료 데이터 보호 규정 준수
- **접근 제어**: 의료진 및 시스템 관리자 권한 관리
- **감사 로깅**: 모든 데이터 접근 및 변경 이력 추적
- **위협 대응**: 보안 침해 시 자동 대응 및 알림

---

## 🔧 AWS 보안 서비스 기본 설정

### 1. AWS Config 설정

#### `aws-config.tf`
```hcl
# AWS Config 설정
resource "aws_config_configuration_recorder" "main" {
  name     = "pacs-config-recorder"
  role_arn = aws_iam_role.config_role.arn

  recording_group {
    all_supported                 = true
    include_global_resource_types = true
  }

  depends_on = [aws_config_delivery_channel.main]
}

# Config 배송 채널
resource "aws_config_delivery_channel" "main" {
  name           = "pacs-config-delivery-channel"
  s3_bucket_name = aws_s3_bucket.config_bucket.bucket
  s3_key_prefix  = "config"
}

# Config S3 버킷
resource "aws_s3_bucket" "config_bucket" {
  bucket        = "pacs-config-bucket-${random_id.bucket_suffix.hex}"
  force_destroy = true

  tags = {
    Name        = "pacs-config-bucket"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Config S3 버킷 버전 관리
resource "aws_s3_bucket_versioning" "config_bucket" {
  bucket = aws_s3_bucket.config_bucket.id
  versioning_configuration {
    status = "Enabled"
  }
}

# Config S3 버킷 암호화
resource "aws_s3_bucket_server_side_encryption_configuration" "config_bucket" {
  bucket = aws_s3_bucket.config_bucket.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# Config S3 버킷 퍼블릭 액세스 차단
resource "aws_s3_bucket_public_access_block" "config_bucket" {
  bucket = aws_s3_bucket.config_bucket.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# Config IAM 역할
resource "aws_iam_role" "config_role" {
  name = "pacs-config-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "config.amazonaws.com"
        }
      }
    ]
  })
}

# Config IAM 정책
resource "aws_iam_role_policy_attachment" "config_role" {
  role       = aws_iam_role.config_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/ConfigRole"
}

# Config IAM 정책 (S3 액세스)
resource "aws_iam_policy" "config_s3_policy" {
  name        = "pacs-config-s3-policy"
  description = "Policy for Config to access S3"

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
        Resource = "${aws_s3_bucket.config_bucket.arn}/*"
      },
      {
        Effect = "Allow"
        Action = [
          "s3:ListBucket"
        ]
        Resource = aws_s3_bucket.config_bucket.arn
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "config_s3_policy" {
  role       = aws_iam_role.config_role.name
  policy_arn = aws_iam_policy.config_s3_policy.arn
}

# 랜덤 ID 생성
resource "random_id" "bucket_suffix" {
  byte_length = 4
}
```

### 2. AWS GuardDuty 설정

#### `guardduty.tf`
```hcl
# GuardDuty 감지기
resource "aws_guardduty_detector" "main" {
  enable = true

  datasources {
    s3_logs {
      enable = true
    }
    kubernetes {
      audit_logs {
        enable = true
      }
    }
    malware_protection {
      scan_ec2_instance_with_findings {
        ebs_volumes {
          enable = true
        }
      }
    }
  }

  tags = {
    Name        = "pacs-guardduty"
    Environment = var.environment
    Project     = var.project_name
  }
}

# GuardDuty 멤버 계정 (다중 계정 환경)
resource "aws_guardduty_member" "member" {
  count = length(var.member_account_ids)

  account_id                 = var.member_account_ids[count.index]
  detector_id                = aws_guardduty_detector.main.id
  email                      = var.member_emails[count.index]
  invite                     = true
  invitation_message         = "PACS 프로젝트 GuardDuty 멤버로 초대합니다."
  disable_email_notification = false
}

# GuardDuty 초대 수락
resource "aws_guardduty_invite_accepter" "member" {
  count = length(var.member_account_ids)

  detector_id       = aws_guardduty_detector.main.id
  master_account_id = var.master_account_id
}
```

### 3. AWS Security Hub 설정

#### `security-hub.tf`
```hcl
# Security Hub 활성화
resource "aws_securityhub_account" "main" {
  enable_default_standards = true
}

# Security Hub 표준 구독
resource "aws_securityhub_standards_subscription" "cis" {
  standards_arn = "arn:aws:securityhub:${var.aws_region}::standards/cis-aws-foundations-benchmark/v/1.2.0"
  depends_on    = [aws_securityhub_account.main]
}

resource "aws_securityhub_standards_subscription" "pci" {
  standards_arn = "arn:aws:securityhub:${var.aws_region}::standards/pci-dss/v/3.2.1"
  depends_on    = [aws_securityhub_account.main]
}

# Security Hub 자동 활성화
resource "aws_securityhub_standards_subscription" "aws_foundational" {
  standards_arn = "arn:aws:securityhub:${var.aws_region}::standards/aws-foundational-security-best-practices/v/1.0.0"
  depends_on    = [aws_securityhub_account.main]
}

# Security Hub 멤버 계정
resource "aws_securityhub_member" "member" {
  count = length(var.member_account_ids)

  account_id = var.member_account_ids[count.index]
  email      = var.member_emails[count.index]
  invite     = true
  depends_on = [aws_securityhub_account.main]
}
```

---

## 🏥 PACS 프로젝트 보안 구성

### 1. 데이터 암호화 설정

#### `encryption.tf`
```hcl
# KMS 키 생성
resource "aws_kms_key" "pacs_data" {
  description             = "PACS 데이터 암호화 키"
  deletion_window_in_days = 30
  enable_key_rotation     = true

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "Enable IAM User Permissions"
        Effect = "Allow"
        Principal = {
          AWS = "arn:aws:iam::${var.aws_account_id}:root"
        }
        Action   = "kms:*"
        Resource = "*"
      },
      {
        Sid    = "Allow CloudWatch Logs"
        Effect = "Allow"
        Principal = {
          Service = "logs.${var.aws_region}.amazonaws.com"
        }
        Action = [
          "kms:Encrypt",
          "kms:Decrypt",
          "kms:ReEncrypt*",
          "kms:GenerateDataKey*"
        ]
        Resource = "*"
        Condition = {
          ArnEquals = {
            "kms:EncryptionContext:aws:logs:arn" = "arn:aws:logs:${var.aws_region}:${var.aws_account_id}:log-group:/aws/ecs/pacs-*"
          }
        }
      }
    ]
  })

  tags = {
    Name        = "pacs-data-key"
    Environment = var.environment
    Project     = var.project_name
  }
}

# KMS 키 별칭
resource "aws_kms_alias" "pacs_data" {
  name          = "alias/pacs-data"
  target_key_id = aws_kms_key.pacs_data.key_id
}

# RDS 암호화 설정
resource "aws_db_instance" "pacs_postgresql" {
  # ... 기존 설정 ...
  
  storage_encrypted = true
  kms_key_id        = aws_kms_key.pacs_data.arn
  
  # ... 기타 설정 ...
}

# S3 버킷 암호화 설정
resource "aws_s3_bucket_server_side_encryption_configuration" "pacs_storage" {
  bucket = aws_s3_bucket.pacs_storage.id

  rule {
    apply_server_side_encryption_by_default {
      kms_master_key_id = aws_kms_key.pacs_data.arn
      sse_algorithm     = "aws:kms"
    }
    bucket_key_enabled = true
  }
}

# EBS 볼륨 암호화 설정
resource "aws_launch_template" "pacs_backend" {
  # ... 기존 설정 ...
  
  block_device_mappings {
    device_name = "/dev/xvda"
    ebs {
      volume_size           = var.volume_size
      volume_type           = "gp3"
      delete_on_termination = true
      encrypted             = true
      kms_key_id           = aws_kms_key.pacs_data.arn
    }
  }
  
  # ... 기타 설정 ...
}
```

### 2. 네트워크 보안 설정

#### `network-security.tf`
```hcl
# VPC Flow Logs
resource "aws_flow_log" "vpc" {
  iam_role_arn    = aws_iam_role.flow_logs.arn
  log_destination = aws_cloudwatch_log_group.vpc_flow.arn
  traffic_type    = "ALL"
  vpc_id          = aws_vpc.main.id
}

# Flow Logs IAM 역할
resource "aws_iam_role" "flow_logs" {
  name = "pacs-flow-logs-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "vpc-flow-logs.amazonaws.com"
        }
      }
    ]
  })
}

# Flow Logs IAM 정책
resource "aws_iam_role_policy_attachment" "flow_logs" {
  role       = aws_iam_role.flow_logs.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/VPCFlowLogsDeliveryRolePolicy"
}

# WAF Web ACL
resource "aws_wafv2_web_acl" "pacs_web_acl" {
  name  = "pacs-web-acl"
  scope = "REGIONAL"

  default_action {
    allow {}
  }

  # SQL Injection 보호
  rule {
    name     = "SQLInjectionRule"
    priority = 1

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesSQLiRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "SQLInjectionRule"
      sampled_requests_enabled   = true
    }
  }

  # XSS 보호
  rule {
    name     = "XSSRule"
    priority = 2

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesCommonRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "XSSRule"
      sampled_requests_enabled   = true
    }
  }

  # Rate Limiting
  rule {
    name     = "RateLimitRule"
    priority = 3

    action {
      block {}
    }

    statement {
      rate_based_statement {
        limit              = 2000
        aggregate_key_type = "IP"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "RateLimitRule"
      sampled_requests_enabled   = true
    }
  }

  # IP 화이트리스트
  rule {
    name     = "IPWhitelistRule"
    priority = 4

    action {
      allow {}
    }

    statement {
      ip_set_reference_statement {
        arn = aws_wafv2_ip_set.whitelist.arn
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "IPWhitelistRule"
      sampled_requests_enabled   = true
    }
  }

  tags = {
    Name        = "pacs-web-acl"
    Environment = var.environment
    Project     = var.project_name
  }
}

# IP 화이트리스트 세트
resource "aws_wafv2_ip_set" "whitelist" {
  name               = "pacs-whitelist"
  scope              = "REGIONAL"
  ip_address_version = "IPV4"
  addresses          = var.whitelist_ips

  tags = {
    Name        = "pacs-whitelist"
    Environment = var.environment
    Project     = var.project_name
  }
}

# WAF 연결
resource "aws_wafv2_web_acl_association" "alb" {
  resource_arn = aws_lb.main.arn
  web_acl_arn  = aws_wafv2_web_acl.pacs_web_acl.arn
}
```

### 3. 접근 제어 및 권한 관리

#### `access-control.tf`
```hcl
# IAM 사용자 그룹
resource "aws_iam_group" "pacs_administrators" {
  name = "pacs-administrators"
  path = "/pacs/"
}

resource "aws_iam_group" "pacs_developers" {
  name = "pacs-developers"
  path = "/pacs/"
}

resource "aws_iam_group" "pacs_operators" {
  name = "pacs-operators"
  path = "/pacs/"
}

# IAM 정책
resource "aws_iam_policy" "pacs_administrator_policy" {
  name        = "pacs-administrator-policy"
  description = "Policy for PACS administrators"
  path        = "/pacs/"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:*",
          "rds:*",
          "ecs:*",
          "ec2:*",
          "iam:*",
          "kms:*",
          "cloudwatch:*",
          "logs:*"
        ]
        Resource = "*"
      }
    ]
  })
}

resource "aws_iam_policy" "pacs_developer_policy" {
  name        = "pacs-developer-policy"
  description = "Policy for PACS developers"
  path        = "/pacs/"

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
          aws_s3_bucket.pacs_storage.arn,
          "${aws_s3_bucket.pacs_storage.arn}/*"
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "rds:DescribeDBInstances",
          "rds:DescribeDBClusters"
        ]
        Resource = "*"
      },
      {
        Effect = "Allow"
        Action = [
          "ecs:DescribeServices",
          "ecs:DescribeTasks",
          "ecs:ListTasks"
        ]
        Resource = "*"
      }
    ]
  })
}

resource "aws_iam_policy" "pacs_operator_policy" {
  name        = "pacs-operator-policy"
  description = "Policy for PACS operators"
  path        = "/pacs/"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:ListBucket"
        ]
        Resource = [
          aws_s3_bucket.pacs_storage.arn,
          "${aws_s3_bucket.pacs_storage.arn}/*"
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "cloudwatch:GetMetricStatistics",
          "cloudwatch:ListMetrics",
          "logs:DescribeLogGroups",
          "logs:DescribeLogStreams",
          "logs:GetLogEvents"
        ]
        Resource = "*"
      }
    ]
  })
}

# 그룹 정책 연결
resource "aws_iam_group_policy_attachment" "administrators" {
  group      = aws_iam_group.pacs_administrators.name
  policy_arn = aws_iam_policy.pacs_administrator_policy.arn
}

resource "aws_iam_group_policy_attachment" "developers" {
  group      = aws_iam_group.pacs_developers.name
  policy_arn = aws_iam_policy.pacs_developer_policy.arn
}

resource "aws_iam_group_policy_attachment" "operators" {
  group      = aws_iam_group.pacs_operators.name
  policy_arn = aws_iam_policy.pacs_operator_policy.arn
}

# MFA 정책
resource "aws_iam_policy" "mfa_required" {
  name        = "pacs-mfa-required"
  description = "Policy requiring MFA for sensitive operations"
  path        = "/pacs/"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Deny"
        Action = [
          "s3:DeleteObject",
          "s3:DeleteBucket",
          "rds:DeleteDBInstance",
          "rds:DeleteDBCluster",
          "iam:DeleteUser",
          "iam:DeleteRole",
          "kms:DeleteKey"
        ]
        Resource = "*"
        Condition = {
          BoolIfExists = {
            "aws:MultiFactorAuthPresent" = "false"
          }
        }
      }
    ]
  })
}

# 모든 그룹에 MFA 정책 연결
resource "aws_iam_group_policy_attachment" "mfa_administrators" {
  group      = aws_iam_group.pacs_administrators.name
  policy_arn = aws_iam_policy.mfa_required.arn
}

resource "aws_iam_group_policy_attachment" "mfa_developers" {
  group      = aws_iam_group.pacs_developers.name
  policy_arn = aws_iam_policy.mfa_required.arn
}

resource "aws_iam_group_policy_attachment" "mfa_operators" {
  group      = aws_iam_group.pacs_operators.name
  policy_arn = aws_iam_policy.mfa_required.arn
}
```

---

## 🏥 HIPAA 컴플라이언스 설정

### 1. HIPAA 요구사항 준수

#### `hipaa-compliance.tf`
```hcl
# CloudTrail 설정 (감사 로깅)
resource "aws_cloudtrail" "pacs_audit" {
  name                          = "pacs-audit-trail"
  s3_bucket_name                = aws_s3_bucket.cloudtrail_bucket.bucket
  include_global_service_events = true
  is_multi_region_trail         = true
  enable_logging                = true
  enable_log_file_validation    = true

  event_selector {
    read_write_type                 = "All"
    include_management_events       = true
    data_resource {
      type   = "AWS::S3::Object"
      values = ["${aws_s3_bucket.pacs_storage.arn}/*"]
    }
  }

  event_selector {
    read_write_type                 = "All"
    include_management_events       = true
    data_resource {
      type   = "AWS::RDS::DBInstance"
      values = [aws_db_instance.pacs_postgresql.arn]
    }
  }

  tags = {
    Name        = "pacs-audit-trail"
    Environment = var.environment
    Project     = var.project_name
    Compliance  = "HIPAA"
  }
}

# CloudTrail S3 버킷
resource "aws_s3_bucket" "cloudtrail_bucket" {
  bucket        = "pacs-cloudtrail-bucket-${random_id.bucket_suffix.hex}"
  force_destroy = true

  tags = {
    Name        = "pacs-cloudtrail-bucket"
    Environment = var.environment
    Project     = var.project_name
    Compliance  = "HIPAA"
  }
}

# CloudTrail S3 버킷 정책
resource "aws_s3_bucket_policy" "cloudtrail_bucket" {
  bucket = aws_s3_bucket.cloudtrail_bucket.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AWSCloudTrailAclCheck"
        Effect = "Allow"
        Principal = {
          Service = "cloudtrail.amazonaws.com"
        }
        Action   = "s3:GetBucketAcl"
        Resource = aws_s3_bucket.cloudtrail_bucket.arn
      },
      {
        Sid    = "AWSCloudTrailWrite"
        Effect = "Allow"
        Principal = {
          Service = "cloudtrail.amazonaws.com"
        }
        Action = "s3:PutObject"
        Resource = "${aws_s3_bucket.cloudtrail_bucket.arn}/*"
        Condition = {
          StringEquals = {
            "s3:x-amz-acl" = "bucket-owner-full-control"
          }
        }
      }
    ]
  })
}

# CloudTrail S3 버킷 버전 관리
resource "aws_s3_bucket_versioning" "cloudtrail_bucket" {
  bucket = aws_s3_bucket.cloudtrail_bucket.id
  versioning_configuration {
    status = "Enabled"
  }
}

# CloudTrail S3 버킷 암호화
resource "aws_s3_bucket_server_side_encryption_configuration" "cloudtrail_bucket" {
  bucket = aws_s3_bucket.cloudtrail_bucket.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# CloudTrail S3 버킷 퍼블릭 액세스 차단
resource "aws_s3_bucket_public_access_block" "cloudtrail_bucket" {
  bucket = aws_s3_bucket.cloudtrail_bucket.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# AWS Macie (민감한 데이터 탐지)
resource "aws_macie2_account" "main" {
  status = "ENABLED"
}

# Macie S3 버킷 분류 작업
resource "aws_macie2_classification_job" "pacs_data" {
  job_type = "ONE_TIME"
  name     = "pacs-data-classification"

  s3_job_definition {
    bucket_definitions {
      account_id = var.aws_account_id
      buckets    = [aws_s3_bucket.pacs_storage.bucket]
    }
  }

  depends_on = [aws_macie2_account.main]
}

# AWS Inspector (보안 취약점 평가)
resource "aws_inspector2_enabler" "main" {
  account_ids    = [var.aws_account_id]
  resource_types = ["EC2", "ECR", "LAMBDA"]
}

# Inspector 평가 템플릿
resource "aws_inspector2_assessment_template" "pacs_security" {
  name       = "pacs-security-assessment"
  target_arn = aws_inspector2_assessment_target.pacs.arn
  duration   = 3600

  rules_package_arns = [
    "arn:aws:inspector:${var.aws_region}:${var.aws_account_id}:rulespackage/0-9hgA1p8",
    "arn:aws:inspector:${var.aws_region}:${var.aws_account_id}:rulespackage/0-H5hpSawc",
    "arn:aws:inspector:${var.aws_region}:${var.aws_account_id}:rulespackage/0-JJOtZiqQ"
  ]

  depends_on = [aws_inspector2_enabler.main]
}

# Inspector 평가 타겟
resource "aws_inspector2_assessment_target" "pacs" {
  name = "pacs-assessment-target"
}
```

### 2. 데이터 보호 및 백업

#### `data-protection.tf`
```hcl
# RDS 자동 백업
resource "aws_db_instance" "pacs_postgresql" {
  # ... 기존 설정 ...
  
  backup_retention_period = 30
  backup_window          = "03:00-04:00"
  maintenance_window     = "sun:04:00-sun:05:00"
  copy_tags_to_snapshot  = true
  deletion_protection    = var.environment == "production"
  
  # ... 기타 설정 ...
}

# RDS 스냅샷
resource "aws_db_snapshot" "pacs_postgresql_daily" {
  count = var.environment == "production" ? 1 : 0
  
  db_instance_identifier = aws_db_instance.pacs_postgresql.id
  db_snapshot_identifier = "pacs-postgresql-daily-${formatdate("YYYY-MM-DD", timestamp())}"
  
  tags = {
    Name        = "pacs-postgresql-daily-snapshot"
    Environment = var.environment
    Project     = var.project_name
    Compliance  = "HIPAA"
  }
}

# S3 버킷 버전 관리
resource "aws_s3_bucket_versioning" "pacs_storage" {
  bucket = aws_s3_bucket.pacs_storage.id
  versioning_configuration {
    status = "Enabled"
  }
}

# S3 버킷 생명주기 정책
resource "aws_s3_bucket_lifecycle_configuration" "pacs_storage" {
  bucket = aws_s3_bucket.pacs_storage.id

  rule {
    id     = "pacs-storage-lifecycle"
    status = "Enabled"

    # 현재 버전 전환
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

    # 이전 버전 삭제
    noncurrent_version_transition {
      noncurrent_days = 30
      storage_class   = "STANDARD_IA"
    }

    noncurrent_version_transition {
      noncurrent_days = 90
      storage_class   = "GLACIER"
    }

    noncurrent_version_expiration {
      noncurrent_days = 2555  # 7년
    }
  }
}

# S3 버킷 복제 (재해 복구)
resource "aws_s3_bucket_replication_configuration" "pacs_storage" {
  count = var.environment == "production" ? 1 : 0
  
  bucket = aws_s3_bucket.pacs_storage.id
  role   = aws_iam_role.replication.arn

  rule {
    id     = "pacs-storage-replication"
    status = "Enabled"

    destination {
      bucket        = aws_s3_bucket.pacs_storage_backup[0].arn
      storage_class = "STANDARD"
    }
  }
}

# 복제용 S3 버킷
resource "aws_s3_bucket" "pacs_storage_backup" {
  count = var.environment == "production" ? 1 : 0
  
  bucket = "pacs-storage-backup-${random_id.bucket_suffix.hex}"

  tags = {
    Name        = "pacs-storage-backup"
    Environment = var.environment
    Project     = var.project_name
    Compliance  = "HIPAA"
  }
}

# 복제 IAM 역할
resource "aws_iam_role" "replication" {
  count = var.environment == "production" ? 1 : 0
  
  name = "pacs-s3-replication-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "s3.amazonaws.com"
        }
      }
    ]
  })
}

# 복제 IAM 정책
resource "aws_iam_policy" "replication" {
  count = var.environment == "production" ? 1 : 0
  
  name        = "pacs-s3-replication-policy"
  description = "Policy for S3 replication"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:GetObjectVersionForReplication",
          "s3:GetObjectVersionAcl",
          "s3:GetObjectVersionTagging"
        ]
        Resource = "${aws_s3_bucket.pacs_storage.arn}/*"
      },
      {
        Effect = "Allow"
        Action = [
          "s3:ReplicateObject",
          "s3:ReplicateDelete",
          "s3:ReplicateTags"
        ]
        Resource = "${aws_s3_bucket.pacs_storage_backup[0].arn}/*"
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "replication" {
  count = var.environment == "production" ? 1 : 0
  
  role       = aws_iam_role.replication[0].name
  policy_arn = aws_iam_policy.replication[0].arn
}
```

---

## 🔧 고급 보안 기능

### 1. 보안 모니터링 및 알림

#### `security-monitoring.tf`
```hcl
# 보안 이벤트 알림을 위한 SNS 토픽
resource "aws_sns_topic" "security_alerts" {
  name = "pacs-security-alerts"

  tags = {
    Name        = "pacs-security-alerts"
    Environment = var.environment
    Project     = var.project_name
    Compliance  = "HIPAA"
  }
}

# 보안 이벤트 SNS 구독
resource "aws_sns_topic_subscription" "security_email" {
  topic_arn = aws_sns_topic.security_alerts.arn
  protocol  = "email"
  endpoint  = var.security_alert_email
}

# GuardDuty 알림
resource "aws_cloudwatch_event_rule" "guardduty_findings" {
  name        = "pacs-guardduty-findings"
  description = "Capture GuardDuty findings"

  event_pattern = jsonencode({
    source      = ["aws.guardduty"]
    detail-type = ["GuardDuty Finding"]
  })
}

resource "aws_cloudwatch_event_target" "guardduty_sns" {
  rule      = aws_cloudwatch_event_rule.guardduty_findings.name
  target_id = "GuardDutySNSTarget"
  arn       = aws_sns_topic.security_alerts.arn
}

# Security Hub 알림
resource "aws_cloudwatch_event_rule" "security_hub_findings" {
  name        = "pacs-security-hub-findings"
  description = "Capture Security Hub findings"

  event_pattern = jsonencode({
    source      = ["aws.securityhub"]
    detail-type = ["Security Hub Findings - Imported"]
  })
}

resource "aws_cloudwatch_event_target" "security_hub_sns" {
  rule      = aws_cloudwatch_event_rule.security_hub_findings.name
  target_id = "SecurityHubSNSTarget"
  arn       = aws_sns_topic.security_alerts.arn
}

# CloudTrail 보안 이벤트 알림
resource "aws_cloudwatch_log_metric_filter" "security_events" {
  name           = "pacs-security-events"
  log_group_name = aws_cloudwatch_log_group.cloudtrail.name
  pattern        = "[timestamp, request_id, event_name, user_identity, source_ip, user_agent, error_code, error_message]"

  metric_transformation {
    name      = "SecurityEvents"
    namespace = "PACS/Security"
    value     = "1"
  }
}

# 보안 이벤트 알람
resource "aws_cloudwatch_metric_alarm" "security_events" {
  alarm_name          = "pacs-security-events"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "1"
  metric_name         = "SecurityEvents"
  namespace           = "PACS/Security"
  period              = "300"
  statistic           = "Sum"
  threshold           = "0"
  alarm_description   = "This metric monitors security events"
  alarm_actions       = [aws_sns_topic.security_alerts.arn]

  tags = {
    Name        = "pacs-security-events"
    Environment = var.environment
    Project     = var.project_name
  }
}

# CloudTrail 로그 그룹
resource "aws_cloudwatch_log_group" "cloudtrail" {
  name              = "/aws/cloudtrail/pacs-audit"
  retention_in_days = var.log_retention_days

  tags = {
    Name        = "pacs-cloudtrail-logs"
    Environment = var.environment
    Project     = var.project_name
    Compliance  = "HIPAA"
  }
}
```

### 2. 자동 보안 대응

#### `security-response.tf`
```hcl
# 보안 이벤트 자동 대응을 위한 Lambda 함수
resource "aws_lambda_function" "security_response" {
  filename         = "security_response.zip"
  function_name    = "pacs-security-response"
  role            = aws_iam_role.lambda_security_response.arn
  handler         = "index.handler"
  runtime         = "python3.9"
  timeout         = 300

  environment {
    variables = {
      SNS_TOPIC_ARN = aws_sns_topic.security_alerts.arn
      S3_BUCKET_NAME = aws_s3_bucket.pacs_storage.bucket
    }
  }

  tags = {
    Name        = "pacs-security-response"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Lambda IAM 역할
resource "aws_iam_role" "lambda_security_response" {
  name = "pacs-lambda-security-response"

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

# Lambda IAM 정책
resource "aws_iam_policy" "lambda_security_response" {
  name        = "pacs-lambda-security-response"
  description = "Policy for security response Lambda"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "sns:Publish",
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject",
          "ec2:DescribeInstances",
          "ec2:StopInstances",
          "ec2:TerminateInstances",
          "iam:ListAccessKeys",
          "iam:UpdateAccessKey",
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
        Resource = "*"
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_security_response" {
  role       = aws_iam_role.lambda_security_response.name
  policy_arn = aws_iam_policy.lambda_security_response.arn
}

# EventBridge 규칙 (보안 이벤트 발생 시)
resource "aws_cloudwatch_event_rule" "security_response" {
  name        = "pacs-security-response"
  description = "Trigger security response for critical events"

  event_pattern = jsonencode({
    source      = ["aws.guardduty", "aws.securityhub"]
    detail-type = ["GuardDuty Finding", "Security Hub Findings - Imported"]
    detail = {
      severity = ["HIGH", "CRITICAL"]
    }
  })
}

resource "aws_cloudwatch_event_target" "security_response" {
  rule      = aws_cloudwatch_event_rule.security_response.name
  target_id = "SecurityResponseTarget"
  arn       = aws_lambda_function.security_response.arn
}

resource "aws_lambda_permission" "allow_eventbridge" {
  statement_id  = "AllowExecutionFromEventBridge"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.security_response.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.security_response.arn
}
```

---

## 🧪 실습 및 테스트

### 1. 보안 설정 테스트

#### `test-security.sh`
```bash
#!/bin/bash
# 보안 설정 테스트 스크립트

echo "Testing security configuration..."

# AWS Config 상태 확인
echo "1. Checking AWS Config status..."
aws configservice describe-configuration-recorders
aws configservice describe-delivery-channels

# GuardDuty 상태 확인
echo "2. Checking GuardDuty status..."
aws guardduty list-detectors

# Security Hub 상태 확인
echo "3. Checking Security Hub status..."
aws securityhub describe-hub

# CloudTrail 상태 확인
echo "4. Checking CloudTrail status..."
aws cloudtrail describe-trails

# KMS 키 상태 확인
echo "5. Checking KMS keys..."
aws kms list-keys --query 'Keys[?KeyManager==`CUSTOMER`]'

# WAF 상태 확인
echo "6. Checking WAF status..."
aws wafv2 list-web_acls --scope REGIONAL

echo "Security configuration test completed! 🎉"
```

### 2. 컴플라이언스 테스트

#### `test-compliance.sh`
```bash
#!/bin/bash
# 컴플라이언스 테스트 스크립트

echo "Testing compliance configuration..."

# HIPAA 요구사항 확인
echo "1. Checking HIPAA compliance requirements..."

# 암호화 상태 확인
echo "2. Checking encryption status..."
aws s3api get-bucket-encryption --bucket $(aws s3api list-buckets --query 'Buckets[?contains(Name, `pacs`)].Name' --output text)

# 백업 상태 확인
echo "3. Checking backup status..."
aws rds describe-db-snapshots --db-instance-identifier pacs-postgresql

# 접근 로그 확인
echo "4. Checking access logs..."
aws logs describe-log-groups --log-group-name-prefix "/aws/cloudtrail"

# 감사 로그 확인
echo "5. Checking audit logs..."
aws logs filter-log-events \
  --log-group-name "/aws/cloudtrail/pacs-audit" \
  --start-time $(date -d '1 hour ago' +%s)000 \
  --end-time $(date +%s)000

echo "Compliance test completed! 🎉"
```

### 3. 보안 모니터링 테스트

#### `test-security-monitoring.sh`
```bash
#!/bin/bash
# 보안 모니터링 테스트 스크립트

echo "Testing security monitoring..."

# 보안 알림 테스트
echo "1. Testing security alerts..."
aws sns publish \
  --topic-arn "arn:aws:sns:ap-northeast-2:123456789012:pacs-security-alerts" \
  --message "Security monitoring test message" \
  --subject "PACS Security Test"

# GuardDuty 테스트
echo "2. Testing GuardDuty..."
aws guardduty list-findings --detector-id $(aws guardduty list-detectors --query 'DetectorIds[0]' --output text)

# Security Hub 테스트
echo "3. Testing Security Hub..."
aws securityhub get-findings --max-items 10

# CloudWatch 보안 메트릭 확인
echo "4. Checking security metrics..."
aws cloudwatch list-metrics --namespace "PACS/Security"

# 보안 이벤트 확인
echo "5. Checking security events..."
aws logs filter-log-events \
  --log-group-name "/aws/cloudtrail/pacs-audit" \
  --filter-pattern "ERROR" \
  --start-time $(date -d '1 hour ago' +%s)000 \
  --end-time $(date +%s)000

echo "Security monitoring test completed! 🎉"
```

---

## 🔧 문제 해결

### 1. AWS Config 설정 실패

**증상**: AWS Config 설정 실패
```
Error: The configuration recorder could not be started
```

**해결 방법**:
```hcl
# Config 역할에 필요한 권한 추가
resource "aws_iam_role_policy_attachment" "config_role" {
  role       = aws_iam_role.config_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/ConfigRole"
}

# Config 서비스 연결 확인
resource "aws_config_configuration_recorder" "main" {
  name     = "pacs-config-recorder"
  role_arn = aws_iam_role.config_role.arn

  recording_group {
    all_supported                 = true
    include_global_resource_types = true
  }

  depends_on = [aws_config_delivery_channel.main]
}
```

### 2. KMS 키 권한 오류

**증상**: KMS 키 접근 권한 오류
```
Error: AccessDenied: User is not authorized to perform kms:Decrypt
```

**해결 방법**:
```hcl
# KMS 키 정책에 사용자 추가
resource "aws_kms_key" "pacs_data" {
  # ... 기존 설정 ...
  
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "Enable IAM User Permissions"
        Effect = "Allow"
        Principal = {
          AWS = "arn:aws:iam::${var.aws_account_id}:root"
        }
        Action   = "kms:*"
        Resource = "*"
      },
      {
        Sid    = "Allow PACS Service"
        Effect = "Allow"
        Principal = {
          AWS = aws_iam_role.pacs_service.arn
        }
        Action = [
          "kms:Encrypt",
          "kms:Decrypt",
          "kms:ReEncrypt*",
          "kms:GenerateDataKey*"
        ]
        Resource = "*"
      }
    ]
  })
}
```

### 3. WAF 규칙 충돌

**증상**: WAF 규칙 충돌
```
Error: Duplicate rule priority
```

**해결 방법**:
```hcl
# WAF 규칙 우선순위 확인 및 조정
resource "aws_wafv2_web_acl" "pacs_web_acl" {
  # ... 기타 설정 ...
  
  rule {
    name     = "SQLInjectionRule"
    priority = 1  # 고유한 우선순위 사용
    # ... 기타 설정 ...
  }

  rule {
    name     = "XSSRule"
    priority = 2  # 다른 우선순위 사용
    # ... 기타 설정 ...
  }
}
```

---

## 📚 다음 단계

이제 보안 및 컴플라이언스 시스템을 성공적으로 설정했으니 다음 문서들을 학습하세요:

1. **비용 최적화** - AWS 비용 관리 및 최적화
2. **재해 복구** - 백업 및 복구 전략
3. **성능 최적화** - 시스템 성능 튜닝

---

## 📖 참고 자료

- [AWS 보안 서비스 공식 문서](https://docs.aws.amazon.com/security/)
- [HIPAA 컴플라이언스 가이드](https://aws.amazon.com/compliance/hipaa-compliance/)
- [AWS 보안 모범 사례](https://aws.amazon.com/security/security-resources/)

이제 PACS 프로젝트의 종합적인 보안 및 컴플라이언스 시스템이 준비되었습니다! 🚀


