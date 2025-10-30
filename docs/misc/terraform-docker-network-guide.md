# 🌐 Docker 네트워크 구성 가이드

Terraform을 사용하여 Docker 컨테이너 간 통신을 위한 네트워크를 구성하는 실습 가이드입니다.

## 📋 목차

1. [Docker 네트워크 기본 개념](#docker-네트워크-기본-개념)
2. [Terraform으로 네트워크 구성](#terraform으로-네트워크-구성)
3. [PACS 프로젝트 네트워크 설계](#pacs-프로젝트-네트워크-설계)
4. [고급 네트워크 설정](#고급-네트워크-설정)
5. [보안 및 격리](#보안-및-격리)
6. [실습 및 테스트](#실습-및-테스트)

---

## 🌐 Docker 네트워크 기본 개념

### 네트워크 드라이버 종류

#### 1. Bridge 네트워크 (기본)
- **용도**: 단일 호스트 내 컨테이너 간 통신
- **특징**: 자동 DNS 해석, 포트 매핑
- **사용 사례**: 개발 환경, 단일 서버 배포

#### 2. Host 네트워크
- **용도**: 컨테이너가 호스트 네트워크 직접 사용
- **특징**: 최고 성능, 포트 충돌 가능성
- **사용 사례**: 고성능 요구사항

#### 3. Overlay 네트워크
- **용도**: 여러 호스트 간 컨테이너 통신
- **특징**: Swarm 모드 필요, 복잡한 설정
- **사용 사례**: 분산 시스템, 클러스터 환경

#### 4. Macvlan 네트워크
- **용도**: 컨테이너에 MAC 주소 할당
- **특징**: 물리적 네트워크와 직접 통신
- **사용 사례**: 레거시 시스템 연동

### 네트워크 구성 요소

```
┌─────────────────────────────────────────┐
│                Host Machine             │
│  ┌─────────────┐    ┌─────────────┐    │
│  │   Container │    │   Container │    │
│  │     A       │    │     B       │    │
│  └─────────────┘    └─────────────┘    │
│         │                   │          │
│         └─────────┬─────────┘          │
│                   │                    │
│         ┌─────────▼─────────┐          │
│         │   Bridge Network  │          │
│         │   (pacs-network)  │          │
│         └─────────┬─────────┘          │
│                   │                    │
│         ┌─────────▼─────────┐          │
│         │   Host Interface  │          │
│         │   (eth0, wlan0)   │          │
│         └───────────────────┘          │
└─────────────────────────────────────────┘
```

---

## 🏗️ Terraform으로 네트워크 구성

### 1. 기본 네트워크 생성

#### `network.tf`
```hcl
# PACS 애플리케이션 네트워크
resource "docker_network" "pacs_network" {
  name = "${var.project_name}-network"
  
  driver = "bridge"
  
  # IP 주소 관리 설정
  ipam_config {
    subnet  = var.network_subnet
    gateway = var.network_gateway
  }
  
  # 네트워크 옵션
  options = {
    com.docker.network.bridge.name = "pacs-br0"
    com.docker.network.driver.mtu  = "1500"
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
    label = "type"
    value = "application"
  }
}

# 데이터베이스 전용 네트워크
resource "docker_network" "database_network" {
  name = "${var.project_name}-database-network"
  
  driver = "bridge"
  
  ipam_config {
    subnet  = var.database_subnet
    gateway = var.database_gateway
  }
  
  # 데이터베이스 네트워크는 외부 접근 차단
  internal = true
  
  labels {
    label = "project"
    value = var.project_name
  }
  
  labels {
    label = "type"
    value = "database"
  }
}

# 모니터링 네트워크
resource "docker_network" "monitoring_network" {
  name = "${var.project_name}-monitoring-network"
  
  driver = "bridge"
  
  ipam_config {
    subnet  = var.monitoring_subnet
    gateway = var.monitoring_gateway
  }
  
  labels {
    label = "project"
    value = var.project_name
  }
  
  labels {
    label = "type"
    value = "monitoring"
  }
}
```

### 2. 네트워크 변수 정의

#### `variables.tf` (네트워크 관련)
```hcl
# 네트워크 설정
variable "network_subnet" {
  description = "Main application network subnet"
  type        = string
  default     = "172.20.0.0/16"
}

variable "network_gateway" {
  description = "Main application network gateway"
  type        = string
  default     = "172.20.0.1"
}

variable "database_subnet" {
  description = "Database network subnet"
  type        = string
  default     = "172.21.0.0/16"
}

variable "database_gateway" {
  description = "Database network gateway"
  type        = string
  default     = "172.21.0.1"
}

variable "monitoring_subnet" {
  description = "Monitoring network subnet"
  type        = string
  default     = "172.22.0.0/16"
}

variable "monitoring_gateway" {
  description = "Monitoring network gateway"
  type        = string
  default     = "172.22.0.1"
}

# 네트워크 보안 설정
variable "enable_network_isolation" {
  description = "Enable network isolation between services"
  type        = bool
  default     = true
}

variable "allow_external_access" {
  description = "Allow external access to services"
  type        = bool
  default     = false
}
```

### 3. 네트워크 출력값

#### `outputs.tf` (네트워크 관련)
```hcl
# 네트워크 정보
output "pacs_network_name" {
  description = "Name of the main PACS network"
  value       = docker_network.pacs_network.name
}

output "pacs_network_id" {
  description = "ID of the main PACS network"
  value       = docker_network.pacs_network.id
}

output "database_network_name" {
  description = "Name of the database network"
  value       = docker_network.database_network.name
}

output "monitoring_network_name" {
  description = "Name of the monitoring network"
  value       = docker_network.monitoring_network.name
}

# 네트워크 설정 정보
output "network_subnet" {
  description = "Main network subnet"
  value       = var.network_subnet
}

output "database_subnet" {
  description = "Database network subnet"
  value       = var.database_subnet
}

output "monitoring_subnet" {
  description = "Monitoring network subnet"
  value       = var.monitoring_subnet
}
```

---

## 🏥 PACS 프로젝트 네트워크 설계

### 네트워크 아키텍처

```
┌─────────────────────────────────────────────────────────────────┐
│                        PACS Network Architecture                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────┐  │
│  │   Web Layer     │    │  Application    │    │  Database   │  │
│  │                 │    │     Layer       │    │   Layer     │  │
│  │ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────┐ │  │
│  │ │   Nginx     │ │    │ │ PACS Server │ │    │ │PostgreSQL│ │  │
│  │ │ (Port 80)   │ │    │ │ (Port 8080) │ │    │ │(Port 5432)│ │  │
│  │ └─────────────┘ │    │ └─────────────┘ │    │ └─────────┘ │  │
│  └─────────────────┘    └─────────────────┘    └─────────────┘  │
│           │                       │                       │      │
│           └───────────────────────┼───────────────────────┘      │
│                                   │                              │
│  ┌─────────────────────────────────┼─────────────────────────────┐  │
│  │        pacs-network            │                              │  │
│  │      (172.20.0.0/16)          │                              │  │
│  └─────────────────────────────────┼─────────────────────────────┘  │
│                                   │                              │
│  ┌─────────────────────────────────┼─────────────────────────────┐  │
│  │      database-network          │                              │  │
│  │      (172.21.0.0/16)          │                              │  │
│  └─────────────────────────────────┼─────────────────────────────┘  │
│                                   │                              │
│  ┌─────────────────────────────────┼─────────────────────────────┐  │
│  │     monitoring-network         │                              │  │
│  │     (172.22.0.0/16)           │                              │  │
│  └─────────────────────────────────┼─────────────────────────────┘  │
│                                   │                              │
│  ┌─────────────────────────────────┼─────────────────────────────┐  │
│  │        Host Interface          │                              │  │
│  │        (External Access)       │                              │  │
│  └─────────────────────────────────┼─────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 서비스별 네트워크 구성

#### `services/web.tf`
```hcl
# Nginx 웹 서버
resource "docker_container" "nginx" {
  name  = "${var.project_name}-nginx"
  image = "nginx:alpine"
  
  # 외부 포트 노출
  ports {
    internal = 80
    external = 80
  }
  
  ports {
    internal = 443
    external = 443
  }
  
  # 메인 애플리케이션 네트워크 연결
  networks_advanced {
    name = docker_network.pacs_network.name
  }
  
  # 볼륨 마운트
  volumes {
    volume_name    = docker_volume.nginx_config.name
    container_path = "/etc/nginx/conf.d"
  }
  
  volumes {
    volume_name    = docker_volume.nginx_logs.name
    container_path = "/var/log/nginx"
  }
  
  restart = "unless-stopped"
  
  labels {
    label = "project"
    value = var.project_name
  }
  
  labels {
    label = "service"
    value = "nginx"
  }
  
  labels {
    label = "tier"
    value = "web"
  }
}
```

#### `services/application.tf`
```hcl
# PACS 애플리케이션 서버
resource "docker_container" "pacs_server" {
  name  = "${var.project_name}-server"
  image = "pacs-server:latest"
  
  # 내부 포트만 노출 (Nginx를 통해 접근)
  ports {
    internal = 8080
    external = 0  # 외부 노출 안함
  }
  
  # 메인 애플리케이션 네트워크 연결
  networks_advanced {
    name = docker_network.pacs_network.name
  }
  
  # 데이터베이스 네트워크 연결
  networks_advanced {
    name = docker_network.database_network.name
  }
  
  # 환경변수
  env = [
    "DATABASE_URL=postgresql://${var.postgres_user}:${var.postgres_password}@${docker_container.postgres.name}:5432/${var.postgres_db}",
    "REDIS_URL=redis://${docker_container.redis.name}:6379",
    "LOG_LEVEL=${var.log_level}"
  ]
  
  # 볼륨 마운트
  volumes {
    volume_name    = docker_volume.app_data.name
    container_path = "/app/data"
  }
  
  restart = "unless-stopped"
  
  depends_on = [
    docker_container.postgres,
    docker_container.redis
  ]
  
  labels {
    label = "project"
    value = var.project_name
  }
  
  labels {
    label = "service"
    value = "pacs-server"
  }
  
  labels {
    label = "tier"
    value = "application"
  }
}
```

#### `services/database.tf`
```hcl
# PostgreSQL 데이터베이스
resource "docker_container" "postgres" {
  name  = "${var.project_name}-postgres"
  image = "postgres:16-alpine"
  
  # 데이터베이스 네트워크만 연결 (외부 접근 차단)
  networks_advanced {
    name = docker_network.database_network.name
  }
  
  # 환경변수
  env = [
    "POSTGRES_USER=${var.postgres_user}",
    "POSTGRES_PASSWORD=${var.postgres_password}",
    "POSTGRES_DB=${var.postgres_db}"
  ]
  
  # 볼륨 마운트
  volumes {
    volume_name    = docker_volume.postgres_data.name
    container_path = "/var/lib/postgresql/data"
  }
  
  restart = "unless-stopped"
  
  labels {
    label = "project"
    value = var.project_name
  }
  
  labels {
    label = "service"
    value = "postgresql"
  }
  
  labels {
    label = "tier"
    value = "database"
  }
}

# Redis 캐시
resource "docker_container" "redis" {
  name  = "${var.project_name}-redis"
  image = "redis:7-alpine"
  
  # 데이터베이스 네트워크 연결
  networks_advanced {
    name = docker_network.database_network.name
  }
  
  # 볼륨 마운트
  volumes {
    volume_name    = docker_volume.redis_data.name
    container_path = "/data"
  }
  
  restart = "unless-stopped"
  
  labels {
    label = "project"
    value = var.project_name
  }
  
  labels {
    label = "service"
    value = "redis"
  }
  
  labels {
    label = "tier"
    value = "cache"
  }
}
```

#### `services/monitoring.tf`
```hcl
# Prometheus 모니터링
resource "docker_container" "prometheus" {
  name  = "${var.project_name}-prometheus"
  image = "prom/prometheus:latest"
  
  # 모니터링 네트워크 연결
  networks_advanced {
    name = docker_network.monitoring_network.name
  }
  
  # 외부 포트 노출 (모니터링용)
  ports {
    internal = 9090
    external = 9090
  }
  
  # 볼륨 마운트
  volumes {
    volume_name    = docker_volume.prometheus_config.name
    container_path = "/etc/prometheus"
  }
  
  volumes {
    volume_name    = docker_volume.prometheus_data.name
    container_path = "/prometheus"
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
  
  labels {
    label = "tier"
    value = "monitoring"
  }
}

# Grafana 대시보드
resource "docker_container" "grafana" {
  name  = "${var.project_name}-grafana"
  image = "grafana/grafana:latest"
  
  # 모니터링 네트워크 연결
  networks_advanced {
    name = docker_network.monitoring_network.name
  }
  
  # 외부 포트 노출 (대시보드용)
  ports {
    internal = 3000
    external = 3000
  }
  
  # 환경변수
  env = [
    "GF_SECURITY_ADMIN_PASSWORD=${var.grafana_password}"
  ]
  
  # 볼륨 마운트
  volumes {
    volume_name    = docker_volume.grafana_data.name
    container_path = "/var/lib/grafana"
  }
  
  restart = "unless-stopped"
  
  labels {
    label = "project"
    value = var.project_name
  }
  
  labels {
    label = "service"
    value = "grafana"
  }
  
  labels {
    label = "tier"
    value = "monitoring"
  }
}
```

---

## 🔒 고급 네트워크 설정

### 1. 네트워크 보안 정책

#### `security.tf`
```hcl
# 네트워크 보안 정책
resource "docker_network" "secure_network" {
  name = "${var.project_name}-secure-network"
  
  driver = "bridge"
  
  ipam_config {
    subnet  = "172.30.0.0/16"
    gateway = "172.30.0.1"
  }
  
  # 네트워크 옵션
  options = {
    com.docker.network.bridge.enable_icc     = "true"
    com.docker.network.bridge.enable_ip_masq = "true"
    com.docker.network.bridge.host_binding_ipv4 = "0.0.0.0"
    com.docker.network.driver.mtu = "1500"
  }
  
  # 보안 라벨
  labels {
    label = "security"
    value = "high"
  }
}

# 방화벽 규칙 (iptables 기반)
resource "null_resource" "firewall_rules" {
  count = var.enable_firewall ? 1 : 0
  
  provisioner "local-exec" {
    command = <<-EOT
      # 기본 정책 설정
      iptables -P INPUT DROP
      iptables -P FORWARD DROP
      iptables -P OUTPUT ACCEPT
      
      # 루프백 허용
      iptables -A INPUT -i lo -j ACCEPT
      iptables -A OUTPUT -o lo -j ACCEPT
      
      # ESTABLISHED, RELATED 연결 허용
      iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT
      
      # HTTP, HTTPS 허용
      iptables -A INPUT -p tcp --dport 80 -j ACCEPT
      iptables -A INPUT -p tcp --dport 443 -j ACCEPT
      
      # SSH 허용 (특정 IP만)
      iptables -A INPUT -p tcp --dport 22 -s 192.168.1.0/24 -j ACCEPT
      
      # Docker 네트워크 허용
      iptables -A INPUT -s 172.20.0.0/16 -j ACCEPT
      iptables -A INPUT -s 172.21.0.0/16 -j ACCEPT
      iptables -A INPUT -s 172.22.0.0/16 -j ACCEPT
    EOT
  }
}
```

### 2. 로드 밸런싱 설정

#### `loadbalancer.tf`
```hcl
# HAProxy 로드 밸런서
resource "docker_container" "haproxy" {
  name  = "${var.project_name}-haproxy"
  image = "haproxy:2.8-alpine"
  
  # 모든 네트워크 연결
  networks_advanced {
    name = docker_network.pacs_network.name
  }
  
  networks_advanced {
    name = docker_network.database_network.name
  }
  
  # 외부 포트 노출
  ports {
    internal = 80
    external = 80
  }
  
  ports {
    internal = 443
    external = 443
  }
  
  ports {
    internal = 8404  # HAProxy 통계 페이지
    external = 8404
  }
  
  # 볼륨 마운트
  volumes {
    volume_name    = docker_volume.haproxy_config.name
    container_path = "/usr/local/etc/haproxy/haproxy.cfg"
  }
  
  restart = "unless-stopped"
  
  labels {
    label = "project"
    value = var.project_name
  }
  
  labels {
    label = "service"
    value = "haproxy"
  }
  
  labels {
    label = "tier"
    value = "loadbalancer"
  }
}
```

### 3. DNS 설정

#### `dns.tf`
```hcl
# 내부 DNS 서버
resource "docker_container" "dns" {
  name  = "${var.project_name}-dns"
  image = "coredns/coredns:latest"
  
  # 모든 네트워크 연결
  networks_advanced {
    name = docker_network.pacs_network.name
  }
  
  networks_advanced {
    name = docker_network.database_network.name
  }
  
  networks_advanced {
    name = docker_network.monitoring_network.name
  }
  
  # DNS 포트
  ports {
    internal = 53
    external = 53
    protocol = "udp"
  }
  
  # Corefile 설정
  volumes {
    volume_name    = docker_volume.dns_config.name
    container_path = "/etc/coredns/Corefile"
  }
  
  restart = "unless-stopped"
  
  labels {
    label = "project"
    value = var.project_name
  }
  
  labels {
    label = "service"
    value = "dns"
  }
}
```

---

## 🧪 실습 및 테스트

### 1. 네트워크 연결 테스트

#### `test_network.sh`
```bash
#!/bin/bash
# 네트워크 연결 테스트 스크립트

echo "Testing PACS network connectivity..."

# 네트워크 생성 확인
echo "1. Checking networks..."
docker network ls | grep pacs

# 컨테이너 네트워크 연결 확인
echo "2. Checking container network connections..."
docker inspect pacs-nginx | jq '.[0].NetworkSettings.Networks'
docker inspect pacs-server | jq '.[0].NetworkSettings.Networks'
docker inspect pacs-postgres | jq '.[0].NetworkSettings.Networks'

# 컨테이너 간 통신 테스트
echo "3. Testing inter-container communication..."

# Nginx -> PACS Server
if docker exec pacs-nginx curl -f http://pacs-server:8080/health; then
    echo "✅ Nginx -> PACS Server: OK"
else
    echo "❌ Nginx -> PACS Server: FAILED"
fi

# PACS Server -> PostgreSQL
if docker exec pacs-server pg_isready -h pacs-postgres -p 5432; then
    echo "✅ PACS Server -> PostgreSQL: OK"
else
    echo "❌ PACS Server -> PostgreSQL: FAILED"
fi

# PACS Server -> Redis
if docker exec pacs-server redis-cli -h pacs-redis ping; then
    echo "✅ PACS Server -> Redis: OK"
else
    echo "❌ PACS Server -> Redis: FAILED"
fi

echo "Network connectivity test completed!"
```

### 2. 네트워크 성능 테스트

#### `test_performance.sh`
```bash
#!/bin/bash
# 네트워크 성능 테스트 스크립트

echo "Testing network performance..."

# 대역폭 테스트
echo "1. Testing bandwidth..."
docker exec pacs-server iperf3 -s -p 5201 &
sleep 2
docker exec pacs-nginx iperf3 -c pacs-server -p 5201 -t 10

# 지연 시간 테스트
echo "2. Testing latency..."
docker exec pacs-nginx ping -c 10 pacs-server
docker exec pacs-server ping -c 10 pacs-postgres

# 연결 수 테스트
echo "3. Testing connection limits..."
docker exec pacs-nginx ab -n 1000 -c 100 http://pacs-server:8080/

echo "Performance test completed!"
```

### 3. 보안 테스트

#### `test_security.sh`
```bash
#!/bin/bash
# 네트워크 보안 테스트 스크립트

echo "Testing network security..."

# 포트 스캔
echo "1. Port scanning..."
nmap -p 1-65535 localhost

# 네트워크 격리 확인
echo "2. Testing network isolation..."

# 데이터베이스 네트워크 외부 접근 차단 확인
if curl -f http://localhost:5432; then
    echo "❌ Database port is accessible from outside"
else
    echo "✅ Database port is properly isolated"
fi

# 모니터링 네트워크 접근 확인
if curl -f http://localhost:9090; then
    echo "✅ Monitoring port is accessible"
else
    echo "❌ Monitoring port is not accessible"
fi

# 방화벽 규칙 확인
echo "3. Checking firewall rules..."
iptables -L -n

echo "Security test completed!"
```

---

## 🔧 문제 해결

### 1. 네트워크 연결 실패

**증상**: 컨테이너 간 통신 불가
```bash
# 네트워크 상태 확인
docker network ls
docker network inspect pacs-network

# 컨테이너 네트워크 설정 확인
docker inspect pacs-server | jq '.[0].NetworkSettings'
```

**해결 방법**:
```hcl
# 네트워크 재생성
resource "docker_network" "pacs_network" {
  name = "${var.project_name}-network"
  
  driver = "bridge"
  
  # 기존 네트워크 강제 삭제
  force_remove = true
  
  ipam_config {
    subnet  = "172.20.0.0/16"
    gateway = "172.20.0.1"
  }
}
```

### 2. DNS 해석 실패

**증상**: 컨테이너 이름으로 접근 불가
```bash
# DNS 설정 확인
docker exec pacs-server nslookup pacs-postgres
docker exec pacs-server cat /etc/resolv.conf
```

**해결 방법**:
```hcl
# DNS 서버 설정
resource "docker_container" "pacs_server" {
  # ... 기존 설정 ...
  
  # DNS 설정
  dns = ["8.8.8.8", "8.8.4.4"]
  
  # 호스트명 설정
  hostname = "pacs-server"
  
  # 추가 호스트 설정
  extra_hosts = [
    "pacs-postgres:172.21.0.2",
    "pacs-redis:172.21.0.3"
  ]
}
```

### 3. 포트 충돌

**증상**: 포트가 이미 사용 중
```bash
# 포트 사용 확인
netstat -tulpn | grep :80
lsof -i :80
```

**해결 방법**:
```hcl
# 동적 포트 할당
resource "docker_container" "nginx" {
  # ... 기존 설정 ...
  
  ports {
    internal = 80
    external = 0  # 동적 포트 할당
  }
}

# 포트 확인
output "nginx_port" {
  value = docker_container.nginx.ports[0].external
}
```

---

## 📊 모니터링 및 로깅

### 1. 네트워크 모니터링

```bash
# 네트워크 통계 확인
docker exec pacs-server netstat -i
docker exec pacs-server ss -tuln

# 네트워크 트래픽 모니터링
docker exec pacs-server iftop -i eth0

# 연결 상태 확인
docker exec pacs-server netstat -an | grep ESTABLISHED
```

### 2. 로그 수집

```bash
# 네트워크 관련 로그 확인
docker logs pacs-nginx 2>&1 | grep -i network
docker logs pacs-server 2>&1 | grep -i connection

# 시스템 로그 확인
journalctl -u docker | grep network
```

### 3. 성능 메트릭

```bash
# 네트워크 성능 메트릭
docker exec pacs-server cat /proc/net/dev
docker exec pacs-server cat /proc/net/snmp

# Prometheus 메트릭 확인
curl http://localhost:9090/api/v1/query?query=container_network_receive_bytes_total
```

---

## 📚 다음 단계

이제 Docker 네트워크를 성공적으로 구성했으니 다음 문서들을 학습하세요:

1. **환경별 설정 관리** - development/production 분리
2. **AWS Provider 설정 가이드** - AWS 클라우드 인프라 시작
3. **S3 버킷 생성 및 관리** - Object Storage 설정

---

## 📖 참고 자료

- [Docker 네트워킹 공식 문서](https://docs.docker.com/network/)
- [Terraform Docker Provider](https://registry.terraform.io/providers/kreuzwerker/docker/latest/docs)
- [Docker 네트워크 보안 가이드](https://docs.docker.com/network/security/)

이제 PACS 프로젝트의 네트워크를 완전히 제어할 수 있게 되었습니다! 🚀
