# GC Batch Job 설계 문서

> **작성일**: 2026-01-12  
> **목적**: Snapshot 및 Mask 리소스의 자동 정리 시스템

---

## 📚 문서 구조

### 1. [현재 상태 검증 결과](./01-현재-상태-검증-결과.md)
- Snapshot 및 Mask 상태 관리 현황
- DB 스키마 검증
- 상태 전이 로직 검증
- GC 준비도 평가

### 2. [DB 마이그레이션 계획](./02-DB-마이그레이션-계획.md)
- GC 로그 테이블 생성 (039_create_gc_deletion_log.sql)
- Mask 상태 관리 추가 (038_add_mask_upload_status.sql)
- 마이그레이션 전략 및 롤백 계획

### 3. [Snapshot GC 구현 가이드](./03-Snapshot-GC-구현-가이드.md)
- PENDING 타임아웃 처리
- FAILED Snapshot S3 삭제
- 설정 관리 및 Rust 구현 예시

### 4. [배포 및 운영 가이드](./04-배포-및-운영-가이드.md)
- Kubernetes CronJob 배포
- Dry-run 검증 절차
- 모니터링 및 트러블슈팅

### 5. [Orphan 정리 가이드](./05-Orphan-정리-가이드.md)
- DB에 없는 S3 오브젝트 탐지
- S3 스캔 알고리즘
- 성능 최적화 및 안전 장치

### 6. [아키텍처 및 바이너리 구조](./06-아키텍처-및-바이너리-구조.md) ⭐
- 별도 바이너리 방식 선택 이유
- 프로젝트 구조 및 Cargo.toml 설정
- API 서버와 GC 배치 바이너리 분리
- 공통 라이브러리 (lib.rs) 구조

### 7. [모듈 설계 및 다이어그램](./07-모듈-설계-및-다이어그램.md) ⭐
- 계층별 모듈 설계 (Domain, Repository, Service, Binary)
- Mermaid 다이어그램 (Activity, Sequence)
- 단일 책임 원칙 및 의존성 방향
- 구현 체크리스트

### 8. [기존 코드베이스 분석](./08-기존-코드베이스-분석.md)
- 재사용 가능한 컴포넌트 확인
- S3 Service, Annotation Entity, ServiceError
- 재사용 방법 및 추가 작업 필요 사항

### 9. [상세 구현 가이드](./09-상세-구현-가이드.md) ⭐⭐ **핵심 문서**
- Step-by-Step 구현 가이드
- DB 마이그레이션 → Domain → Repository → Service → Binary
- 테스트 작성 및 실행
- Kubernetes 배포

### 10. [최종 요약 및 체크리스트](./10-최종-요약-및-체크리스트.md) ⭐⭐ **시작 문서**
- 프로젝트 개요 및 파일 구조
- 구현 체크리스트 (Phase별)
- 빠른 시작 가이드
- 모니터링 및 성능 최적화

---

## 🎯 프로젝트 목표

### 문제 정의
- **Snapshot**: 업로드 실패 시 S3에 불완전한 파일 잔류
- **Mask**: 어노테이션 삭제 시 구버전 Mask 미삭제
- **Orphan**: DB 레코드 없이 S3에만 존재하는 파일

### 해결 방안
1. **상태 기반 정리**: PENDING/FAILED 상태를 활용한 자동 정리
2. **Grace Period**: 즉시 삭제 대신 유예 기간 적용 (3-7일)
3. **Dry-run 모드**: 프로덕션 배포 전 충분한 검증
4. **로그 기록**: 모든 삭제 작업 감사 추적

---

## 📊 개발 로드맵

### Phase 1: Snapshot GC (2주) - ✅ 즉시 시작 가능

**목표**: Snapshot 리소스 자동 정리

**작업**:
- [x] 현재 상태 검증 (완료)
- [x] 아키텍처 설계 - 별도 바이너리 방식 (완료)
- [ ] 프로젝트 구조 리팩토링 (lib.rs 분리)
- [ ] GC 로그 테이블 마이그레이션
- [ ] GC Service 구현
- [ ] gc_runner 바이너리 구현
- [ ] Dockerfile 수정 (멀티 바이너리)
- [ ] Dry-run 1주일 검증
- [ ] 프로덕션 배포

**산출물**:
- `src/lib.rs` (공통 라이브러리)
- `src/bin/gc_runner.rs` (GC 배치 바이너리)
- `src/application/services/gc_service.rs`
- `src/infrastructure/repositories/gc_repository.rs`
- Kubernetes CronJob YAML
- 운영 가이드

---

### Phase 2: Mask 상태 관리 추가 (2주)

**목표**: Mask 업로드 상태 추적 체계 구축

**작업**:
- [ ] DB 마이그레이션 (mask_upload_status 추가)
- [ ] Mask 업로드 URL 생성 API 수정
- [ ] Mask 업로드 완료 API 추가
- [ ] 기존 데이터 마이그레이션
- [ ] 통합 테스트

**산출물**:
- `038_add_mask_upload_status.sql`
- API 수정 사항
- 마이그레이션 가이드

