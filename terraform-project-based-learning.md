# 🎯 프로젝트 기반 Terraform 학습 가이드

현재 PACS Extension Server 프로젝트에서 실제로 사용하고 있는 인프라 구성 요소들을 바탕으로 한 Terraform 학습 주제 목록입니다.

## 📋 현재 프로젝트 인프라 구성 요소

### 1. **데이터베이스 (PostgreSQL)**
- Docker Compose로 관리되는 PostgreSQL 16
- 포트: 5432
- 볼륨: `postgres_data`
- 헬스체크 설정

### 2. **Object Storage (AWS S3 / MinIO)**
- AWS S3 (프로덕션)
- MinIO (로컬 개발)
- Signed URL을 통한 직접 업로드 패턴
- CORS 설정

### 3. **웹 서버 (Rust Actix-web)**
- 포트: 8080
- CORS 설정
- JWT 인증
- 로깅 설정

### 4. **인증 시스템 (Keycloak)**
- OAuth 2.0 / OpenID Connect
- JWT 토큰 관리

---

## 🎓 프로젝트 기반 학습 주제

### **Phase 1: 기본 인프라 구성 (1-2주)**

#### 1.1 Docker Compose → Terraform 마이그레이션
**현재 상태**: `infra/docker-compose.yml`
```yaml
version: '3.8'
services:
  postgres:
    image: postgres:16-alpine
    container_name: pacs-postgres
    environment:
      POSTGRES_USER: admin
      POSTGRES_PASSWORD: admin123
      POSTGRES_DB: pacs_db
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
```

**학습 목표**:
- Docker Provider 사용법
- Terraform으로 컨테이너 관리
- 볼륨 및 네트워크 설정
- 환경변수 관리

**실습 과제**:
```hcl
# terraform/docker/main.tf
resource "docker_image" "postgres" {
  name = "postgres:16-alpine"
}

resource "docker_container" "postgres" {
  name  = "pacs-postgres"
  image = docker_image.postgres.image_id
  
  env = [
    "POSTGRES_USER=admin",
    "POSTGRES_PASSWORD=admin123",
    "POSTGRES_DB=pacs_db"
  ]
  
  ports {
    internal = 5432
    external = 5432
  }
  
  volumes {
    volume_name    = docker_volume.postgres_data.name
    container_path = "/var/lib/postgresql/data"
  }
}

resource "docker_volume" "postgres_data" {
  name = "postgres_data"
}
```

#### 1.2 로컬 개발 환경 구성
**학습 목표**:
- Terraform Workspaces 사용
- 환경별 설정 분리
- 로컬 개발용 MinIO 설정

**실습 과제**:
```hcl
# terraform/environments/development/main.tf
resource "docker_container" "minio" {
  name  = "pacs-minio"
  image = "quay.io/minio/minio:latest"
  
  command = ["server", "/data", "--console-address", ":9001"]
  
  ports {
    internal = 9000
    external = 9000
  }
  
  ports {
    internal = 9001
    external = 9001
  }
  
  env = [
    "MINIO_ROOT_USER=minioadmin",
    "MINIO_ROOT_PASSWORD=minioadmin"
  ]
}
```

### **Phase 2: AWS 클라우드 인프라 (2-3주)**

#### 2.1 AWS S3 버킷 구성
**현재 상태**: AWS S3 연동 코드
```rust
pub struct ObjectStorageConfig {
    pub provider: String,      // "s3" or "minio"
    pub bucket_name: String,   // S3 버킷 이름
    pub region: String,        // AWS 리전
    pub endpoint: String,      // MinIO 엔드포인트
    pub access_key: String,    // AWS Access Key ID
    pub secret_key: String,    // AWS Secret Access Key
}
```

**학습 목표**:
- AWS Provider 설정
- S3 버킷 생성 및 설정
- IAM 정책 및 사용자 생성
- CORS 설정

