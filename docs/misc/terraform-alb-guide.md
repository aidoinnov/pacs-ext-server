# ⚖️ Application Load Balancer 가이드

Terraform을 사용하여 AWS Application Load Balancer (ALB)를 구성하고 관리하는 방법을 학습합니다. PACS 프로젝트의 고가용성 로드 밸런싱과 SSL 종료를 위한 ALB 설정을 중심으로 다룹니다.

## 📋 목차

1. [Application Load Balancer란?](#application-load-balancer란)
2. [기본 ALB 구성](#기본-alb-구성)
3. [PACS 프로젝트 ALB 설정](#pacs-프로젝트-alb-설정)
4. [SSL/TLS 및 보안](#ssltls-및-보안)
5. [고급 라우팅 및 모니터링](#고급-라우팅-및-모니터링)
6. [실습 및 테스트](#실습-및-테스트)

---

## 🎯 Application Load Balancer란?

**AWS Application Load Balancer (ALB)**는 OSI 7계층에서 동작하는 로드 밸런서입니다.

### 주요 특징
- **Layer 7 로드 밸런싱**: HTTP/HTTPS 트래픽 처리
- **고가용성**: Multi-AZ 배포로 99.99% 가용성
- **자동 스케일링**: 트래픽에 따른 자동 확장
- **SSL 종료**: SSL/TLS 암호화 처리
- **고급 라우팅**: 경로, 호스트, 헤더 기반 라우팅

### PACS 프로젝트에서의 활용
- **API 게이트웨이**: PACS Backend API 라우팅
- **인증 서비스**: Keycloak 라우팅
- **웹 애플리케이션**: Frontend 라우팅
- **SSL 종료**: HTTPS 트래픽 처리
- **헬스 체크**: 서비스 상태 모니터링

---

## 🔧 기본 ALB 구성

### 1. 기본 ALB 생성

#### `alb-basic.tf`
```hcl
# Application Load Balancer
resource "aws_lb" "main" {
  name               = "${var.project_name}-${var.environment}-alb"
  internal           = var.alb_internal
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets            = var.alb_subnet_ids

  enable_deletion_protection = var.environment == "production"

  # 액세스 로그 설정
  access_logs {
    bucket  = aws_s3_bucket.alb_logs.bucket
    prefix  = "alb-logs"
    enabled = true
  }

  tags = {
    Name        = "${var.project_name}-${var.environment}-alb"
    Environment = var.environment
    Project     = var.project_name
  }
}

# ALB 보안 그룹
resource "aws_security_group" "alb" {
  name_prefix = "${var.project_name}-alb-"
  vpc_id      = var.vpc_id

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

  # 모든 아웃바운드 허용
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name        = "${var.project_name}-alb-sg"
    Environment = var.environment
    Project     = var.project_name
  }
}

# ALB 로그용 S3 버킷
resource "aws_s3_bucket" "alb_logs" {
  bucket = "${var.project_name}-${var.environment}-alb-logs"

  tags = {
    Name        = "${var.project_name}-alb-logs"
    Environment = var.environment
    Project     = var.project_name
  }
}

resource "aws_s3_bucket_public_access_block" "alb_logs" {
  bucket = aws_s3_bucket.alb_logs.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}
```

### 2. Target Group 구성

#### `target-groups.tf`
```hcl
# PACS Backend Target Group
resource "aws_lb_target_group" "pacs_backend" {
  name     = "${var.project_name}-backend-tg"
  port     = 8080
  protocol = "HTTP"
  vpc_id   = var.vpc_id

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

  # Sticky Session 설정
  stickiness {
    type            = "lb_cookie"
    cookie_duration = 86400
    enabled         = var.enable_sticky_session
  }

  tags = {
    Name        = "${var.project_name}-backend-tg"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Keycloak Target Group
resource "aws_lb_target_group" "keycloak" {
  name     = "${var.project_name}-keycloak-tg"
  port     = 8080
  protocol = "HTTP"
  vpc_id   = var.vpc_id

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

# Frontend Target Group
resource "aws_lb_target_group" "frontend" {
  name     = "${var.project_name}-frontend-tg"
  port     = 80
  protocol = "HTTP"
  vpc_id   = var.vpc_id

  health_check {
    enabled             = true
    healthy_threshold   = 2
    interval            = 30
    matcher             = "200"
    path                = "/"
    port                = "traffic-port"
    protocol            = "HTTP"
    timeout             = 5
    unhealthy_threshold = 2
  }

  tags = {
    Name        = "${var.project_name}-frontend-tg"
    Environment = var.environment
    Project     = var.project_name
  }
}
```

### 3. Listener 구성

#### `listeners.tf`
```hcl
# HTTP Listener
resource "aws_lb_listener" "http" {
  load_balancer_arn = aws_lb.main.arn
  port              = "80"
  protocol          = "HTTP"

  # HTTP to HTTPS 리다이렉트
  default_action {
    type = "redirect"
    redirect {
      port        = "443"
      protocol    = "HTTPS"
      status_code = "HTTP_301"
    }
  }
}

# HTTPS Listener
resource "aws_lb_listener" "https" {
  load_balancer_arn = aws_lb.main.arn
  port              = "443"
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS-1-2-2017-01"
  certificate_arn   = var.ssl_certificate_arn

  # 기본 액션 (Frontend)
  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.frontend.arn
  }
}

# PACS API Listener Rule
resource "aws_lb_listener_rule" "pacs_api" {
  listener_arn = aws_lb_listener.https.arn
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

  condition {
    host_header {
      values = [var.api_domain]
    }
  }
}

# Keycloak Listener Rule
resource "aws_lb_listener_rule" "keycloak" {
  listener_arn = aws_lb_listener.https.arn
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

  condition {
    host_header {
      values = [var.auth_domain]
    }
  }
}

# Frontend Listener Rule
resource "aws_lb_listener_rule" "frontend" {
  listener_arn = aws_lb_listener.https.arn
  priority     = 300

  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.frontend.arn
  }

  condition {
    host_header {
      values = [var.frontend_domain]
    }
  }
}
```

---

## 🏥 PACS 프로젝트 ALB 설정

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

# ALB 설정
variable "alb_internal" {
  description = "Whether the ALB is internal"
  type        = bool
  default     = false
}

variable "alb_subnet_ids" {
  description = "Subnet IDs for ALB"
  type        = list(string)
}

variable "vpc_id" {
  description = "VPC ID"
  type        = string
}

# 도메인 설정
variable "api_domain" {
  description = "API domain name"
  type        = string
  default     = "api.pacs.local"
}

variable "auth_domain" {
  description = "Authentication domain name"
  type        = string
  default     = "auth.pacs.local"
}

variable "frontend_domain" {
  description = "Frontend domain name"
  type        = string
  default     = "pacs.local"
}

# SSL 설정
variable "ssl_certificate_arn" {
  description = "SSL certificate ARN"
  type        = string
}

# 기능 설정
variable "enable_sticky_session" {
  description = "Enable sticky session"
  type        = bool
  default     = false
}

# 타겟 설정
variable "pacs_backend_targets" {
  description = "PACS Backend target instances"
  type        = list(string)
  default     = []
}

variable "keycloak_targets" {
  description = "Keycloak target instances"
  type        = list(string)
  default     = []
}

variable "frontend_targets" {
  description = "Frontend target instances"
  type        = list(string)
  default     = []
}
```

### 2. Target 등록

#### `target-registration.tf`
```hcl
# PACS Backend Target 등록
resource "aws_lb_target_group_attachment" "pacs_backend" {
  count = length(var.pacs_backend_targets)

  target_group_arn = aws_lb_target_group.pacs_backend.arn
  target_id        = var.pacs_backend_targets[count.index]
  port             = 8080
}

# Keycloak Target 등록
resource "aws_lb_target_group_attachment" "keycloak" {
  count = length(var.keycloak_targets)

  target_group_arn = aws_lb_target_group.keycloak.arn
  target_id        = var.keycloak_targets[count.index]
  port             = 8080
}

# Frontend Target 등록
resource "aws_lb_target_group_attachment" "frontend" {
  count = length(var.frontend_targets)

  target_group_arn = aws_lb_target_group.frontend.arn
  target_id        = var.frontend_targets[count.index]
  port             = 80
}
```

### 3. 고급 라우팅 규칙

#### `advanced-routing.tf`
```hcl
# API 버전별 라우팅
resource "aws_lb_listener_rule" "api_v1" {
  listener_arn = aws_lb_listener.https.arn
  priority     = 110

  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.pacs_backend.arn
  }

  condition {
    path_pattern {
      values = ["/api/v1/*"]
    }
  }

  condition {
    host_header {
      values = [var.api_domain]
    }
  }
}

# API 버전별 라우팅 (v2)
resource "aws_lb_listener_rule" "api_v2" {
  listener_arn = aws_lb_listener.https.arn
  priority     = 120

  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.pacs_backend.arn
  }

  condition {
    path_pattern {
      values = ["/api/v2/*"]
    }
  }

  condition {
    host_header {
      values = [var.api_domain]
    }
  }
}

# 헤더 기반 라우팅 (모바일 앱)
resource "aws_lb_listener_rule" "mobile_api" {
  listener_arn = aws_lb_listener.https.arn
  priority     = 130

  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.pacs_backend.arn
  }

  condition {
    http_header {
      http_header_name = "User-Agent"
      values           = ["*Mobile*", "*Android*", "*iOS*"]
    }
  }

  condition {
    path_pattern {
      values = ["/api/mobile/*"]
    }
  }
}

# 조건부 라우팅 (A/B 테스트)
resource "aws_lb_listener_rule" "ab_test" {
  listener_arn = aws_lb_listener.https.arn
  priority     = 140

  action {
    type = "weighted-forward"
    weighted_forward {
      target_group {
        target_group_arn = aws_lb_target_group.pacs_backend.arn
        weight           = 80
      }
      target_group {
        target_group_arn = aws_lb_target_group.pacs_backend_v2.arn
        weight           = 20
      }
    }
  }

  condition {
    path_pattern {
      values = ["/api/experimental/*"]
    }
  }
}
```

---

## 🔒 SSL/TLS 및 보안

### 1. SSL 인증서 설정

#### `ssl-certificates.tf`
```hcl
# ACM 인증서 요청
resource "aws_acm_certificate" "main" {
  domain_name               = var.domain_name
  subject_alternative_names = var.subject_alternative_names
  validation_method         = "DNS"

  lifecycle {
    create_before_destroy = true
  }

  tags = {
    Name        = "${var.project_name}-ssl-cert"
    Environment = var.environment
    Project     = var.project_name
  }
}

# DNS 검증 레코드
resource "aws_route53_record" "cert_validation" {
  for_each = {
    for dvo in aws_acm_certificate.main.domain_validation_options : dvo.domain_name => {
      name   = dvo.resource_record_name
      record = dvo.resource_record_value
      type   = dvo.resource_record_type
    }
  }

  allow_overwrite = true
  name            = each.value.name
  records         = [each.value.record]
  ttl             = 60
  type            = each.value.type
  zone_id         = var.hosted_zone_id
}

# 인증서 검증
resource "aws_acm_certificate_validation" "main" {
  certificate_arn         = aws_acm_certificate.main.arn
  validation_record_fqdns = [for record in aws_route53_record.cert_validation : record.fqdn]
}

# WAF Web ACL
resource "aws_wafv2_web_acl" "alb" {
  name  = "${var.project_name}-alb-waf"
  scope = "REGIONAL"

  default_action {
    allow {}
  }

  # SQL Injection 보호
  rule {
    name     = "SQLInjectionRule"
    priority = 1

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesSQLiRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "SQLInjectionRule"
      sampled_requests_enabled   = true
    }
  }

  # XSS 보호
  rule {
    name     = "XSSRule"
    priority = 2

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesCommonRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "XSSRule"
      sampled_requests_enabled   = true
    }
  }

  # Rate Limiting
  rule {
    name     = "RateLimitRule"
    priority = 3

    action {
      block {}
    }

    statement {
      rate_based_statement {
        limit              = 2000
        aggregate_key_type = "IP"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "RateLimitRule"
      sampled_requests_enabled   = true
    }
  }

  tags = {
    Name        = "${var.project_name}-alb-waf"
    Environment = var.environment
    Project     = var.project_name
  }
}

# WAF 연결
resource "aws_wafv2_web_acl_association" "alb" {
  resource_arn = aws_lb.main.arn
  web_acl_arn  = aws_wafv2_web_acl.alb.arn
}
```

### 2. 보안 헤더 설정

#### `security-headers.tf`
```hcl
# 보안 헤더를 위한 Lambda@Edge 함수
resource "aws_lambda_function" "security_headers" {
  filename         = "security_headers.zip"
  function_name    = "${var.project_name}-security-headers"
  role            = aws_iam_role.lambda_edge.arn
  handler         = "index.handler"
  runtime         = "nodejs18.x"
  timeout         = 5

  tags = {
    Name        = "${var.project_name}-security-headers"
    Environment = var.environment
    Project     = var.project_name
  }
}

# Lambda@Edge IAM 역할
resource "aws_iam_role" "lambda_edge" {
  name = "${var.project_name}-lambda-edge-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = ["lambda.amazonaws.com", "edgelambda.amazonaws.com"]
        }
      }
    ]
  })
}

