# 🐘 Terraform으로 PostgreSQL 구성하기

PACS Extension Server 프로젝트의 PostgreSQL 데이터베이스를 Terraform으로 구성하는 실습 가이드입니다.

## 📋 목차

1. [프로젝트 구조 설정](#프로젝트-구조-설정)
2. [기본 PostgreSQL 구성](#기본-postgresql-구성)
3. [고급 설정 및 최적화](#고급-설정-및-최적화)
4. [환경별 설정 분리](#환경별-설정-분리)
5. [실습 및 테스트](#실습-및-테스트)
6. [문제 해결](#문제-해결)

---

## 🏗️ 프로젝트 구조 설정

### 디렉토리 구조
```
terraform-postgres/
├── main.tf                 # 메인 리소스 정의
├── variables.tf            # 변수 정의
├── outputs.tf             # 출력값 정의
├── terraform.tfvars       # 변수 값 설정
├── versions.tf            # Provider 버전 관리
├── modules/               # 재사용 가능한 모듈
│   └── postgres/
│       ├── main.tf
│       ├── variables.tf
│       ├── outputs.tf
│       └── versions.tf
└── environments/          # 환경별 설정
    ├── development/
    │   ├── main.tf
    │   └── terraform.tfvars
    └── production/
        ├── main.tf
        └── terraform.tfvars
```

### 초기 설정

#### `versions.tf`
```hcl
terraform {
  required_version = ">= 1.0"
  
  required_providers {
    docker = {
      source  = "kreuzwerker/docker"
      version = "~> 3.0"
    }
  }
}
```

---

## 🐘 기본 PostgreSQL 구성

### 1. 메인 리소스 정의

#### `main.tf`
```hcl
# Docker Provider 설정
provider "docker" {
  host = "unix:///var/run/docker.sock"
}

# PostgreSQL 이미지
resource "docker_image" "postgres" {
  name = "postgres:16-alpine"
  
  keep_locally = false
}

# 데이터 볼륨
resource "docker_volume" "postgres_data" {
  name = "${var.project_name}-postgres-data"
  
  driver = "local"
}

# PostgreSQL 컨테이너
resource "docker_container" "postgres" {
  name  = "${var.project_name}-postgres"
  image = docker_image.postgres.image_id
  
  # 환경변수
  env = [
    "POSTGRES_USER=${var.postgres_user}",
    "POSTGRES_PASSWORD=${var.postgres_password}",
    "POSTGRES_DB=${var.postgres_db}",
    "POSTGRES_INITDB_ARGS=--encoding=UTF-8 --lc-collate=C --lc-ctype=C"
  ]
  
  # 포트 매핑
  ports {
    internal = 5432
    external = var.postgres_port
  }
  
  # 볼륨 마운트
  volumes {
    volume_name    = docker_volume.postgres_data.name
    container_path = "/var/lib/postgresql/data"
  }
  
  # 재시작 정책
  restart = "unless-stopped"
  
  # 헬스체크
  healthcheck {
    test     = ["CMD-SHELL", "pg_isready -U ${var.postgres_user} -d ${var.postgres_db}"]
    interval = "10s"
    timeout  = "5s"
    retries  = 5
    start_period = "30s"
  }
  
  # 라벨
  labels {
    label = "project"
    value = var.project_name
  }
  
  labels {
    label = "environment"
    value = var.environment
  }
  
  labels {
    label = "service"
    value = "postgresql"
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
  description = "Environment (development, staging, production)"
  type        = string
  default     = "development"
  
  validation {
    condition     = contains(["development", "staging", "production"], var.environment)
    error_message = "Environment must be one of: development, staging, production."
  }
}

# PostgreSQL 설정
variable "postgres_user" {
  description = "PostgreSQL username"
  type        = string
  default     = "admin"
}

variable "postgres_password" {
  description = "PostgreSQL password"
  type        = string
  sensitive   = true
  default     = "admin123"
}

variable "postgres_db" {
  description = "PostgreSQL database name"
  type        = string
  default     = "pacs_db"
}

variable "postgres_port" {
  description = "PostgreSQL port"
  type        = number
  default     = 5432
  
  validation {
    condition     = var.postgres_port > 1024 && var.postgres_port < 65536
    error_message = "PostgreSQL port must be between 1024 and 65535."
  }
}

# 컨테이너 설정
variable "container_restart_policy" {
  description = "Container restart policy"
  type        = string
  default     = "unless-stopped"
  
  validation {
    condition     = contains(["no", "on-failure", "always", "unless-stopped"], var.container_restart_policy)
    error_message = "Restart policy must be one of: no, on-failure, always, unless-stopped."
  }
}

# 헬스체크 설정
variable "healthcheck_interval" {
  description = "Health check interval"
  type        = string
  default     = "10s"
}

variable "healthcheck_timeout" {
  description = "Health check timeout"
  type        = string
  default     = "5s"
}

variable "healthcheck_retries" {
  description = "Health check retries"
  type        = number
  default     = 5
}

variable "healthcheck_start_period" {
  description = "Health check start period"
  type        = string
  default     = "30s"
}
```

### 3. 출력값 정의

#### `outputs.tf`
```hcl
# 컨테이너 정보
output "postgres_container_name" {
  description = "Name of the PostgreSQL container"
  value       = docker_container.postgres.name
}

output "postgres_container_id" {
  description = "ID of the PostgreSQL container"
  value       = docker_container.postgres.id
}

# 네트워크 정보
output "postgres_host" {
  description = "PostgreSQL host"
  value       = "localhost"
}

output "postgres_port" {
  description = "PostgreSQL port"
  value       = docker_container.postgres.ports[0].external
}

# 데이터베이스 정보
output "postgres_database" {
  description = "PostgreSQL database name"
  value       = var.postgres_db
}

output "postgres_user" {
  description = "PostgreSQL username"
  value       = var.postgres_user
}

# 연결 정보
output "postgres_connection_string" {
  description = "PostgreSQL connection string"
  value       = "postgresql://${var.postgres_user}:${var.postgres_password}@localhost:${docker_container.postgres.ports[0].external}/${var.postgres_db}"
  sensitive   = true
}

# 볼륨 정보
output "postgres_volume_name" {
  description = "Name of the PostgreSQL data volume"
  value       = docker_volume.postgres_data.name
}

# 헬스체크 정보
output "postgres_health_status" {
  description = "PostgreSQL health status"
  value       = docker_container.postgres.health
}
```

### 4. 변수 값 설정

#### `terraform.tfvars`
```hcl
# 프로젝트 설정
project_name = "pacs"
environment  = "development"

# PostgreSQL 설정
postgres_user     = "admin"
postgres_password = "admin123"
postgres_db       = "pacs_db"
postgres_port     = 5432

# 컨테이너 설정
container_restart_policy = "unless-stopped"

# 헬스체크 설정
healthcheck_interval    = "10s"
healthcheck_timeout     = "5s"
healthcheck_retries     = 5
healthcheck_start_period = "30s"
```

---

## 🚀 고급 설정 및 최적화

### 1. 네트워크 구성

#### `network.tf`
```hcl
# 애플리케이션 네트워크
resource "docker_network" "pacs_network" {
  name = "${var.project_name}-network"
  
  driver = "bridge"
  
  ipam_config {
    subnet = "172.20.0.0/16"
  }
  
  labels {
    label = "project"
    value = var.project_name
  }
}

# PostgreSQL 컨테이너 (네트워크 연결)
resource "docker_container" "postgres" {
  # ... 기존 설정 ...
  
  networks_advanced {
    name = docker_network.pacs_network.name
  }
  
  # 외부 포트 노출 제거 (내부 네트워크에서만 접근)
  # ports 블록 제거
}
```

### 2. 백업 설정

#### `backup.tf`
```hcl
# 백업 볼륨
resource "docker_volume" "backup_data" {
  name = "${var.project_name}-backup-data"
  
  driver = "local"
}

# 백업 컨테이너
resource "docker_container" "backup" {
  name  = "${var.project_name}-backup"
  image = "postgres:16-alpine"
  
  command = [
    "sh", "-c",
    "while true; do pg_dump -h ${docker_container.postgres.name} -U ${var.postgres_user} -d ${var.postgres_db} > /backup/backup_$(date +%Y%m%d_%H%M%S).sql; sleep 86400; done"
  ]
  
  volumes {
    volume_name    = docker_volume.backup_data.name
    container_path = "/backup"
  }
  
  networks_advanced {
    name = docker_network.pacs_network.name
  }
  
  depends_on = [docker_container.postgres]
  
  restart = "unless-stopped"
  
  labels {
    label = "project"
    value = var.project_name
  }
  
  labels {
    label = "service"
    value = "backup"
  }
}
```

### 3. 모니터링 설정

#### `monitoring.tf`
```hcl
# Prometheus 설정
resource "docker_container" "prometheus" {
  name  = "${var.project_name}-prometheus"
  image = "prom/prometheus:latest"
  
  ports {
    internal = 9090
    external = 9090
  }
  
  volumes {
    volume_name    = docker_volume.prometheus_data.name
    container_path = "/prometheus"
  }
  
  networks_advanced {
    name = docker_network.pacs_network.name
  }
  
  restart = "unless-stopped"
  
  labels {
    label = "project"
    value = var.project_name
  }
  
  labels {
    label = "service"
    value = "prometheus"
  }
}

# Grafana 설정
resource "docker_container" "grafana" {
  name  = "${var.project_name}-grafana"
  image = "grafana/grafana:latest"
  
  ports {
    internal = 3000
    external = 3000
  }
  
  volumes {
    volume_name    = docker_volume.grafana_data.name
    container_path = "/var/lib/grafana"
  }
  
  networks_advanced {
    name = docker_network.pacs_network.name
  }
  
  restart = "unless-stopped"
  
  env = [
    "GF_SECURITY_ADMIN_PASSWORD=admin123"
  ]
  
  labels {
    label = "project"
    value = var.project_name
  }
  
  labels {
    label = "service"
    value = "grafana"
  }
}

# 모니터링 볼륨들
resource "docker_volume" "prometheus_data" {
  name = "${var.project_name}-prometheus-data"
}

resource "docker_volume" "grafana_data" {
  name = "${var.project_name}-grafana-data"
}
```

---

## 🌍 환경별 설정 분리

### 1. 모듈 구조

#### `modules/postgres/main.tf`
```hcl
# PostgreSQL 이미지
resource "docker_image" "postgres" {
  name = "postgres:16-alpine"
}

# 데이터 볼륨
resource "docker_volume" "postgres_data" {
  name = "${var.project_name}-postgres-data-${var.environment}"
}

# PostgreSQL 컨테이너
resource "docker_container" "postgres" {
  name  = "${var.project_name}-postgres-${var.environment}"
  image = docker_image.postgres.image_id
  
  env = [
    "POSTGRES_USER=${var.postgres_user}",
    "POSTGRES_PASSWORD=${var.postgres_password}",
    "POSTGRES_DB=${var.postgres_db}",
    "POSTGRES_INITDB_ARGS=--encoding=UTF-8 --lc-collate=C --lc-ctype=C"
  ]
  
  ports {
    internal = 5432
    external = var.postgres_port
  }
  
  volumes {
    volume_name    = docker_volume.postgres_data.name
    container_path = "/var/lib/postgresql/data"
  }
  
  restart = var.container_restart_policy
  
  healthcheck {
    test         = ["CMD-SHELL", "pg_isready -U ${var.postgres_user} -d ${var.postgres_db}"]
    interval     = var.healthcheck_interval
    timeout      = var.healthcheck_timeout
    retries      = var.healthcheck_retries
    start_period = var.healthcheck_start_period
  }
  
  labels = {
    project     = var.project_name
    environment = var.environment
    service     = "postgresql"
  }
}
```

#### `modules/postgres/variables.tf`
```hcl
# 프로젝트 설정
variable "project_name" {
  description = "Name of the project"
  type        = string
}

variable "environment" {
  description = "Environment"
  type        = string
}

# PostgreSQL 설정
variable "postgres_user" {
  description = "PostgreSQL username"
  type        = string
}

variable "postgres_password" {
  description = "PostgreSQL password"
  type        = string
  sensitive   = true
}

variable "postgres_db" {
  description = "PostgreSQL database name"
  type        = string
}

variable "postgres_port" {
  description = "PostgreSQL port"
  type        = number
}

# 컨테이너 설정
variable "container_restart_policy" {
  description = "Container restart policy"
  type        = string
  default     = "unless-stopped"
}

# 헬스체크 설정
variable "healthcheck_interval" {
  description = "Health check interval"
  type        = string
  default     = "10s"
}

variable "healthcheck_timeout" {
  description = "Health check timeout"
  type        = string
  default     = "5s"
}

variable "healthcheck_retries" {
  description = "Health check retries"
  type        = number
  default     = 5
}

variable "healthcheck_start_period" {
  description = "Health check start period"
  type        = string
  default     = "30s"
}
```

#### `modules/postgres/outputs.tf`
```hcl
output "postgres_container_name" {
  description = "Name of the PostgreSQL container"
  value       = docker_container.postgres.name
}

output "postgres_container_id" {
  description = "ID of the PostgreSQL container"
  value       = docker_container.postgres.id
}

output "postgres_port" {
  description = "PostgreSQL port"
  value       = docker_container.postgres.ports[0].external
}

output "postgres_volume_name" {
  description = "Name of the PostgreSQL data volume"
  value       = docker_volume.postgres_data.name
}
```

### 2. 환경별 설정

#### `environments/development/main.tf`
```hcl
module "postgres" {
  source = "../../modules/postgres"
  
  project_name = "pacs"
  environment  = "development"
  
  postgres_user     = "admin"
  postgres_password = "admin123"
  postgres_db       = "pacs_db_dev"
  postgres_port     = 5432
  
  container_restart_policy = "unless-stopped"
  
  healthcheck_interval    = "10s"
  healthcheck_timeout     = "5s"
  healthcheck_retries     = 5
  healthcheck_start_period = "30s"
}
```

#### `environments/development/terraform.tfvars`
```hcl
# Development 환경 설정
project_name = "pacs"
environment  = "development"

postgres_user     = "admin"
postgres_password = "admin123"
postgres_db       = "pacs_db_dev"
postgres_port     = 5432
```

#### `environments/production/main.tf`
```hcl
module "postgres" {
  source = "../../modules/postgres"
  
  project_name = "pacs"
  environment  = "production"
  
  postgres_user     = "pacs_admin"
  postgres_password = var.postgres_password  # 시크릿에서 가져옴
  postgres_db       = "pacs_db_prod"
  postgres_port     = 5432
  
  container_restart_policy = "always"
  
  healthcheck_interval    = "5s"
  healthcheck_timeout     = "3s"
  healthcheck_retries     = 3
  healthcheck_start_period = "60s"
}
```

---

## 🧪 실습 및 테스트

### 1. 기본 구성 테스트

#### 단계 1: 프로젝트 초기화
```bash
# 프로젝트 디렉토리 생성
mkdir terraform-postgres
cd terraform-postgres

# Terraform 파일들 생성 (위의 코드 사용)
```

#### 단계 2: Terraform 실행
```bash
# 초기화
terraform init

# 계획 확인
terraform plan

# 적용
terraform apply
```

#### 단계 3: 결과 확인
```bash
# 컨테이너 확인
docker ps

# 볼륨 확인
docker volume ls

# 네트워크 확인
docker network ls

# 데이터베이스 연결 테스트
docker exec -it pacs-postgres psql -U admin -d pacs_db
```

### 2. 데이터베이스 연결 테스트

#### 연결 테스트 스크립트
```bash
#!/bin/bash
# test_connection.sh

echo "Testing PostgreSQL connection..."

# 컨테이너 상태 확인
if docker ps | grep -q "pacs-postgres"; then
    echo "✅ PostgreSQL container is running"
else
    echo "❌ PostgreSQL container is not running"
    exit 1
fi

# 데이터베이스 연결 테스트
if docker exec pacs-postgres pg_isready -U admin -d pacs_db; then
    echo "✅ PostgreSQL is ready to accept connections"
else
    echo "❌ PostgreSQL is not ready"
    exit 1
fi

# 테이블 생성 테스트
docker exec pacs-postgres psql -U admin -d pacs_db -c "
CREATE TABLE IF NOT EXISTS test_table (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
"

if [ $? -eq 0 ]; then
    echo "✅ Database operations successful"
else
    echo "❌ Database operations failed"
    exit 1
fi

echo "🎉 All tests passed!"
```

### 3. 성능 테스트

#### 성능 테스트 스크립트
```bash
#!/bin/bash
# performance_test.sh

echo "Running PostgreSQL performance test..."

# 연결 시간 테스트
time docker exec pacs-postgres psql -U admin -d pacs_db -c "SELECT 1;"

# 대량 데이터 삽입 테스트
docker exec pacs-postgres psql -U admin -d pacs_db -c "
INSERT INTO test_table (name) 
SELECT 'test_' || generate_series(1, 10000);
"

# 쿼리 성능 테스트
time docker exec pacs-postgres psql -U admin -d pacs_db -c "
SELECT COUNT(*) FROM test_table;
"

echo "Performance test completed!"
```

---

## 🔧 문제 해결

### 1. 컨테이너 시작 실패

**증상**: 컨테이너가 시작되지 않음
```bash
# 컨테이너 로그 확인
docker logs pacs-postgres

# 컨테이너 상태 확인
docker inspect pacs-postgres
```

**해결 방법**:
```hcl
# 환경변수 검증 추가
resource "docker_container" "postgres" {
  # ... 기존 설정 ...
  
  # 환경변수 검증
  env = [
    "POSTGRES_USER=${var.postgres_user}",
    "POSTGRES_PASSWORD=${var.postgres_password}",
    "POSTGRES_DB=${var.postgres_db}",
    "POSTGRES_INITDB_ARGS=--encoding=UTF-8 --lc-collate=C --lc-ctype=C"
  ]
  
  # 시작 대기 시간 증가
  healthcheck {
    test         = ["CMD-SHELL", "pg_isready -U ${var.postgres_user} -d ${var.postgres_db}"]
    interval     = "10s"
    timeout      = "5s"
    retries      = 10  # 재시도 횟수 증가
    start_period = "60s"  # 시작 대기 시간 증가
  }
}
```

### 2. 포트 충돌

**증상**: 포트가 이미 사용 중
```bash
# 포트 사용 확인
netstat -tulpn | grep :5432
lsof -i :5432
```

**해결 방법**:
```hcl
# 다른 포트 사용
variable "postgres_port" {
  description = "PostgreSQL port"
  type        = number
  default     = 5433  # 5432 대신 5433 사용
}
```

### 3. 볼륨 권한 문제

**증상**: 볼륨 마운트 실패
```bash
# 볼륨 정보 확인
docker volume inspect pacs-postgres-data
```

**해결 방법**:
```hcl
# 볼륨 드라이버 옵션 설정
resource "docker_volume" "postgres_data" {
  name = "${var.project_name}-postgres-data"
  
  driver = "local"
  
  driver_opts = {
    type   = "none"
    o      = "bind"
    device = "/var/lib/docker/volumes/${var.project_name}-postgres-data/_data"
  }
}
```

### 4. 네트워크 연결 문제

**증상**: 컨테이너 간 통신 실패
```bash
# 네트워크 확인
docker network ls
docker network inspect pacs-network
```

**해결 방법**:
```hcl
# 네트워크 설정 확인
resource "docker_network" "pacs_network" {
  name = "${var.project_name}-network"
  
  driver = "bridge"
  
  ipam_config {
    subnet = "172.20.0.0/16"
    gateway = "172.20.0.1"
  }
}
```

---

## 📊 모니터링 및 로깅

### 1. 로그 수집

```bash
# 컨테이너 로그 확인
docker logs pacs-postgres

# 실시간 로그 모니터링
docker logs -f pacs-postgres

# 로그 파일로 저장
docker logs pacs-postgres > postgres.log 2>&1
```

### 2. 성능 모니터링

```bash
# 컨테이너 리소스 사용량 확인
docker stats pacs-postgres

# 데이터베이스 성능 확인
docker exec pacs-postgres psql -U admin -d pacs_db -c "
SELECT 
    datname,
    numbackends,
    xact_commit,
    xact_rollback,
    blks_read,
    blks_hit
FROM pg_stat_database 
WHERE datname = 'pacs_db';
"
```

### 3. 헬스체크 모니터링

```bash
# 헬스체크 상태 확인
docker inspect pacs-postgres | jq '.[0].State.Health'

# 헬스체크 로그 확인
docker inspect pacs-postgres | jq '.[0].State.Health.Log'
```

---

## 📚 다음 단계

이제 PostgreSQL을 Terraform으로 성공적으로 구성했으니 다음 문서들을 학습하세요:

1. **Docker 네트워크 구성** - 컨테이너 간 통신 설정
2. **환경별 설정 관리** - development/production 분리
3. **AWS S3 버킷 구성** - Object Storage 설정

---

## 📖 참고 자료

- [PostgreSQL 공식 문서](https://www.postgresql.org/docs/)
- [Docker PostgreSQL 이미지](https://hub.docker.com/_/postgres)
- [Terraform Docker Provider](https://registry.terraform.io/providers/kreuzwerker/docker/latest/docs)

이제 PACS 프로젝트의 PostgreSQL을 Terraform으로 완전히 관리할 수 있게 되었습니다! 🚀


