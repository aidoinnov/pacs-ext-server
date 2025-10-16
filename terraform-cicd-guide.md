# 🔄 CI/CD 파이프라인 가이드

Terraform을 사용하여 GitHub Actions 기반 CI/CD 파이프라인을 구성하고 관리하는 방법을 학습합니다. PACS 프로젝트의 자동화된 배포와 인프라 관리를 위한 CI/CD 설정을 중심으로 다룹니다.

## 📋 목차

1. [CI/CD 파이프라인이란?](#cicd-파이프라인이란)
2. [GitHub Actions 기본 설정](#github-actions-기본-설정)
3. [PACS 프로젝트 CI/CD 구성](#pacs-프로젝트-cicd-구성)
4. [인프라 배포 자동화](#인프라-배포-자동화)
5. [애플리케이션 배포 자동화](#애플리케이션-배포-자동화)
6. [고급 CI/CD 기능](#고급-cicd-기능)
7. [실습 및 테스트](#실습-및-테스트)

---

## 🎯 CI/CD 파이프라인이란?

**CI/CD (Continuous Integration/Continuous Deployment)**는 코드 변경사항을 자동으로 통합, 테스트, 배포하는 프로세스입니다.

### 주요 특징
- **지속적 통합**: 코드 변경사항을 자동으로 빌드 및 테스트
- **지속적 배포**: 테스트 통과 시 자동으로 프로덕션 배포
- **자동화**: 수동 작업 최소화로 인적 오류 방지
- **빠른 피드백**: 개발자에게 즉시 결과 제공
- **롤백 지원**: 문제 발생 시 이전 버전으로 복구

### PACS 프로젝트에서의 활용
- **인프라 배포**: Terraform을 통한 AWS 리소스 자동 배포
- **애플리케이션 배포**: Docker 이미지 빌드 및 EKS 배포
- **테스트 자동화**: 단위 테스트, 통합 테스트, 보안 스캔
- **환경 관리**: Development, Staging, Production 환경 분리
- **모니터링**: 배포 상태 및 성능 모니터링

---

## 🔧 GitHub Actions 기본 설정

### 1. GitHub Actions 워크플로우 구조

#### `.github/workflows/ci-cd.yml`
```yaml
name: PACS CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  AWS_REGION: ap-northeast-2
  ECR_REGISTRY: ${{ secrets.AWS_ACCOUNT_ID }}.dkr.ecr.ap-northeast-2.amazonaws.com
  ECR_REPOSITORY: pacs-server
  IMAGE_TAG: ${{ github.sha }}

jobs:
  # 인프라 테스트
  infrastructure-test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v3
        with:
          terraform_version: 1.6.0

      - name: Terraform Format Check
        run: terraform fmt -check -recursive

      - name: Terraform Init
        run: terraform init

      - name: Terraform Validate
        run: terraform validate

      - name: Terraform Plan
        run: terraform plan -out=tfplan

  # 애플리케이션 테스트
  application-test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Cache Rust dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: |
          cargo test --verbose
          cargo clippy -- -D warnings
          cargo fmt -- --check

  # 인프라 배포
  infrastructure-deploy:
    needs: [infrastructure-test]
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    environment: production
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ env.AWS_REGION }}

      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v3
        with:
          terraform_version: 1.6.0

      - name: Terraform Init
        run: terraform init

      - name: Terraform Apply
        run: terraform apply -auto-approve

  # 애플리케이션 배포
  application-deploy:
    needs: [application-test, infrastructure-deploy]
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    environment: production
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ env.AWS_REGION }}

      - name: Login to Amazon ECR
        id: login-ecr
        uses: aws-actions/amazon-ecr-login@v2

      - name: Build, tag, and push image
        run: |
          docker build -t $ECR_REGISTRY/$ECR_REPOSITORY:$IMAGE_TAG .
          docker push $ECR_REGISTRY/$ECR_REPOSITORY:$IMAGE_TAG

      - name: Deploy to EKS
        run: |
          aws eks update-kubeconfig --region $AWS_REGION --name pacs-cluster
          kubectl set image deployment/pacs-backend pacs-backend=$ECR_REGISTRY/$ECR_REPOSITORY:$IMAGE_TAG
```

### 2. 환경별 워크플로우

#### `.github/workflows/development.yml`
```yaml
name: Development Deployment

on:
  push:
    branches: [ develop ]
  pull_request:
    branches: [ develop ]

env:
  ENVIRONMENT: development
  AWS_REGION: ap-northeast-2

jobs:
  deploy:
    runs-on: ubuntu-latest
    environment: development
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ env.AWS_REGION }}

      - name: Deploy to Development
        run: |
          terraform init
          terraform workspace select development
          terraform apply -var="environment=development" -auto-approve
```

#### `.github/workflows/staging.yml`
```yaml
name: Staging Deployment

on:
  push:
    branches: [ staging ]
  workflow_dispatch:

env:
  ENVIRONMENT: staging
  AWS_REGION: ap-northeast-2

jobs:
  deploy:
    runs-on: ubuntu-latest
    environment: staging
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ env.AWS_REGION }}

      - name: Deploy to Staging
        run: |
          terraform init
          terraform workspace select staging
          terraform apply -var="environment=staging" -auto-approve
```

---

## 🏥 PACS 프로젝트 CI/CD 구성

### 1. Terraform 백엔드 설정

#### `backend.tf`
```hcl
# S3 백엔드 설정
terraform {
  backend "s3" {
    bucket         = "pacs-terraform-state"
    key            = "infrastructure/terraform.tfstate"
    region         = "ap-northeast-2"
    encrypt        = true
    dynamodb_table = "pacs-terraform-locks"
  }
}

# DynamoDB 테이블 (상태 잠금)
resource "aws_dynamodb_table" "terraform_locks" {
  name           = "pacs-terraform-locks"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "LockID"

  attribute {
    name = "LockID"
    type = "S"
  }

  tags = {
    Name        = "pacs-terraform-locks"
    Environment = "shared"
    Project     = "pacs"
  }
}
```

### 2. 환경별 변수 설정

#### `terraform.tfvars.example`
```hcl
# 프로젝트 설정
project_name = "pacs"
environment  = "development"

# AWS 설정
aws_region = "ap-northeast-2"
aws_account_id = "123456789012"

# 네트워크 설정
vpc_cidr = "10.0.0.0/16"
public_subnet_cidrs = ["10.0.1.0/24", "10.0.2.0/24"]
private_subnet_cidrs = ["10.0.10.0/24", "10.0.20.0/24"]

# 데이터베이스 설정
db_instance_class = "db.t3.micro"
db_allocated_storage = 20

# 애플리케이션 설정
app_instance_type = "t3.medium"
app_min_size = 1
app_max_size = 3
app_desired_capacity = 2

# 도메인 설정
domain_name = "pacs.example.com"
ssl_certificate_arn = "arn:aws:acm:ap-northeast-2:123456789012:certificate/12345678-1234-1234-1234-123456789012"
```

### 3. GitHub Secrets 설정

#### `secrets.md`
```markdown
# GitHub Secrets 설정 가이드

## 필수 Secrets
- `AWS_ACCESS_KEY_ID`: AWS 액세스 키 ID
- `AWS_SECRET_ACCESS_KEY`: AWS 시크릿 액세스 키
- `AWS_ACCOUNT_ID`: AWS 계정 ID
- `DOCKER_USERNAME`: Docker Hub 사용자명
- `DOCKER_PASSWORD`: Docker Hub 비밀번호

## 선택적 Secrets
- `SLACK_WEBHOOK_URL`: Slack 알림용 웹훅 URL
- `TEAMS_WEBHOOK_URL`: Microsoft Teams 알림용 웹훅 URL
- `SONAR_TOKEN`: SonarQube 토큰
- `CODECOV_TOKEN`: Codecov 토큰
```

---

## 🏗️ 인프라 배포 자동화

### 1. 인프라 배포 워크플로우

#### `.github/workflows/infrastructure.yml`
```yaml
name: Infrastructure Deployment

on:
  push:
    branches: [ main, develop ]
    paths: [ 'terraform/**', 'infrastructure/**' ]
  pull_request:
    branches: [ main ]
    paths: [ 'terraform/**', 'infrastructure/**' ]

env:
  TF_VERSION: 1.6.0
  AWS_REGION: ap-northeast-2

jobs:
  plan:
    name: Terraform Plan
    runs-on: ubuntu-latest
    outputs:
      plan-exitcode: ${{ steps.plan.outputs.exitcode }}
      plan-output: ${{ steps.plan.outputs.stdout }}
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ env.AWS_REGION }}

      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v3
        with:
          terraform_version: ${{ env.TF_VERSION }}

      - name: Terraform Init
        run: terraform init

      - name: Terraform Format Check
        run: terraform fmt -check -recursive

      - name: Terraform Validate
        run: terraform validate

      - name: Terraform Plan
        id: plan
        run: |
          terraform plan -out=tfplan
          echo "exitcode=$?" >> $GITHUB_OUTPUT
          terraform show -no-color tfplan > tfplan.txt
          echo "stdout<<EOF" >> $GITHUB_OUTPUT
          cat tfplan.txt >> $GITHUB_OUTPUT
          echo "EOF" >> $GITHUB_OUTPUT

      - name: Upload Plan Artifact
        uses: actions/upload-artifact@v3
        with:
          name: terraform-plan
          path: tfplan

  apply:
    name: Terraform Apply
    needs: plan
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main' && needs.plan.outputs.plan-exitcode == 0
    environment: production
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ env.AWS_REGION }}

      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v3
        with:
          terraform_version: ${{ env.TF_VERSION }}

      - name: Download Plan Artifact
        uses: actions/download-artifact@v3
        with:
          name: terraform-plan
          path: .

      - name: Terraform Init
        run: terraform init

      - name: Terraform Apply
        run: terraform apply -auto-approve tfplan

      - name: Output Infrastructure Info
        run: |
          echo "## Infrastructure Deployment Complete" >> $GITHUB_STEP_SUMMARY
          echo "### Deployed Resources:" >> $GITHUB_STEP_SUMMARY
          terraform output -json | jq -r 'to_entries[] | "- \(.key): \(.value.value)"' >> $GITHUB_STEP_SUMMARY
```

### 2. 환경별 인프라 배포

#### `.github/workflows/environment-deployment.yml`
```yaml
name: Environment Deployment

on:
  workflow_dispatch:
    inputs:
      environment:
        description: 'Environment to deploy'
        required: true
        default: 'development'
        type: choice
        options:
          - development
          - staging
          - production

env:
  TF_VERSION: 1.6.0
  AWS_REGION: ap-northeast-2

jobs:
  deploy:
    name: Deploy to ${{ github.event.inputs.environment }}
    runs-on: ubuntu-latest
    environment: ${{ github.event.inputs.environment }}
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ env.AWS_REGION }}

      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v3
        with:
          terraform_version: ${{ env.TF_VERSION }}

      - name: Create Terraform Workspace
        run: |
          terraform init
          terraform workspace select ${{ github.event.inputs.environment }} || terraform workspace new ${{ github.event.inputs.environment }}

      - name: Terraform Plan
        run: |
          terraform plan -var="environment=${{ github.event.inputs.environment }}" -out=tfplan

      - name: Terraform Apply
        run: |
          terraform apply -auto-approve tfplan

      - name: Notify Deployment
        uses: 8398a7/action-slack@v3
        with:
          status: success
          text: "Infrastructure deployed to ${{ github.event.inputs.environment }}"
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
```

---

## 🚀 애플리케이션 배포 자동화

### 1. Docker 이미지 빌드 및 배포

#### `.github/workflows/application.yml`
```yaml
name: Application Deployment

on:
  push:
    branches: [ main, develop ]
    paths: [ 'pacs-server/**', 'Dockerfile' ]
  pull_request:
    branches: [ main ]
    paths: [ 'pacs-server/**', 'Dockerfile' ]

env:
  AWS_REGION: ap-northeast-2
  ECR_REGISTRY: ${{ secrets.AWS_ACCOUNT_ID }}.dkr.ecr.ap-northeast-2.amazonaws.com
  ECR_REPOSITORY: pacs-server
  IMAGE_TAG: ${{ github.sha }}

jobs:
  test:
    name: Run Tests
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Cache Rust dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: |
          cd pacs-server
          cargo test --verbose
          cargo clippy -- -D warnings
          cargo fmt -- --check

  build:
    name: Build Docker Image
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main' || github.ref == 'refs/heads/develop'
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ env.AWS_REGION }}

      - name: Login to Amazon ECR
        id: login-ecr
        uses: aws-actions/amazon-ecr-login@v2

      - name: Build, tag, and push image
        run: |
          docker build -t $ECR_REGISTRY/$ECR_REPOSITORY:$IMAGE_TAG .
          docker tag $ECR_REGISTRY/$ECR_REPOSITORY:$IMAGE_TAG $ECR_REGISTRY/$ECR_REPOSITORY:latest
          docker push $ECR_REGISTRY/$ECR_REPOSITORY:$IMAGE_TAG
          docker push $ECR_REGISTRY/$ECR_REPOSITORY:latest

  deploy:
    name: Deploy to EKS
    needs: [test, build]
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    environment: production
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ env.AWS_REGION }}

      - name: Update kubeconfig
        run: |
          aws eks update-kubeconfig --region $AWS_REGION --name pacs-cluster

      - name: Deploy to EKS
        run: |
          kubectl set image deployment/pacs-backend pacs-backend=$ECR_REGISTRY/$ECR_REPOSITORY:$IMAGE_TAG
          kubectl rollout status deployment/pacs-backend

      - name: Verify deployment
        run: |
          kubectl get pods -l app=pacs-backend
          kubectl get services -l app=pacs-backend
```

### 2. EKS 배포 매니페스트

#### `k8s/deployment.yaml`
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pacs-backend
  namespace: pacs
  labels:
    app: pacs-backend
    version: v1
spec:
  replicas: 3
  selector:
    matchLabels:
      app: pacs-backend
  template:
    metadata:
      labels:
        app: pacs-backend
        version: v1
    spec:
      containers:
      - name: pacs-backend
        image: 123456789012.dkr.ecr.ap-northeast-2.amazonaws.com/pacs-server:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: pacs-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: pacs-secrets
              key: redis-url
        - name: S3_BUCKET_NAME
          valueFrom:
            configMapKeyRef:
              name: pacs-config
              key: s3-bucket-name
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: pacs-backend-service
  namespace: pacs
  labels:
    app: pacs-backend
spec:
  selector:
    app: pacs-backend
  ports:
  - port: 80
    targetPort: 8080
    protocol: TCP
  type: ClusterIP
```

---

## 🔧 고급 CI/CD 기능

### 1. 보안 스캔 및 정적 분석

#### `.github/workflows/security.yml`
```yaml
name: Security Scan

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  security-scan:
    name: Security Scan
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          scan-ref: '.'
          format: 'sarif'
          output: 'trivy-results.sarif'

      - name: Upload Trivy scan results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: 'trivy-results.sarif'

      - name: Run Snyk to check for vulnerabilities
        uses: snyk/actions/rust@master
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}

      - name: Run CodeQL Analysis
        uses: github/codeql-action/analyze@v2
        with:
          languages: rust
```

### 2. 성능 테스트

#### `.github/workflows/performance.yml`
```yaml
name: Performance Test

on:
  push:
    branches: [ main ]
  workflow_dispatch:

jobs:
  performance-test:
    name: Performance Test
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install k6
        run: |
          sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
          echo "deb https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
          sudo apt-get update
          sudo apt-get install k6

      - name: Run performance tests
        run: |
          cd tests/performance
          k6 run load-test.js

      - name: Upload performance results
        uses: actions/upload-artifact@v3
        with:
          name: performance-results
          path: tests/performance/results/
```

### 3. 롤백 자동화

#### `.github/workflows/rollback.yml`
```yaml
name: Rollback Deployment

on:
  workflow_dispatch:
    inputs:
      environment:
        description: 'Environment to rollback'
        required: true
        default: 'production'
        type: choice
        options:
          - production
          - staging
      version:
        description: 'Version to rollback to'
        required: true
        type: string

env:
  AWS_REGION: ap-northeast-2

jobs:
  rollback:
    name: Rollback to ${{ github.event.inputs.version }}
    runs-on: ubuntu-latest
    environment: ${{ github.event.inputs.environment }}
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ env.AWS_REGION }}

      - name: Update kubeconfig
        run: |
          aws eks update-kubeconfig --region $AWS_REGION --name pacs-cluster

      - name: Rollback deployment
        run: |
          kubectl rollout undo deployment/pacs-backend --to-revision=${{ github.event.inputs.version }}
          kubectl rollout status deployment/pacs-backend

      - name: Verify rollback
        run: |
          kubectl get pods -l app=pacs-backend
          kubectl describe deployment pacs-backend

      - name: Notify rollback
        uses: 8398a7/action-slack@v3
        with:
          status: success
          text: "Rolled back to version ${{ github.event.inputs.version }} in ${{ github.event.inputs.environment }}"
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
```

---

## 🧪 실습 및 테스트

### 1. CI/CD 파이프라인 테스트

#### `test-cicd-pipeline.sh`
```bash
#!/bin/bash
# CI/CD 파이프라인 테스트 스크립트

echo "Testing CI/CD pipeline..."

# GitHub Actions 워크플로우 테스트
echo "1. Testing GitHub Actions workflows..."

# 워크플로우 파일 검증
echo "2. Validating workflow files..."
for workflow in .github/workflows/*.yml; do
  echo "Validating $workflow..."
  yamllint "$workflow"
done

# Terraform 파일 검증
echo "3. Validating Terraform files..."
terraform init
terraform validate

# Docker 이미지 빌드 테스트
echo "4. Testing Docker image build..."
docker build -t pacs-server:test .

# EKS 배포 매니페스트 검증
echo "5. Validating Kubernetes manifests..."
kubectl apply --dry-run=client -f k8s/

echo "CI/CD pipeline test completed! 🎉"
```

### 2. 배포 테스트

#### `test-deployment.sh`
```bash
#!/bin/bash
# 배포 테스트 스크립트

echo "Testing deployment..."

# 환경 변수 설정
ENVIRONMENT=${1:-development}
AWS_REGION="ap-northeast-2"

echo "Deploying to $ENVIRONMENT environment..."

# AWS 자격 증명 확인
echo "1. Checking AWS credentials..."
aws sts get-caller-identity

# Terraform 배포
echo "2. Deploying infrastructure..."
terraform init
terraform workspace select $ENVIRONMENT || terraform workspace new $ENVIRONMENT
terraform plan -var="environment=$ENVIRONMENT"
terraform apply -var="environment=$ENVIRONMENT" -auto-approve

# EKS 클러스터 연결
echo "3. Connecting to EKS cluster..."
aws eks update-kubeconfig --region $AWS_REGION --name pacs-cluster

# 애플리케이션 배포
echo "4. Deploying application..."
kubectl apply -f k8s/

# 배포 상태 확인
echo "5. Checking deployment status..."
kubectl get pods -l app=pacs-backend
kubectl get services -l app=pacs-backend

# 헬스 체크
echo "6. Running health checks..."
kubectl get pods -l app=pacs-backend -o jsonpath='{.items[0].status.containerStatuses[0].ready}'

echo "Deployment test completed! 🎉"
```

### 3. 롤백 테스트

#### `test-rollback.sh`
```bash
#!/bin/bash
# 롤백 테스트 스크립트

echo "Testing rollback..."

# 현재 배포 상태 확인
echo "1. Checking current deployment status..."
kubectl get deployment pacs-backend -o jsonpath='{.spec.template.spec.containers[0].image}'

# 이전 버전으로 롤백
echo "2. Rolling back to previous version..."
kubectl rollout undo deployment/pacs-backend

# 롤백 상태 확인
echo "3. Checking rollback status..."
kubectl rollout status deployment/pacs-backend

# 최종 상태 확인
echo "4. Checking final status..."
kubectl get deployment pacs-backend -o jsonpath='{.spec.template.spec.containers[0].image}'

echo "Rollback test completed! 🎉"
```

---

## 🔧 문제 해결

### 1. GitHub Actions 실패

**증상**: GitHub Actions 워크플로우 실패
```
Error: Resource not accessible by integration
```

**해결 방법**:
```yaml
# 권한 설정 추가
permissions:
  contents: read
  pull-requests: write
  issues: write
  statuses: write
```

### 2. Terraform 상태 잠금

**증상**: Terraform 상태 파일 잠금
```
Error: Error acquiring the state lock
```

**해결 방법**:
```bash
# 상태 잠금 강제 해제
terraform force-unlock <lock-id>

# 또는 DynamoDB에서 직접 삭제
aws dynamodb delete-item \
  --table-name pacs-terraform-locks \
  --key '{"LockID": {"S": "<lock-id>"}}'
```

### 3. EKS 배포 실패

**증상**: EKS 배포 실패
```
Error: error: the server doesn't have a resource type "deployment"
```

**해결 방법**:
```bash
# EKS 클러스터 연결 확인
aws eks update-kubeconfig --region ap-northeast-2 --name pacs-cluster

# 클러스터 상태 확인
kubectl cluster-info
kubectl get nodes
```

---

## 📚 다음 단계

이제 CI/CD 파이프라인을 성공적으로 설정했으니 다음 문서들을 학습하세요:

1. **모니터링 및 로깅** - 전체 시스템 모니터링
2. **보안 및 컴플라이언스** - 보안 정책 및 감사
3. **비용 최적화** - AWS 비용 관리 및 최적화

---

## 📖 참고 자료

- [GitHub Actions 공식 문서](https://docs.github.com/en/actions)
- [Terraform CI/CD 가이드](https://learn.hashicorp.com/tutorials/terraform/github-actions)
- [AWS EKS 배포 가이드](https://docs.aws.amazon.com/eks/latest/userguide/deploy-applications.html)

이제 PACS 프로젝트의 자동화된 배포를 위한 CI/CD 파이프라인이 준비되었습니다! 🚀
