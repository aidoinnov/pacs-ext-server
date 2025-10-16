# ☸️ EKS 클러스터 구성 가이드

Terraform을 사용하여 AWS EKS (Elastic Kubernetes Service) 클러스터를 구성하고 관리하는 방법을 학습합니다. PACS 프로젝트의 컨테이너 오케스트레이션을 위한 고가용성 Kubernetes 환경 설정을 중심으로 다룹니다.

## 📋 목차

1. [EKS 클러스터란?](#eks-클러스터란)
2. [기본 EKS 구성](#기본-eks-구성)
3. [PACS 프로젝트 EKS 설정](#pacs-프로젝트-eks-설정)
4. [노드 그룹 및 오토스케일링](#노드-그룹-및-오토스케일링)
5. [보안 및 모니터링](#보안-및-모니터링)
6. [실습 및 테스트](#실습-및-테스트)

---

## 🎯 EKS 클러스터란?

**Amazon EKS (Elastic Kubernetes Service)**는 AWS에서 관리되는 Kubernetes 클러스터 서비스입니다.

### 주요 특징
- **관리형 서비스**: 컨트롤 플레인 자동 관리
- **고가용성**: Multi-AZ 배포로 99.95% 가용성
- **확장성**: 수천 개의 노드까지 자동 스케일링
- **보안**: IAM 통합, VPC 네트워킹, 암호화
- **호환성**: 표준 Kubernetes API 지원

### PACS 프로젝트에서의 활용
- **마이크로서비스**: PACS Backend, Keycloak, Frontend 분리
- **자동 스케일링**: 트래픽에 따른 자동 확장/축소
- **롤링 업데이트**: 무중단 배포
- **서비스 메시**: Istio를 통한 서비스 간 통신

---

## 🔧 기본 EKS 구성

### 1. EKS 클러스터 생성

#### `eks-cluster.tf`
```hcl
# EKS 클러스터
resource "aws_eks_cluster" "main" {
  name     = "${var.project_name}-${var.environment}"
  role_arn = aws_iam_role.eks_cluster.arn
  version  = var.kubernetes_version

  vpc_config {
    subnet_ids              = var.private_subnet_ids
    endpoint_private_access = true
    endpoint_public_access  = var.enable_public_endpoint
    public_access_cidrs     = var.public_access_cidrs
  }

  # 클러스터 로깅
  enabled_cluster_log_types = [
    "api",
    "audit",
    "authenticator",
    "controllerManager",
    "scheduler"
  ]

  # 암호화 설정
  encryption_config {
    provider {
      key_arn = aws_kms_key.eks.arn
    }
    resources = ["secrets"]
  }

  depends_on = [
    aws_iam_role_policy_attachment.eks_cluster_policy,
    aws_iam_role_policy_attachment.eks_vpc_resource_controller,
    aws_cloudwatch_log_group.eks_cluster
  ]

  tags = {
    Name        = "${var.project_name}-eks-cluster"
    Environment = var.environment
    Project     = var.project_name
  }
}

# EKS 클러스터 IAM 역할
resource "aws_iam_role" "eks_cluster" {
  name = "${var.project_name}-eks-cluster-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "eks.amazonaws.com"
        }
      }
    ]
  })

  tags = {
    Name        = "${var.project_name}-eks-cluster-role"
    Environment = var.environment
    Project     = var.project_name
  }
}

# EKS 클러스터 정책 연결
resource "aws_iam_role_policy_attachment" "eks_cluster_policy" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKSClusterPolicy"
  role       = aws_iam_role.eks_cluster.name
}

resource "aws_iam_role_policy_attachment" "eks_vpc_resource_controller" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKSVPCResourceController"
  role       = aws_iam_role.eks_cluster.name
}
```

### 2. 노드 그룹 IAM 역할

#### `eks-node-iam.tf`
```hcl
# EKS 노드 그룹 IAM 역할
resource "aws_iam_role" "eks_node_group" {
  name = "${var.project_name}-eks-node-group-role"

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
    Name        = "${var.project_name}-eks-node-group-role"
    Environment = var.environment
    Project     = var.project_name
  }
}

# EKS 노드 그룹 정책 연결
resource "aws_iam_role_policy_attachment" "eks_worker_node_policy" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKSWorkerNodePolicy"
  role       = aws_iam_role.eks_node_group.name
}

resource "aws_iam_role_policy_attachment" "eks_cni_policy" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKS_CNI_Policy"
  role       = aws_iam_role.eks_node_group.name
}

resource "aws_iam_role_policy_attachment" "eks_container_registry_policy" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonEC2ContainerRegistryReadOnly"
  role       = aws_iam_role.eks_node_group.name
}
```

### 3. CloudWatch 로그 그룹

#### `eks-logging.tf`
```hcl
# EKS 클러스터 로그 그룹
resource "aws_cloudwatch_log_group" "eks_cluster" {
  name              = "/aws/eks/${var.project_name}-${var.environment}/cluster"
  retention_in_days = var.log_retention_days
  kms_key_id        = aws_kms_key.eks.arn

  tags = {
    Name        = "${var.project_name}-eks-cluster-logs"
    Environment = var.environment
    Project     = var.project_name
  }
}

# KMS 키 for EKS
resource "aws_kms_key" "eks" {
  description             = "KMS key for EKS cluster encryption"
  deletion_window_in_days = 7

  tags = {
    Name        = "${var.project_name}-eks-kms-key"
    Environment = var.environment
    Project     = var.project_name
  }
}

resource "aws_kms_alias" "eks" {
  name          = "alias/${var.project_name}-eks"
  target_key_id = aws_kms_key.eks.key_id
}
```

---

## 🏥 PACS 프로젝트 EKS 설정

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

# EKS 설정
variable "kubernetes_version" {
  description = "Kubernetes version"
  type        = string
  default     = "1.28"
}

variable "enable_public_endpoint" {
  description = "Enable public endpoint for EKS cluster"
  type        = bool
  default     = false
}

variable "public_access_cidrs" {
  description = "CIDR blocks for public access"
  type        = list(string)
  default     = ["0.0.0.0/0"]
}

variable "private_subnet_ids" {
  description = "Private subnet IDs for EKS cluster"
  type        = list(string)
}

variable "log_retention_days" {
  description = "CloudWatch log retention days"
  type        = number
  default     = 7
}

# 노드 그룹 설정
variable "node_groups" {
  description = "EKS node groups configuration"
  type = map(object({
    instance_types = list(string)
    capacity_type  = string
    min_size       = number
    max_size       = number
    desired_size   = number
    disk_size      = number
    ami_type       = string
  }))
  default = {
    general = {
      instance_types = ["t3.medium"]
      capacity_type  = "ON_DEMAND"
      min_size       = 1
      max_size       = 3
      desired_size   = 2
      disk_size      = 20
      ami_type       = "AL2_x86_64"
    }
    pacs_backend = {
      instance_types = ["m5.large"]
      capacity_type  = "ON_DEMAND"
      min_size       = 1
      max_size       = 2
      desired_size   = 1
      disk_size      = 50
      ami_type       = "AL2_x86_64"
    }
  }
}
```

### 2. 노드 그룹 구성

#### `eks-node-groups.tf`
```hcl
# EKS 노드 그룹
resource "aws_eks_node_group" "main" {
  for_each = var.node_groups

  cluster_name    = aws_eks_cluster.main.name
  node_group_name = each.key
  node_role_arn   = aws_iam_role.eks_node_group.arn
  subnet_ids      = var.private_subnet_ids

  instance_types = each.value.instance_types
  capacity_type  = each.value.capacity_type
  ami_type       = each.value.ami_type
  disk_size      = each.value.disk_size

  scaling_config {
    desired_size = each.value.desired_size
    max_size     = each.value.max_size
    min_size     = each.value.min_size
  }

  update_config {
    max_unavailable_percentage = 25
  }

  # Taint 설정 (PACS Backend 전용)
  dynamic "taint" {
    for_each = each.key == "pacs_backend" ? [1] : []
    content {
      key    = "workload"
      value  = "pacs-backend"
      effect = "NO_SCHEDULE"
    }
  }

  # 라벨 설정
  labels = {
    "node-group" = each.key
    "environment" = var.environment
    "project"     = var.project_name
  }

  depends_on = [
    aws_iam_role_policy_attachment.eks_worker_node_policy,
    aws_iam_role_policy_attachment.eks_cni_policy,
    aws_iam_role_policy_attachment.eks_container_registry_policy
  ]

  tags = {
    Name        = "${var.project_name}-${each.key}-node-group"
    Environment = var.environment
    Project     = var.project_name
    NodeGroup   = each.key
  }
}
```

### 3. PACS 전용 네임스페이스 및 리소스

#### `pacs-kubernetes-resources.tf`
```hcl
# PACS 네임스페이스
resource "kubernetes_namespace" "pacs" {
  metadata {
    name = "pacs"
    labels = {
      name        = "pacs"
      environment = var.environment
      project     = var.project_name
    }
  }

  depends_on = [aws_eks_node_group.main]
}

# PACS ConfigMap
resource "kubernetes_config_map" "pacs_config" {
  metadata {
    name      = "pacs-config"
    namespace = kubernetes_namespace.pacs.metadata[0].name
  }

  data = {
    DATABASE_URL = "postgresql://pacs_admin:${var.pacs_db_password}@${var.pacs_db_endpoint}:5432/pacs_main"
    REDIS_URL    = "redis://${var.redis_endpoint}:6379"
    S3_BUCKET    = var.s3_bucket_name
    S3_REGION    = var.aws_region
  }

  depends_on = [kubernetes_namespace.pacs]
}

# PACS Secret
resource "kubernetes_secret" "pacs_secrets" {
  metadata {
    name      = "pacs-secrets"
    namespace = kubernetes_namespace.pacs.metadata[0].name
  }

  data = {
    database-password = base64encode(var.pacs_db_password)
    redis-password    = base64encode(var.redis_password)
    s3-access-key     = base64encode(var.s3_access_key)
    s3-secret-key     = base64encode(var.s3_secret_key)
  }

  type = "Opaque"

  depends_on = [kubernetes_namespace.pacs]
}

# PACS Service Account
resource "kubernetes_service_account" "pacs" {
  metadata {
    name      = "pacs-service-account"
    namespace = kubernetes_namespace.pacs.metadata[0].name
    annotations = {
      "eks.amazonaws.com/role-arn" = aws_iam_role.pacs_pod_role.arn
    }
  }

  depends_on = [kubernetes_namespace.pacs]
}
```

### 4. PACS 애플리케이션 배포

#### `pacs-deployments.tf`
```hcl
# PACS Backend Deployment
resource "kubernetes_deployment" "pacs_backend" {
  metadata {
    name      = "pacs-backend"
    namespace = kubernetes_namespace.pacs.metadata[0].name
  }

  spec {
    replicas = var.pacs_backend_replicas

    selector {
      match_labels = {
        app = "pacs-backend"
      }
    }

    template {
      metadata {
        labels = {
          app = "pacs-backend"
        }
      }

      spec {
        service_account_name = kubernetes_service_account.pacs.metadata[0].name

        # PACS Backend 전용 노드에 배치
        affinity {
          node_affinity {
            required_during_scheduling_ignored_during_execution {
              node_selector_term {
                match_expressions {
                  key      = "workload"
                  operator = "In"
                  values   = ["pacs-backend"]
                }
              }
            }
          }
        }

        container {
          name  = "pacs-backend"
          image = "${var.pacs_backend_image}:${var.pacs_backend_tag}"

          port {
            container_port = 8080
            name          = "http"
          }

          env {
            name = "DATABASE_URL"
            value_from {
              config_map_key_ref {
                name = kubernetes_config_map.pacs_config.metadata[0].name
                key  = "DATABASE_URL"
              }
            }
          }

          env {
            name = "REDIS_URL"
            value_from {
              config_map_key_ref {
                name = kubernetes_config_map.pacs_config.metadata[0].name
                key  = "REDIS_URL"
              }
            }
          }

          env {
            name = "S3_BUCKET"
            value_from {
              config_map_key_ref {
                name = kubernetes_config_map.pacs_config.metadata[0].name
                key  = "S3_BUCKET"
              }
            }
          }

          resources {
            requests = {
              cpu    = "500m"
              memory = "1Gi"
            }
            limits = {
              cpu    = "2000m"
              memory = "4Gi"
            }
          }

          liveness_probe {
            http_get {
              path = "/health"
              port = 8080
            }
            initial_delay_seconds = 30
            period_seconds        = 10
          }

          readiness_probe {
            http_get {
              path = "/ready"
              port = 8080
            }
            initial_delay_seconds = 5
            period_seconds        = 5
          }
        }
      }
    }
  }

  depends_on = [kubernetes_namespace.pacs]
}

# Keycloak Deployment
resource "kubernetes_deployment" "keycloak" {
  metadata {
    name      = "keycloak"
    namespace = kubernetes_namespace.pacs.metadata[0].name
  }

  spec {
    replicas = var.keycloak_replicas

    selector {
      match_labels = {
        app = "keycloak"
      }
    }

    template {
      metadata {
        labels = {
          app = "keycloak"
        }
      }

      spec {
        container {
          name  = "keycloak"
          image = "quay.io/keycloak/keycloak:${var.keycloak_version}"

          port {
            container_port = 8080
            name          = "http"
          }

          env {
            name  = "KEYCLOAK_ADMIN"
            value = var.keycloak_admin_user
          }

          env {
            name = "KEYCLOAK_ADMIN_PASSWORD"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.pacs_secrets.metadata[0].name
                key  = "keycloak-admin-password"
              }
            }
          }

          env {
            name  = "KC_DB"
            value = "postgres"
          }

          env {
            name = "KC_DB_URL"
            value_from {
              config_map_key_ref {
                name = kubernetes_config_map.pacs_config.metadata[0].name
                key  = "KEYCLOAK_DB_URL"
              }
            }
          }

          command = ["/opt/keycloak/bin/kc.sh", "start-dev"]

          resources {
            requests = {
              cpu    = "250m"
              memory = "512Mi"
            }
            limits = {
              cpu    = "1000m"
              memory = "2Gi"
            }
          }
        }
      }
    }
  }

  depends_on = [kubernetes_namespace.pacs]
}
```

### 5. 서비스 및 Ingress

#### `pacs-services.tf`
```hcl
# PACS Backend Service
resource "kubernetes_service" "pacs_backend" {
  metadata {
    name      = "pacs-backend"
    namespace = kubernetes_namespace.pacs.metadata[0].name
  }

  spec {
    selector = {
      app = "pacs-backend"
    }

    port {
      port        = 80
      target_port = 8080
      protocol    = "TCP"
      name        = "http"
    }

    type = "ClusterIP"
  }

  depends_on = [kubernetes_deployment.pacs_backend]
}

# Keycloak Service
resource "kubernetes_service" "keycloak" {
  metadata {
    name      = "keycloak"
    namespace = kubernetes_namespace.pacs.metadata[0].name
  }

  spec {
    selector = {
      app = "keycloak"
    }

    port {
      port        = 80
      target_port = 8080
      protocol    = "TCP"
      name        = "http"
    }

    type = "ClusterIP"
  }

  depends_on = [kubernetes_deployment.keycloak]
}

# Ingress
resource "kubernetes_ingress_v1" "pacs" {
  metadata {
    name      = "pacs-ingress"
    namespace = kubernetes_namespace.pacs.metadata[0].name
    annotations = {
      "kubernetes.io/ingress.class"                = "alb"
      "alb.ingress.kubernetes.io/scheme"          = "internal"
      "alb.ingress.kubernetes.io/target-type"     = "ip"
      "alb.ingress.kubernetes.io/load-balancer-name" = "${var.project_name}-pacs-alb"
    }
  }

  spec {
    rule {
      host = "api.${var.environment}.pacs.local"
      http {
        path {
          path      = "/api"
          path_type = "Prefix"
          backend {
            service {
              name = kubernetes_service.pacs_backend.metadata[0].name
              port {
                number = 80
              }
            }
          }
        }
      }
    }

    rule {
      host = "auth.${var.environment}.pacs.local"
      http {
        path {
          path      = "/auth"
          path_type = "Prefix"
          backend {
            service {
              name = kubernetes_service.keycloak.metadata[0].name
              port {
                number = 80
              }
            }
          }
        }
      }
    }
  }

  depends_on = [kubernetes_service.pacs_backend, kubernetes_service.keycloak]
}
```

---

## 🔄 노드 그룹 및 오토스케일링

### 1. Cluster Autoscaler

#### `cluster-autoscaler.tf`
```hcl
# Cluster Autoscaler IAM 정책
resource "aws_iam_policy" "cluster_autoscaler" {
  name        = "${var.project_name}-cluster-autoscaler"
  description = "Policy for Cluster Autoscaler"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "autoscaling:DescribeAutoScalingGroups",
          "autoscaling:DescribeAutoScalingInstances",
          "autoscaling:DescribeLaunchConfigurations",
          "autoscaling:DescribeTags",
          "autoscaling:SetDesiredCapacity",
          "autoscaling:TerminateInstanceInAutoScalingGroup",
          "ec2:DescribeLaunchTemplateVersions"
        ]
        Resource = "*"
      }
    ]
  })
}

