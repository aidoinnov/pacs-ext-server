# Terraform 핵심 개념 완전 가이드

## 🎯 Terraform이란?

**Terraform**은 HashiCorp에서 개발한 **Infrastructure as Code (IaC)** 도구입니다. 코드로 인프라를 정의하고 관리할 수 있게 해주는 도구로, 클라우드 리소스를 선언적으로 관리할 수 있습니다.

### 주요 특징
- **선언적 (Declarative)**: 원하는 상태를 정의하면 Terraform이 알아서 구현
- **멀티 클라우드**: AWS, Azure, GCP 등 다양한 클라우드 지원
- **상태 관리**: 현재 인프라 상태를 추적하고 관리
- **의존성 관리**: 리소스 간 의존성을 자동으로 해결

---

## 🏗️ 핵심 개념들

### 1. Configuration Language (HCL)

Terraform은 **HCL (HashiCorp Configuration Language)**을 사용합니다.

```hcl
# 기본 문법 예시
resource "aws_instance" "web" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t2.micro"
  
  tags = {
    Name = "HelloWorld"
  }
}
```

**HCL 특징:**
- JSON과 유사하지만 더 읽기 쉬움
- 주석 지원 (`#` 또는 `//`)
- 변수와 함수 사용 가능
- 블록 구조로 리소스 정의

### 2. Provider (프로바이더)

**Provider**는 특정 클라우드나 서비스와 상호작용하는 플러그인입니다.

```hcl
# AWS Provider 설정
provider "aws" {
  region = "us-west-2"
  profile = "default"
}

# Azure Provider 설정
provider "azurerm" {
  features {}
}

# Google Cloud Provider 설정
provider "google" {
  project = "my-project-id"
  region  = "us-central1"
}
```

**주요 Provider들:**
- `aws`: Amazon Web Services
- `azurerm`: Microsoft Azure
- `google`: Google Cloud Platform
- `kubernetes`: Kubernetes 클러스터
- `docker`: Docker 컨테이너
- `local`: 로컬 파일 시스템

### 3. Resource (리소스)

**Resource**는 관리하고자 하는 인프라 구성 요소입니다.

```hcl
# AWS EC2 인스턴스 리소스
resource "aws_instance" "web_server" {
  ami           = var.ami_id
  instance_type = "t2.micro"
  
  vpc_security_group_ids = [aws_security_group.web_sg.id]
  subnet_id              = aws_subnet.public.id
  
  tags = {
    Name        = "Web Server"
    Environment = "production"
  }
}

# AWS 보안 그룹 리소스
resource "aws_security_group" "web_sg" {
  name_prefix = "web-sg-"
  
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}
```

**Resource 구조:**
- `resource "TYPE" "NAME"`: 리소스 타입과 로컬 이름
- `{}`: 리소스 설정 블록
- 속성: 리소스별 고유 설정값

### 4. State (상태)

**State**는 현재 관리 중인 인프라의 실제 상태를 저장하는 파일입니다.

```hcl
# terraform.tfstate 파일 예시 (JSON 형식)
{
  "version": 4,
  "terraform_version": "1.0.0",
  "resources": [
    {
      "mode": "managed",
      "type": "aws_instance",
      "name": "web_server",
      "provider": "provider[\"registry.terraform.io/hashicorp/aws\"]",
      "instances": [
        {
          "schema_version": 1,
          "attributes": {
            "ami": "ami-0c55b159cbfafe1d0",
            "instance_type": "t2.micro",
            "id": "i-1234567890abcdef0"
          }
        }
      ]
    }
  ]
}
```

**State의 역할:**
- 현재 인프라 상태 추적
- 계획(plan) 실행 시 변경사항 계산
- 리소스 간 의존성 관리
- 리소스 삭제 시 참조

### 5. Variables (변수)

**Variables**는 재사용 가능한 값들을 정의하는 방법입니다.

```hcl
# variables.tf
variable "region" {
  description = "AWS region"
  type        = string
  default     = "us-west-2"
}

variable "instance_count" {
  description = "Number of instances"
  type        = number
  default     = 1
}

variable "tags" {
  description = "Tags to apply to resources"
  type        = map(string)
  default     = {
    Environment = "development"
    Project     = "my-project"
  }
}

# 사용 예시
resource "aws_instance" "web" {
  count         = var.instance_count
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t2.micro"
  
  tags = var.tags
}
```

