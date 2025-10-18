# 🌐 VPC 및 네트워킹 가이드

Terraform을 사용하여 AWS VPC와 네트워킹 인프라를 구성하고 관리하는 방법을 학습합니다. PACS 프로젝트의 보안 요구사항에 맞는 3계층 네트워크 아키텍처를 중심으로 다룹니다.

## 📋 목차

1. [VPC 네트워킹이란?](#vpc-네트워킹이란)
2. [기본 VPC 구성](#기본-vpc-구성)
3. [PACS 프로젝트 네트워크 아키텍처](#pacs-프로젝트-네트워크-아키텍처)
4. [보안 그룹 및 NACL](#보안-그룹-및-nacl)
5. [로드 밸런서 및 NAT](#로드-밸런서-및-nat)
6. [실습 및 테스트](#실습-및-테스트)

---

## 🎯 VPC 네트워킹이란?

**Amazon VPC (Virtual Private Cloud)**는 AWS 클라우드에서 논리적으로 격리된 네트워크 환경을 제공합니다.

### 주요 구성 요소
- **VPC**: 논리적으로 격리된 네트워크 환경
- **서브넷**: VPC 내의 IP 주소 범위
- **라우팅 테이블**: 네트워크 트래픽 라우팅 규칙
- **인터넷 게이트웨이**: 인터넷과의 통신
- **NAT 게이트웨이**: 프라이빗 서브넷의 아웃바운드 인터넷 접근

### PACS 프로젝트에서의 활용
- **3계층 아키텍처**: 웹/앱/DB 계층 분리
- **보안 격리**: 각 계층별 네트워크 분리
- **고가용성**: Multi-AZ 배포
- **확장성**: Auto Scaling 그룹 지원

---

## 🔧 기본 VPC 구성

### 1. VPC 생성

#### `vpc-basic.tf`
```hcl
# VPC 생성
resource "aws_vpc" "main" {
  cidr_block           = var.vpc_cidr
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name        = "${var.project_name}-vpc"
    Environment = var.environment
    Project     = var.project_name
  }
}

# 인터넷 게이트웨이
resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id

  tags = {
    Name        = "${var.project_name}-igw"
    Environment = var.environment
    Project     = var.project_name
  }
}

# 기본 라우팅 테이블
resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }

  tags = {
    Name        = "${var.project_name}-public-rt"
    Environment = var.environment
    Project     = var.project_name
  }
}

# 기본 라우팅 테이블 연결
resource "aws_main_route_table_association" "main" {
  vpc_id         = aws_vpc.main.id
  route_table_id = aws_route_table.public.id
}
```

### 2. 서브넷 구성

#### `subnets.tf`
```hcl
# 퍼블릭 서브넷
resource "aws_subnet" "public" {
  count = length(var.availability_zones)

  vpc_id                  = aws_vpc.main.id
  cidr_block              = var.public_subnet_cidrs[count.index]
  availability_zone       = var.availability_zones[count.index]
  map_public_ip_on_launch = true

  tags = {
    Name        = "${var.project_name}-public-subnet-${count.index + 1}"
    Environment = var.environment
    Project     = var.project_name
    Type        = "Public"
    Tier        = "Web"
  }
}

# 프라이빗 서브넷 (앱 계층)
resource "aws_subnet" "private_app" {
  count = length(var.availability_zones)

  vpc_id            = aws_vpc.main.id
  cidr_block        = var.private_app_subnet_cidrs[count.index]
  availability_zone = var.availability_zones[count.index]

  tags = {
    Name        = "${var.project_name}-private-app-subnet-${count.index + 1}"
    Environment = var.environment
    Project     = var.project_name
    Type        = "Private"
    Tier        = "Application"
  }
}

# 프라이빗 서브넷 (DB 계층)
resource "aws_subnet" "private_db" {
  count = length(var.availability_zones)

  vpc_id            = aws_vpc.main.id
  cidr_block        = var.private_db_subnet_cidrs[count.index]
  availability_zone = var.availability_zones[count.index]

  tags = {
    Name        = "${var.project_name}-private-db-subnet-${count.index + 1}"
    Environment = var.environment
    Project     = var.project_name
    Type        = "Private"
    Tier        = "Database"
  }
}

# 퍼블릭 서브넷 라우팅 테이블 연결
resource "aws_route_table_association" "public" {
  count = length(aws_subnet.public)

  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}
```

### 3. NAT 게이트웨이

#### `nat-gateway.tf`
```hcl
# Elastic IP for NAT Gateway
resource "aws_eip" "nat" {
  count = length(var.availability_zones)

  domain = "vpc"

  tags = {
    Name        = "${var.project_name}-nat-eip-${count.index + 1}"
    Environment = var.environment
    Project     = var.project_name
  }

  depends_on = [aws_internet_gateway.main]
}

# NAT Gateway
resource "aws_nat_gateway" "main" {
  count = length(var.availability_zones)

  allocation_id = aws_eip.nat[count.index].id
  subnet_id     = aws_subnet.public[count.index].id

  tags = {
    Name        = "${var.project_name}-nat-gateway-${count.index + 1}"
    Environment = var.environment
    Project     = var.project_name
  }

  depends_on = [aws_internet_gateway.main]
}

# 프라이빗 라우팅 테이블 (앱 계층)
resource "aws_route_table" "private_app" {
  count = length(var.availability_zones)

  vpc_id = aws_vpc.main.id

  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.main[count.index].id
  }

  tags = {
    Name        = "${var.project_name}-private-app-rt-${count.index + 1}"
    Environment = var.environment
    Project     = var.project_name
    Tier        = "Application"
  }
}

# 프라이빗 라우팅 테이블 (DB 계층)
resource "aws_route_table" "private_db" {
  vpc_id = aws_vpc.main.id

  # DB 계층은 인터넷 접근 불가
  tags = {
    Name        = "${var.project_name}-private-db-rt"
    Environment = var.environment
    Project     = var.project_name
    Tier        = "Database"
  }
}

# 프라이빗 서브넷 라우팅 테이블 연결
resource "aws_route_table_association" "private_app" {
  count = length(aws_subnet.private_app)

  subnet_id      = aws_subnet.private_app[count.index].id
  route_table_id = aws_route_table.private_app[count.index].id
}

resource "aws_route_table_association" "private_db" {
  count = length(aws_subnet.private_db)

  subnet_id      = aws_subnet.private_db[count.index].id
  route_table_id = aws_route_table.private_db.id
}
```

---

## 🏥 PACS 프로젝트 네트워크 아키텍처

### 1. 3계층 아키텍처

#### `pacs-network-architecture.tf`
```hcl
# PACS 웹 계층 서브넷
resource "aws_subnet" "pacs_web" {
  count = length(var.availability_zones)

  vpc_id                  = aws_vpc.main.id
  cidr_block              = var.pacs_web_subnet_cidrs[count.index]
  availability_zone       = var.availability_zones[count.index]
  map_public_ip_on_launch = true

  tags = {
    Name        = "${var.project_name}-web-subnet-${count.index + 1}"
    Environment = var.environment
    Project     = var.project_name
    Tier        = "Web"
    Purpose     = "Load Balancer, Web Servers"
  }
}

# PACS 앱 계층 서브넷
resource "aws_subnet" "pacs_app" {
  count = length(var.availability_zones)

  vpc_id            = aws_vpc.main.id
  cidr_block        = var.pacs_app_subnet_cidrs[count.index]
  availability_zone = var.availability_zones[count.index]

  tags = {
    Name        = "${var.project_name}-app-subnet-${count.index + 1}"
    Environment = var.environment
    Project     = var.project_name
    Tier        = "Application"
    Purpose     = "PACS Backend, Keycloak"
  }
}

# PACS DB 계층 서브넷
resource "aws_subnet" "pacs_db" {
  count = length(var.availability_zones)

  vpc_id            = aws_vpc.main.id
  cidr_block        = var.pacs_db_subnet_cidrs[count.index]
  availability_zone = var.availability_zones[count.index]

  tags = {
    Name        = "${var.project_name}-db-subnet-${count.index + 1}"
    Environment = var.environment
    Project     = var.project_name
    Tier        = "Database"
    Purpose     = "RDS, ElastiCache"
  }
}

# PACS 관리 계층 서브넷
resource "aws_subnet" "pacs_mgmt" {
  count = length(var.availability_zones)

  vpc_id            = aws_vpc.main.id
  cidr_block        = var.pacs_mgmt_subnet_cidrs[count.index]
  availability_zone = var.availability_zones[count.index]

  tags = {
    Name        = "${var.project_name}-mgmt-subnet-${count.index + 1}"
    Environment = var.environment
    Project     = var.project_name
    Tier        = "Management"
    Purpose     = "Bastion Host, Monitoring"
  }
}
```

### 2. 변수 정의

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

# VPC 설정
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

# 서브넷 CIDR 블록
variable "pacs_web_subnet_cidrs" {
  description = "CIDR blocks for PACS web subnets"
  type        = list(string)
  default     = ["10.0.1.0/24", "10.0.2.0/24"]
}

variable "pacs_app_subnet_cidrs" {
  description = "CIDR blocks for PACS app subnets"
  type        = list(string)
  default     = ["10.0.10.0/24", "10.0.20.0/24"]
}

variable "pacs_db_subnet_cidrs" {
  description = "CIDR blocks for PACS DB subnets"
  type        = list(string)
  default     = ["10.0.100.0/24", "10.0.200.0/24"]
}

variable "pacs_mgmt_subnet_cidrs" {
  description = "CIDR blocks for PACS management subnets"
  type        = list(string)
  default     = ["10.0.250.0/24", "10.0.251.0/24"]
}

# 기존 서브넷 (호환성)
variable "public_subnet_cidrs" {
  description = "CIDR blocks for public subnets"
  type        = list(string)
  default     = ["10.0.1.0/24", "10.0.2.0/24"]
}

variable "private_app_subnet_cidrs" {
  description = "CIDR blocks for private app subnets"
  type        = list(string)
  default     = ["10.0.10.0/24", "10.0.20.0/24"]
}

variable "private_db_subnet_cidrs" {
  description = "CIDR blocks for private DB subnets"
  type        = list(string)
  default     = ["10.0.100.0/24", "10.0.200.0/24"]
}
```

### 3. DNS 설정

#### `dns.tf`
```hcl
# Private Hosted Zone
resource "aws_route53_zone" "private" {
  name = "${var.environment}.pacs.local"

  vpc {
    vpc_id = aws_vpc.main.id
  }

  tags = {
    Name        = "${var.project_name}-private-zone"
    Environment = var.environment
    Project     = var.project_name
  }
}

# DNS 레코드
resource "aws_route53_record" "pacs_api" {
  zone_id = aws_route53_zone.private.zone_id
  name    = "api.${var.environment}.pacs.local"
  type    = "A"
  ttl     = 300
  records = [aws_lb.pacs_app.private_ip]
}

resource "aws_route53_record" "pacs_db" {
  zone_id = aws_route53_zone.private.zone_id
  name    = "db.${var.environment}.pacs.local"
  type    = "CNAME"
  ttl     = 300
  records = [aws_db_instance.pacs_main.endpoint]
}

resource "aws_route53_record" "keycloak" {
  zone_id = aws_route53_zone.private.zone_id
  name    = "auth.${var.environment}.pacs.local"
  type    = "A"
  ttl     = 300
  records = [aws_lb.pacs_app.private_ip]
}
```

---

## 🔒 보안 그룹 및 NACL

### 1. 보안 그룹 구성

#### `security-groups.tf`
```hcl
# 웹 계층 보안 그룹
resource "aws_security_group" "pacs_web" {
  name_prefix = "${var.project_name}-web-"
  vpc_id      = aws_vpc.main.id

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
    cidr_blocks = var.admin_cidr_blocks
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name        = "${var.project_name}-web-sg"
    Environment = var.environment
    Project     = var.project_name
    Tier        = "Web"
  }
}

# 앱 계층 보안 그룹
resource "aws_security_group" "pacs_app" {
  name_prefix = "${var.project_name}-app-"
  vpc_id      = aws_vpc.main.id

  # 웹 계층에서만 접근
  ingress {
    from_port       = 8080
    to_port         = 8080
    protocol        = "tcp"
    security_groups = [aws_security_group.pacs_web.id]
  }

  # Keycloak 포트
  ingress {
    from_port       = 8080
    to_port         = 8080
    protocol        = "tcp"
    security_groups = [aws_security_group.pacs_web.id]
  }

  # 데이터베이스 접근
  egress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.pacs_db.id]
  }

  # Redis 접근
  egress {
    from_port       = 6379
    to_port         = 6379
    protocol        = "tcp"
    security_groups = [aws_security_group.pacs_cache.id]
  }

  # S3 접근
  egress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name        = "${var.project_name}-app-sg"
    Environment = var.environment
    Project     = var.project_name
    Tier        = "Application"
  }
}

# DB 계층 보안 그룹
resource "aws_security_group" "pacs_db" {
  name_prefix = "${var.project_name}-db-"
  vpc_id      = aws_vpc.main.id

  # 앱 계층에서만 접근
  ingress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.pacs_app.id]
  }

  # 관리 계층에서 접근 (백업, 모니터링)
  ingress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.pacs_mgmt.id]
  }

  tags = {
    Name        = "${var.project_name}-db-sg"
    Environment = var.environment
    Project     = var.project_name
    Tier        = "Database"
  }
}

