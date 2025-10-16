# 📊 모니터링 및 로깅 가이드

Terraform을 사용하여 AWS CloudWatch, Prometheus, Grafana를 통한 종합적인 모니터링 및 로깅 시스템을 구성하는 방법을 학습합니다. PACS 프로젝트의 전체 시스템 가시성과 운영 관리를 위한 모니터링 설정을 중심으로 다룹니다.

## 📋 목차

1. [모니터링 및 로깅이란?](#모니터링-및-로깅이란)
2. [CloudWatch 기본 설정](#cloudwatch-기본-설정)
3. [PACS 프로젝트 모니터링 구성](#pacs-프로젝트-모니터링-구성)
4. [Prometheus 및 Grafana 설정](#prometheus-및-grafana-설정)
5. [고급 모니터링 기능](#고급-모니터링-기능)
6. [실습 및 테스트](#실습-및-테스트)

---

## 🎯 모니터링 및 로깅이란?

**모니터링 및 로깅**은 시스템의 상태, 성능, 보안을 실시간으로 추적하고 분석하는 프로세스입니다.

### 주요 특징
- **실시간 모니터링**: 시스템 상태를 실시간으로 추적
- **성능 분석**: 애플리케이션 및 인프라 성능 측정
- **장애 감지**: 문제 발생 시 즉시 알림
- **트렌드 분석**: 장기적인 성능 패턴 분석
- **용량 계획**: 리소스 사용량 기반 확장 계획

### PACS 프로젝트에서의 활용
- **인프라 모니터링**: AWS 리소스 상태 및 성능
- **애플리케이션 모니터링**: PACS Backend 성능 및 오류
- **데이터베이스 모니터링**: PostgreSQL 성능 및 연결 상태
- **보안 모니터링**: 접근 로그 및 보안 이벤트
- **비즈니스 메트릭**: DICOM 처리량, 사용자 활동

---

## 🔧 CloudWatch 기본 설정

### 1. CloudWatch 로그 그룹

#### `cloudwatch-logs.tf`
```hcl
# PACS Backend 로그 그룹
resource "aws_cloudwatch_log_group" "pacs_backend" {
  name              = "/aws/ecs/pacs-backend"
  retention_in_days = var.log_retention_days

  tags = {
    Name        = "pacs-backend-logs"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Keycloak 로그 그룹
resource "aws_cloudwatch_log_group" "keycloak" {
  name              = "/aws/ecs/keycloak"
  retention_in_days = var.log_retention_days

  tags = {
    Name        = "keycloak-logs"
    Environment = var.environment
    Project     = var.project_name
  }
}

# ALB 액세스 로그 그룹
resource "aws_cloudwatch_log_group" "alb_access" {
  name              = "/aws/applicationloadbalancer/pacs-alb"
  retention_in_days = var.log_retention_days

  tags = {
    Name        = "pacs-alb-access-logs"
    Environment = var.environment
    Project     = var.project_name
  }
}

# VPC Flow Logs 그룹
resource "aws_cloudwatch_log_group" "vpc_flow" {
  name              = "/aws/vpc/flowlogs"
  retention_in_days = var.log_retention_days

  tags = {
    Name        = "vpc-flow-logs"
    Environment = var.environment
    Project     = var.project_name
  }
}

# RDS 로그 그룹
resource "aws_cloudwatch_log_group" "rds_postgresql" {
  name              = "/aws/rds/instance/pacs-postgresql/postgresql"
  retention_in_days = var.log_retention_days

  tags = {
    Name        = "pacs-postgresql-logs"
    Environment = var.environment
    Project     = var.project_name
  }
}
```

### 2. CloudWatch 메트릭 필터

#### `cloudwatch-metrics.tf`
```hcl
# PACS Backend 오류 메트릭 필터
resource "aws_cloudwatch_log_metric_filter" "pacs_backend_errors" {
  name           = "pacs-backend-errors"
  log_group_name = aws_cloudwatch_log_group.pacs_backend.name
  pattern        = "[timestamp, request_id, level=\"ERROR\", ...]"

  metric_transformation {
    name      = "PACSBackendErrors"
    namespace = "PACS/Application"
    value     = "1"
  }
}

# PACS Backend 응답 시간 메트릭 필터
resource "aws_cloudwatch_log_metric_filter" "pacs_backend_response_time" {
  name           = "pacs-backend-response-time"
  log_group_name = aws_cloudwatch_log_group.pacs_backend.name
  pattern        = "[timestamp, request_id, level=\"INFO\", message=\"Request completed\", duration=*]"

  metric_transformation {
    name      = "PACSBackendResponseTime"
    namespace = "PACS/Application"
    value     = "$duration"
  }
}

# DICOM 처리 메트릭 필터
resource "aws_cloudwatch_log_metric_filter" "dicom_processing" {
  name           = "dicom-processing"
  log_group_name = aws_cloudwatch_log_group.pacs_backend.name
  pattern        = "[timestamp, request_id, level=\"INFO\", message=\"DICOM processed\", study_uid=*]"

  metric_transformation {
    name      = "DICOMProcessed"
    namespace = "PACS/Business"
    value     = "1"
  }
}

# 인증 실패 메트릭 필터
resource "aws_cloudwatch_log_metric_filter" "auth_failures" {
  name           = "auth-failures"
  log_group_name = aws_cloudwatch_log_group.keycloak.name
  pattern        = "[timestamp, level=\"WARN\", message=\"Authentication failed\", ...]"

  metric_transformation {
    name      = "AuthenticationFailures"
    namespace = "PACS/Security"
    value     = "1"
  }
}
```

### 3. CloudWatch 알람

#### `cloudwatch-alarms.tf`
```hcl
# PACS Backend 오류율 알람
resource "aws_cloudwatch_metric_alarm" "pacs_backend_error_rate" {
  alarm_name          = "pacs-backend-error-rate"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "PACSBackendErrors"
  namespace           = "PACS/Application"
  period              = "300"
  statistic           = "Sum"
  threshold           = "10"
  alarm_description   = "This metric monitors PACS Backend error rate"
  alarm_actions       = [aws_sns_topic.alerts.arn]

  tags = {
    Name        = "pacs-backend-error-rate"
    Environment = var.environment
    Project     = var.project_name
  }
}

# PACS Backend 응답 시간 알람
resource "aws_cloudwatch_metric_alarm" "pacs_backend_response_time" {
  alarm_name          = "pacs-backend-response-time"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "PACSBackendResponseTime"
  namespace           = "PACS/Application"
  period              = "300"
  statistic           = "Average"
  threshold           = "5000"
  alarm_description   = "This metric monitors PACS Backend response time"
  alarm_actions       = [aws_sns_topic.alerts.arn]

  tags = {
    Name        = "pacs-backend-response-time"
    Environment = var.environment
    Project     = var.project_name
  }
}

# RDS CPU 사용률 알람
resource "aws_cloudwatch_metric_alarm" "rds_cpu_utilization" {
  alarm_name          = "rds-cpu-utilization"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "CPUUtilization"
  namespace           = "AWS/RDS"
  period              = "300"
  statistic           = "Average"
  threshold           = "80"
  alarm_description   = "This metric monitors RDS CPU utilization"
  alarm_actions       = [aws_sns_topic.alerts.arn]

  dimensions = {
    DBInstanceIdentifier = aws_db_instance.pacs_postgresql.id
  }

  tags = {
    Name        = "rds-cpu-utilization"
    Environment = var.environment
    Project     = var.project_name
  }
}

# RDS 연결 수 알람
resource "aws_cloudwatch_metric_alarm" "rds_connections" {
  alarm_name          = "rds-connections"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "DatabaseConnections"
  namespace           = "AWS/RDS"
  period              = "300"
  statistic           = "Average"
  threshold           = "80"
  alarm_description   = "This metric monitors RDS connection count"
  alarm_actions       = [aws_sns_topic.alerts.arn]

  dimensions = {
    DBInstanceIdentifier = aws_db_instance.pacs_postgresql.id
  }

  tags = {
    Name        = "rds-connections"
    Environment = var.environment
    Project     = var.project_name
  }
}

# S3 버킷 크기 알람
resource "aws_cloudwatch_metric_alarm" "s3_bucket_size" {
  alarm_name          = "s3-bucket-size"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "BucketSizeBytes"
  namespace           = "AWS/S3"
  period              = "86400"
  statistic           = "Average"
  threshold           = "100000000000"  # 100GB
  alarm_description   = "This metric monitors S3 bucket size"
  alarm_actions       = [aws_sns_topic.alerts.arn]

  dimensions = {
    BucketName  = aws_s3_bucket.pacs_storage.bucket
    StorageType = "StandardStorage"
  }

  tags = {
    Name        = "s3-bucket-size"
    Environment = var.environment
    Project     = var.project_name
  }
}
```

---

## 🏥 PACS 프로젝트 모니터링 구성

### 1. 종합 대시보드

#### `cloudwatch-dashboard.tf`
```hcl
# PACS 프로젝트 종합 대시보드
resource "aws_cloudwatch_dashboard" "pacs_main" {
  dashboard_name = "PACS-Main-Dashboard"

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
            ["AWS/ApplicationELB", "RequestCount", "LoadBalancer", aws_lb.main.arn_suffix],
            [".", "TargetResponseTime", ".", "."],
            [".", "HTTPCode_Target_2XX_Count", ".", "."],
            [".", "HTTPCode_Target_4XX_Count", ".", "."],
            [".", "HTTPCode_Target_5XX_Count", ".", "."]
          ]
          view    = "timeSeries"
          stacked = false
          region  = var.aws_region
          title   = "ALB Metrics"
          period  = 300
        }
      },
      {
        type   = "metric"
        x      = 0
        y      = 6
        width  = 12
        height = 6

        properties = {
          metrics = [
            ["AWS/ECS", "CPUUtilization", "ServiceName", "pacs-backend", "ClusterName", "pacs-cluster"],
            [".", "MemoryUtilization", ".", ".", ".", "."],
            ["AWS/ECS", "RunningTaskCount", ".", ".", ".", "."]
          ]
          view    = "timeSeries"
          stacked = false
          region  = var.aws_region
          title   = "ECS Service Metrics"
          period  = 300
        }
      },
      {
        type   = "metric"
        x      = 0
        y      = 12
        width  = 12
        height = 6

        properties = {
          metrics = [
            ["AWS/RDS", "CPUUtilization", "DBInstanceIdentifier", aws_db_instance.pacs_postgresql.id],
            [".", "DatabaseConnections", ".", "."],
            [".", "FreeableMemory", ".", "."],
            [".", "FreeStorageSpace", ".", "."]
          ]
          view    = "timeSeries"
          stacked = false
          region  = var.aws_region
          title   = "RDS Metrics"
          period  = 300
        }
      },
      {
        type   = "metric"
        x      = 0
        y      = 18
        width  = 12
        height = 6

        properties = {
          metrics = [
            ["AWS/S3", "BucketSizeBytes", "BucketName", aws_s3_bucket.pacs_storage.bucket, "StorageType", "StandardStorage"],
            [".", "NumberOfObjects", ".", ".", ".", "."]
          ]
          view    = "timeSeries"
          stacked = false
          region  = var.aws_region
          title   = "S3 Storage Metrics"
          period  = 86400
        }
      },
      {
        type   = "metric"
        x      = 0
        y      = 24
        width  = 12
        height = 6

        properties = {
          metrics = [
            ["PACS/Application", "PACSBackendErrors"],
            [".", "PACSBackendResponseTime"],
            ["PACS/Business", "DICOMProcessed"],
            ["PACS/Security", "AuthenticationFailures"]
          ]
          view    = "timeSeries"
          stacked = false
          region  = var.aws_region
          title   = "Application Metrics"
          period  = 300
        }
      }
    ]
  })
}

# 인프라 대시보드
resource "aws_cloudwatch_dashboard" "pacs_infrastructure" {
  dashboard_name = "PACS-Infrastructure-Dashboard"

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
            ["AWS/AutoScaling", "GroupDesiredCapacity", "AutoScalingGroupName", aws_autoscaling_group.pacs_backend.name],
            [".", "GroupInServiceInstances", ".", "."],
            [".", "GroupTotalInstances", ".", "."]
          ]
          view    = "timeSeries"
          stacked = false
          region  = var.aws_region
          title   = "Auto Scaling Group"
          period  = 300
        }
      },
      {
        type   = "metric"
        x      = 0
        y      = 6
        width  = 12
        height = 6

        properties = {
          metrics = [
            ["AWS/VPC", "PacketsIn", "VpcId", aws_vpc.main.id],
            [".", "PacketsOut", ".", "."],
            [".", "BytesIn", ".", "."],
            [".", "BytesOut", ".", "."]
          ]
          view    = "timeSeries"
          stacked = false
          region  = var.aws_region
          title   = "VPC Network Metrics"
          period  = 300
        }
      }
    ]
  })
}
```

### 2. SNS 알림 설정

#### `sns-notifications.tf`
```hcl
# SNS 토픽
resource "aws_sns_topic" "alerts" {
  name = "pacs-alerts"

  tags = {
    Name        = "pacs-alerts"
    Environment = var.environment
    Project     = var.project_name
  }
}

# 이메일 구독
resource "aws_sns_topic_subscription" "email" {
  topic_arn = aws_sns_topic.alerts.arn
  protocol  = "email"
  endpoint  = var.alert_email
}

# Slack 웹훅 구독
resource "aws_sns_topic_subscription" "slack" {
  count     = var.slack_webhook_url != "" ? 1 : 0
  topic_arn = aws_sns_topic.alerts.arn
  protocol  = "https"
  endpoint  = var.slack_webhook_url
}

# Teams 웹훅 구독
resource "aws_sns_topic_subscription" "teams" {
  count     = var.teams_webhook_url != "" ? 1 : 0
  topic_arn = aws_sns_topic.alerts.arn
  protocol  = "https"
  endpoint  = var.teams_webhook_url
}
```

---

## 📈 Prometheus 및 Grafana 설정

### 1. Prometheus 설정

#### `prometheus.tf`
```hcl
# Prometheus ECS 태스크 정의
resource "aws_ecs_task_definition" "prometheus" {
  family                   = "prometheus"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = "512"
  memory                   = "1024"
  execution_role_arn       = aws_iam_role.ecs_execution_role.arn
  task_role_arn           = aws_iam_role.ecs_task_role.arn

  container_definitions = jsonencode([
    {
      name  = "prometheus"
      image = "prom/prometheus:latest"
      portMappings = [
        {
          containerPort = 9090
          protocol      = "tcp"
        }
      ]
      environment = [
        {
          name  = "PROMETHEUS_CONFIG"
          value = "prometheus.yml"
        }
      ]
      mountPoints = [
        {
          sourceVolume  = "prometheus-config"
          containerPath = "/etc/prometheus"
          readOnly      = true
        }
      ]
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = aws_cloudwatch_log_group.prometheus.name
          "awslogs-region"        = var.aws_region
          "awslogs-stream-prefix" = "ecs"
        }
      }
    }
  ])

  volume {
    name = "prometheus-config"
    efs_volume_configuration {
      file_system_id = aws_efs_file_system.prometheus_config.id
      root_directory = "/"
    }
  }

  tags = {
    Name        = "prometheus"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Prometheus EFS 파일 시스템
resource "aws_efs_file_system" "prometheus_config" {
  creation_token = "prometheus-config"
  encrypted      = true

  tags = {
    Name        = "prometheus-config"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Prometheus EFS 마운트 타겟
resource "aws_efs_mount_target" "prometheus_config" {
  count           = length(var.private_subnet_ids)
  file_system_id  = aws_efs_file_system.prometheus_config.id
  subnet_id       = var.private_subnet_ids[count.index]
  security_groups = [aws_security_group.efs.id]
}

# Prometheus EFS 보안 그룹
resource "aws_security_group" "efs" {
  name_prefix = "pacs-efs-"
  vpc_id      = var.vpc_id

  ingress {
    from_port   = 2049
    to_port     = 2049
    protocol    = "tcp"
    cidr_blocks = [var.vpc_cidr]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name        = "pacs-efs-sg"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Prometheus 로그 그룹
resource "aws_cloudwatch_log_group" "prometheus" {
  name              = "/aws/ecs/prometheus"
  retention_in_days = var.log_retention_days

  tags = {
    Name        = "prometheus-logs"
    Environment = var.environment
    Project     = var.project_name
  }
}
```

### 2. Grafana 설정

#### `grafana.tf`
```hcl
# Grafana ECS 태스크 정의
resource "aws_ecs_task_definition" "grafana" {
  family                   = "grafana"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = "1024"
  memory                   = "2048"
  execution_role_arn       = aws_iam_role.ecs_execution_role.arn
  task_role_arn           = aws_iam_role.ecs_task_role.arn

  container_definitions = jsonencode([
    {
      name  = "grafana"
      image = "grafana/grafana:latest"
      portMappings = [
        {
          containerPort = 3000
          protocol      = "tcp"
        }
      ]
      environment = [
        {
          name  = "GF_SECURITY_ADMIN_PASSWORD"
          value = var.grafana_admin_password
        },
        {
          name  = "GF_INSTALL_PLUGINS"
          value = "grafana-piechart-panel,grafana-worldmap-panel"
        }
      ]
      mountPoints = [
        {
          sourceVolume  = "grafana-storage"
          containerPath = "/var/lib/grafana"
          readOnly      = false
        }
      ]
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = aws_cloudwatch_log_group.grafana.name
          "awslogs-region"        = var.aws_region
          "awslogs-stream-prefix" = "ecs"
        }
      }
    }
  ])

  volume {
    name = "grafana-storage"
    efs_volume_configuration {
      file_system_id = aws_efs_file_system.grafana_storage.id
      root_directory = "/"
    }
  }

  tags = {
    Name        = "grafana"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Grafana EFS 파일 시스템
resource "aws_efs_file_system" "grafana_storage" {
  creation_token = "grafana-storage"
  encrypted      = true

  tags = {
    Name        = "grafana-storage"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Grafana EFS 마운트 타겟
resource "aws_efs_mount_target" "grafana_storage" {
  count           = length(var.private_subnet_ids)
  file_system_id  = aws_efs_file_system.grafana_storage.id
  subnet_id       = var.private_subnet_ids[count.index]
  security_groups = [aws_security_group.efs.id]
}

# Grafana 로그 그룹
resource "aws_cloudwatch_log_group" "grafana" {
  name              = "/aws/ecs/grafana"
  retention_in_days = var.log_retention_days

  tags = {
    Name        = "grafana-logs"
    Environment = var.environment
    Project     = var.project_name
  }
}
```

### 3. Prometheus 설정 파일

#### `prometheus-config.yml`
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "rules/*.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'pacs-backend'
    static_configs:
      - targets: ['pacs-backend-service:8080']
    metrics_path: '/metrics'
    scrape_interval: 30s

  - job_name: 'postgresql'
    static_configs:
      - targets: ['postgres-exporter:9187']
    scrape_interval: 30s

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']
    scrape_interval: 30s

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
    scrape_interval: 30s

  - job_name: 'cadvisor'
    static_configs:
      - targets: ['cadvisor:8080']
    scrape_interval: 30s
```

---

## 🔧 고급 모니터링 기능

### 1. 커스텀 메트릭 수집

#### `custom-metrics.tf`
```hcl
# CloudWatch 커스텀 메트릭을 위한 Lambda 함수
resource "aws_lambda_function" "custom_metrics_collector" {
  filename         = "custom_metrics_collector.zip"
  function_name    = "pacs-custom-metrics-collector"
  role            = aws_iam_role.lambda_custom_metrics.arn
  handler         = "index.handler"
  runtime         = "python3.9"
  timeout         = 60

  environment {
    variables = {
      CLOUDWATCH_NAMESPACE = "PACS/Custom"
      S3_BUCKET_NAME       = aws_s3_bucket.pacs_storage.bucket
      RDS_ENDPOINT         = aws_db_instance.pacs_postgresql.endpoint
    }
  }

  tags = {
    Name        = "pacs-custom-metrics-collector"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Lambda IAM 역할
resource "aws_iam_role" "lambda_custom_metrics" {
  name = "pacs-lambda-custom-metrics"

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

# Lambda 정책
resource "aws_iam_policy" "lambda_custom_metrics" {
  name        = "pacs-lambda-custom-metrics"
  description = "Policy for custom metrics Lambda"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "cloudwatch:PutMetricData",
          "s3:GetObject",
          "s3:ListBucket",
          "rds:DescribeDBInstances",
          "rds:DescribeDBClusters"
        ]
        Resource = "*"
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_custom_metrics" {
  role       = aws_iam_role.lambda_custom_metrics.name
  policy_arn = aws_iam_policy.lambda_custom_metrics.arn
}

# EventBridge 규칙 (5분마다 실행)
resource "aws_cloudwatch_event_rule" "custom_metrics" {
  name                = "pacs-custom-metrics"
  description         = "Trigger custom metrics Lambda"
  schedule_expression = "rate(5 minutes)"
}

resource "aws_cloudwatch_event_target" "custom_metrics" {
  rule      = aws_cloudwatch_event_rule.custom_metrics.name
  target_id = "CustomMetricsTarget"
  arn       = aws_lambda_function.custom_metrics_collector.arn
}

resource "aws_lambda_permission" "allow_eventbridge" {
  statement_id  = "AllowExecutionFromEventBridge"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.custom_metrics_collector.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.custom_metrics.arn
}
```

### 2. 로그 분석 및 검색

#### `log-insights.tf`
```hcl
# CloudWatch Insights 쿼리
resource "aws_cloudwatch_query_definition" "pacs_error_analysis" {
  name = "PACS Error Analysis"

  log_group_names = [
    aws_cloudwatch_log_group.pacs_backend.name
  ]

  query_string = <<EOF
fields @timestamp, @message
| filter @message like /ERROR/
| sort @timestamp desc
| limit 100
EOF
}

resource "aws_cloudwatch_query_definition" "pacs_performance_analysis" {
  name = "PACS Performance Analysis"

  log_group_names = [
    aws_cloudwatch_log_group.pacs_backend.name
  ]

  query_string = <<EOF
fields @timestamp, @message
| filter @message like /Request completed/
| parse @message /duration=(?<duration>\d+)/
| stats avg(duration) by bin(5m)
EOF
}

resource "aws_cloudwatch_query_definition" "dicom_processing_analysis" {
  name = "DICOM Processing Analysis"

  log_group_names = [
    aws_cloudwatch_log_group.pacs_backend.name
  ]

  query_string = <<EOF
fields @timestamp, @message
| filter @message like /DICOM processed/
| parse @message /study_uid=(?<study_uid>\S+)/
| stats count() by bin(1h)
EOF
}
```

### 3. 모니터링 자동화

#### `monitoring-automation.tf`
```hcl
# 자동 스케일링을 위한 CloudWatch 메트릭
resource "aws_cloudwatch_metric_alarm" "auto_scaling_cpu" {
  alarm_name          = "pacs-auto-scaling-cpu"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "CPUUtilization"
  namespace           = "AWS/ECS"
  period              = "300"
  statistic           = "Average"
  threshold           = "70"
  alarm_description   = "This metric monitors ECS CPU utilization for auto scaling"
  alarm_actions       = [aws_autoscaling_policy.scale_out_cpu.arn]

  dimensions = {
    ServiceName = "pacs-backend"
    ClusterName = "pacs-cluster"
  }

  tags = {
    Name        = "pacs-auto-scaling-cpu"
    Environment = var.environment
    Project     = var.project_name
  }
}

# 자동 스케일링을 위한 CloudWatch 메트릭 (메모리)
resource "aws_cloudwatch_metric_alarm" "auto_scaling_memory" {
  alarm_name          = "pacs-auto-scaling-memory"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "MemoryUtilization"
  namespace           = "AWS/ECS"
  period              = "300"
  statistic           = "Average"
  threshold           = "80"
  alarm_description   = "This metric monitors ECS memory utilization for auto scaling"
  alarm_actions       = [aws_autoscaling_policy.scale_out_memory.arn]

  dimensions = {
    ServiceName = "pacs-backend"
    ClusterName = "pacs-cluster"
  }

  tags = {
    Name        = "pacs-auto-scaling-memory"
    Environment = var.environment
    Project     = var.project_name
  }
}
```

---

## 🧪 실습 및 테스트

### 1. 모니터링 시스템 테스트

#### `test-monitoring.sh`
```bash
#!/bin/bash
# 모니터링 시스템 테스트 스크립트

echo "Testing monitoring system..."

# CloudWatch 로그 그룹 확인
echo "1. Checking CloudWatch log groups..."
aws logs describe-log-groups --log-group-name-prefix "/aws/ecs/pacs"

# CloudWatch 메트릭 확인
echo "2. Checking CloudWatch metrics..."
aws cloudwatch list-metrics --namespace "PACS/Application"

# CloudWatch 알람 확인
echo "3. Checking CloudWatch alarms..."
aws cloudwatch describe-alarms --alarm-name-prefix "pacs-"

# SNS 토픽 확인
echo "4. Checking SNS topics..."
aws sns list-topics

# Prometheus 상태 확인
echo "5. Checking Prometheus status..."
kubectl get pods -l app=prometheus

# Grafana 상태 확인
echo "6. Checking Grafana status..."
kubectl get pods -l app=grafana

echo "Monitoring system test completed! 🎉"
```

### 2. 로그 수집 테스트

#### `test-log-collection.sh`
```bash
#!/bin/bash
# 로그 수집 테스트 스크립트

echo "Testing log collection..."

# PACS Backend 로그 확인
echo "1. Checking PACS Backend logs..."
aws logs describe-log-streams \
  --log-group-name "/aws/ecs/pacs-backend" \
  --order-by LastEventTime \
  --descending

# 최근 로그 이벤트 확인
echo "2. Checking recent log events..."
aws logs filter-log-events \
  --log-group-name "/aws/ecs/pacs-backend" \
  --start-time $(date -d '1 hour ago' +%s)000 \
  --end-time $(date +%s)000

# 오류 로그 확인
echo "3. Checking error logs..."
aws logs filter-log-events \
  --log-group-name "/aws/ecs/pacs-backend" \
  --filter-pattern "ERROR" \
  --start-time $(date -d '1 hour ago' +%s)000 \
  --end-time $(date +%s)000

echo "Log collection test completed! 🎉"
```

### 3. 알림 테스트

#### `test-notifications.sh`
```bash
#!/bin/bash
# 알림 테스트 스크립트

echo "Testing notifications..."

# SNS 토픽에 테스트 메시지 발송
echo "1. Sending test message to SNS topic..."
aws sns publish \
  --topic-arn "arn:aws:sns:ap-northeast-2:123456789012:pacs-alerts" \
  --message "Test message from monitoring system" \
  --subject "PACS Monitoring Test"

# CloudWatch 알람 테스트
echo "2. Testing CloudWatch alarm..."
aws cloudwatch set-alarm-state \
  --alarm-name "pacs-backend-error-rate" \
  --state-value ALARM \
  --state-reason "Testing alarm state"

# 5초 대기
sleep 5

# 알람 상태 복원
echo "3. Restoring alarm state..."
aws cloudwatch set-alarm-state \
  --alarm-name "pacs-backend-error-rate" \
  --state-value OK \
  --state-reason "Restoring alarm state"

echo "Notification test completed! 🎉"
```

---

## 🔧 문제 해결

### 1. CloudWatch 로그 수집 실패

**증상**: CloudWatch 로그가 수집되지 않음
```
Error: Log group does not exist
```

**해결 방법**:
```hcl
# 로그 그룹 생성 확인
resource "aws_cloudwatch_log_group" "pacs_backend" {
  name              = "/aws/ecs/pacs-backend"
  retention_in_days = 30
  
  # 로그 그룹이 존재하지 않을 경우 생성
  lifecycle {
    create_before_destroy = true
  }
}
```

### 2. Prometheus 메트릭 수집 실패

**증상**: Prometheus가 메트릭을 수집하지 못함
```
Error: connection refused
```

**해결 방법**:
```yaml
# Prometheus 설정에서 타겟 확인
scrape_configs:
  - job_name: 'pacs-backend'
    static_configs:
      - targets: ['pacs-backend-service:8080']
    metrics_path: '/metrics'
    scrape_interval: 30s
    scrape_timeout: 10s
```

### 3. SNS 알림 전송 실패

**증상**: SNS 알림이 전송되지 않음
```
Error: InvalidParameterValue
```

**해결 방법**:
```hcl
# SNS 토픽 구독 확인
resource "aws_sns_topic_subscription" "email" {
  topic_arn = aws_sns_topic.alerts.arn
  protocol  = "email"
  endpoint  = var.alert_email
  
  # 구독 확인 대기
  confirmation_timeout_in_minutes = 10
}
```

---

## 📚 다음 단계

이제 모니터링 및 로깅 시스템을 성공적으로 설정했으니 다음 문서들을 학습하세요:

1. **보안 및 컴플라이언스** - 보안 정책 및 감사
2. **비용 최적화** - AWS 비용 관리 및 최적화
3. **재해 복구** - 백업 및 복구 전략

---

## 📖 참고 자료

- [AWS CloudWatch 공식 문서](https://docs.aws.amazon.com/cloudwatch/)
- [Prometheus 공식 문서](https://prometheus.io/docs/)
- [Grafana 공식 문서](https://grafana.com/docs/)

이제 PACS 프로젝트의 종합적인 모니터링 및 로깅 시스템이 준비되었습니다! 🚀
