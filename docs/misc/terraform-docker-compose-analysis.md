# 📊 현재 docker-compose.yml 분석

PACS Extension Server 프로젝트의 현재 Docker Compose 설정을 분석하고 Terraform으로 마이그레이션하기 위한 준비 작업입니다.

## 📋 목차

1. [현재 설정 분석](#현재-설정-분석)
2. [Docker Compose vs Terraform 비교](#docker-compose-vs-terraform-비교)
3. [마이그레이션 전략](#마이그레이션-전략)
4. [Terraform 변환 계획](#terraform-변환-계획)
5. [실습 예제](#실습-예제)

---

## 🔍 현재 설정 분석

### 현재 `infra/docker-compose.yml` 파일

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
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U admin -d pacs_db"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
```

### 📊 구성 요소 분석

#### 1. **PostgreSQL 서비스**
- **이미지**: `postgres:16-alpine` (PostgreSQL 16, Alpine Linux 기반)
- **컨테이너명**: `pacs-postgres`
- **포트**: `5432:5432` (호스트:컨테이너)
- **데이터베이스**: `pacs_db`
- **사용자**: `admin`
- **비밀번호**: `admin123`

#### 2. **환경변수 설정**
```yaml
environment:
  POSTGRES_USER: admin
  POSTGRES_PASSWORD: admin123
  POSTGRES_DB: pacs_db
```

#### 3. **볼륨 설정**
```yaml
volumes:
  - postgres_data:/var/lib/postgresql/data
```
- **볼륨명**: `postgres_data`
- **마운트 경로**: `/var/lib/postgresql/data` (PostgreSQL 데이터 디렉토리)

#### 4. **헬스체크 설정**
```yaml
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U admin -d pacs_db"]
  interval: 10s
  timeout: 5s
  retries: 5
```
- **체크 명령**: `pg_isready` (PostgreSQL 연결 가능 여부 확인)
- **체크 간격**: 10초
- **타임아웃**: 5초
- **재시도**: 5회

---

## ⚖️ Docker Compose vs Terraform 비교

### Docker Compose의 장점
- **간단한 설정**: YAML 기반으로 직관적
- **빠른 시작**: `docker-compose up` 한 번으로 실행
- **개발 친화적**: 로컬 개발에 최적화
- **의존성 관리**: 서비스 간 의존성 자동 처리

### Terraform의 장점
- **상태 관리**: 현재 상태를 추적하고 관리
- **변경 계획**: `terraform plan`으로 변경사항 미리보기
- **버전 관리**: Git과 함께 인프라 코드 버전 관리
- **확장성**: 복잡한 인프라 구성 가능
- **멀티 환경**: development, staging, production 환경 분리

### 비교표

| 기능 | Docker Compose | Terraform |
|------|----------------|-----------|
| 설정 복잡도 | ⭐⭐ (간단) | ⭐⭐⭐⭐ (복잡) |
| 상태 관리 | ❌ | ✅ |
| 변경 계획 | ❌ | ✅ |
| 버전 관리 | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| 멀티 환경 | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| 개발 속도 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| 운영 안정성 | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

---

## 🚀 마이그레이션 전략

### 1단계: 현재 설정 이해
- [x] Docker Compose 파일 분석 완료
- [x] 각 구성 요소의 역할 파악
- [x] 의존성 관계 확인

### 2단계: Terraform 구조 설계
```
terraform/
├── main.tf              # 메인 리소스 정의
├── variables.tf          # 변수 정의
├── outputs.tf           # 출력값 정의
├── terraform.tfvars     # 변수 값 설정
└── environments/        # 환경별 설정
    ├── development/
    └── production/
```

### 3단계: 점진적 마이그레이션
1. **Phase 1**: PostgreSQL만 Terraform으로 이전
2. **Phase 2**: 네트워크 및 볼륨 관리 추가
3. **Phase 3**: 환경별 설정 분리
4. **Phase 4**: 모니터링 및 로깅 추가

### 4단계: 검증 및 테스트
- [ ] Terraform plan 실행
- [ ] 컨테이너 생성 확인
- [ ] 데이터베이스 연결 테스트
- [ ] 헬스체크 동작 확인

---

## 📝 Terraform 변환 계획

### 1. 기본 구조

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

# PostgreSQL 이미지
resource "docker_image" "postgres" {
  name = "postgres:16-alpine"
}

# 데이터 볼륨
resource "docker_volume" "postgres_data" {
  name = "postgres_data"
}

# PostgreSQL 컨테이너
resource "docker_container" "postgres" {
  name  = "pacs-postgres"
  image = docker_image.postgres.image_id
  
  env = [
    "POSTGRES_USER=${var.postgres_user}",
    "POSTGRES_PASSWORD=${var.postgres_password}",
    "POSTGRES_DB=${var.postgres_db}"
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
    test     = ["CMD-SHELL", "pg_isready -U ${var.postgres_user} -d ${var.postgres_db}"]
    interval = "10s"
    timeout  = "5s"
    retries  = 5
  }
}
```

#### `variables.tf`
```hcl
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
}
```

#### `outputs.tf`
```hcl
output "postgres_container_name" {
  description = "Name of the PostgreSQL container"
  value       = docker_container.postgres.name
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

#### `terraform.tfvars`
```hcl
# PostgreSQL 설정
postgres_user     = "admin"
postgres_password = "admin123"
postgres_db       = "pacs_db"
postgres_port     = 5432
```

---

## 🧪 실습 예제

### 1. 기본 마이그레이션

#### 단계 1: Terraform 파일 생성
```bash
# 프로젝트 디렉토리 생성
mkdir terraform-postgres
cd terraform-postgres

# Terraform 파일들 생성 (위의 코드 사용)
```

#### 단계 2: 초기화 및 실행
```bash
# Terraform 초기화
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

# 데이터베이스 연결 테스트
docker exec -it pacs-postgres psql -U admin -d pacs_db
```

### 2. 환경별 설정 분리

#### `environments/development/main.tf`
```hcl
module "postgres" {
  source = "../../modules/postgres"
  
  postgres_user     = "admin"
  postgres_password = "admin123"
  postgres_db       = "pacs_db_dev"
  postgres_port     = 5432
}
```

#### `environments/production/main.tf`
```hcl
module "postgres" {
  source = "../../modules/postgres"
  
  postgres_user     = "pacs_admin"
  postgres_password = var.postgres_password  # 시크릿에서 가져옴
  postgres_db       = "pacs_db_prod"
  postgres_port     = 5432
}
```

### 3. 네트워크 추가

#### `network.tf`
```hcl
# 애플리케이션 네트워크
resource "docker_network" "pacs_network" {
  name = "pacs-network"
  
  driver = "bridge"
  
  ipam_config {
    subnet = "172.20.0.0/16"
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

---

## 🔧 고급 설정

### 1. 데이터 백업 설정

#### `backup.tf`
```hcl
# 백업 볼륨
resource "docker_volume" "backup_data" {
  name = "postgres-backup"
}

# 백업 컨테이너
resource "docker_container" "backup" {
  name  = "postgres-backup"
  image = "postgres:16-alpine"
  
  command = [
    "sh", "-c",
    "while true; do pg_dump -h postgres -U ${var.postgres_user} -d ${var.postgres_db} > /backup/backup_$(date +%Y%m%d_%H%M%S).sql; sleep 86400; done"
  ]
  
  volumes {
    volume_name    = docker_volume.backup_data.name
    container_path = "/backup"
  }
  
  networks_advanced {
    name = docker_network.pacs_network.name
  }
  
  depends_on = [docker_container.postgres]
}
```

### 2. 모니터링 설정

#### `monitoring.tf`
```hcl
# Prometheus 컨테이너
resource "docker_container" "prometheus" {
  name  = "prometheus"
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
}

# Grafana 컨테이너
resource "docker_container" "grafana" {
  name  = "grafana"
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
}
```

---

## 📊 마이그레이션 체크리스트

### 사전 준비
- [ ] Docker Compose 파일 분석 완료
- [ ] Terraform 환경 설정
- [ ] 백업 데이터 확인

### 마이그레이션 실행
- [ ] Terraform 파일 작성
- [ ] `terraform init` 실행
- [ ] `terraform plan` 확인
- [ ] `terraform apply` 실행
- [ ] 컨테이너 동작 확인

### 검증 테스트
- [ ] 데이터베이스 연결 테스트
- [ ] 헬스체크 동작 확인
- [ ] 데이터 영속성 확인
- [ ] 포트 접근 확인

### 정리 작업
- [ ] 기존 Docker Compose 중지
- [ ] 불필요한 리소스 정리
- [ ] 문서 업데이트

---

## 🚨 주의사항

### 1. 데이터 보존
```hcl
# 중요한 데이터 볼륨은 destroy 방지
resource "docker_volume" "postgres_data" {
  name = "postgres_data"
  
  lifecycle {
    prevent_destroy = true
  }
}
```

### 2. 환경변수 보안
```hcl
# 민감한 정보는 변수로 관리
variable "postgres_password" {
  description = "PostgreSQL password"
  type        = string
  sensitive   = true
}
```

### 3. 네트워크 격리
```hcl
# 프로덕션 환경에서는 외부 포트 노출 최소화
resource "docker_container" "postgres" {
  # 외부 포트 노출하지 않음
  # networks_advanced로만 접근 허용
}
```

---

## 📚 다음 단계

이제 현재 Docker Compose 설정을 완전히 이해했으니 다음 문서들을 학습하세요:

1. **Terraform으로 PostgreSQL 구성하기** - 실제 Terraform 코드 작성
2. **Docker 네트워크 구성** - 컨테이너 간 통신 설정
3. **환경별 설정 관리** - development/production 분리

---

## 📖 참고 자료

- [Docker Compose 공식 문서](https://docs.docker.com/compose/)
- [Terraform Docker Provider](https://registry.terraform.io/providers/kreuzwerker/docker/latest/docs)
- [PostgreSQL Docker 이미지](https://hub.docker.com/_/postgres)

이 분석을 통해 Docker Compose에서 Terraform으로의 마이그레이션 준비가 완료되었습니다! 🚀