# Lambda@Edge 정책
resource "aws_iam_role_policy_attachment" "lambda_edge" {
  role       = aws_iam_role.lambda_edge.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

# CloudFront Distribution (Lambda@Edge 사용)
resource "aws_cloudfront_distribution" "main" {
  origin {
    domain_name = aws_lb.main.dns_name
    origin_id   = "ALB-${aws_lb.main.id}"

    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }

  enabled             = true
  is_ipv6_enabled     = true
  default_root_object = "index.html"

  default_cache_behavior {
    allowed_methods        = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "ALB-${aws_lb.main.id}"
    compress               = true
    viewer_protocol_policy = "redirect-to-https"

    forwarded_values {
      query_string = false
      cookies {
        forward = "none"
      }
    }

    # Lambda@Edge 연결
    lambda_function_association {
      event_type   = "viewer-response"
      lambda_arn   = aws_lambda_function.security_headers.qualified_arn
      include_body = false
    }
  }

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }

  viewer_certificate {
    acm_certificate_arn      = aws_acm_certificate.main.arn
    ssl_support_method       = "sni-only"
    minimum_protocol_version = "TLSv1.2_2021"
  }

  tags = {
    Name        = "${var.project_name}-cloudfront"
    Environment = var.environment
    Project     = var.project_name
  }
}
```

---

## 📊 고급 라우팅 및 모니터링

### 1. 고급 라우팅 규칙

#### `advanced-routing-rules.tf`
```hcl
# 가중치 기반 라우팅
resource "aws_lb_listener_rule" "weighted_routing" {
  listener_arn = aws_lb_listener.https.arn
  priority     = 150

  action {
    type = "weighted-forward"
    weighted_forward {
      target_group {
        target_group_arn = aws_lb_target_group.pacs_backend.arn
        weight           = 70
      }
      target_group {
        target_group_arn = aws_lb_target_group.pacs_backend_v2.arn
        weight           = 30
      }
    }
  }

  condition {
    path_pattern {
      values = ["/api/canary/*"]
    }
  }
}

