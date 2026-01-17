# GC Batch Job 구현 완료 ✅

> **작성일**: 2026-01-12
> **완료일**: 2026-01-13
> **목적**: Snapshot 리소스의 자동 정리 시스템
> **상태**: ✅ **구현 완료 및 테스트 통과**

---

## 🎉 구현 완료 요약

### ✅ 완료된 작업
1. **ObjectStorageService 추상화 계층** - Trait 기반 S3 연동
2. **GC Service Layer** - Job A/B 비즈니스 로직 (트랜잭션 일관성 개선)
3. **Repository Layer** - DB 쿼리 및 로그 기록
4. **Binary Layer** - CLI 인터페이스 (`gc_runner`) + Advisory Lock
5. **Database Migration** - snapshot 컬럼 및 GC 로그 테이블
6. **E2E 테스트** - 14개 시나리오 모두 통과
7. **file_size 기록** - S3 파일 크기 추적 기능
8. **동시 실행 방지** - PostgreSQL Advisory Lock

### 📊 테스트 결과
```
✅ Unit Tests: 6/6 통과
✅ E2E Tests: 14/14 통과
✅ 빌드 테스트: 성공
✅ 데이터베이스 마이그레이션: 성공
✅ Dry-run 모드: 정상 작동
✅ Advisory Lock: 동시 실행 방지 검증 완료
```

### 🆕 주요 개선사항 (2026-01-13)
1. **트랜잭션 일관성 문제 해결** - 로그 기록 실패 시에도 GC 작업 계속 진행
2. **멱등성 보장** - 중복 실행 시 안전하게 처리
3. **Grace Period 경계값 처리** - 정확히 일치하는 경우도 처리
4. **빈 결과 처리** - 처리할 항목 없어도 정상 종료
5. **Batch Size 경계값 처리** - 정확한 배치 크기 준수
6. **NULL snapshot_image_key 처리** - SQL WHERE 절에서 자동 필터링
7. **동시 실행 방지** - PostgreSQL Advisory Lock (Job A: 1001, Job B: 1002)
8. **file_size 기록** - 삭제된 파일 크기 추적 (용량 모니터링)

### 🚀 빠른 시작
```bash
# 빌드
cd pacs-server
cargo build --bin gc_runner

# Job A: PENDING 타임아웃 처리 (Dry-run)
./target/debug/gc_runner timeout-pending --grace-days 3 --batch-size 1000 --dry-run

# Job B: FAILED 스냅샷 정리 (Dry-run)
./target/debug/gc_runner cleanup-failed --grace-days 7 --batch-size 1000 --dry-run

# E2E 테스트 실행
cd tests/e2e
./run_e2e_tests.sh
# 또는
python3 test_gc_e2e.py
```

---

## 📚 문서 구조

### 1. [현재 상태 검증 결과](./01-현재-상태-검증-결과.md)
- Snapshot 및 Mask 상태 관리 현황
- DB 스키마 검증
- 상태 전이 로직 검증
- GC 준비도 평가

### 2. [DB 마이그레이션 계획](./02-DB-마이그레이션-계획.md) ✅ **완료**
- GC 로그 테이블 생성 (039_create_gc_deletion_log.sql)
- file_size 컬럼 추가 (BIGINT)
- 마이그레이션 전략 및 롤백 계획

### 3. [Snapshot GC 구현 가이드](./03-Snapshot-GC-구현-가이드.md) ✅ **완료**
- PENDING 타임아웃 처리
- FAILED Snapshot S3 삭제
- 설정 관리 및 Rust 구현 예시
- Advisory Lock을 통한 동시 실행 방지

### 4. [배포 및 운영 가이드](./04-배포-및-운영-가이드.md)
- Kubernetes CronJob 배포
- Dry-run 검증 절차
- 모니터링 및 트러블슈팅
- file_size 기반 용량 모니터링

### 5. [Orphan 정리 가이드](./05-Orphan-정리-가이드.md) ⏸️ **보류**
- DB에 없는 S3 오브젝트 탐지
- S3 스캔 알고리즘
- 성능 최적화 및 안전 장치
- **현재 구현 계획 없음**

### 6. [아키텍처 및 바이너리 구조](./06-아키텍처-및-바이너리-구조.md) ⭐ ✅ **완료**
- 별도 바이너리 방식 선택 이유
- 프로젝트 구조 및 Cargo.toml 설정
- API 서버와 GC 배치 바이너리 분리
- 공통 라이브러리 (lib.rs) 구조

### 7. [모듈 설계 및 다이어그램](./07-모듈-설계-및-다이어그램.md) ⭐ ✅ **완료**
- 계층별 모듈 설계 (Domain, Repository, Service, Binary)
- Mermaid 다이어그램 (Activity, Sequence)
- 단일 책임 원칙 및 의존성 방향
- 구현 체크리스트

### 8. [기존 코드베이스 분석](./08-기존-코드베이스-분석.md) ✅ **완료**
- 재사용 가능한 컴포넌트 확인
- S3 Service, Annotation Entity, ServiceError
- 재사용 방법 및 추가 작업 필요 사항

### 9. [상세 구현 가이드](./09-상세-구현-가이드.md) ⭐⭐ **핵심 문서** ✅ **완료**
- Step-by-Step 구현 가이드
- DB 마이그레이션 → Domain → Repository → Service → Binary
- 테스트 작성 및 실행 (14개 E2E 시나리오)
- Kubernetes 배포