# 캐시 계층 보안 그룹
resource "aws_security_group" "pacs_cache" {
  name_prefix = "${var.project_name}-cache-"
  vpc_id      = aws_vpc.main.id

  # 앱 계층에서만 접근
  ingress {
    from_port       = 6379
    to_port         = 6379
    protocol        = "tcp"
    security_groups = [aws_security_group.pacs_app.id]
  }

  tags = {
    Name        = "${var.project_name}-cache-sg"
    Environment = var.environment
    Project     = var.project_name
    Tier        = "Cache"
  }
}

# 관리 계층 보안 그룹
resource "aws_security_group" "pacs_mgmt" {
  name_prefix = "${var.project_name}-mgmt-"
  vpc_id      = aws_vpc.main.id

  # SSH (관리자만)
  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = var.admin_cidr_blocks
  }

  # 모든 아웃바운드 허용
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name        = "${var.project_name}-mgmt-sg"
    Environment = var.environment
    Project     = var.project_name
    Tier        = "Management"
  }
}
```

### 2. Network ACL 설정

#### `network-acls.tf`
```hcl
# 퍼블릭 서브넷 NACL
resource "aws_network_acl" "public" {
  vpc_id = aws_vpc.main.id

  # HTTP
  ingress {
    protocol   = "tcp"
    rule_no    = 100
    action     = "allow"
    cidr_block = "0.0.0.0/0"
    from_port  = 80
    to_port    = 80
  }

  # HTTPS
  ingress {
    protocol   = "tcp"
    rule_no    = 110
    action     = "allow"
    cidr_block = "0.0.0.0/0"
    from_port  = 443
    to_port    = 443
  }

  # SSH
  ingress {
    protocol   = "tcp"
    rule_no    = 120
    action     = "allow"
    cidr_block = var.admin_cidr_blocks[0]
    from_port  = 22
    to_port    = 22
  }

  # Ephemeral ports
  ingress {
    protocol   = "tcp"
    rule_no    = 130
    action     = "allow"
    cidr_block = "0.0.0.0/0"
    from_port  = 1024
    to_port    = 65535
  }

  # All outbound traffic
  egress {
    protocol   = "-1"
    rule_no    = 100
    action     = "allow"
    cidr_block = "0.0.0.0/0"
    from_port  = 0
    to_port    = 0
  }

  tags = {
    Name        = "${var.project_name}-public-nacl"
    Environment = var.environment
    Project     = var.project_name
  }
}