# 리다이렉트 규칙
resource "aws_lb_listener_rule" "redirect_rule" {
  listener_arn = aws_lb_listener.https.arn
  priority     = 160

  action {
    type = "redirect"
    redirect {
      host        = "www.${var.frontend_domain}"
      path        = "/#{path}"
      port        = "443"
      protocol    = "HTTPS"
      query       = "#{query}"
      status_code = "HTTP_301"
    }
  }

  condition {
    host_header {
      values = [var.frontend_domain]
    }
  }
}

# 고정 응답 규칙
resource "aws_lb_listener_rule" "fixed_response" {
  listener_arn = aws_lb_listener.https.arn
  priority     = 170

  action {
    type = "fixed-response"
    fixed_response {
      content_type = "text/plain"
      message_body = "Service temporarily unavailable"
      status_code  = "503"
    }
  }

  condition {
    path_pattern {
      values = ["/maintenance/*"]
    }
  }
}
```

### 2. 모니터링 및 알림

#### `monitoring.tf`
```hcl
# CloudWatch 대시보드
resource "aws_cloudwatch_dashboard" "alb" {
  dashboard_name = "${var.project_name}-alb-dashboard"

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
      }
    ]
  })
}

# CloudWatch 알람
resource "aws_cloudwatch_metric_alarm" "alb_high_response_time" {
  alarm_name          = "${var.project_name}-alb-high-response-time"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "TargetResponseTime"
  namespace           = "AWS/ApplicationELB"
  period              = "300"
  statistic           = "Average"
  threshold           = "2"
  alarm_description   = "This metric monitors ALB response time"

  dimensions = {
    LoadBalancer = aws_lb.main.arn_suffix
  }

  tags = {
    Name        = "${var.project_name}-alb-high-response-time"
    Environment = var.environment
    Project     = var.project_name
  }
}

