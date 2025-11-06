# Terraform 학습 가이드

## 📚 학습 문서 목록

### 1. 기초 개념 및 이론
- [Terraform 공식 문서 - Introduction](https://developer.hashicorp.com/terraform/intro)
- [Terraform 공식 문서 - Core Concepts](https://developer.hashicorp.com/terraform/intro/core-concepts)
- [Terraform 공식 문서 - Configuration Language](https://developer.hashicorp.com/terraform/language)
- [Terraform 공식 문서 - State Management](https://developer.hashicorp.com/terraform/language/state)

### 2. 실습 및 튜토리얼
- [Terraform 공식 튜토리얼 - Get Started](https://developer.hashicorp.com/terraform/tutorials)
- [Terraform 공식 튜토리얼 - AWS](https://developer.hashicorp.com/terraform/tutorials/aws-get-started)
- [Terraform 공식 튜토리얼 - Azure](https://developer.hashicorp.com/terraform/tutorials/azure-get-started)
- [Terraform 공식 튜토리얼 - GCP](https://developer.hashicorp.com/terraform/tutorials/gcp-get-started)

### 3. 고급 주제
- [Terraform 공식 문서 - Modules](https://developer.hashicorp.com/terraform/language/modules)
- [Terraform 공식 문서 - Workspaces](https://developer.hashicorp.com/terraform/language/state/workspaces)
- [Terraform 공식 문서 - Data Sources](https://developer.hashicorp.com/terraform/language/data-sources)
- [Terraform 공식 문서 - Providers](https://developer.hashicorp.com/terraform/language/providers)

### 4. 모범 사례 및 설계
- [Terraform 공식 문서 - Best Practices](https://developer.hashicorp.com/terraform/language/modules/develop)
- [Terraform 공식 문서 - Security](https://developer.hashicorp.com/terraform/language/state/sensitive-data)
- [Terraform 공식 문서 - Performance](https://developer.hashicorp.com/terraform/language/performance)
- [Terraform 공식 문서 - Testing](https://developer.hashicorp.com/terraform/language/testing)

### 5. 도구 및 생태계
- [Terraform 공식 문서 - CLI](https://developer.hashicorp.com/terraform/cli)
- [Terraform 공식 문서 - Cloud](https://developer.hashicorp.com/terraform/cloud)
- [Terraform 공식 문서 - Enterprise](https://developer.hashicorp.com/terraform/enterprise)
- [Terraform 공식 문서 - Registry](https://developer.hashicorp.com/terraform/registry)

### 6. 특정 클라우드 제공업체
- [AWS Provider 문서](https://registry.terraform.io/providers/hashicorp/aws/latest/docs)
- [Azure Provider 문서](https://registry.terraform.io/providers/hashicorp/azurerm/latest/docs)
- [Google Cloud Provider 문서](https://registry.terraform.io/providers/hashicorp/google/latest/docs)
- [Kubernetes Provider 문서](https://registry.terraform.io/providers/hashicorp/kubernetes/latest/docs)

### 7. 실무 예제 및 케이스 스터디
- [Terraform 공식 문서 - Use Cases](https://developer.hashicorp.com/terraform/use-cases)
- [Terraform 공식 문서 - Migration Guides](https://developer.hashicorp.com/terraform/language/upgrade-guides)
- [Terraform 공식 문서 - Troubleshooting](https://developer.hashicorp.com/terraform/language/troubleshooting)

### 8. 추가 학습 자료
- [Terraform 공식 문서 - Glossary](https://developer.hashicorp.com/terraform/glossary)
- [Terraform 공식 문서 - FAQ](https://developer.hashicorp.com/terraform/faq)
- [Terraform 공식 문서 - Community](https://developer.hashicorp.com/terraform/community)

## 🎯 학습 순서 추천

### 초급자 (1-2주)
1. **기초 개념** - Terraform이 무엇인지, 왜 사용하는지 이해
2. **Configuration Language** - HCL 문법 학습
3. **Core Concepts** - Provider, Resource, State 개념 이해
4. **첫 번째 실습** - 간단한 AWS/Azure/GCP 리소스 생성

### 중급자 (2-4주)
1. **Modules** - 재사용 가능한 코드 구조화
2. **State Management** - State 파일 관리 및 백엔드 설정
3. **Data Sources** - 기존 리소스 정보 조회
4. **Workspaces** - 환경별 관리

### 고급자 (1-2개월)
1. **Best Practices** - 코드 구조, 네이밍, 보안
2. **Testing** - 테스트 전략 및 도구
3. **CI/CD 통합** - GitHub Actions, GitLab CI 등
4. **Enterprise 기능** - Terraform Cloud/Enterprise

## 🛠️ 실습 환경 설정

### 필수 도구
- [Terraform CLI 설치](https://developer.hashicorp.com/terraform/downloads)
- [AWS CLI 설치](https://aws.amazon.com/cli/) (AWS 사용 시)
- [Azure CLI 설치](https://docs.microsoft.com/en-us/cli/azure/install-azure-cli) (Azure 사용 시)
- [Google Cloud CLI 설치](https://cloud.google.com/sdk/docs/install) (GCP 사용 시)

### IDE 및 에디터
- [Terraform VS Code 확장](https://marketplace.visualstudio.com/items?itemName=HashiCorp.terraform)
- [IntelliJ Terraform 플러그인](https://plugins.jetbrains.com/plugin/7808-terraform-and-hcl)

## 📝 실습 프로젝트 아이디어

### 초급 프로젝트
1. **간단한 웹 서버** - EC2 인스턴스 + 보안 그룹
2. **정적 웹사이트** - S3 버킷 + CloudFront
3. **데이터베이스** - RDS 인스턴스 생성

### 중급 프로젝트
1. **VPC 구성** - 네트워크 아키텍처 설계
2. **Kubernetes 클러스터** - EKS/AKS/GKE 생성
3. **모니터링 스택** - CloudWatch/Prometheus 설정

### 고급 프로젝트
1. **멀티 클라우드 아키텍처** - AWS + Azure 하이브리드
2. **마이크로서비스 인프라** - 서비스 메시 + API 게이트웨이
3. **GitOps 파이프라인** - ArgoCD + Terraform 통합

## 🔗 유용한 링크

- [Terraform Registry](https://registry.terraform.io/) - 공식 모듈 및 프로바이더
- [Terraform GitHub](https://github.com/hashicorp/terraform) - 소스 코드 및 이슈
- [Terraform Community](https://discuss.hashicorp.com/c/terraform-core) - 커뮤니티 포럼
- [Terraform YouTube](https://www.youtube.com/c/HashiCorp) - 공식 채널

## 📚 추천 도서

1. **"Terraform: Up & Running"** - Yevgeniy Brikman
2. **"Infrastructure as Code"** - Kief Morris
3. **"The DevOps Handbook"** - Gene Kim, Patrick Debois, John Willis

이 가이드를 따라 학습하시면 Terraform을 체계적으로 이해하고 실무에 적용할 수 있을 것입니다!