**실습 과제**:
```hcl
# terraform/aws/s3.tf
resource "aws_s3_bucket" "pacs_masks" {
  bucket = "pacs-masks-${random_id.bucket_suffix.hex}"
  
  tags = {
    Name        = "PACS Masks Storage"
    Environment = var.environment
    Project     = "pacs-extension"
  }
}

resource "aws_s3_bucket_cors_configuration" "pacs_masks" {
  bucket = aws_s3_bucket.pacs_masks.id

  cors_rule {
    allowed_headers = ["*"]
    allowed_methods = ["GET", "PUT", "POST", "DELETE"]
    allowed_origins = var.allowed_origins
    expose_headers  = ["ETag"]
    max_age_seconds = 3000
  }
}

resource "aws_s3_bucket_versioning" "pacs_masks" {
  bucket = aws_s3_bucket.pacs_masks.id
  versioning_configuration {
    status = "Disabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "pacs_masks" {
  bucket = aws_s3_bucket.pacs_masks.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}
```

#### 2.2 IAM 정책 및 사용자 생성
**학습 목표**:
- IAM 정책 작성
- IAM 사용자 생성
- Access Key 관리
- 최소 권한 원칙 적용

**실습 과제**:
```hcl
# terraform/aws/iam.tf
resource "aws_iam_policy" "pacs_s3_policy" {
  name        = "pacs-s3-mask-policy"
  description = "Policy for PACS mask upload to S3"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:PutObject",
          "s3:GetObject",
          "s3:DeleteObject"
        ]
        Resource = "${aws_s3_bucket.pacs_masks.arn}/mask/*"
      },
      {
        Effect = "Allow"
        Action = "s3:ListBucket"
        Resource = aws_s3_bucket.pacs_masks.arn
      }
    ]
  })
}

resource "aws_iam_user" "pacs_s3_user" {
  name = "pacs-s3-uploader"
  
  tags = {
    Name        = "PACS S3 Uploader"
    Environment = var.environment
  }
}

resource "aws_iam_user_policy_attachment" "pacs_s3_policy_attach" {
  user       = aws_iam_user.pacs_s3_user.name
  policy_arn = aws_iam_policy.pacs_s3_policy.arn
}

resource "aws_iam_access_key" "pacs_s3_user_key" {
  user = aws_iam_user.pacs_s3_user.name
}
```

#### 2.3 RDS PostgreSQL 구성
**학습 목표**:
- RDS 인스턴스 생성
- 보안 그룹 설정
- 서브넷 그룹 구성
- 백업 및 모니터링 설정

**실습 과제**:
```hcl
# terraform/aws/rds.tf
resource "aws_db_instance" "pacs_postgres" {
  identifier = "pacs-postgres-${var.environment}"
  
  engine         = "postgres"
  engine_version = "16.1"
  instance_class = var.db_instance_class
  
  allocated_storage     = 20
  max_allocated_storage = 100
  storage_type          = "gp3"
  storage_encrypted     = true
  
  db_name  = "pacs_db"
  username = "admin"
  password = var.db_password
  
  vpc_security_group_ids = [aws_security_group.rds.id]
  db_subnet_group_name   = aws_db_subnet_group.pacs.name
  
  backup_retention_period = 7
  backup_window          = "03:00-04:00"
  maintenance_window     = "sun:04:00-sun:05:00"
  
  skip_final_snapshot = var.environment == "development"
  
  tags = {
    Name        = "PACS PostgreSQL"
    Environment = var.environment
  }
}
```

### **Phase 3: 네트워킹 및 보안 (2-3주)**

#### 3.1 VPC 및 서브넷 구성
**학습 목표**:
- VPC 생성 및 설정
- 퍼블릭/프라이빗 서브넷 분리
- 인터넷 게이트웨이 설정
- NAT 게이트웨이 구성

**실습 과제**:
```hcl
# terraform/aws/vpc.tf
resource "aws_vpc" "pacs_vpc" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  enable_dns_support   = true
  
  tags = {
    Name        = "pacs-vpc"
    Environment = var.environment
  }
}

resource "aws_internet_gateway" "pacs_igw" {
  vpc_id = aws_vpc.pacs_vpc.id
  
  tags = {
    Name = "pacs-igw"
  }
}

resource "aws_subnet" "public" {
  count = 2
  
  vpc_id                  = aws_vpc.pacs_vpc.id
  cidr_block              = "10.0.${count.index + 1}.0/24"
  availability_zone       = data.aws_availability_zones.available.names[count.index]
  map_public_ip_on_launch = true
  
  tags = {
    Name = "pacs-public-subnet-${count.index + 1}"
    Type = "Public"
  }
}

resource "aws_subnet" "private" {
  count = 2
  
  vpc_id            = aws_vpc.pacs_vpc.id
  cidr_block        = "10.0.${count.index + 10}.0/24"
  availability_zone = data.aws_availability_zones.available.names[count.index]
  
  tags = {
    Name = "pacs-private-subnet-${count.index + 1}"
    Type = "Private"
  }
}
```