# Cluster Autoscaler IAM 역할
resource "aws_iam_role" "cluster_autoscaler" {
  name = "${var.project_name}-cluster-autoscaler"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRoleWithWebIdentity"
        Effect = "Allow"
        Principal = {
          Federated = aws_iam_openid_connect_provider.eks.arn
        }
        Condition = {
          StringEquals = {
            "${replace(aws_iam_openid_connect_provider.eks.url, "https://", "")}:sub" = "system:serviceaccount:kube-system:cluster-autoscaler"
            "${replace(aws_iam_openid_connect_provider.eks.url, "https://", "")}:aud" = "sts.amazonaws.com"
          }
        }
      }
    ]
  })
}

# OIDC Identity Provider
resource "aws_iam_openid_connect_provider" "eks" {
  client_id_list  = ["sts.amazonaws.com"]
  thumbprint_list = [data.tls_certificate.eks.certificates[0].sha1_fingerprint]
  url             = aws_eks_cluster.main.identity[0].oidc[0].issuer
}

data "tls_certificate" "eks" {
  url = aws_eks_cluster.main.identity[0].oidc[0].issuer
}

# 정책 연결
resource "aws_iam_role_policy_attachment" "cluster_autoscaler" {
  policy_arn = aws_iam_policy.cluster_autoscaler.arn
  role       = aws_iam_role.cluster_autoscaler.name
}
```

### 2. Horizontal Pod Autoscaler

#### `hpa.tf`
```hcl
# PACS Backend HPA
resource "kubernetes_horizontal_pod_autoscaler_v2" "pacs_backend" {
  metadata {
    name      = "pacs-backend-hpa"
    namespace = kubernetes_namespace.pacs.metadata[0].name
  }

  spec {
    scale_target_ref {
      api_version = "apps/v1"
      kind        = "Deployment"
      name        = kubernetes_deployment.pacs_backend.metadata[0].name
    }

    min_replicas = var.pacs_backend_min_replicas
    max_replicas = var.pacs_backend_max_replicas

    metric {
      type = "Resource"
      resource {
        name = "cpu"
        target {
          type                = "Utilization"
          average_utilization = 70
        }
      }
    }

    metric {
      type = "Resource"
      resource {
        name = "memory"
        target {
          type                = "Utilization"
          average_utilization = 80
        }
      }
    }
  }

  depends_on = [kubernetes_deployment.pacs_backend]
}