# 프라이빗 서브넷 NACL
resource "aws_network_acl" "private" {
  vpc_id = aws_vpc.main.id

  # HTTP from public subnets
  ingress {
    protocol   = "tcp"
    rule_no    = 100
    action     = "allow"
    cidr_block = var.pacs_web_subnet_cidrs[0]
    from_port  = 80
    to_port    = 80
  }

  ingress {
    protocol   = "tcp"
    rule_no    = 110
    action     = "allow"
    cidr_block = var.pacs_web_subnet_cidrs[1]
    from_port  = 80
    to_port    = 80
  }

  # HTTPS from public subnets
  ingress {
    protocol   = "tcp"
    rule_no    = 120
    action     = "allow"
    cidr_block = var.pacs_web_subnet_cidrs[0]
    from_port  = 443
    to_port    = 443
  }

  ingress {
    protocol   = "tcp"
    rule_no    = 130
    action     = "allow"
    cidr_block = var.pacs_web_subnet_cidrs[1]
    from_port  = 443
    to_port    = 443
  }

  # Ephemeral ports
  ingress {
    protocol   = "tcp"
    rule_no    = 140
    action     = "allow"
    cidr_block = "0.0.0.0/0"
    from_port  = 1024
    to_port    = 65535
  }

  # All outbound traffic
  egress {
    protocol   = "-1"
    rule_no    = 100
    action     = "allow"
    cidr_block = "0.0.0.0/0"
    from_port  = 0
    to_port    = 0
  }

  tags = {
    Name        = "${var.project_name}-private-nacl"
    Environment = var.environment
    Project     = var.project_name
  }
}