resource "aws_cloudwatch_metric_alarm" "alb_high_5xx_errors" {
  alarm_name          = "${var.project_name}-alb-high-5xx-errors"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "HTTPCode_Target_5XX_Count"
  namespace           = "AWS/ApplicationELB"
  period              = "300"
  statistic           = "Sum"
  threshold           = "10"
  alarm_description   = "This metric monitors ALB 5xx errors"

  dimensions = {
    LoadBalancer = aws_lb.main.arn_suffix
  }

  tags = {
    Name        = "${var.project_name}-alb-high-5xx-errors"
    Environment = var.environment
    Project     = var.project_name
  }
}

# SNS 토픽
resource "aws_sns_topic" "alb_alerts" {
  name = "${var.project_name}-alb-alerts"

  tags = {
    Name        = "${var.project_name}-alb-alerts"
    Environment = var.environment
    Project     = var.project_name
  }
}

# SNS 토픽 구독
resource "aws_sns_topic_subscription" "alb_alerts" {
  topic_arn = aws_sns_topic.alb_alerts.arn
  protocol  = "email"
  endpoint  = var.alert_email
}

# 알람과 SNS 연결
resource "aws_cloudwatch_metric_alarm" "alb_high_response_time" {
  # ... 기존 설정 ...
  
  alarm_actions = [aws_sns_topic.alb_alerts.arn]
}