# Keycloak HPA
resource "kubernetes_horizontal_pod_autoscaler_v2" "keycloak" {
  metadata {
    name      = "keycloak-hpa"
    namespace = kubernetes_namespace.pacs.metadata[0].name
  }

  spec {
    scale_target_ref {
      api_version = "apps/v1"
      kind        = "Deployment"
      name        = kubernetes_deployment.keycloak.metadata[0].name
    }

    min_replicas = var.keycloak_min_replicas
    max_replicas = var.keycloak_max_replicas

    metric {
      type = "Resource"
      resource {
        name = "cpu"
        target {
          type                = "Utilization"
          average_utilization = 70
        }
      }
    }
  }

  depends_on = [kubernetes_deployment.keycloak]
}
```

---

## 🔒 보안 및 모니터링

### 1. Pod Security Policy

#### `pod-security.tf`
```hcl
# Pod Security Policy
resource "kubernetes_pod_security_policy" "pacs" {
  metadata {
    name = "pacs-psp"
  }

  spec {
    privileged                 = false
    allow_privilege_escalation = false
    required_drop_capabilities = ["ALL"]

    volumes = [
      "configMap",
      "emptyDir",
      "projected",
      "secret",
      "downwardAPI",
      "persistentVolumeClaim"
    ]

    run_as_user {
      rule = "MustRunAsNonRoot"
    }

    se_linux {
      rule = "RunAsAny"
    }

    fs_group {
      rule = "RunAsAny"
    }
  }
}

