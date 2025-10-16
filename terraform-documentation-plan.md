# 📋 Terraform 학습 문서 작성 계획

현재 PACS Extension Server 프로젝트 기반으로 작성해야 할 Terraform 학습 문서 목록입니다.

## 🎯 Phase 1: 기본 인프라 구성 (1-2주)

### 1.1 Docker Compose → Terraform 마이그레이션
- [ ] **Docker Provider 기초 가이드**
  - Docker Provider 설치 및 설정
  - 기본 문법 및 리소스 타입
  - 컨테이너, 이미지, 볼륨 관리

- [ ] **현재 docker-compose.yml 분석**
  - PostgreSQL 컨테이너 설정 분석
  - 환경변수 및 포트 매핑
  - 볼륨 및 헬스체크 설정

- [ ] **Terraform으로 PostgreSQL 구성하기**
  - docker_image 리소스 사용법
  - docker_container 리소스 설정
  - docker_volume 관리
  - 환경변수 및 포트 설정

- [ ] **Docker 네트워크 구성**
  - docker_network 리소스
  - 컨테이너 간 통신 설정
  - 외부 접근 설정

### 1.2 로컬 개발 환경 구성
- [ ] **Terraform Workspaces 가이드**
  - Workspace 개념 및 사용법
  - 환경별 설정 분리
  - workspace 명령어 사용법

- [ ] **MinIO 로컬 개발 환경 구성**
  - MinIO 컨테이너 설정
  - 포트 및 환경변수 구성
  - 데이터 영속성 설정

- [ ] **환경별 설정 관리**
  - development.tf, production.tf 분리
  - 변수를 통한 환경별 설정
  - .tfvars 파일 활용

## 🎯 Phase 2: AWS 클라우드 인프라 (2-3주)

### 2.1 AWS S3 버킷 구성
- [ ] **AWS Provider 설정 가이드**
  - AWS Provider 설치 및 인증
  - AWS CLI 설정
  - IAM 역할 및 정책

- [ ] **S3 버킷 생성 및 관리**
  - aws_s3_bucket 리소스
  - 버킷 정책 설정
  - 버킷 버전 관리

- [ ] **S3 CORS 설정**
  - CORS 정책 이해
  - aws_s3_bucket_cors_configuration
  - 프론트엔드 연동 설정

- [ ] **S3 암호화 및 보안**
  - 서버 측 암호화 설정
  - 접근 로그 설정
  - 퍼블릭 액세스 차단

### 2.2 IAM 정책 및 사용자 생성
- [ ] **IAM 기본 개념**
  - IAM 사용자, 그룹, 역할
  - 정책 문서 작성법
  - 최소 권한 원칙

- [ ] **S3 전용 IAM 정책 작성**
  - JSON 정책 문서 작성
  - aws_iam_policy 리소스
  - 리소스 ARN 이해

- [ ] **IAM 사용자 및 키 관리**
  - aws_iam_user 리소스
  - aws_iam_access_key 관리
  - 보안 모범 사례

### 2.3 RDS PostgreSQL 구성
- [ ] **RDS 기본 개념**
  - RDS vs EC2 인스턴스
  - 데이터베이스 엔진 선택
  - 인스턴스 클래스 및 스토리지

- [ ] **RDS 인스턴스 생성**
  - aws_db_instance 리소스
  - 파라미터 그룹 설정
  - 옵션 그룹 구성

- [ ] **RDS 보안 설정**
  - 보안 그룹 구성
  - 서브넷 그룹 설정
  - 암호화 설정

- [ ] **RDS 백업 및 모니터링**
  - 자동 백업 설정
  - 스냅샷 관리
  - CloudWatch 모니터링

## 🎯 Phase 3: 네트워킹 및 보안 (2-3주)

### 3.1 VPC 및 서브넷 구성
- [ ] **VPC 기본 개념**
  - VPC, 서브넷, 라우팅 테이블
  - CIDR 블록 이해
  - 가용 영역 활용

- [ ] **VPC 생성 및 설정**
  - aws_vpc 리소스
  - DNS 설정
  - IPv6 설정

- [ ] **서브넷 구성**
  - 퍼블릭/프라이빗 서브넷
  - aws_subnet 리소스
  - 가용 영역 분산

- [ ] **인터넷 게이트웨이 및 NAT**
  - aws_internet_gateway
  - aws_nat_gateway
  - 라우팅 테이블 설정