**Variable 타입들:**
- `string`: 문자열
- `number`: 숫자
- `bool`: 불린값
- `list`: 리스트
- `map`: 키-값 쌍
- `object`: 복합 객체
- `any`: 모든 타입

### 6. Outputs (출력)

**Outputs**는 리소스의 정보를 외부로 노출하는 방법입니다.

```hcl
# outputs.tf
output "instance_id" {
  description = "ID of the EC2 instance"
  value       = aws_instance.web.id
}

output "instance_public_ip" {
  description = "Public IP address of the EC2 instance"
  value       = aws_instance.web.public_ip
}

output "instance_public_dns" {
  description = "Public DNS name of the EC2 instance"
  value       = aws_instance.web.public_dns
  sensitive   = false
}
```

### 7. Data Sources (데이터 소스)

**Data Sources**는 기존 리소스의 정보를 조회하는 방법입니다.

```hcl
# 기존 VPC 조회
data "aws_vpc" "existing" {
  filter {
    name   = "tag:Name"
    values = ["my-existing-vpc"]
  }
}

# 기존 서브넷 조회
data "aws_subnet" "existing" {
  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.existing.id]
  }
}

# 사용 예시
resource "aws_instance" "web" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t2.micro"
  subnet_id     = data.aws_subnet.existing.id
}
```

### 8. Modules (모듈)

**Modules**는 재사용 가능한 Terraform 코드의 컨테이너입니다.

```hcl
# modules/ec2/main.tf
variable "instance_type" {
  description = "EC2 instance type"
  type        = string
  default     = "t2.micro"
}

variable "ami_id" {
  description = "AMI ID"
  type        = string
}

resource "aws_instance" "this" {
  ami           = var.ami_id
  instance_type = var.instance_type
  
  tags = {
    Name = "EC2 Instance"
  }
}

output "instance_id" {
  value = aws_instance.this.id
}

# main.tf에서 모듈 사용
module "web_server" {
  source = "./modules/ec2"
  
  instance_type = "t2.small"
  ami_id        = "ami-0c55b159cbfafe1d0"
}
```

---

## 🔄 Terraform 워크플로우

### 1. 초기화 (terraform init)
```bash
terraform init
```
- Provider 다운로드
- 백엔드 설정
- 모듈 다운로드

### 2. 계획 (terraform plan)
```bash
terraform plan
```
- 현재 상태와 원하는 상태 비교
- 변경사항 미리보기
- 실행 계획 생성

### 3. 적용 (terraform apply)
```bash
terraform apply
```
- 실제 인프라 변경사항 적용
- State 파일 업데이트
- 리소스 생성/수정/삭제

### 4. 파괴 (terraform destroy)
```bash
terraform destroy
```
- 모든 리소스 삭제
- State 파일 정리

---

## 🎯 핵심 원칙

### 1. 선언적 접근법
- **명령형**: "서버를 시작하고, 포트를 열고, 방화벽을 설정하라"
- **선언적**: "웹 서버가 포트 80에서 실행되어야 한다"

### 2. 의존성 관리
```hcl
# 명시적 의존성
resource "aws_instance" "web" {
  depends_on = [aws_security_group.web_sg]
  # ...
}

# 암시적 의존성 (참조를 통한)
resource "aws_instance" "web" {
  vpc_security_group_ids = [aws_security_group.web_sg.id]
  # ...
}
```

### 3. 상태 관리
- **로컬 상태**: 단일 사용자, 단일 환경
- **원격 상태**: 팀 협업, 백엔드 저장소 사용

### 4. 멱등성 (Idempotency)
- 같은 설정으로 여러 번 실행해도 결과가 동일
- 현재 상태를 확인하고 필요한 변경만 수행

---

## 🚀 다음 단계

이제 기본 개념을 이해했으니 다음 단계로 진행하세요:

1. **실습 환경 설정** - Terraform CLI 설치
2. **첫 번째 리소스 생성** - 간단한 EC2 인스턴스
3. **변수와 출력 사용** - 동적 설정
4. **모듈 생성** - 재사용 가능한 코드
5. **상태 관리** - 원격 백엔드 설정

이 핵심 개념들을 이해하면 Terraform을 효과적으로 사용할 수 있습니다!