# Cluster Role
resource "kubernetes_cluster_role" "pacs_psp" {
  metadata {
    name = "pacs-psp"
  }

  rule {
    api_groups     = ["policy"]
    resources      = ["podsecuritypolicies"]
    verbs          = ["use"]
    resource_names = [kubernetes_pod_security_policy.pacs.metadata[0].name]
  }
}

# Cluster Role Binding
resource "kubernetes_cluster_role_binding" "pacs_psp" {
  metadata {
    name = "pacs-psp"
  }

  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.pacs_psp.metadata[0].name
  }

  subject {
    kind      = "ServiceAccount"
    name      = "default"
    namespace = kubernetes_namespace.pacs.metadata[0].name
  }
}
```

### 2. 모니터링 설정

#### `monitoring.tf`
```hcl
# Prometheus ServiceMonitor
resource "kubernetes_manifest" "pacs_backend_service_monitor" {
  manifest = {
    apiVersion = "monitoring.coreos.com/v1"
    kind       = "ServiceMonitor"
    metadata = {
      name      = "pacs-backend"
      namespace = kubernetes_namespace.pacs.metadata[0].name
      labels = {
        app = "pacs-backend"
      }
    }
    spec = {
      selector = {
        matchLabels = {
          app = "pacs-backend"
        }
      }
      endpoints = [
        {
          port = "http"
          path = "/metrics"
        }
      ]
    }
  }

  depends_on = [kubernetes_service.pacs_backend]
}