### 3.2 보안 그룹 구성
- [ ] **보안 그룹 기본 개념**
  - 보안 그룹 vs NACL
  - 인바운드/아웃바운드 규칙
  - 상태 기반 필터링

- [ ] **웹 서버 보안 그룹**
  - HTTP/HTTPS 포트 설정
  - SSH 접근 제한
  - aws_security_group 리소스

- [ ] **데이터베이스 보안 그룹**
  - 데이터베이스 포트 설정
  - 웹 서버에서만 접근 허용
  - 보안 그룹 간 참조

## 🎯 Phase 4: 애플리케이션 배포 (2-3주)

### 4.1 EC2 인스턴스 구성
- [ ] **EC2 기본 개념**
  - 인스턴스 타입 선택
  - AMI 선택 기준
  - 키 페어 관리

- [ ] **EC2 인스턴스 생성**
  - aws_instance 리소스
  - 사용자 데이터 스크립트
  - 탄력적 IP 설정

- [ ] **EC2 보안 설정**
  - 키 페어 생성 및 관리
  - 보안 그룹 연결
  - IAM 역할 연결

### 4.2 Application Load Balancer 구성
- [ ] **ALB 기본 개념**
  - ALB vs NLB vs CLB
  - 타겟 그룹 이해
  - 헬스체크 설정

- [ ] **ALB 생성 및 설정**
  - aws_lb 리소스
  - aws_lb_target_group
  - aws_lb_listener

- [ ] **SSL/TLS 인증서 관리**
  - AWS Certificate Manager
  - HTTPS 리스너 설정
  - 인증서 갱신

## 🎯 Phase 5: 모니터링 및 로깅 (1-2주)

### 5.1 CloudWatch 구성
- [ ] **CloudWatch 기본 개념**
  - 메트릭, 로그, 알람
  - 네임스페이스 이해
  - 대시보드 구성

- [ ] **로그 그룹 설정**
  - aws_cloudwatch_log_group
  - 로그 스트림 관리
  - 로그 보존 정책

- [ ] **메트릭 알람 설정**
  - aws_cloudwatch_metric_alarm
  - 임계값 설정
  - SNS 알림 연동

- [ ] **대시보드 생성**
  - aws_cloudwatch_dashboard
  - 위젯 구성
  - 실시간 모니터링

## 🎯 Phase 6: CI/CD 파이프라인 (2-3주)

### 6.1 GitHub Actions 통합
- [ ] **Terraform 상태 관리**
  - 로컬 vs 원격 상태
  - S3 백엔드 설정
  - 상태 잠금 (DynamoDB)

- [ ] **GitHub Actions 기본**
  - 워크플로우 파일 작성
  - 시크릿 관리
  - 환경별 배포

- [ ] **자동화된 배포 파이프라인**
  - terraform plan 자동화
  - terraform apply 자동화
  - 롤백 전략

- [ ] **환경별 배포 전략**
  - development → staging → production
  - 브랜치별 배포
  - 승인 프로세스

## 📚 추가 문서

### 실습 가이드
- [ ] **로컬 개발 환경 설정**
  - Terraform CLI 설치
  - AWS CLI 설정
  - 개발 도구 설정

- [ ] **실습 프로젝트**
  - 단계별 실습 가이드
  - 문제 해결 가이드
  - 모범 사례 적용

### 참고 자료
- [ ] **Terraform 명령어 레퍼런스**
  - 주요 명령어 정리
  - 옵션 및 플래그 설명
  - 실무 활용 팁

- [ ] **AWS 리소스 매핑**
  - Terraform 리소스 ↔ AWS 서비스
  - 자주 사용하는 리소스 정리
  - 설정 옵션 가이드

---

## 📅 작성 우선순위

### 1단계 (즉시 작성)
1. Docker Provider 기초 가이드
2. 현재 docker-compose.yml 분석
3. Terraform으로 PostgreSQL 구성하기

### 2단계 (1주차)
4. AWS Provider 설정 가이드
5. S3 버킷 생성 및 관리
6. IAM 기본 개념

### 3단계 (2주차)
7. RDS PostgreSQL 구성
8. VPC 기본 개념
9. 보안 그룹 구성

### 4단계 (3-4주차)
10. EC2 인스턴스 구성
11. ALB 구성
12. CloudWatch 구성

### 5단계 (5-6주차)
13. GitHub Actions 통합
14. 자동화된 배포 파이프라인
15. 실습 가이드

이 계획에 따라 단계별로 문서를 작성하면 체계적인 Terraform 학습이 가능합니다!