### 10. [최종 요약 및 체크리스트](./10-최종-요약-및-체크리스트.md) ⭐⭐ **시작 문서** ✅ **완료**
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

### Phase 1: Snapshot GC ✅ **완료** (2026-01-13)

**목표**: Snapshot 리소스 자동 정리

**작업**:
- [x] 현재 상태 검증
- [x] 아키텍처 설계 - 별도 바이너리 방식
- [x] 프로젝트 구조 리팩토링 (lib.rs 분리)
- [x] GC 로그 테이블 마이그레이션
- [x] GC Service 구현
- [x] gc_runner 바이너리 구현
- [x] Advisory Lock 구현 (동시 실행 방지)
- [x] file_size 기록 기능
- [x] E2E 테스트 (14개 시나리오)
- [x] 트랜잭션 일관성 개선
- [x] 멱등성 보장
- [x] 경계값 처리
- [ ] Dockerfile 수정 (멀티 바이너리)
- [ ] Dry-run 1주일 검증
- [ ] 프로덕션 배포

**산출물**:
- ✅ `src/lib.rs` (공통 라이브러리)
- ✅ `src/bin/gc_runner.rs` (GC 배치 바이너리)
- ✅ `src/application/services/gc_service.rs`
- ✅ `src/application/services/gc_service_impl.rs`
- ✅ `src/infrastructure/repositories/gc_repository_impl.rs`
- ✅ `src/infrastructure/repositories/gc_log_repository_impl.rs`
- ✅ `tests/e2e/test_gc_e2e.py` (14개 시나리오)
- ⏳ Kubernetes CronJob YAML
- ⏳ 운영 가이드

---

### Phase 2: Mask 상태 관리 추가 ⏸️ **보류**

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

**상태**: 현재 구현 계획 없음

---

### Phase 3: Mask GC 구현 ⏸️ **보류**

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

**상태**: Phase 2 완료 후 진행

---

### Phase 4: Orphan 정리 ⏸️ **보류**

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

**상태**: 현재 구현 계획 없음

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
| 2026-01-13 | 3.0 | **Phase 1 구현 완료** 🎉 |
| | | - ObjectStorageService 추상화 계층 구현 |
| | | - GC Service Layer 구현 (트랜잭션 일관성) |
| | | - Repository Layer 구현 |
| | | - Binary Layer 구현 (gc_runner) |
| | | - Database Migration 완료 |
| | | - E2E 테스트 14개 시나리오 통과 |
| | | - Advisory Lock 구현 (동시 실행 방지) |
| | | - file_size 기록 기능 추가 |
| | | - 멱등성 보장 및 경계값 처리 |
| | | - NULL snapshot_image_key 처리 |
| | | - Job A/B 독립성 테스트 |
| | | - 락 자동 해제 테스트 |

---

## ✅ 다음 단계

### 🎯 Phase 1 완료! 다음은 배포 준비

**Phase 1 (Snapshot GC 구현) 완료!** 이제 프로덕션 배포를 준비할 수 있습니다.

### 📋 구현 체크리스트

#### Phase 1: DB 마이그레이션 ✅ **완료**
- [x] `migrations/039_create_gc_deletion_log.sql` 작성
- [x] 로컬 DB에 마이그레이션 실행
- [x] 테이블 생성 확인
- [x] file_size 컬럼 추가

#### Phase 2: Domain Layer ✅ **완료**
- [x] `src/domain/entities/gc_deletion_log.rs` 작성
- [x] `src/domain/repositories/gc_repository.rs` 작성
- [x] `src/domain/repositories/gc_log_repository.rs` 작성

#### Phase 3: Repository Layer ✅ **완료**
- [x] `src/infrastructure/repositories/gc_repository_impl.rs` 작성
- [x] `src/infrastructure/repositories/gc_log_repository_impl.rs` 작성
- [x] Repository 단위 테스트

#### Phase 4: Service Layer ✅ **완료**
- [x] `src/application/services/gc_service.rs` 작성
- [x] `src/application/services/gc_service_impl.rs` 작성
- [x] Service 단위 테스트 (Mock)
- [x] 트랜잭션 일관성 개선
- [x] 멱등성 보장
- [x] file_size 기록 기능

#### Phase 5: Binary Layer ✅ **완료**
- [x] `src/bin/gc_runner.rs` 작성
- [x] `Cargo.toml` 업데이트
- [x] 로컬 빌드 및 실행 테스트
- [x] Advisory Lock 구현

#### Phase 6: 통합 테스트 ✅ **완료**
- [x] E2E 테스트 작성 (14개 시나리오)
- [x] Dry-run 모드 검증
- [x] Job A/B 독립성 테스트
- [x] 락 자동 해제 테스트

#### Phase 7: 배포 ⏳ **진행 예정**
- [ ] `Dockerfile` 수정 (멀티 바이너리)
- [ ] Kubernetes CronJob YAML 작성
- [ ] ConfigMap/Secret 생성
- [ ] Dry-run 1주일 검증
- [ ] 프로덕션 배포

---

### 🚀 배포 준비

**다음 작업**:

1. **Dockerfile 수정** - 멀티 바이너리 빌드
2. **Kubernetes CronJob 작성** - Job A/B 스케줄링
3. **Dry-run 검증** - 1주일 모니터링
4. **프로덕션 배포** - 실제 환경 적용

**참고 문서**:
- **[04-배포-및-운영-가이드.md](./04-배포-및-운영-가이드.md)** - Kubernetes 배포
- **[09-상세-구현-가이드.md](./09-상세-구현-가이드.md)** - 배포 섹션