# Grafana Dashboard ConfigMap
resource "kubernetes_config_map" "grafana_dashboard" {
  metadata {
    name      = "pacs-dashboard"
    namespace = kubernetes_namespace.pacs.metadata[0].name
    labels = {
      grafana_dashboard = "1"
    }
  }

  data = {
    "pacs-dashboard.json" = jsonencode({
      dashboard = {
        title = "PACS Dashboard"
        panels = [
          {
            title = "PACS Backend CPU Usage"
            type  = "graph"
            targets = [
              {
                expr = "rate(container_cpu_usage_seconds_total{pod=~\"pacs-backend-.*\"}[5m])"
              }
            ]
          }
        ]
      }
    })
  }
}
```

---

## 🧪 실습 및 테스트

### 1. EKS 클러스터 생성 테스트

#### `test-eks-creation.sh`
```bash
#!/bin/bash
# EKS 클러스터 생성 테스트 스크립트

echo "Testing EKS cluster creation..."

# Terraform 초기화
echo "1. Initializing Terraform..."
terraform init

# Terraform 검증
echo "2. Validating configuration..."
terraform validate

# EKS 클러스터 생성
echo "3. Creating EKS cluster..."
terraform apply -target=aws_eks_cluster.main -auto-approve

# 노드 그룹 생성
echo "4. Creating node groups..."
terraform apply -target=aws_eks_node_group.main -auto-approve