#### 3.2 보안 그룹 구성
**학습 목표**:
- 보안 그룹 규칙 작성
- 인바운드/아웃바운드 규칙 설정
- 보안 그룹 간 참조

**실습 과제**:
```hcl
# terraform/aws/security_groups.tf
resource "aws_security_group" "web" {
  name_prefix = "pacs-web-"
  vpc_id      = aws_vpc.pacs_vpc.id
  
  ingress {
    from_port   = 8080
    to_port     = 8080
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  
  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = [var.ssh_cidr_blocks]
  }
  
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
  
  tags = {
    Name = "pacs-web-sg"
  }
}

resource "aws_security_group" "rds" {
  name_prefix = "pacs-rds-"
  vpc_id      = aws_vpc.pacs_vpc.id
  
  ingress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.web.id]
  }
  
  tags = {
    Name = "pacs-rds-sg"
  }
}
```

### **Phase 4: 애플리케이션 배포 (2-3주)**

#### 4.1 EC2 인스턴스 구성
**학습 목표**:
- EC2 인스턴스 생성
- 사용자 데이터 스크립트
- 키 페어 관리
- 탄력적 IP 설정

**실습 과제**:
```hcl
# terraform/aws/ec2.tf
resource "aws_instance" "pacs_server" {
  ami                    = data.aws_ami.ubuntu.id
  instance_type          = var.instance_type
  key_name               = aws_key_pair.pacs_key.key_name
  vpc_security_group_ids = [aws_security_group.web.id]
  subnet_id              = aws_subnet.public[0].id
  
  user_data = base64encode(templatefile("${path.module}/user_data.sh", {
    db_host     = aws_db_instance.pacs_postgres.endpoint
    db_name     = aws_db_instance.pacs_postgres.db_name
    db_username = aws_db_instance.pacs_postgres.username
    db_password = var.db_password
    s3_bucket   = aws_s3_bucket.pacs_masks.bucket
    s3_region   = var.aws_region
  }))
  
  tags = {
    Name        = "pacs-server"
    Environment = var.environment
  }
}

resource "aws_eip" "pacs_server" {
  instance = aws_instance.pacs_server.id
  domain   = "vpc"
  
  tags = {
    Name = "pacs-server-eip"
  }
}
```

#### 4.2 Application Load Balancer 구성
**학습 목표**:
- ALB 생성 및 설정
- 타겟 그룹 구성
- 헬스체크 설정
- SSL/TLS 인증서 관리

**실습 과제**:
```hcl
# terraform/aws/alb.tf
resource "aws_lb" "pacs_alb" {
  name               = "pacs-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets            = aws_subnet.public[*].id
  
  tags = {
    Name = "pacs-alb"
  }
}

resource "aws_lb_target_group" "pacs_tg" {
  name     = "pacs-tg"
  port     = 8080
  protocol = "HTTP"
  vpc_id   = aws_vpc.pacs_vpc.id
  
  health_check {
    enabled             = true
    healthy_threshold   = 2
    unhealthy_threshold = 2
    timeout             = 5
    interval            = 30
    path                = "/health"
    matcher             = "200"
    port                = "traffic-port"
    protocol            = "HTTP"
  }
}

resource "aws_lb_listener" "pacs_listener" {
  load_balancer_arn = aws_lb.pacs_alb.arn
  port              = "80"
  protocol          = "HTTP"
  
  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.pacs_tg.arn
  }
}
```

### **Phase 5: 모니터링 및 로깅 (1-2주)**

#### 5.1 CloudWatch 구성
**학습 목표**:
- CloudWatch 로그 그룹 생성
- 메트릭 필터 설정
- 알람 구성
- 대시보드 생성