---

### Phase 3: Mask GC 구현 (1주)

**목표**: Mask 리소스 자동 정리

**작업**:
- [ ] PENDING Mask 타임아웃 구현
- [ ] FAILED Mask 삭제 구현
- [ ] 어노테이션 삭제 시 Mask 정리
- [ ] Dry-run 검증
- [ ] 프로덕션 배포

**산출물**:
- Mask GC 로직 추가
- 통합 테스트

---

### Phase 4: Orphan 정리 (2주) - 선택

**목표**: DB에 없는 S3 오브젝트 정리

**작업**:
- [ ] S3 스캔 알고리즘 구현
- [ ] Snapshot Orphan 탐지
- [ ] Mask Orphan 탐지
- [ ] 성능 최적화 (병렬 처리)
- [ ] Dry-run 2주 검증
- [ ] 프로덕션 배포

**산출물**:
- Orphan 정리 로직
- 성능 최적화 가이드

---

## 🔧 기술 스택

### 언어 및 프레임워크
- **Rust**: 배치 작업 구현
  - **멀티 바이너리**: API 서버 (`pacs-server`) + GC 배치 (`gc_runner`)
  - **공통 라이브러리**: `lib.rs`로 코드 공유
- **SQLx**: DB 쿼리
- **AWS SDK**: S3 작업
- **Clap**: CLI 인자 파싱 (GC 배치 전용)

### 인프라
- **Kubernetes**: CronJob 스케줄링
- **PostgreSQL**: 메타데이터 저장
- **AWS S3**: 파일 스토리지
- **Docker**: 멀티 바이너리 이미지

### 모니터링 (선택)
- **Prometheus**: 메트릭 수집
- **Grafana**: 대시보드
- **AlertManager**: 알림

---

## 📈 예상 효과

### 비용 절감
- **Snapshot**: 월 100GB 절감 예상 (실패 업로드 정리)
- **Mask**: 월 500GB 절감 예상 (구버전 정리)
- **Orphan**: 월 200GB 절감 예상 (고아 파일 정리)

**총 예상 절감**: 월 800GB ≈ $18/월 (S3 Standard 기준)

### 운영 효율
- 수동 정리 작업 제거
- 스토리지 사용량 가시성 향상
- 감사 추적 자동화

---

## 🚀 빠른 시작

### 📖 문서 읽기 순서

#### 처음 시작하는 경우
1. **[10-최종-요약-및-체크리스트.md](./10-최종-요약-및-체크리스트.md)** - 전체 개요 파악
2. **[09-상세-구현-가이드.md](./09-상세-구현-가이드.md)** - 구현 시작
3. **[07-모듈-설계-및-다이어그램.md](./07-모듈-설계-및-다이어그램.md)** - 다이어그램 확인

#### 상세 설계 확인
4. **[06-아키텍처-및-바이너리-구조.md](./06-아키텍처-및-바이너리-구조.md)** - 아키텍처 이해
5. **[08-기존-코드베이스-분석.md](./08-기존-코드베이스-분석.md)** - 재사용 컴포넌트 확인

#### 배포 및 운영
6. **[04-배포-및-운영-가이드.md](./04-배포-및-운영-가이드.md)** - Kubernetes 배포

---

### 💻 구현 순서 (총 9-10시간)

```
1. DB 마이그레이션 (30분)
   ↓
2. Domain Layer (1시간)
   ↓
3. Repository Layer (2시간)
   ↓
4. Service Layer (2시간)
   ↓
5. Binary Layer (1시간)
   ↓
6. 테스트 작성 (2시간)
   ↓
7. Kubernetes 배포 (1시간)
```

**상세 가이드**: [09-상세-구현-가이드.md](./09-상세-구현-가이드.md)

---

### 🔧 로컬 테스트

```bash
# 1. DB 마이그레이션
sqlx migrate run

# 2. 빌드
cargo build --bin gc_runner

# 3. Dry-run 테스트
./target/debug/gc_runner timeout-pending --dry-run=true
./target/debug/gc_runner cleanup-failed --dry-run=true

# 4. 실제 실행 (주의!)
./target/debug/gc_runner timeout-pending --grace-days=3 --batch-size=100
./target/debug/gc_runner cleanup-failed --grace-days=7 --batch-size=100
```

---

### 📊 현재 상태 확인

```bash
# Snapshot 상태 분포
psql -d pacs_production -c "
SELECT
    snapshot_upload_status,
    COUNT(*) as count
FROM annotations
WHERE snapshot_upload_status IS NOT NULL
GROUP BY snapshot_upload_status;
"

# PENDING 타임아웃 대상
psql -d pacs_production -c "
SELECT COUNT(*)
FROM annotations
WHERE snapshot_upload_status = 'pending'
  AND updated_at < NOW() - INTERVAL '3 days';
"
```

---

### 🐳 Docker 빌드 및 실행