# EKS 클러스터 확인
echo "5. Verifying EKS cluster..."
aws eks describe-cluster --name pacs-development

echo "EKS cluster creation test completed! 🎉"
```

### 2. Kubernetes 리소스 배포 테스트

#### `test-k8s-deployment.sh`
```bash
#!/bin/bash
# Kubernetes 리소스 배포 테스트 스크립트

echo "Testing Kubernetes resource deployment..."

# kubectl 설정
echo "1. Configuring kubectl..."
aws eks update-kubeconfig --region ap-northeast-2 --name pacs-development

# 네임스페이스 확인
echo "2. Checking namespaces..."
kubectl get namespaces

# PACS 네임스페이스 확인
echo "3. Checking PACS namespace..."
kubectl get namespaces pacs

# ConfigMap 확인
echo "4. Checking ConfigMaps..."
kubectl get configmaps -n pacs

# Secret 확인
echo "5. Checking Secrets..."
kubectl get secrets -n pacs

# Deployment 확인
echo "6. Checking Deployments..."
kubectl get deployments -n pacs

# Service 확인
echo "7. Checking Services..."
kubectl get services -n pacs

# Pod 확인
echo "8. Checking Pods..."
kubectl get pods -n pacs

echo "Kubernetes deployment test completed! 🎉"
```

### 3. 애플리케이션 연결 테스트

#### `test-app-connectivity.sh`
```bash
#!/bin/bash
# 애플리케이션 연결 테스트 스크립트

