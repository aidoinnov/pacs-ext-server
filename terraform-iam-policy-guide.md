# 🔐 IAM 정책 및 사용자 생성 가이드

Terraform을 사용하여 AWS IAM 정책과 사용자를 생성하고 관리하는 방법을 학습합니다. PACS 프로젝트의 보안 요구사항에 맞는 최소 권한 원칙을 적용합니다.

## 📋 목차

1. [IAM이란?](#iam이란)
2. [기본 IAM 구성](#기본-iam-구성)
3. [PACS 프로젝트 IAM 설정](#pacs-프로젝트-iam-설정)
4. [역할 기반 접근 제어](#역할-기반-접근-제어)
5. [고급 보안 설정](#고급-보안-설정)
6. [실습 및 테스트](#실습-및-테스트)

---

## 🎯 IAM이란?

**AWS IAM (Identity and Access Management)**은 AWS 리소스에 대한 접근을 안전하게 제어하는 서비스입니다.

### 주요 구성 요소
- **사용자 (Users)**: AWS 서비스에 접근하는 개인 또는 애플리케이션
- **그룹 (Groups)**: 사용자들을 논리적으로 묶는 컨테이너
- **역할 (Roles)**: AWS 서비스 간 권한 위임
- **정책 (Policies)**: 권한을 정의하는 JSON 문서

### PACS 프로젝트에서의 활용
- **개발자 계정**: 개발 환경 접근 권한
- **운영자 계정**: 프로덕션 환경 관리 권한
- **서비스 역할**: 애플리케이션별 최소 권한
- **크로스 계정 액세스**: 여러 AWS 계정 간 권한 공유

---

## 🔧 기본 IAM 구성

### 1. 사용자 생성

#### `iam-users.tf`
```hcl
# 개발자 사용자
resource "aws_iam_user" "developer" {
  name = "pacs-developer"
  path = "/pacs/"

  tags = {
    Name        = "PACS Developer"
    Environment = "development"
    Role        = "developer"
  }
}

# 운영자 사용자
resource "aws_iam_user" "operator" {
  name = "pacs-operator"
  path = "/pacs/"

  tags = {
    Name        = "PACS Operator"
    Environment = "production"
    Role        = "operator"
  }
}

# 서비스 사용자
resource "aws_iam_user" "service" {
  name = "pacs-service"
  path = "/pacs/"

  tags = {
    Name        = "PACS Service Account"
    Environment = "all"
    Role        = "service"
  }
}
```

### 2. 그룹 생성

#### `iam-groups.tf`
```hcl
# 개발자 그룹
resource "aws_iam_group" "developers" {
  name = "pacs-developers"
  path = "/pacs/"
}

# 운영자 그룹
resource "aws_iam_group" "operators" {
  name = "pacs-operators"
  path = "/pacs/"
}

# 읽기 전용 그룹
resource "aws_iam_group" "readonly" {
  name = "pacs-readonly"
  path = "/pacs/"
}
```

### 3. 사용자 그룹 멤버십

#### `iam-memberships.tf`
```hcl
# 개발자 그룹 멤버십
resource "aws_iam_user_group_membership" "developer" {
  user = aws_iam_user.developer.name
  groups = [
    aws_iam_group.developers.name,
    aws_iam_group.readonly.name
  ]
}

# 운영자 그룹 멤버십
resource "aws_iam_user_group_membership" "operator" {
  user = aws_iam_user.operator.name
  groups = [
    aws_iam_group.operators.name,
    aws_iam_group.readonly.name
  ]
}
```

---

## 🏥 PACS 프로젝트 IAM 설정

### 1. 개발자 정책

#### `pacs-developer-policy.tf`
```hcl
# 개발자 정책
resource "aws_iam_policy" "pacs_developer" {
  name        = "pacs-developer-policy"
  description = "Policy for PACS developers"
  path        = "/pacs/"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "EC2ReadAccess"
        Effect = "Allow"
        Action = [
          "ec2:Describe*",
          "ec2:Get*"
        ]
        Resource = "*"
      },
      {
        Sid    = "S3DevelopmentAccess"
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject",
          "s3:ListBucket"
        ]
        Resource = [
          "arn:aws:s3:::pacs-development-*",
          "arn:aws:s3:::pacs-development-*/*"
        ]
      },
      {
        Sid    = "RDSReadAccess"
        Effect = "Allow"
        Action = [
          "rds:Describe*",
          "rds:ListTagsForResource"
        ]
        Resource = "*"
      },
      {
        Sid    = "CloudWatchLogsAccess"
        Effect = "Allow"
        Action = [
          "logs:DescribeLogGroups",
          "logs:DescribeLogStreams",
          "logs:GetLogEvents",
          "logs:FilterLogEvents"
        ]
        Resource = "arn:aws:logs:*:*:log-group:/pacs/*"
      }
    ]
  })
}

# 개발자 그룹에 정책 연결
resource "aws_iam_group_policy_attachment" "pacs_developer" {
  group      = aws_iam_group.developers.name
  policy_arn = aws_iam_policy.pacs_developer.arn
}
```

### 2. 운영자 정책

#### `pacs-operator-policy.tf`
```hcl
# 운영자 정책
resource "aws_iam_policy" "pacs_operator" {
  name        = "pacs-operator-policy"
  description = "Policy for PACS operators"
  path        = "/pacs/"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "FullEC2Access"
        Effect = "Allow"
        Action = [
          "ec2:*"
        ]
        Resource = "*"
      },
      {
        Sid    = "FullS3Access"
        Effect = "Allow"
        Action = [
          "s3:*"
        ]
        Resource = [
          "arn:aws:s3:::pacs-*",
          "arn:aws:s3:::pacs-*/*"
        ]
      },
      {
        Sid    = "FullRDSAccess"
        Effect = "Allow"
        Action = [
          "rds:*"
        ]
        Resource = "*"
      },
      {
        Sid    = "FullCloudWatchAccess"
        Effect = "Allow"
        Action = [
          "logs:*",
          "cloudwatch:*"
        ]
        Resource = "*"
      },
      {
        Sid    = "IAMReadAccess"
        Effect = "Allow"
        Action = [
          "iam:Get*",
          "iam:List*"
        ]
        Resource = "*"
      }
    ]
  })
}

# 운영자 그룹에 정책 연결
resource "aws_iam_group_policy_attachment" "pacs_operator" {
  group      = aws_iam_group.operators.name
  policy_arn = aws_iam_policy.pacs_operator.arn
}
```

### 3. 서비스 역할

#### `pacs-service-roles.tf`
```hcl
# PACS 애플리케이션 역할
resource "aws_iam_role" "pacs_application" {
  name = "pacs-application-role"
  path = "/pacs/"

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

  tags = {
    Name        = "PACS Application Role"
    Environment = "all"
    Purpose     = "Application Service"
  }
}

# S3 액세스 정책
resource "aws_iam_policy" "pacs_s3_access" {
  name        = "pacs-s3-access-policy"
  description = "S3 access policy for PACS application"
  path        = "/pacs/"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "S3BucketAccess"
        Effect = "Allow"
        Action = [
          "s3:ListBucket"
        ]
        Resource = [
          "arn:aws:s3:::pacs-*"
        ]
      },
      {
        Sid    = "S3ObjectAccess"
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject"
        ]
        Resource = [
          "arn:aws:s3:::pacs-*/*"
        ]
      }
    ]
  })
}

# RDS 액세스 정책
resource "aws_iam_policy" "pacs_rds_access" {
  name        = "pacs-rds-access-policy"
  description = "RDS access policy for PACS application"
  path        = "/pacs/"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "RDSConnectAccess"
        Effect = "Allow"
        Action = [
          "rds-db:connect"
        ]
        Resource = [
          "arn:aws:rds-db:*:*:dbuser:*/pacs_app"
        ]
      }
    ]
  })
}

# 정책 연결
resource "aws_iam_role_policy_attachment" "pacs_s3_access" {
  role       = aws_iam_role.pacs_application.name
  policy_arn = aws_iam_policy.pacs_s3_access.arn
}

resource "aws_iam_role_policy_attachment" "pacs_rds_access" {
  role       = aws_iam_role.pacs_application.name
  policy_arn = aws_iam_policy.pacs_rds_access.arn
}
```

---

## 👥 역할 기반 접근 제어

### 1. 크로스 계정 역할

#### `cross-account-roles.tf`
```hcl
# 개발 계정에서 운영 계정 액세스 역할
resource "aws_iam_role" "cross_account_operator" {
  name = "pacs-cross-account-operator"
  path = "/pacs/"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          AWS = "arn:aws:iam::${var.development_account_id}:root"
        }
        Condition = {
          StringEquals = {
            "sts:ExternalId" = var.cross_account_external_id
          }
        }
      }
    ]
  })

  tags = {
    Name        = "Cross Account Operator Role"
    Environment = "production"
    Purpose     = "Cross Account Access"
  }
}

# 크로스 계정 정책
resource "aws_iam_policy" "cross_account_policy" {
  name        = "pacs-cross-account-policy"
  description = "Cross account access policy"
  path        = "/pacs/"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "ReadOnlyAccess"
        Effect = "Allow"
        Action = [
          "ec2:Describe*",
          "s3:GetObject",
          "s3:ListBucket",
          "rds:Describe*",
          "logs:GetLogEvents"
        ]
        Resource = "*"
      }
    ]
  })
}

# 크로스 계정 역할에 정책 연결
resource "aws_iam_role_policy_attachment" "cross_account_policy" {
  role       = aws_iam_role.cross_account_operator.name
  policy_arn = aws_iam_policy.cross_account_policy.arn
}
```

### 2. 임시 자격 증명

#### `temporary-credentials.tf`
```hcl
# STS 임시 자격 증명 정책
resource "aws_iam_policy" "sts_temporary_credentials" {
  name        = "pacs-sts-temporary-credentials"
  description = "Policy for temporary credentials"
  path        = "/pacs/"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AssumeRoleAccess"
        Effect = "Allow"
        Action = [
          "sts:AssumeRole",
          "sts:GetSessionToken"
        ]
        Resource = [
          "arn:aws:iam::*:role/pacs-*"
        ]
      }
    ]
  })
}

# 개발자에게 임시 자격 증명 권한 부여
resource "aws_iam_user_policy_attachment" "developer_sts" {
  user       = aws_iam_user.developer.name
  policy_arn = aws_iam_policy.sts_temporary_credentials.arn
}
```

---

## 🔒 고급 보안 설정

### 1. MFA 정책

#### `mfa-policies.tf`
```hcl
# MFA 강제 정책
resource "aws_iam_policy" "mfa_required" {
  name        = "pacs-mfa-required"
  description = "Policy requiring MFA for sensitive operations"
  path        = "/pacs/"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "DenyAllExceptListedIfNoMFA"
        Effect = "Deny"
        NotAction = [
          "iam:CreateVirtualMFADevice",
          "iam:EnableMFADevice",
          "iam:GetUser",
          "iam:ListMFADevices",
          "iam:ListVirtualMFADevices",
          "iam:ResyncMFADevice",
          "sts:GetSessionToken"
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

# 운영자에게 MFA 정책 적용
resource "aws_iam_user_policy_attachment" "operator_mfa" {
  user       = aws_iam_user.operator.name
  policy_arn = aws_iam_policy.mfa_required.arn
}
```

### 2. IP 제한 정책

#### `ip-restriction-policies.tf`
```hcl
# IP 제한 정책
resource "aws_iam_policy" "ip_restriction" {
  name        = "pacs-ip-restriction"
  description = "Policy restricting access by IP address"
  path        = "/pacs/"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "DenyAccessFromUnapprovedIPs"
        Effect = "Deny"
        Action = "*"
        Resource = "*"
        Condition = {
          IpAddressIfExists = {
            "aws:SourceIp" = [
              "0.0.0.0/0"
            ]
          }
          StringNotEquals = {
            "aws:SourceIp" = var.allowed_ip_ranges
          }
        }
      }
    ]
  })
}

# 개발자에게 IP 제한 적용
resource "aws_iam_user_policy_attachment" "developer_ip_restriction" {
  user       = aws_iam_user.developer.name
  policy_arn = aws_iam_policy.ip_restriction.arn
}
```

### 3. 시간 제한 정책

#### `time-restriction-policies.tf`
```hcl
# 시간 제한 정책
resource "aws_iam_policy" "time_restriction" {
  name        = "pacs-time-restriction"
  description = "Policy restricting access by time"
  path        = "/pacs/"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "DenyAccessOutsideBusinessHours"
        Effect = "Deny"
        Action = [
          "ec2:StopInstances",
          "ec2:TerminateInstances",
          "rds:StopDBInstance",
          "rds:DeleteDBInstance"
        ]
        Resource = "*"
        Condition = {
          DateGreaterThan = {
            "aws:CurrentTime" = "2024-01-01T18:00:00Z"
          }
          DateLessThan = {
            "aws:CurrentTime" = "2024-01-02T09:00:00Z"
          }
        }
      }
    ]
  })
}

# 운영자에게 시간 제한 적용
resource "aws_iam_user_policy_attachment" "operator_time_restriction" {
  user       = aws_iam_user.operator.name
  policy_arn = aws_iam_policy.time_restriction.arn
}
```

---

## 🧪 실습 및 테스트

### 1. IAM 사용자 생성 테스트

#### `test-iam-creation.sh`
```bash
#!/bin/bash
# IAM 사용자 생성 테스트 스크립트

echo "Testing IAM user creation..."

# Terraform 초기화
echo "1. Initializing Terraform..."
terraform init

# Terraform 검증
echo "2. Validating configuration..."
terraform validate

# IAM 사용자 생성
echo "3. Creating IAM users..."
terraform apply -target=aws_iam_user.developer -auto-approve
terraform apply -target=aws_iam_user.operator -auto-approve

# 사용자 확인
echo "4. Verifying user creation..."
aws iam get-user --user-name pacs-developer
aws iam get-user --user-name pacs-operator

echo "IAM user creation test completed! 🎉"
```

### 2. 정책 테스트

#### `test-policies.sh`
```bash
#!/bin/bash
# IAM 정책 테스트 스크립트

echo "Testing IAM policies..."

# 정책 확인
echo "1. Checking policies..."
aws iam list-policies --path-prefix "/pacs/"

# 그룹 정책 확인
echo "2. Checking group policies..."
aws iam list-attached-group-policies --group-name pacs-developers
aws iam list-attached-group-policies --group-name pacs-operators

# 사용자 정책 확인
echo "3. Checking user policies..."
aws iam list-attached-user-policies --user-name pacs-developer
aws iam list-attached-user-policies --user-name pacs-operator

echo "Policy test completed! 🎉"
```

### 3. 역할 테스트

#### `test-roles.sh`
```bash
#!/bin/bash
# IAM 역할 테스트 스크립트

echo "Testing IAM roles..."

# 역할 확인
echo "1. Checking roles..."
aws iam get-role --role-name pacs-application-role

# 역할 정책 확인
echo "2. Checking role policies..."
aws iam list-attached-role-policies --role-name pacs-application-role

# 역할 가정 테스트
echo "3. Testing role assumption..."
aws sts assume-role \
  --role-arn "arn:aws:iam::$(aws sts get-caller-identity --query Account --output text):role/pacs-application-role" \
  --role-session-name "test-session"

echo "Role test completed! 🎉"
```

### 4. 보안 설정 테스트

#### `test-security.sh`
```bash
#!/bin/bash
# IAM 보안 설정 테스트 스크립트

echo "Testing IAM security settings..."

# MFA 설정 확인
echo "1. Checking MFA settings..."
aws iam list-mfa-devices --user-name pacs-operator

# 액세스 키 확인
echo "2. Checking access keys..."
aws iam list-access-keys --user-name pacs-developer

# 로그인 프로필 확인
echo "3. Checking login profiles..."
aws iam get-login-profile --user-name pacs-operator 2>/dev/null || echo "No login profile found"

echo "Security test completed! 🎉"
```

---

## 🔧 문제 해결

### 1. 권한 부족 오류

**증상**: IAM 리소스 생성 권한 부족
```
Error: AccessDenied: User is not authorized to perform: iam:CreateUser
```

**해결 방법**:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "iam:CreateUser",
        "iam:CreateGroup",
        "iam:CreatePolicy",
        "iam:AttachUserPolicy",
        "iam:AttachGroupPolicy"
      ],
      "Resource": "*"
    }
  ]
}
```

### 2. 정책 문법 오류

**증상**: JSON 정책 문법 오류
```
Error: MalformedPolicyDocument: The policy document is malformed
```

**해결 방법**:
```hcl
# JSON 유효성 검사
resource "aws_iam_policy" "test_policy" {
  name = "test-policy"
  
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = "s3:GetObject"
        Resource = "arn:aws:s3:::bucket/*"
      }
    ]
  })
}
```

### 3. 역할 가정 실패

**증상**: 역할 가정 권한 부족
```
Error: AccessDenied: User is not authorized to perform: sts:AssumeRole
```

**해결 방법**:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": "sts:AssumeRole",
      "Resource": "arn:aws:iam::*:role/pacs-*"
    }
  ]
}
```

---

## 📚 다음 단계

이제 IAM 정책과 사용자를 성공적으로 설정했으니 다음 문서들을 학습하세요:

1. **RDS PostgreSQL 구성** - 데이터베이스 설정
2. **VPC 및 네트워킹** - 네트워크 보안 구성
3. **EKS 클러스터 구성** - 컨테이너 오케스트레이션

---

## 📖 참고 자료

- [AWS IAM 공식 문서](https://docs.aws.amazon.com/iam/)
- [IAM 정책 예제](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies_examples.html)
- [최소 권한 원칙](https://docs.aws.amazon.com/IAM/latest/UserGuide/best-practices.html)

이제 PACS 프로젝트의 보안을 위한 IAM 설정이 완료되었습니다! 🚀
