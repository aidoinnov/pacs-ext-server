# 🐳 Docker Provider 기초 가이드

Terraform에서 Docker 컨테이너를 관리하기 위한 Docker Provider 사용법을 학습합니다.

## 📋 목차

1. [Docker Provider란?](#docker-provider란)
2. [설치 및 설정](#설치-및-설정)
3. [기본 리소스 타입](#기본-리소스-타입)
4. [실습 예제](#실습-예제)
5. [모범 사례](#모범-사례)
6. [문제 해결](#문제-해결)

---

## 🎯 Docker Provider란?

**Docker Provider**는 Terraform에서 Docker 컨테이너, 이미지, 볼륨, 네트워크 등을 관리할 수 있게 해주는 플러그인입니다.

### 주요 특징
- **로컬 Docker 데몬과 연동**: Docker Desktop 또는 Docker Engine과 통신
- **컨테이너 라이프사이클 관리**: 생성, 수정, 삭제 자동화
- **상태 추적**: 현재 Docker 상태를 Terraform state로 관리
- **의존성 관리**: 컨테이너 간 의존성 자동 해결

### 사용 사례
- 로컬 개발 환경 구성
- 테스트 환경 자동화
- Docker Compose 대체
- CI/CD 파이프라인에서 컨테이너 관리

---

## 🔧 설치 및 설정

### 1. 사전 요구사항

```bash
# Docker 설치 확인
docker --version
# Docker version 24.0.7, build afdd53b

# Docker 데몬 실행 확인
docker ps
# CONTAINER ID   IMAGE     COMMAND   CREATED   STATUS    PORTS     NAMES
```

### 2. Terraform 프로젝트 초기화

```bash
# 프로젝트 디렉토리 생성
mkdir terraform-docker-guide
cd terraform-docker-guide

# Terraform 초기화
terraform init
```

### 3. Provider 설정

#### `providers.tf`
```hcl
terraform {
  required_providers {
    docker = {
      source  = "kreuzwerker/docker"
      version = "~> 3.0"
    }
  }
}

provider "docker" {
  # Docker 데몬이 기본 위치에 있다면 설정 불필요
  # host = "unix:///var/run/docker.sock"
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
- Finding kreuzwerker/docker versions matching "~> 3.0"...
- Installing kreuzwerker/docker v3.0.2...
- Installed kreuzwerker/docker v3.0.2 (self-signed key, key ID 0x...)

Terraform has been successfully initialized!
```

---

## 🏗️ 기본 리소스 타입

### 1. docker_image

Docker 이미지를 관리합니다.

```hcl
resource "docker_image" "nginx" {
  name = "nginx:alpine"
  
  # 이미지가 없으면 자동으로 pull
  keep_locally = false
}
```

**주요 속성:**
- `name`: 이미지 이름 (태그 포함)
- `keep_locally`: Terraform destroy 시 이미지 유지 여부
- `pull_triggers`: 이미지 재다운로드 트리거

### 2. docker_container

Docker 컨테이너를 관리합니다.

```hcl
resource "docker_container" "web" {
  name  = "nginx-web"
  image = docker_image.nginx.image_id
  
  ports {
    internal = 80
    external = 8080
  }
  
  env = [
    "NGINX_HOST=localhost",
    "NGINX_PORT=80"
  ]
  
  volumes {
    volume_name    = docker_volume.web_data.name
    container_path = "/var/www/html"
  }
}
```

**주요 속성:**
- `name`: 컨테이너 이름
- `image`: 사용할 이미지 ID
- `ports`: 포트 매핑
- `env`: 환경변수
- `volumes`: 볼륨 마운트
- `restart`: 재시작 정책

### 3. docker_volume

Docker 볼륨을 관리합니다.

```hcl
resource "docker_volume" "web_data" {
  name = "web-data"
  
  driver = "local"
  
  driver_opts = {
    type   = "none"
    o      = "bind"
    device = "/path/to/host/directory"
  }
}
```

**주요 속성:**
- `name`: 볼륨 이름
- `driver`: 볼륨 드라이버
- `driver_opts`: 드라이버 옵션

### 4. docker_network

Docker 네트워크를 관리합니다.

```hcl
resource "docker_network" "app_network" {
  name = "app-network"
  
  driver = "bridge"
  
  ipam_config {
    subnet = "172.20.0.0/16"
  }
}
```

**주요 속성:**
- `name`: 네트워크 이름
- `driver`: 네트워크 드라이버
- `ipam_config`: IP 주소 관리 설정

---

## 🚀 실습 예제

### 예제 1: 기본 웹 서버 구성

#### `main.tf`
```hcl
terraform {
  required_providers {
    docker = {
      source  = "kreuzwerker/docker"
      version = "~> 3.0"
    }
  }
}

provider "docker" {}

# Nginx 이미지
resource "docker_image" "nginx" {
  name = "nginx:alpine"
}

# 웹 서버 컨테이너
resource "docker_container" "web" {
  name  = "nginx-web"
  image = docker_image.nginx.image_id
  
  ports {
    internal = 80
    external = 8080
  }
  
  restart = "unless-stopped"
  
  labels {
    label = "environment"
    value = "development"
  }
}
```

#### 실행 명령어
```bash
# 계획 확인
terraform plan

# 적용
terraform apply

# 컨테이너 확인
docker ps

# 웹 서버 접속 테스트
curl http://localhost:8080
```

### 예제 2: PostgreSQL 데이터베이스 구성

#### `postgres.tf`
```hcl
# PostgreSQL 이미지
resource "docker_image" "postgres" {
  name = "postgres:16-alpine"
}

# 데이터 볼륨
resource "docker_volume" "postgres_data" {
  name = "postgres-data"
}

# PostgreSQL 컨테이너
resource "docker_container" "postgres" {
  name  = "postgres-db"
  image = docker_image.postgres.image_id
  
  env = [
    "POSTGRES_DB=pacs_db",
    "POSTGRES_USER=admin",
    "POSTGRES_PASSWORD=admin123"
  ]
  
  ports {
    internal = 5432
    external = 5432
  }
  
  volumes {
    volume_name    = docker_volume.postgres_data.name
    container_path = "/var/lib/postgresql/data"
  }
  
  restart = "unless-stopped"
  
  healthcheck {
    test     = ["CMD-SHELL", "pg_isready -U admin -d pacs_db"]
    interval = "10s"
    timeout  = "5s"
    retries  = 5
  }
}
```

### 예제 3: 네트워크를 사용한 멀티 컨테이너 구성

#### `network.tf`
```hcl
# 애플리케이션 네트워크
resource "docker_network" "app_network" {
  name = "pacs-network"
  
  driver = "bridge"
  
  ipam_config {
    subnet = "172.20.0.0/16"
  }
}

# 웹 서버 (네트워크 연결)
resource "docker_container" "web" {
  name  = "pacs-web"
  image = docker_image.nginx.image_id
  
  networks_advanced {
    name = docker_network.app_network.name
  }
  
  ports {
    internal = 80
    external = 8080
  }
}

# 데이터베이스 (네트워크 연결)
resource "docker_container" "postgres" {
  name  = "pacs-db"
  image = docker_image.postgres.image_id
  
  networks_advanced {
    name = docker_network.app_network.name
  }
  
  env = [
    "POSTGRES_DB=pacs_db",
    "POSTGRES_USER=admin",
    "POSTGRES_PASSWORD=admin123"
  ]
  
  # 외부 포트 노출하지 않음 (내부 네트워크에서만 접근)
}
```

---

## 📚 모범 사례

### 1. 리소스 네이밍 규칙

```hcl
# 좋은 예
resource "docker_container" "pacs_postgres" {
  name = "pacs-postgres-${var.environment}"
}

# 나쁜 예
resource "docker_container" "db" {
  name = "container1"
}
```

### 2. 환경변수 관리

```hcl
# variables.tf
variable "postgres_password" {
  description = "PostgreSQL password"
  type        = string
  sensitive   = true
}

# main.tf
resource "docker_container" "postgres" {
  env = [
    "POSTGRES_PASSWORD=${var.postgres_password}"
  ]
}
```

### 3. 볼륨 데이터 보존

```hcl
# 데이터 볼륨 (destroy 시 유지)
resource "docker_volume" "postgres_data" {
  name = "postgres-data"
  
  lifecycle {
    prevent_destroy = true
  }
}

# 임시 볼륨 (destroy 시 삭제)
resource "docker_volume" "temp_data" {
  name = "temp-data"
}
```

### 4. 헬스체크 설정

```hcl
resource "docker_container" "web" {
  # ... 기타 설정
  
  healthcheck {
    test     = ["CMD", "curl", "-f", "http://localhost:80/health"]
    interval = "30s"
    timeout  = "10s"
    retries  = 3
    start_period = "40s"
  }
}
```

### 5. 리소스 의존성 관리

```hcl
# 명시적 의존성
resource "docker_container" "app" {
  depends_on = [
    docker_container.database,
    docker_container.redis
  ]
  
  # ... 기타 설정
}

# 암시적 의존성 (권장)
resource "docker_container" "app" {
  env = [
    "DATABASE_URL=postgresql://admin:${docker_container.database.env[0]}@database:5432/pacs_db"
  ]
}
```

---

## 🔧 문제 해결

### 1. Docker 데몬 연결 오류

**오류 메시지:**
```
Error: Cannot connect to the Docker daemon at unix:///var/run/docker.sock
```

**해결 방법:**
```bash
# Docker 데몬 상태 확인
sudo systemctl status docker

# Docker 데몬 시작
sudo systemctl start docker

# 사용자를 docker 그룹에 추가
sudo usermod -aG docker $USER
newgrp docker
```

### 2. 포트 충돌 오류

**오류 메시지:**
```
Error: port is already allocated
```

**해결 방법:**
```bash
# 사용 중인 포트 확인
netstat -tulpn | grep :8080

# 다른 포트 사용
ports {
  internal = 80
  external = 8081  # 8080 대신 8081 사용
}
```

### 3. 이미지 Pull 실패

**오류 메시지:**
```
Error: pull access denied for nginx
```

**해결 방법:**
```bash
# Docker 로그인
docker login

# 이미지 수동 pull
docker pull nginx:alpine

# Terraform 재실행
terraform apply
```

### 4. 볼륨 권한 오류

**오류 메시지:**
```
Error: permission denied
```

**해결 방법:**
```hcl
# 볼륨 드라이버 옵션 설정
resource "docker_volume" "data" {
  name = "app-data"
  
  driver_opts = {
    type   = "none"
    o      = "bind"
    device = "/home/user/app-data"
  }
}
```

---

## 📖 다음 단계

이제 Docker Provider의 기본을 이해했으니 다음 문서들을 학습하세요:

1. **현재 docker-compose.yml 분석** - PACS 프로젝트의 Docker 설정 분석
2. **Terraform으로 PostgreSQL 구성하기** - 실제 데이터베이스 설정
3. **Docker 네트워크 구성** - 컨테이너 간 통신 설정

---

## 📚 참고 자료

- [Terraform Docker Provider 공식 문서](https://registry.terraform.io/providers/kreuzwerker/docker/latest/docs)
- [Docker 공식 문서](https://docs.docker.com/)
- [Terraform 공식 문서](https://developer.hashicorp.com/terraform/docs)

이 가이드를 통해 Docker Provider의 기본을 마스터하고, 다음 단계로 진행할 준비가 되었습니다! 🚀