echo "Testing application connectivity..."

# Port Forward 설정
echo "1. Setting up port forwarding..."
kubectl port-forward -n pacs service/pacs-backend 8080:80 &
KUBECTL_PID=$!

# 잠시 대기
sleep 5

# PACS Backend Health Check
echo "2. Testing PACS Backend health..."
curl -f http://localhost:8080/health || echo "PACS Backend health check failed"

# Keycloak Health Check
echo "3. Testing Keycloak health..."
kubectl port-forward -n pacs service/keycloak 8081:80 &
KUBECTL_PID2=$!

sleep 5
curl -f http://localhost:8081/auth/health || echo "Keycloak health check failed"

# 정리
echo "4. Cleaning up..."
kill $KUBECTL_PID $KUBECTL_PID2 2>/dev/null

echo "Application connectivity test completed! 🎉"
```

### 4. 오토스케일링 테스트

#### `test-autoscaling.sh`
```bash
#!/bin/bash
# 오토스케일링 테스트 스크립트

echo "Testing autoscaling..."

# HPA 상태 확인
echo "1. Checking HPA status..."
kubectl get hpa -n pacs

# 노드 상태 확인
echo "2. Checking node status..."
kubectl get nodes

# Pod 리소스 사용량 확인
echo "3. Checking pod resource usage..."
kubectl top pods -n pacs