**실습 과제**:
```hcl
# terraform/aws/cloudwatch.tf
resource "aws_cloudwatch_log_group" "pacs_logs" {
  name              = "/aws/ec2/pacs-server"
  retention_in_days = 30
  
  tags = {
    Name = "pacs-server-logs"
  }
}

resource "aws_cloudwatch_metric_alarm" "high_cpu" {
  alarm_name          = "pacs-high-cpu"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "CPUUtilization"
  namespace           = "AWS/EC2"
  period              = "300"
  statistic           = "Average"
  threshold           = "80"
  alarm_description   = "This metric monitors ec2 cpu utilization"
  
  dimensions = {
    InstanceId = aws_instance.pacs_server.id
  }
}

resource "aws_cloudwatch_dashboard" "pacs_dashboard" {
  dashboard_name = "pacs-dashboard"
  
  dashboard_body = jsonencode({
    widgets = [
      {
        type   = "metric"
        x      = 0
        y      = 0
        width  = 12
        height = 6
        
        properties = {
          metrics = [
            ["AWS/EC2", "CPUUtilization", "InstanceId", aws_instance.pacs_server.id],
            [".", "NetworkIn", ".", "."],
            [".", "NetworkOut", ".", "."]
          ]
          period = 300
          stat   = "Average"
          region = var.aws_region
          title  = "PACS Server Metrics"
        }
      }
    ]
  })
}
```

### **Phase 6: CI/CD 파이프라인 (2-3주)**

#### 6.1 GitHub Actions 통합
**학습 목표**:
- Terraform 상태 관리
- 원격 백엔드 설정
- 자동화된 배포 파이프라인
- 환경별 배포 전략

**실습 과제**:
```yaml
# .github/workflows/terraform.yml
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
      working-directory: ./terraform
      
    - name: Terraform Validate
      run: terraform validate
      working-directory: ./terraform
      
    - name: Terraform Plan
      run: terraform plan
      working-directory: ./terraform
      env:
        TF_VAR_environment: ${{ github.ref == 'refs/heads/main' && 'production' || 'development' }}
        
    - name: Terraform Apply
      if: github.ref == 'refs/heads/main'
      run: terraform apply -auto-approve
      working-directory: ./terraform
      env:
        TF_VAR_environment: production
```

---

## 🎯 학습 순서 및 체크리스트

### **Week 1-2: 기본 환경 구성**
- [ ] Docker Provider 학습 및 실습
- [ ] 현재 docker-compose.yml을 Terraform으로 변환
- [ ] 로컬 개발 환경 구성 (MinIO 포함)
- [ ] 환경별 설정 분리 (development/production)

### **Week 3-5: AWS 기본 인프라**
- [ ] AWS Provider 설정 및 인증
- [ ] S3 버킷 생성 및 CORS 설정
- [ ] IAM 정책 및 사용자 생성
- [ ] RDS PostgreSQL 구성

### **Week 6-8: 네트워킹 및 보안**
- [ ] VPC 및 서브넷 구성
- [ ] 보안 그룹 설정
- [ ] 인터넷 게이트웨이 및 NAT 게이트웨이
- [ ] SSL/TLS 인증서 관리

### **Week 9-11: 애플리케이션 배포**
- [ ] EC2 인스턴스 구성
- [ ] Application Load Balancer 설정
- [ ] 자동 스케일링 그룹 구성
- [ ] 배포 스크립트 작성

### **Week 12-13: 모니터링 및 로깅**
- [ ] CloudWatch 로그 그룹 설정
- [ ] 메트릭 알람 구성
- [ ] 대시보드 생성
- [ ] 로그 분석 및 모니터링

### **Week 14-16: CI/CD 파이프라인**
- [ ] GitHub Actions 설정
- [ ] Terraform 상태 백엔드 구성
- [ ] 자동화된 배포 파이프라인
- [ ] 환경별 배포 전략 구현

---

## 📚 참고 자료

### **공식 문서**
- [Terraform AWS Provider](https://registry.terraform.io/providers/hashicorp/aws/latest/docs)
- [Terraform Docker Provider](https://registry.terraform.io/providers/kreuzwerker/docker/latest/docs)
- [AWS S3 서비스](https://docs.aws.amazon.com/s3/)
- [AWS RDS 서비스](https://docs.aws.amazon.com/rds/)

### **실습 프로젝트**
- 현재 PACS Extension Server의 실제 인프라 구성
- Docker Compose → Terraform 마이그레이션
- AWS 클라우드 인프라 구축
- CI/CD 파이프라인 구축

이 가이드를 따라 학습하시면 현재 프로젝트의 실제 인프라를 Terraform으로 관리할 수 있게 됩니다!