resource "aws_cloudwatch_metric_alarm" "alb_high_5xx_errors" {
  # ... 기존 설정 ...
  
  alarm_actions = [aws_sns_topic.alb_alerts.arn]
}
```

---

## 🧪 실습 및 테스트

### 1. ALB 생성 테스트

#### `test-alb-creation.sh`
```bash
#!/bin/bash
# ALB 생성 테스트 스크립트

echo "Testing ALB creation..."

# Terraform 초기화
echo "1. Initializing Terraform..."
terraform init

# Terraform 검증
echo "2. Validating configuration..."
terraform validate

# ALB 생성
echo "3. Creating ALB..."
terraform apply -target=aws_lb.main -auto-approve

# Target Group 생성
echo "4. Creating Target Groups..."
terraform apply -target=aws_lb_target_group.pacs_backend -auto-approve
terraform apply -target=aws_lb_target_group.keycloak -auto-approve
terraform apply -target=aws_lb_target_group.frontend -auto-approve

# Listener 생성
echo "5. Creating Listeners..."
terraform apply -target=aws_lb_listener.http -auto-approve
terraform apply -target=aws_lb_listener.https -auto-approve

# ALB 확인
echo "6. Verifying ALB creation..."
aws elbv2 describe-load-balancers --names pacs-development-alb

echo "ALB creation test completed! 🎉"
```

### 2. 라우팅 테스트

#### `test-routing.sh`
```bash
#!/bin/bash
# 라우팅 테스트 스크립트

echo "Testing ALB routing..."

# ALB DNS 이름 가져오기
ALB_DNS=$(aws elbv2 describe-load-balancers \
  --names pacs-development-alb \
  --query 'LoadBalancers[0].DNSName' \
  --output text)

echo "ALB DNS: $ALB_DNS"

# HTTP to HTTPS 리다이렉트 테스트
echo "1. Testing HTTP to HTTPS redirect..."
curl -I http://$ALB_DNS/api/health

# API 라우팅 테스트
echo "2. Testing API routing..."
curl -H "Host: api.pacs.local" https://$ALB_DNS/api/health

# 인증 라우팅 테스트
echo "3. Testing Auth routing..."
curl -H "Host: auth.pacs.local" https://$ALB_DNS/auth/health

# Frontend 라우팅 테스트
echo "4. Testing Frontend routing..."
curl -H "Host: pacs.local" https://$ALB_DNS/

echo "Routing test completed! 🎉"
```

### 3. 헬스 체크 테스트

#### `test-health-checks.sh`
```bash
#!/bin/bash
# 헬스 체크 테스트 스크립트

echo "Testing health checks..."

# Target Group 상태 확인
echo "1. Checking Target Group status..."
aws elbv2 describe-target-health \
  --target-group-arn $(aws elbv2 describe-target-groups \
    --names pacs-backend-tg \
    --query 'TargetGroups[0].TargetGroupArn' \
    --output text)

# Target 등록
echo "2. Registering targets..."
aws elbv2 register-targets \
  --target-group-arn $(aws elbv2 describe-target-groups \
    --names pacs-backend-tg \
    --query 'TargetGroups[0].TargetGroupArn' \
    --output text) \
  --targets Id=i-1234567890abcdef0,Port=8080