# 부하 테스트 (간단한 예시)
echo "4. Running load test..."
for i in {1..10}; do
  kubectl run load-test-$i --image=busybox --rm -i --restart=Never -- /bin/sh -c "while true; do wget -q -O- http://pacs-backend.pacs.svc.cluster.local/health; sleep 1; done" &
done

# 30초 대기
sleep 30

# 스케일링 확인
echo "5. Checking scaling..."
kubectl get hpa -n pacs
kubectl get pods -n pacs

# 정리
echo "6. Cleaning up load test..."
kubectl delete pods -l run=load-test --force --grace-period=0

echo "Autoscaling test completed! 🎉"
```

---

## 🔧 문제 해결

### 1. 노드 그룹 생성 실패

**증상**: 노드 그룹 생성 실패
```
Error: creating EKS Node Group: InvalidParameterException: The provided role doesn't have the Amazon EKS Managed Node Group service linked role
```

**해결 방법**:
```hcl
# EKS 서비스 연결 역할 생성
resource "aws_iam_service_linked_role" "eks_nodegroup" {
  aws_service_name = "eks-nodegroup.amazonaws.com"
}

# 노드 그룹 생성 전 역할 확인
resource "aws_eks_node_group" "main" {
  # ... 기타 설정 ...
  
  depends_on = [
    aws_iam_service_linked_role.eks_nodegroup,
    aws_iam_role_policy_attachment.eks_worker_node_policy
  ]
}
```

### 2. Pod 스케줄링 실패

**증상**: Pod가 스케줄링되지 않음
```
Warning: 0/2 nodes are available: 2 Insufficient cpu
```

**해결 방법**:
```hcl
# 노드 그룹 인스턴스 타입 변경
variable "node_groups" {
  default = {
    general = {
      instance_types = ["t3.large"]  # 더 큰 인스턴스 사용
      # ... 기타 설정 ...
    }
  }
}

# 리소스 요청량 조정
resource "kubernetes_deployment" "pacs_backend" {
  spec {
    template {
      spec {
        container {
          resources {
            requests = {
              cpu    = "100m"  # 요청량 줄이기
              memory = "256Mi"
            }
          }
        }
      }
    }
  }
}
```

### 3. 서비스 연결 실패

**증상**: 서비스 간 연결 실패
```
Error: connection refused
```

**해결 방법**:
```hcl
# 서비스 포트 확인
resource "kubernetes_service" "pacs_backend" {
  spec {
    port {
      port        = 80
      target_port = 8080  # 컨테이너 포트와 일치하는지 확인
      protocol    = "TCP"
    }
  }
}

# 네트워크 정책 확인
resource "kubernetes_network_policy" "pacs" {
  metadata {
    name      = "pacs-network-policy"
    namespace = kubernetes_namespace.pacs.metadata[0].name
  }

  spec {
    pod_selector {
      match_labels = {
        app = "pacs-backend"
      }
    }

    ingress {
      from {
        pod_selector {
          match_labels = {
            app = "keycloak"
          }
        }
      }
    }
  }
}
```

---

## 📚 다음 단계

이제 EKS 클러스터를 성공적으로 설정했으니 다음 문서들을 학습하세요:

1. **Application Load Balancer** - 로드 밸런싱 설정
2. **Auto Scaling 그룹** - 자동 스케일링 설정
3. **CI/CD 파이프라인** - 배포 자동화

---

## 📖 참고 자료

- [AWS EKS 공식 문서](https://docs.aws.amazon.com/eks/)
- [Kubernetes 공식 문서](https://kubernetes.io/docs/)
- [EKS Best Practices](https://aws.github.io/aws-eks-best-practices/)

이제 PACS 프로젝트의 컨테이너 오케스트레이션을 위한 EKS 클러스터가 준비되었습니다! 🚀