```bash
# 1. 이미지 빌드
docker build -t pacs-server:gc-test .

# 2. 컨테이너 실행
docker run --rm \
  -e DATABASE_URL=postgresql://... \
  -e S3_BUCKET=... \
  -e S3_REGION=... \
  -e S3_ACCESS_KEY=... \
  -e S3_SECRET_KEY=... \
  pacs-server:gc-test \
  /usr/local/bin/gc_runner timeout-pending --dry-run=true
```

---

### ☸️ Kubernetes 배포

```bash
# 1. ConfigMap/Secret 생성
kubectl apply -f k8s/config/pacs-config.yaml
kubectl apply -f k8s/secrets/pacs-secrets.yaml

# 2. CronJob 배포
kubectl apply -f k8s/cronjobs/gc-job-a-timeout-pending.yaml
kubectl apply -f k8s/cronjobs/gc-job-b-cleanup-failed.yaml

# 3. 상태 확인
kubectl get cronjobs -n pacs-system
kubectl get jobs -n pacs-system

# 4. 수동 실행 (테스트)
kubectl create job --from=cronjob/gc-job-a-timeout-pending gc-test-a -n pacs-system

# 5. 로그 확인
kubectl logs -f job/gc-test-a -n pacs-system
```

---

## 📞 문의 및 지원

### 문서 관련
- 각 문서의 상세 내용 참조
- 예제 코드 및 SQL 쿼리 제공

### 구현 관련
- `03-Snapshot-GC-구현-가이드.md` 참조
- Rust 코드 예시 포함

### 운영 관련
- `04-배포-및-운영-가이드.md` 참조
- 트러블슈팅 가이드 포함

---

## 📝 변경 이력

| 날짜 | 버전 | 변경 내용 |
|------|------|----------|
| 2026-01-12 | 1.0 | 초기 문서 작성 |
| | | - 현재 상태 검증 완료 |
| | | - Snapshot GC 즉시 시작 가능 확인 |
| | | - Mask 상태 관리 추가 필요 확인 |
| 2026-01-12 | 1.1 | 아키텍처 설계 추가 |
| | | - 별도 바이너리 방식 선택 |
| | | - 멀티 바이너리 구조 설계 |
| | | - Dockerfile 및 배포 전략 수립 |
| 2026-01-12 | 2.0 | **상세 구현 가이드 완성** ⭐ |
| | | - 07-모듈-설계-및-다이어그램.md 추가 |
| | | - 08-기존-코드베이스-분석.md 추가 |
| | | - 09-상세-구현-가이드.md 추가 (핵심) |
| | | - 10-최종-요약-및-체크리스트.md 추가 |
| | | - Mermaid 다이어그램 렌더링 |
| | | - Step-by-Step 구현 가이드 |
| | | - 재사용 가능 컴포넌트 분석 |
| | | - 구현 체크리스트 및 빠른 시작 가이드 |

---

## ✅ 다음 단계

### 🎯 즉시 시작 가능

**모든 설계 및 가이드 문서 완성!** 이제 코드 작성을 시작할 수 있습니다.

### 📋 구현 체크리스트

#### Phase 1: DB 마이그레이션 (30분)
- [ ] `migrations/039_create_gc_deletion_log.sql` 작성
- [ ] 로컬 DB에 마이그레이션 실행
- [ ] 테이블 생성 확인

#### Phase 2: Domain Layer (1시간)
- [ ] `src/domain/entities/gc_deletion_log.rs` 작성
- [ ] `src/domain/repositories/gc_repository.rs` 작성
- [ ] `src/domain/repositories/gc_log_repository.rs` 작성

#### Phase 3: Repository Layer (2시간)
- [ ] `src/infrastructure/repositories/gc_repository_impl.rs` 작성
- [ ] `src/infrastructure/repositories/gc_log_repository_impl.rs` 작성
- [ ] Repository 단위 테스트

#### Phase 4: Service Layer (2시간)
- [ ] `src/application/services/gc_service.rs` 작성
- [ ] `src/application/services/gc_service_impl.rs` 작성
- [ ] Service 단위 테스트 (Mock)

#### Phase 5: Binary Layer (1시간)
- [ ] `src/bin/gc_runner.rs` 작성
- [ ] `Cargo.toml` 업데이트
- [ ] 로컬 빌드 및 실행 테스트

#### Phase 6: 통합 테스트 (2시간)
- [ ] 통합 테스트 작성
- [ ] Docker Compose 환경 구성
- [ ] Dry-run 모드 검증

#### Phase 7: 배포 (1시간)
- [ ] `Dockerfile` 수정
- [ ] Kubernetes CronJob YAML 작성
- [ ] ConfigMap/Secret 생성
- [ ] CronJob 배포 및 테스트

---

### 🚀 시작하기

**다음 문서를 읽고 구현을 시작하세요**:

1. **[10-최종-요약-및-체크리스트.md](./10-최종-요약-및-체크리스트.md)** - 전체 개요
2. **[09-상세-구현-가이드.md](./09-상세-구현-가이드.md)** - Step-by-Step 가이드

**예상 소요 시간**: 약 **9-10시간**

**권장 접근 방식**: Phase 1부터 순서대로 진행하여 점진적으로 확장

