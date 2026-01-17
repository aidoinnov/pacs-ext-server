# Kubernetes 배포 가이드

> **목적**: PACS Server 및 GC Batch Job Kubernetes 배포  
> **작성일**: 2026-01-13

---

## 📁 디렉토리 구조

```
k8s/
├── README.md                           # 이 파일
├── config/
│   └── pacs-config.yaml                # ConfigMap
├── secrets/
│   └── pacs-secrets.yaml.example       # Secret 예시
└── cronjobs/
    ├── gc-job-a-timeout-pending.yaml   # Job A CronJob
    └── gc-job-b-cleanup-failed.yaml    # Job B CronJob
```

---

## 🚀 배포 순서

### 1. Namespace 생성

```bash
kubectl create namespace pacs-system
```

### 2. ConfigMap 생성

```bash
# pacs-config.yaml 수정 (S3 버킷, 리전 등)
vi k8s/config/pacs-config.yaml

# 적용
kubectl apply -f k8s/config/pacs-config.yaml
```

### 3. Secret 생성

**⚠️ 주의: Secret은 Git에 커밋하지 마세요!**

```bash
# 방법 1: kubectl 명령어로 직접 생성 (권장)
kubectl create secret generic pacs-secrets \
  --namespace=pacs-system \
  --from-literal=database-url='postgresql://user:pass@host:5432/db' \
  --from-literal=s3-access-key='YOUR_ACCESS_KEY' \
  --from-literal=s3-secret-key='YOUR_SECRET_KEY'

# 방법 2: YAML 파일 사용 (주의: 실제 값 입력 후 즉시 삭제)
cp k8s/secrets/pacs-secrets.yaml.example k8s/secrets/pacs-secrets.yaml
vi k8s/secrets/pacs-secrets.yaml  # 실제 값 입력
kubectl apply -f k8s/secrets/pacs-secrets.yaml
rm k8s/secrets/pacs-secrets.yaml  # 즉시 삭제!
```

### 4. Docker 이미지 빌드 및 푸시

```bash
# 이미지 빌드
docker build -t your-registry/pacs-server:latest -f pacs-server/Dockerfile pacs-server/

# 이미지 푸시
docker push your-registry/pacs-server:latest
```

### 5. CronJob 배포 (Dry-run 모드)

**⚠️ 프로덕션 배포 전 1주일 Dry-run 테스트 필수!**

```bash
# CronJob YAML 파일에서 --dry-run 주석 해제 확인
# args에 "--dry-run" 추가되어 있는지 확인

# Job A 배포
kubectl apply -f k8s/cronjobs/gc-job-a-timeout-pending.yaml

# Job B 배포
kubectl apply -f k8s/cronjobs/gc-job-b-cleanup-failed.yaml
```

### 6. 수동 실행 테스트

```bash
# Job A 수동 실행
kubectl create job --from=cronjob/gc-job-a-timeout-pending gc-test-a -n pacs-system

# 로그 확인
kubectl logs -f job/gc-test-a -n pacs-system

# Job B 수동 실행
kubectl create job --from=cronjob/gc-job-b-cleanup-failed gc-test-b -n pacs-system

# 로그 확인
kubectl logs -f job/gc-test-b -n pacs-system
```

### 7. Dry-run 검증 (1주일)

```bash
# CronJob 상태 확인
kubectl get cronjobs -n pacs-system

# Job 실행 이력 확인
kubectl get jobs -n pacs-system

# 로그 확인
kubectl logs -l app=pacs-gc -n pacs-system --tail=100

# DB에서 로그 확인
psql $DATABASE_URL -c "
SELECT 
    job_type,
    status,
    COUNT(*) as count,
    SUM(file_size) as total_bytes
FROM gc_deletion_log
WHERE created_at >= NOW() - INTERVAL '7 days'
GROUP BY job_type, status;
"
```

### 8. 프로덕션 배포

**Dry-run 1주일 검증 완료 후:**

```bash
# CronJob YAML에서 --dry-run 제거
vi k8s/cronjobs/gc-job-a-timeout-pending.yaml
vi k8s/cronjobs/gc-job-b-cleanup-failed.yaml

# 재배포
kubectl apply -f k8s/cronjobs/gc-job-a-timeout-pending.yaml
kubectl apply -f k8s/cronjobs/gc-job-b-cleanup-failed.yaml
```

---

## 📊 모니터링

### CronJob 상태 확인

```bash
# CronJob 목록
kubectl get cronjobs -n pacs-system

# 출력 예시:
# NAME                        SCHEDULE      SUSPEND   ACTIVE   LAST SCHEDULE   AGE
# gc-job-a-timeout-pending    0 17 * * *    False     0        2h              7d
# gc-job-b-cleanup-failed     0 18 * * *    False     0        1h              7d
```

### Job 실행 이력

```bash
# 최근 Job 목록
kubectl get jobs -n pacs-system --sort-by=.metadata.creationTimestamp

# 특정 Job 상세 정보
kubectl describe job gc-job-a-timeout-pending-28471234 -n pacs-system
```

### 로그 확인

```bash
# 최근 로그
kubectl logs -l app=pacs-gc -n pacs-system --tail=100

# 특정 Job 로그
kubectl logs job/gc-job-a-timeout-pending-28471234 -n pacs-system

# 실시간 로그
kubectl logs -f -l app=pacs-gc -n pacs-system
```

---

## 🔧 트러블슈팅

### CronJob이 실행되지 않음

```bash
# CronJob 상세 정보 확인
kubectl describe cronjob gc-job-a-timeout-pending -n pacs-system

# Events 확인
kubectl get events -n pacs-system --sort-by='.lastTimestamp'
```

### Job 실패

```bash
# 실패한 Job 로그 확인
kubectl logs job/gc-job-a-timeout-pending-28471234 -n pacs-system

# Pod 상태 확인
kubectl get pods -l app=pacs-gc -n pacs-system

# Pod 상세 정보
kubectl describe pod gc-job-a-timeout-pending-28471234-xxxxx -n pacs-system
```

### DB 연결 실패

```bash
# Secret 확인
kubectl get secret pacs-secrets -n pacs-system -o yaml

# DATABASE_URL 확인 (base64 디코딩)
kubectl get secret pacs-secrets -n pacs-system -o jsonpath='{.data.database-url}' | base64 -d
```

---

## 🗑️ 삭제

```bash
# CronJob 삭제
kubectl delete cronjob gc-job-a-timeout-pending -n pacs-system
kubectl delete cronjob gc-job-b-cleanup-failed -n pacs-system

# ConfigMap 삭제
kubectl delete configmap pacs-config -n pacs-system

# Secret 삭제
kubectl delete secret pacs-secrets -n pacs-system

# Namespace 삭제 (모든 리소스 삭제)
kubectl delete namespace pacs-system
```

---

## 📚 참고 문서

- [04-배포-및-운영-가이드.md](../../docs/gc-batch-job/04-배포-및-운영-가이드.md)
- [11-구현-완료-요약.md](../../docs/gc-batch-job/11-구현-완료-요약.md)