# 헬스 체크 상태 확인
echo "3. Checking health status..."
aws elbv2 describe-target-health \
  --target-group-arn $(aws elbv2 describe-target-groups \
    --names pacs-backend-tg \
    --query 'TargetGroups[0].TargetGroupArn' \
    --output text)

echo "Health check test completed! 🎉"
```

### 4. SSL 인증서 테스트

#### `test-ssl.sh`
```bash
#!/bin/bash
# SSL 인증서 테스트 스크립트

echo "Testing SSL certificates..."

# ALB DNS 이름 가져오기
ALB_DNS=$(aws elbv2 describe-load-balancers \
  --names pacs-development-alb \
  --query 'LoadBalancers[0].DNSName' \
  --output text)

# SSL 인증서 확인
echo "1. Checking SSL certificate..."
openssl s_client -connect $ALB_DNS:443 -servername $ALB_DNS < /dev/null 2>/dev/null | openssl x509 -noout -text

# SSL Labs 테스트 (선택사항)
echo "2. Running SSL Labs test..."
curl -s "https://api.ssllabs.com/api/v3/analyze?host=$ALB_DNS" | jq '.status'

# TLS 버전 확인
echo "3. Checking TLS versions..."
for version in ssl2 ssl3 tls1 tls1_1 tls1_2 tls1_3; do
  echo -n "Testing $version: "
  timeout 5 openssl s_client -connect $ALB_DNS:443 -$version < /dev/null 2>/dev/null && echo "SUPPORTED" || echo "NOT SUPPORTED"
done

echo "SSL test completed! 🎉"
```

---

## 🔧 문제 해결

### 1. Target Group 등록 실패

**증상**: Target 등록 실패
```
Error: Error registering targets: InvalidTarget: The target does not exist
```

**해결 방법**:
```hcl
# Target ID 확인
data "aws_instances" "pacs_backend" {
  filter {
    name   = "tag:Name"
    values = ["pacs-backend-*"]
  }
}

# Target 등록
resource "aws_lb_target_group_attachment" "pacs_backend" {
  count = length(data.aws_instances.pacs_backend.ids)

  target_group_arn = aws_lb_target_group.pacs_backend.arn
  target_id        = data.aws_instances.pacs_backend.ids[count.index]
  port             = 8080
}
```

### 2. SSL 인증서 검증 실패

**증상**: SSL 인증서 검증 실패
```
Error: Error creating certificate: ValidationException: DNS validation failed
```

**해결 방법**:
```hcl
# Route53 호스팅 존 확인
data "aws_route53_zone" "main" {
  name = var.domain_name
}

# DNS 검증 레코드 생성
resource "aws_route53_record" "cert_validation" {
  for_each = {
    for dvo in aws_acm_certificate.main.domain_validation_options : dvo.domain_name => {
      name   = dvo.resource_record_name
      record = dvo.resource_record_value
      type   = dvo.resource_record_type
    }
  }

  allow_overwrite = true
  name            = each.value.name
  records         = [each.value.record]
  ttl             = 60
  type            = each.value.type
  zone_id         = data.aws_route53_zone.main.zone_id
}
```

### 3. 라우팅 규칙 충돌

**증상**: 라우팅 규칙 충돌
```
Error: Error creating listener rule: DuplicateListenerRule
```

**해결 방법**:
```hcl
# 우선순위 확인 및 조정
resource "aws_lb_listener_rule" "pacs_api" {
  listener_arn = aws_lb_listener.https.arn
  priority     = 100  # 고유한 우선순위 사용

  # ... 기타 설정 ...
}

resource "aws_lb_listener_rule" "keycloak" {
  listener_arn = aws_lb_listener.https.arn
  priority     = 200  # 다른 우선순위 사용

  # ... 기타 설정 ...
}
```

---

## 📚 다음 단계

이제 Application Load Balancer를 성공적으로 설정했으니 다음 문서들을 학습하세요:

1. **Auto Scaling 그룹** - 자동 스케일링 설정
2. **CI/CD 파이프라인** - 배포 자동화
3. **모니터링 및 로깅** - 전체 시스템 모니터링

---

## 📖 참고 자료

- [AWS ALB 공식 문서](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/)
- [ALB 라우팅 규칙](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/load-balancer-listeners.html)
- [SSL/TLS 설정 가이드](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/create-https-listener.html)

이제 PACS 프로젝트의 고가용성 로드 밸런싱을 위한 ALB가 준비되었습니다! 🚀


