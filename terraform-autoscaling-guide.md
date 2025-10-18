# 📈 Auto Scaling 그룹 가이드

Terraform을 사용하여 AWS Auto Scaling Group (ASG)을 구성하고 관리하는 방법을 학습합니다. PACS 프로젝트의 자동 스케일링과 고가용성을 위한 ASG 설정을 중심으로 다룹니다.

## 📋 목차

1. [Auto Scaling Group이란?](#auto-scaling-group이란)
2. [기본 ASG 구성](#기본-asg-구성)
3. [PACS 프로젝트 ASG 설정](#pacs-프로젝트-asg-설정)
4. [스케일링 정책 및 메트릭](#스케일링-정책-및-메트릭)
5. [고급 설정 및 모니터링](#고급-설정-및-모니터링)
6. [실습 및 테스트](#실습-및-테스트)

---

## 🎯 Auto Scaling Group이란?

**AWS Auto Scaling Group (ASG)**은 애플리케이션의 수요에 따라 EC2 인스턴스를 자동으로 확장하거나 축소하는 서비스입니다.

### 주요 특징
- **자동 스케일링**: CPU, 메모리, 커스텀 메트릭 기반
- **고가용성**: Multi-AZ 배포로 장애 복구
- **비용 최적화**: 수요에 따른 인스턴스 수 조절
- **롤링 업데이트**: 무중단 배포
- **생명주기 훅**: 인스턴스 상태 전환 시 커스텀 작업

### PACS 프로젝트에서의 활용
- **PACS Backend**: DICOM 처리 부하에 따른 스케일링
- **Keycloak**: 인증 요청 증가 시 확장
- **Frontend**: 웹 트래픽 증가 시 확장
- **배치 작업**: 대용량 데이터 처리용 임시 인스턴스

---

## 🔧 기본 ASG 구성

### 1. Launch Template 생성

#### `launch-template.tf`
```hcl
# Launch Template
resource "aws_launch_template" "pacs_backend" {
  name_prefix   = "${var.project_name}-backend-"
  image_id      = var.ami_id
  instance_type = var.instance_type
  key_name      = var.key_pair_name

  vpc_security_group_ids = [aws_security_group.asg.id]

  # IAM 인스턴스 프로필
  iam_instance_profile {
    name = aws_iam_instance_profile.asg.name
  }

  # 사용자 데이터
  user_data = base64encode(templatefile("${path.module}/user_data.sh", {
    project_name = var.project_name
    environment  = var.environment
  }))

  # 블록 디바이스 매핑
  block_device_mappings {
    device_name = "/dev/xvda"
    ebs {
      volume_size           = var.volume_size
      volume_type           = "gp3"
      delete_on_termination = true
      encrypted             = true
    }
  }

  # 태그 설정
  tag_specifications {
    resource_type = "instance"
    tags = {
      Name        = "${var.project_name}-backend"
      Environment = var.environment
      Project     = var.project_name
      Type        = "PACS Backend"
    }
  }

  tags = {
    Name        = "${var.project_name}-backend-template"
    Environment = var.environment
    Project     = var.project_name
  }
}

# ASG 보안 그룹
resource "aws_security_group" "asg" {
  name_prefix = "${var.project_name}-asg-"
  vpc_id      = var.vpc_id

  # HTTP
  ingress {
    from_port       = 8080
    to_port         = 8080
    protocol        = "tcp"
    security_groups = [var.alb_security_group_id]
  }

  # SSH
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
    Name        = "${var.project_name}-asg-sg"
    Environment = var.environment
    Project     = var.project_name
  }
}
```

### 2. IAM 역할 및 정책

#### `asg-iam.tf`
```hcl
# ASG IAM 역할
resource "aws_iam_role" "asg" {
  name = "${var.project_name}-asg-role"

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
    Name        = "${var.project_name}-asg-role"
    Environment = var.environment
    Project     = var.project_name
  }
}

# ASG IAM 정책
resource "aws_iam_policy" "asg" {
  name        = "${var.project_name}-asg-policy"
  description = "Policy for ASG instances"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject"
        ]
        Resource = [
          "arn:aws:s3:::${var.s3_bucket_name}/*"
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "rds-db:connect"
        ]
        Resource = [
          "arn:aws:rds-db:${var.aws_region}:${var.aws_account_id}:dbuser:${var.rds_instance_id}/pacs_app"
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "elasticache:Connect"
        ]
        Resource = [
          "arn:aws:elasticache:${var.aws_region}:${var.aws_account_id}:cluster:${var.redis_cluster_id}"
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents",
          "logs:DescribeLogStreams"
        ]
        Resource = "arn:aws:logs:${var.aws_region}:${var.aws_account_id}:log-group:/aws/ec2/${var.project_name}/*"
      }
    ]
  })
}

# 정책 연결
resource "aws_iam_role_policy_attachment" "asg" {
  role       = aws_iam_role.asg.name
  policy_arn = aws_iam_policy.asg.arn
}

# IAM 인스턴스 프로필
resource "aws_iam_instance_profile" "asg" {
  name = "${var.project_name}-asg-profile"
  role = aws_iam_role.asg.name

  tags = {
    Name        = "${var.project_name}-asg-profile"
    Environment = var.environment
    Project     = var.project_name
  }
}
```

### 3. Auto Scaling Group 생성

#### `auto-scaling-group.tf`
```hcl
# Auto Scaling Group
resource "aws_autoscaling_group" "pacs_backend" {
  name                = "${var.project_name}-backend-asg"
  vpc_zone_identifier = var.private_subnet_ids
  target_group_arns   = [var.target_group_arn]
  health_check_type   = "ELB"
  health_check_grace_period = 300

  min_size         = var.min_size
  max_size         = var.max_size
  desired_capacity = var.desired_capacity

  launch_template {
    id      = aws_launch_template.pacs_backend.id
    version = "$Latest"
  }

  # 인스턴스 보호 설정
  protect_from_scale_in = var.environment == "production"

  # 태그 설정
  tag {
    key                 = "Name"
    value               = "${var.project_name}-backend"
    propagate_at_launch = true
  }

  tag {
    key                 = "Environment"
    value               = var.environment
    propagate_at_launch = true
  }

  tag {
    key                 = "Project"
    value               = var.project_name
    propagate_at_launch = true
  }

  # 생명주기 훅
  initial_lifecycle_hook {
    name                 = "pacs-backend-launch"
    default_result       = "CONTINUE"
    heartbeat_timeout    = 2000
    lifecycle_transition = "autoscaling:EC2_INSTANCE_LAUNCHING"
  }

  initial_lifecycle_hook {
    name                 = "pacs-backend-terminate"
    default_result       = "CONTINUE"
    heartbeat_timeout    = 300
    lifecycle_transition = "autoscaling:EC2_INSTANCE_TERMINATING"
  }

  depends_on = [aws_launch_template.pacs_backend]
}
```

---

## 🏥 PACS 프로젝트 ASG 설정

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

# ASG 설정
variable "ami_id" {
  description = "AMI ID for instances"
  type        = string
}

variable "instance_type" {
  description = "Instance type"
  type        = string
  default     = "t3.medium"
}

variable "key_pair_name" {
  description = "Key pair name for SSH access"
  type        = string
}

variable "volume_size" {
  description = "EBS volume size in GB"
  type        = number
  default     = 20
}

# 스케일링 설정
variable "min_size" {
  description = "Minimum number of instances"
  type        = number
  default     = 1
}

variable "max_size" {
  description = "Maximum number of instances"
  type        = number
  default     = 10
}

variable "desired_capacity" {
  description = "Desired number of instances"
  type        = number
  default     = 2
}

# 네트워크 설정
variable "vpc_id" {
  description = "VPC ID"
  type        = string
}

variable "private_subnet_ids" {
  description = "Private subnet IDs"
  type        = list(string)
}

variable "alb_security_group_id" {
  description = "ALB security group ID"
  type        = string
}

variable "admin_cidr_blocks" {
  description = "Admin CIDR blocks for SSH access"
  type        = list(string)
  default     = ["0.0.0.0/0"]
}

# 외부 서비스 설정
variable "s3_bucket_name" {
  description = "S3 bucket name"
  type        = string
}

variable "rds_instance_id" {
  description = "RDS instance ID"
  type        = string
}

variable "redis_cluster_id" {
  description = "Redis cluster ID"
  type        = string
}

variable "target_group_arn" {
  description = "Target group ARN"
  type        = string
}

variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "ap-northeast-2"
}

variable "aws_account_id" {
  description = "AWS account ID"
  type        = string
}
```

### 2. 스케일링 정책

#### `scaling-policies.tf`
```hcl
# CPU 기반 스케일 아웃 정책
resource "aws_autoscaling_policy" "scale_out_cpu" {
  name                   = "${var.project_name}-scale-out-cpu"
  scaling_adjustment     = 1
  adjustment_type        = "ChangeInCapacity"
  cooldown               = 300
  autoscaling_group_name = aws_autoscaling_group.pacs_backend.name
}

# CPU 기반 스케일 인 정책
resource "aws_autoscaling_policy" "scale_in_cpu" {
  name                   = "${var.project_name}-scale-in-cpu"
  scaling_adjustment     = -1
  adjustment_type        = "ChangeInCapacity"
  cooldown               = 300
  autoscaling_group_name = aws_autoscaling_group.pacs_backend.name
}

# 메모리 기반 스케일 아웃 정책
resource "aws_autoscaling_policy" "scale_out_memory" {
  name                   = "${var.project_name}-scale-out-memory"
  scaling_adjustment     = 1
  adjustment_type        = "ChangeInCapacity"
  cooldown               = 300
  autoscaling_group_name = aws_autoscaling_group.pacs_backend.name
}

# 메모리 기반 스케일 인 정책
resource "aws_autoscaling_policy" "scale_in_memory" {
  name                   = "${var.project_name}-scale-in-memory"
  scaling_adjustment     = -1
  adjustment_type        = "ChangeInCapacity"
  cooldown               = 300
  autoscaling_group_name = aws_autoscaling_group.pacs_backend.name
}

# 커스텀 메트릭 기반 스케일 아웃 (대기열 길이)
resource "aws_autoscaling_policy" "scale_out_queue" {
  name                   = "${var.project_name}-scale-out-queue"
  scaling_adjustment     = 2
  adjustment_type        = "ChangeInCapacity"
  cooldown               = 300
  autoscaling_group_name = aws_autoscaling_group.pacs_backend.name
}

# 커스텀 메트릭 기반 스케일 인 (대기열 길이)
resource "aws_autoscaling_policy" "scale_in_queue" {
  name                   = "${var.project_name}-scale-in-queue"
  scaling_adjustment     = -1
  adjustment_type        = "ChangeInCapacity"
  cooldown               = 300
  autoscaling_group_name = aws_autoscaling_group.pacs_backend.name
}
```

### 3. CloudWatch 알람

#### `cloudwatch-alarms.tf`
```hcl
# CPU 사용률 높음 알람
resource "aws_cloudwatch_metric_alarm" "cpu_high" {
  alarm_name          = "${var.project_name}-cpu-high"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "CPUUtilization"
  namespace           = "AWS/EC2"
  period              = "300"
  statistic           = "Average"
  threshold           = "70"
  alarm_description   = "This metric monitors CPU utilization"
  alarm_actions       = [aws_autoscaling_policy.scale_out_cpu.arn]

  dimensions = {
    AutoScalingGroupName = aws_autoscaling_group.pacs_backend.name
  }

  tags = {
    Name        = "${var.project_name}-cpu-high"
    Environment = var.environment
    Project     = var.project_name
  }
}

# CPU 사용률 낮음 알람
resource "aws_cloudwatch_metric_alarm" "cpu_low" {
  alarm_name          = "${var.project_name}-cpu-low"
  comparison_operator = "LessThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "CPUUtilization"
  namespace           = "AWS/EC2"
  period              = "300"
  statistic           = "Average"
  threshold           = "20"
  alarm_description   = "This metric monitors CPU utilization"
  alarm_actions       = [aws_autoscaling_policy.scale_in_cpu.arn]

  dimensions = {
    AutoScalingGroupName = aws_autoscaling_group.pacs_backend.name
  }

  tags = {
    Name        = "${var.project_name}-cpu-low"
    Environment = var.environment
    Project     = var.project_name
  }
}

# 메모리 사용률 높음 알람
resource "aws_cloudwatch_metric_alarm" "memory_high" {
  alarm_name          = "${var.project_name}-memory-high"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "MemoryUtilization"
  namespace           = "CWAgent"
  period              = "300"
  statistic           = "Average"
  threshold           = "80"
  alarm_description   = "This metric monitors memory utilization"
  alarm_actions       = [aws_autoscaling_policy.scale_out_memory.arn]

  dimensions = {
    AutoScalingGroupName = aws_autoscaling_group.pacs_backend.name
  }

  tags = {
    Name        = "${var.project_name}-memory-high"
    Environment = var.environment
    Project     = var.project_name
  }
}

# 메모리 사용률 낮음 알람
resource "aws_cloudwatch_metric_alarm" "memory_low" {
  alarm_name          = "${var.project_name}-memory-low"
  comparison_operator = "LessThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "MemoryUtilization"
  namespace           = "CWAgent"
  period              = "300"
  statistic           = "Average"
  threshold           = "30"
  alarm_description   = "This metric monitors memory utilization"
  alarm_actions       = [aws_autoscaling_policy.scale_in_memory.arn]

  dimensions = {
    AutoScalingGroupName = aws_autoscaling_group.pacs_backend.name
  }

  tags = {
    Name        = "${var.project_name}-memory-low"
    Environment = var.environment
    Project     = var.project_name
  }
}

# 커스텀 메트릭: 대기열 길이
resource "aws_cloudwatch_metric_alarm" "queue_length_high" {
  alarm_name          = "${var.project_name}-queue-length-high"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "QueueLength"
  namespace           = "PACS/Custom"
  period              = "300"
  statistic           = "Average"
  threshold           = "100"
  alarm_description   = "This metric monitors queue length"
  alarm_actions       = [aws_autoscaling_policy.scale_out_queue.arn]

  dimensions = {
    AutoScalingGroupName = aws_autoscaling_group.pacs_backend.name
  }

  tags = {
    Name        = "${var.project_name}-queue-length-high"
    Environment = var.environment
    Project     = var.project_name
  }
}

# 커스텀 메트릭: 대기열 길이 낮음
resource "aws_cloudwatch_metric_alarm" "queue_length_low" {
  alarm_name          = "${var.project_name}-queue-length-low"
  comparison_operator = "LessThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "QueueLength"
  namespace           = "PACS/Custom"
  period              = "300"
  statistic           = "Average"
  threshold           = "10"
  alarm_description   = "This metric monitors queue length"
  alarm_actions       = [aws_autoscaling_policy.scale_in_queue.arn]

  dimensions = {
    AutoScalingGroupName = aws_autoscaling_group.pacs_backend.name
  }

  tags = {
    Name        = "${var.project_name}-queue-length-low"
    Environment = var.environment
    Project     = var.project_name
  }
}
```

---

## 📊 스케일링 정책 및 메트릭

### 1. 예측적 스케일링

#### `predictive-scaling.tf`
```hcl
# 예측적 스케일링 설정
resource "aws_autoscaling_policy" "predictive_scaling" {
  name                   = "${var.project_name}-predictive-scaling"
  autoscaling_group_name = aws_autoscaling_group.pacs_backend.name
  policy_type            = "PredictiveScaling"

  predictive_scaling_configuration {
    metric_specification {
      target_value = 70
      predefined_metric_specification {
        predefined_metric_type = "ASGAverageCPUUtilization"
      }
    }
    mode                         = "ForecastAndScale"
    scheduling_buffer_time       = 10
    max_capacity_breach_behavior = "HonorMaxCapacity"
    max_capacity_buffer          = 10
  }
}
```

### 2. 스케일링 일정

#### `scheduled-scaling.tf`
```hcl
# 스케일링 일정 (업무 시간)
resource "aws_autoscaling_schedule" "scale_up_work_hours" {
  scheduled_action_name  = "scale-up-work-hours"
  min_size              = 3
  max_size              = 8
  desired_capacity      = 5
  recurrence            = "0 9 * * MON-FRI"  # 평일 오전 9시
  autoscaling_group_name = aws_autoscaling_group.pacs_backend.name
}

# 스케일링 일정 (야간 시간)
resource "aws_autoscaling_schedule" "scale_down_night" {
  scheduled_action_name  = "scale-down-night"
  min_size              = 1
  max_size              = 3
  desired_capacity      = 1
  recurrence            = "0 22 * * MON-FRI"  # 평일 오후 10시
  autoscaling_group_name = aws_autoscaling_group.pacs_backend.name
}

# 스케일링 일정 (주말)
resource "aws_autoscaling_schedule" "scale_down_weekend" {
  scheduled_action_name  = "scale-down-weekend"
  min_size              = 1
  max_size              = 2
  desired_capacity      = 1
  recurrence            = "0 18 * * SAT"  # 토요일 오후 6시
  autoscaling_group_name = aws_autoscaling_group.pacs_backend.name
}
```

### 3. 커스텀 메트릭

#### `custom-metrics.tf`
```hcl
# 커스텀 메트릭을 위한 Lambda 함수
resource "aws_lambda_function" "custom_metrics" {
  filename         = "custom_metrics.zip"
  function_name    = "${var.project_name}-custom-metrics"
  role            = aws_iam_role.lambda_custom_metrics.arn
  handler         = "index.handler"
  runtime         = "python3.9"
  timeout         = 60

  environment {
    variables = {
      ASG_NAME = aws_autoscaling_group.pacs_backend.name
    }
  }

  tags = {
    Name        = "${var.project_name}-custom-metrics"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Lambda IAM 역할
resource "aws_iam_role" "lambda_custom_metrics" {
  name = "${var.project_name}-lambda-custom-metrics"

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
  name        = "${var.project_name}-lambda-custom-metrics"
  description = "Policy for custom metrics Lambda"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "cloudwatch:PutMetricData",
          "autoscaling:DescribeAutoScalingGroups",
          "ec2:DescribeInstances"
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
  name                = "${var.project_name}-custom-metrics"
  description         = "Trigger custom metrics Lambda"
  schedule_expression = "rate(5 minutes)"
}

resource "aws_cloudwatch_event_target" "custom_metrics" {
  rule      = aws_cloudwatch_event_rule.custom_metrics.name
  target_id = "CustomMetricsTarget"
  arn       = aws_lambda_function.custom_metrics.arn
}

resource "aws_lambda_permission" "allow_eventbridge" {
  statement_id  = "AllowExecutionFromEventBridge"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.custom_metrics.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.custom_metrics.arn
}
```

---

## 🔧 고급 설정 및 모니터링

### 1. 생명주기 훅

#### `lifecycle-hooks.tf`
```hcl
# 생명주기 훅을 위한 SNS 토픽
resource "aws_sns_topic" "asg_lifecycle" {
  name = "${var.project_name}-asg-lifecycle"

  tags = {
    Name        = "${var.project_name}-asg-lifecycle"
    Environment = var.environment
    Project     = var.project_name
  }
}

# 생명주기 훅 Lambda 함수
resource "aws_lambda_function" "lifecycle_hook" {
  filename         = "lifecycle_hook.zip"
  function_name    = "${var.project_name}-lifecycle-hook"
  role            = aws_iam_role.lambda_lifecycle.arn
  handler         = "index.handler"
  runtime         = "python3.9"
  timeout         = 300

  environment {
    variables = {
      ASG_NAME = aws_autoscaling_group.pacs_backend.name
    }
  }

  tags = {
    Name        = "${var.project_name}-lifecycle-hook"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Lambda IAM 역할
resource "aws_iam_role" "lambda_lifecycle" {
  name = "${var.project_name}-lambda-lifecycle"

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
resource "aws_iam_policy" "lambda_lifecycle" {
  name        = "${var.project_name}-lambda-lifecycle"
  description = "Policy for lifecycle hook Lambda"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "autoscaling:CompleteLifecycleAction",
          "autoscaling:RecordLifecycleActionHeartbeat",
          "ec2:DescribeInstances",
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
        Resource = "*"
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_lifecycle" {
  role       = aws_iam_role.lambda_lifecycle.name
  policy_arn = aws_iam_policy.lambda_lifecycle.arn
}

# SNS 구독
resource "aws_sns_topic_subscription" "lifecycle_hook" {
  topic_arn = aws_sns_topic.asg_lifecycle.arn
  protocol  = "lambda"
  endpoint  = aws_lambda_function.lifecycle_hook.arn
}

resource "aws_lambda_permission" "allow_sns" {
  statement_id  = "AllowExecutionFromSNS"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.lifecycle_hook.function_name
  principal     = "sns.amazonaws.com"
  source_arn    = aws_sns_topic.asg_lifecycle.arn
}
```

### 2. 모니터링 대시보드

#### `monitoring-dashboard.tf`
```hcl
# CloudWatch 대시보드
resource "aws_cloudwatch_dashboard" "asg" {
  dashboard_name = "${var.project_name}-asg-dashboard"

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
            [".", "GroupTotalInstances", ".", "."],
            [".", "GroupMinSize", ".", "."],
            [".", "GroupMaxSize", ".", "."]
          ]
          view    = "timeSeries"
          stacked = false
          region  = var.aws_region
          title   = "ASG Capacity"
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
            ["AWS/EC2", "CPUUtilization", "AutoScalingGroupName", aws_autoscaling_group.pacs_backend.name],
            ["CWAgent", "MemoryUtilization", ".", "."],
            ["PACS/Custom", "QueueLength", ".", "."]
          ]
          view    = "timeSeries"
          stacked = false
          region  = var.aws_region
          title   = "Instance Metrics"
          period  = 300
        }
      }
    ]
  })
}
```

---

## 🧪 실습 및 테스트

### 1. ASG 생성 테스트

#### `test-asg-creation.sh`
```bash
#!/bin/bash
# ASG 생성 테스트 스크립트

echo "Testing ASG creation..."

# Terraform 초기화
echo "1. Initializing Terraform..."
terraform init

# Terraform 검증
echo "2. Validating configuration..."
terraform validate

# Launch Template 생성
echo "3. Creating Launch Template..."
terraform apply -target=aws_launch_template.pacs_backend -auto-approve

# ASG 생성
echo "4. Creating Auto Scaling Group..."
terraform apply -target=aws_autoscaling_group.pacs_backend -auto-approve

# 스케일링 정책 생성
echo "5. Creating scaling policies..."
terraform apply -target=aws_autoscaling_policy.scale_out_cpu -auto-approve
terraform apply -target=aws_autoscaling_policy.scale_in_cpu -auto-approve

# ASG 확인
echo "6. Verifying ASG creation..."
aws autoscaling describe-auto-scaling-groups --auto-scaling-group-names pacs-backend-asg

echo "ASG creation test completed! 🎉"
```

### 2. 스케일링 테스트

#### `test-scaling.sh`
```bash
#!/bin/bash
# 스케일링 테스트 스크립트

echo "Testing ASG scaling..."

# 현재 ASG 상태 확인
echo "1. Checking current ASG status..."
aws autoscaling describe-auto-scaling-groups \
  --auto-scaling-group-names pacs-backend-asg \
  --query 'AutoScalingGroups[0].{DesiredCapacity:DesiredCapacity,MinSize:MinSize,MaxSize:MaxSize,Instances:length(Instances)}'

# 수동 스케일 아웃
echo "2. Testing manual scale out..."
aws autoscaling set-desired-capacity \
  --auto-scaling-group-name pacs-backend-asg \
  --desired-capacity 3

# 30초 대기
echo "3. Waiting for scaling to complete..."
sleep 30

# 스케일링 결과 확인
echo "4. Checking scaling result..."
aws autoscaling describe-auto-scaling-groups \
  --auto-scaling-group-names pacs-backend-asg \
  --query 'AutoScalingGroups[0].{DesiredCapacity:DesiredCapacity,Instances:length(Instances)}'

# 수동 스케일 인
echo "5. Testing manual scale in..."
aws autoscaling set-desired-capacity \
  --auto-scaling-group-name pacs-backend-asg \
  --desired-capacity 1

# 30초 대기
echo "6. Waiting for scaling to complete..."
sleep 30

# 최종 상태 확인
echo "7. Checking final status..."
aws autoscaling describe-auto-scaling-groups \
  --auto-scaling-group-names pacs-backend-asg \
  --query 'AutoScalingGroups[0].{DesiredCapacity:DesiredCapacity,Instances:length(Instances)}'

echo "Scaling test completed! 🎉"
```

### 3. 부하 테스트

#### `test-load.sh`
```bash
#!/bin/bash
# 부하 테스트 스크립트

echo "Testing load and auto scaling..."

# ALB DNS 이름 가져오기
ALB_DNS=$(aws elbv2 describe-load-balancers \
  --names pacs-development-alb \
  --query 'LoadBalancers[0].DNSName' \
  --output text)

echo "ALB DNS: $ALB_DNS"

# 부하 테스트 실행
echo "1. Running load test..."
for i in {1..10}; do
  (
    while true; do
      curl -s http://$ALB_DNS/api/health > /dev/null
      sleep 0.1
    done
  ) &
done

# 60초 대기
echo "2. Running load test for 60 seconds..."
sleep 60

# 스케일링 확인
echo "3. Checking if scaling occurred..."
aws autoscaling describe-auto-scaling-groups \
  --auto-scaling-group-names pacs-backend-asg \
  --query 'AutoScalingGroups[0].{DesiredCapacity:DesiredCapacity,Instances:length(Instances)}'

# 부하 테스트 중지
echo "4. Stopping load test..."
pkill -f "curl.*$ALB_DNS"

# 30초 대기 후 스케일 인 확인
echo "5. Waiting for scale in..."
sleep 30

# 최종 상태 확인
echo "6. Checking final status..."
aws autoscaling describe-auto-scaling-groups \
  --auto-scaling-group-names pacs-backend-asg \
  --query 'AutoScalingGroups[0].{DesiredCapacity:DesiredCapacity,Instances:length(Instances)}'

echo "Load test completed! 🎉"
```

### 4. 헬스 체크 테스트

#### `test-health-checks.sh`
```bash
#!/bin/bash
# 헬스 체크 테스트 스크립트

echo "Testing health checks..."

# ASG 인스턴스 목록 가져오기
echo "1. Getting ASG instances..."
INSTANCES=$(aws autoscaling describe-auto-scaling-groups \
  --auto-scaling-group-names pacs-backend-asg \
  --query 'AutoScalingGroups[0].Instances[].InstanceId' \
  --output text)

echo "Instances: $INSTANCES"

# 각 인스턴스의 헬스 체크 상태 확인
echo "2. Checking instance health..."
for instance in $INSTANCES; do
  echo "Instance $instance:"
  aws autoscaling describe-auto-scaling-instances \
    --instance-ids $instance \
    --query 'AutoScalingInstances[0].{HealthStatus:HealthStatus,LifecycleState:LifecycleState}'
done

# Target Group 헬스 상태 확인
echo "3. Checking target group health..."
TARGET_GROUP_ARN=$(aws elbv2 describe-target-groups \
  --names pacs-backend-tg \
  --query 'TargetGroups[0].TargetGroupArn' \
  --output text)

aws elbv2 describe-target-health \
  --target-group-arn $TARGET_GROUP_ARN

echo "Health check test completed! 🎉"
```

---

## 🔧 문제 해결

### 1. 인스턴스 시작 실패

**증상**: 인스턴스가 시작되지 않음
```
Error: Error launching instance: InvalidParameterValue
```

**해결 방법**:
```hcl
# AMI ID 확인
data "aws_ami" "latest" {
  most_recent = true
  owners      = ["amazon"]

  filter {
    name   = "name"
    values = ["amzn2-ami-hvm-*-x86_64-gp2"]
  }
}

# Launch Template에서 올바른 AMI 사용
resource "aws_launch_template" "pacs_backend" {
  image_id = data.aws_ami.latest.id
  # ... 기타 설정 ...
}
```

### 2. 스케일링 정책 작동 안함

**증상**: 스케일링 정책이 작동하지 않음
```
Error: No scaling activity occurred
```

**해결 방법**:
```hcl
# 알람과 정책 연결 확인
resource "aws_cloudwatch_metric_alarm" "cpu_high" {
  # ... 기타 설정 ...
  alarm_actions = [aws_autoscaling_policy.scale_out_cpu.arn]
}

# ASG에 정책 연결 확인
resource "aws_autoscaling_policy" "scale_out_cpu" {
  autoscaling_group_name = aws_autoscaling_group.pacs_backend.name
  # ... 기타 설정 ...
}
```

### 3. 생명주기 훅 타임아웃

**증상**: 생명주기 훅이 타임아웃됨
```
Error: Lifecycle hook timeout
```

**해결 방법**:
```hcl
# 생명주기 훅 타임아웃 증가
resource "aws_autoscaling_group" "pacs_backend" {
  # ... 기타 설정 ...
  
  initial_lifecycle_hook {
    name                 = "pacs-backend-launch"
    default_result       = "CONTINUE"
    heartbeat_timeout    = 3600  # 1시간으로 증가
    lifecycle_transition = "autoscaling:EC2_INSTANCE_LAUNCHING"
  }
}
```

---

## 📚 다음 단계

이제 Auto Scaling Group을 성공적으로 설정했으니 다음 문서들을 학습하세요:

1. **CI/CD 파이프라인** - 배포 자동화
2. **모니터링 및 로깅** - 전체 시스템 모니터링
3. **보안 및 컴플라이언스** - 보안 정책 및 감사

---

## 📖 참고 자료

- [AWS Auto Scaling 공식 문서](https://docs.aws.amazon.com/autoscaling/)
- [Auto Scaling Best Practices](https://docs.aws.amazon.com/autoscaling/ec2/userguide/auto-scaling-benefits.html)
- [CloudWatch 메트릭](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/working_with_metrics.html)

이제 PACS 프로젝트의 자동 스케일링을 위한 Auto Scaling Group이 준비되었습니다! 🚀