# NACL 연결
resource "aws_network_acl_association" "public" {
  count = length(aws_subnet.pacs_web)

  subnet_id      = aws_subnet.pacs_web[count.index].id
  network_acl_id = aws_network_acl.public.id
}

resource "aws_network_acl_association" "private_app" {
  count = length(aws_subnet.pacs_app)

  subnet_id      = aws_subnet.pacs_app[count.index].id
  network_acl_id = aws_network_acl.private.id
}

resource "aws_network_acl_association" "private_db" {
  count = length(aws_subnet.pacs_db)

  subnet_id      = aws_subnet.pacs_db[count.index].id
  network_acl_id = aws_network_acl.private.id
}
```

---

## ⚖️ 로드 밸런서 및 NAT

### 1. Application Load Balancer

#### `load-balancer.tf`
```hcl
# Application Load Balancer
resource "aws_lb" "pacs_web" {
  name               = "${var.project_name}-web-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.pacs_web.id]
  subnets            = aws_subnet.pacs_web[*].id

  enable_deletion_protection = var.environment == "production"

  tags = {
    Name        = "${var.project_name}-web-alb"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Target Group
resource "aws_lb_target_group" "pacs_web" {
  name     = "${var.project_name}-web-tg"
  port     = 80
  protocol = "HTTP"
  vpc_id   = aws_vpc.main.id

  health_check {
    enabled             = true
    healthy_threshold   = 2
    interval            = 30
    matcher             = "200"
    path                = "/health"
    port                = "traffic-port"
    protocol            = "HTTP"
    timeout             = 5
    unhealthy_threshold = 2
  }

  tags = {
    Name        = "${var.project_name}-web-tg"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Listener
resource "aws_lb_listener" "pacs_web" {
  load_balancer_arn = aws_lb.pacs_web.arn
  port              = "80"
  protocol          = "HTTP"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.pacs_web.arn
  }
}

# HTTPS Listener (SSL 인증서 필요)
resource "aws_lb_listener" "pacs_web_https" {
  count = var.enable_https ? 1 : 0

  load_balancer_arn = aws_lb.pacs_web.arn
  port              = "443"
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS-1-2-2017-01"
  certificate_arn   = var.ssl_certificate_arn

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.pacs_web.arn
  }
}
```

### 2. Internal Load Balancer

#### `internal-load-balancer.tf`
```hcl
# Internal Application Load Balancer
resource "aws_lb" "pacs_app" {
  name               = "${var.project_name}-app-alb"
  internal           = true
  load_balancer_type = "application"
  security_groups    = [aws_security_group.pacs_app.id]
  subnets            = aws_subnet.pacs_app[*].id

  enable_deletion_protection = var.environment == "production"

  tags = {
    Name        = "${var.project_name}-app-alb"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Target Group for PACS Backend
resource "aws_lb_target_group" "pacs_backend" {
  name     = "${var.project_name}-backend-tg"
  port     = 8080
  protocol = "HTTP"
  vpc_id   = aws_vpc.main.id

  health_check {
    enabled             = true
    healthy_threshold   = 2
    interval            = 30
    matcher             = "200"
    path                = "/health"
    port                = "traffic-port"
    protocol            = "HTTP"
    timeout             = 5
    unhealthy_threshold = 2
  }

  tags = {
    Name        = "${var.project_name}-backend-tg"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Target Group for Keycloak
resource "aws_lb_target_group" "keycloak" {
  name     = "${var.project_name}-keycloak-tg"
  port     = 8080
  protocol = "HTTP"
  vpc_id   = aws_vpc.main.id

  health_check {
    enabled             = true
    healthy_threshold   = 2
    interval            = 30
    matcher             = "200"
    path                = "/auth/health"
    port                = "traffic-port"
    protocol            = "HTTP"
    timeout             = 5
    unhealthy_threshold = 2
  }

  tags = {
    Name        = "${var.project_name}-keycloak-tg"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Listener
resource "aws_lb_listener" "pacs_app" {
  load_balancer_arn = aws_lb.pacs_app.arn
  port              = "80"
  protocol          = "HTTP"

  default_action {
    type = "fixed-response"
    fixed_response {
      content_type = "text/plain"
      message_body = "Not Found"
      status_code  = "404"
    }
  }
}

# PACS Backend Listener Rule
resource "aws_lb_listener_rule" "pacs_backend" {
  listener_arn = aws_lb_listener.pacs_app.arn
  priority     = 100

  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.pacs_backend.arn
  }

  condition {
    path_pattern {
      values = ["/api/*"]
    }
  }
}

# Keycloak Listener Rule
resource "aws_lb_listener_rule" "keycloak" {
  listener_arn = aws_lb_listener.pacs_app.arn
  priority     = 200

  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.keycloak.arn
  }

  condition {
    path_pattern {
      values = ["/auth/*"]
    }
  }
}
```

---

## 🧪 실습 및 테스트

### 1. VPC 생성 테스트

#### `test-vpc-creation.sh`
```bash
#!/bin/bash
# VPC 생성 테스트 스크립트

echo "Testing VPC creation..."

# Terraform 초기화
echo "1. Initializing Terraform..."
terraform init

# Terraform 검증
echo "2. Validating configuration..."
terraform validate

# VPC 생성
echo "3. Creating VPC..."
terraform apply -target=aws_vpc.main -auto-approve

# 인터넷 게이트웨이 생성
echo "4. Creating Internet Gateway..."
terraform apply -target=aws_internet_gateway.main -auto-approve

# 서브넷 생성
echo "5. Creating subnets..."
terraform apply -target=aws_subnet.pacs_web -auto-approve
terraform apply -target=aws_subnet.pacs_app -auto-approve
terraform apply -target=aws_subnet.pacs_db -auto-approve

# VPC 확인
echo "6. Verifying VPC creation..."
aws ec2 describe-vpcs --filters "Name=tag:Name,Values=pacs-vpc"

echo "VPC creation test completed! 🎉"
```

### 2. 네트워크 연결 테스트

#### `test-network-connectivity.sh`
```bash
#!/bin/bash
# 네트워크 연결 테스트 스크립트

echo "Testing network connectivity..."

# VPC ID 가져오기
VPC_ID=$(aws ec2 describe-vpcs \
  --filters "Name=tag:Name,Values=pacs-vpc" \
  --query 'Vpcs[0].VpcId' \
  --output text)

echo "VPC ID: $VPC_ID"

# 서브넷 확인
echo "1. Checking subnets..."
aws ec2 describe-subnets \
  --filters "Name=vpc-id,Values=$VPC_ID" \
  --query 'Subnets[].{SubnetId:SubnetId,CidrBlock:CidrBlock,AvailabilityZone:AvailabilityZone,Tags:Tags[?Key==`Tier`].Value|[0]}'

# 라우팅 테이블 확인
echo "2. Checking route tables..."
aws ec2 describe-route-tables \
  --filters "Name=vpc-id,Values=$VPC_ID" \
  --query 'RouteTables[].{RouteTableId:RouteTableId,Routes:Routes[?DestinationCidrBlock==`0.0.0.0/0`]}'

# 보안 그룹 확인
echo "3. Checking security groups..."
aws ec2 describe-security-groups \
  --filters "Name=vpc-id,Values=$VPC_ID" \
  --query 'SecurityGroups[].{GroupId:GroupId,GroupName:GroupName,Description:Description}'

echo "Network connectivity test completed! 🎉"
```

### 3. 로드 밸런서 테스트

#### `test-load-balancer.sh`
```bash
#!/bin/bash
# 로드 밸런서 테스트 스크립트

echo "Testing load balancer..."

# ALB 확인
echo "1. Checking Application Load Balancer..."
aws elbv2 describe-load-balancers \
  --names pacs-web-alb \
  --query 'LoadBalancers[].{LoadBalancerArn:LoadBalancerArn,DNSName:DNSName,State:State}'

# Target Group 확인
echo "2. Checking Target Groups..."
aws elbv2 describe-target-groups \
  --names pacs-web-tg \
  --query 'TargetGroups[].{TargetGroupArn:TargetGroupArn,Port:Port,Protocol:Protocol,HealthCheckPath:HealthCheckPath}'

# Listener 확인
echo "3. Checking Listeners..."
aws elbv2 describe-listeners \
  --load-balancer-arn $(aws elbv2 describe-load-balancers --names pacs-web-alb --query 'LoadBalancers[0].LoadBalancerArn' --output text) \
  --query 'Listeners[].{Port:Port,Protocol:Protocol,DefaultActions:DefaultActions[0].Type}'

echo "Load balancer test completed! 🎉"
```

### 4. 보안 설정 테스트

#### `test-security.sh`
```bash
#!/bin/bash
# 보안 설정 테스트 스크립트

echo "Testing security configuration..."

# 보안 그룹 확인
echo "1. Checking security groups..."
aws ec2 describe-security-groups \
  --filters "Name=group-name,Values=pacs-*" \
  --query 'SecurityGroups[].{GroupName:GroupName,GroupId:GroupId,IngressRules:length(IpPermissions),EgressRules:length(IpPermissionsEgress)}'

# NACL 확인
echo "2. Checking Network ACLs..."
aws ec2 describe-network-acls \
  --filters "Name=tag:Name,Values=pacs-*" \
  --query 'NetworkAcls[].{NetworkAclId:NetworkAclId,IsDefault:IsDefault,Entries:length(Entries)}'

# VPC Flow Logs 확인
echo "3. Checking VPC Flow Logs..."
aws ec2 describe-flow-logs \
  --filter "Name=resource-type,Values=VPC" \
  --query 'FlowLogs[].{FlowLogId:FlowLogId,FlowLogStatus:FlowLogStatus,LogDestination:LogDestination}'

echo "Security test completed! 🎉"
```

---

## 🔧 문제 해결

### 1. 서브넷 CIDR 충돌

**증상**: 서브넷 CIDR 블록이 겹침
```
Error: InvalidSubnet.Conflict: The CIDR '10.0.1.0/24' conflicts with another subnet
```

**해결 방법**:
```hcl
# CIDR 블록 확인
variable "pacs_web_subnet_cidrs" {
  description = "CIDR blocks for PACS web subnets"
  type        = list(string)
  default     = ["10.0.1.0/24", "10.0.2.0/24"]  # 겹치지 않는 CIDR 사용
}

variable "pacs_app_subnet_cidrs" {
  description = "CIDR blocks for PACS app subnets"
  type        = list(string)
  default     = ["10.0.10.0/24", "10.0.20.0/24"]  # 다른 범위 사용
}
```

### 2. 라우팅 테이블 오류

**증상**: 라우팅 테이블 연결 실패
```
Error: InvalidRouteTableID.NotFound: The route table ID 'rt-xxx' does not exist
```

**해결 방법**:
```hcl
# 라우팅 테이블 생성 후 연결
resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id
  # ... 라우팅 규칙 ...
}

resource "aws_route_table_association" "public" {
  subnet_id      = aws_subnet.public.id
  route_table_id = aws_route_table.public.id  # 생성된 라우팅 테이블 사용
}
```

### 3. 보안 그룹 규칙 오류

**증상**: 보안 그룹 규칙이 너무 제한적
```
Error: Security group rule conflicts with existing rule
```

**해결 방법**:
```hcl
# 보안 그룹 규칙 정리
resource "aws_security_group" "pacs_web" {
  name_prefix = "${var.project_name}-web-"
  vpc_id      = aws_vpc.main.id

  # 필요한 포트만 열기
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # 중복 규칙 제거
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}
```

---

## 📚 다음 단계

이제 VPC와 네트워킹을 성공적으로 설정했으니 다음 문서들을 학습하세요:

1. **EKS 클러스터 구성** - 컨테이너 오케스트레이션
2. **Application Load Balancer** - 로드 밸런싱 설정
3. **Auto Scaling 그룹** - 자동 스케일링 설정

---

## 📖 참고 자료

- [AWS VPC 공식 문서](https://docs.aws.amazon.com/vpc/)
- [VPC 네트워킹 가이드](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_Networking.html)
- [보안 그룹 규칙](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html)

이제 PACS 프로젝트의 안전하고 확장 가능한 네트워크 인프라가 준비되었습니다! 🚀


