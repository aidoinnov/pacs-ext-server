# 🚀 PACS 마스크 업로드 v2 구현 계획서

## 📋 개요
이 문서는 `design.md`와 `worklist.md`를 바탕으로 실제 구현을 위한 구체적인 작업 계획을 제시합니다.

## 🎯 현재 상황 분석
- ✅ Annotation 시스템이 완성되어 있음
- ✅ PostgreSQL 데이터베이스 설정 완료
- ✅ Rust + Actix Web 기반 API 서버 구축
- ✅ Swagger/OpenAPI 문서화 완료
- ✅ Object Storage (S3/MinIO) 연동 구현 완료
- ✅ 마스크 업로드 관련 테이블 생성 완료
- ✅ Repository 구현체 완료
- ✅ DTO 설계 및 구현 완료
- ❌ Use Case 및 Service 레이어 미구현
- ❌ API 엔드포인트 미구현

## 🏗️ 구현 단계별 계획

### Phase 1: 데이터베이스 스키마 구현 (1-2일) ✅ **완료**
**목표**: 마스크 관련 테이블 생성 및 마이그레이션

#### 1.1 데이터베이스 마이그레이션 스크립트 작성
```sql
-- annotation_mask_group 테이블 생성
CREATE TABLE annotation_mask_group (
    id SERIAL PRIMARY KEY,
    annotation_id INTEGER NOT NULL REFERENCES annotation_annotation(id) ON DELETE CASCADE,
    group_name TEXT,
    model_name TEXT,
    version TEXT,
    modality TEXT,
    slice_count INTEGER DEFAULT 1,
    mask_type TEXT DEFAULT 'segmentation',
    description TEXT,
    created_by INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- annotation_mask 테이블 생성
CREATE TABLE annotation_mask (
    id SERIAL PRIMARY KEY,
    mask_group_id INTEGER NOT NULL REFERENCES annotation_mask_group(id) ON DELETE CASCADE,
    slice_index INTEGER,
    sop_instance_uid TEXT,
    label_name TEXT,
    file_path TEXT NOT NULL,
    mime_type TEXT DEFAULT 'image/png',
    file_size BIGINT,
    checksum TEXT,
    width INTEGER,
    height INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 인덱스 생성

CREATE INDEX idx_mask_group_annotation_id ON annotation_mask_group(annotation_id);
CREATE INDEX idx_mask_mask_group_id ON annotation_mask(mask_group_id);
```

#### 1.2 Rust 엔티티 및 DTO 생성
- `MaskGroup` 엔티티
- `Mask` 엔티티
- `CreateMaskGroupRequest` DTO
- `MaskGroupResponse` DTO
- `SignedUrlRequest` DTO
- `SignedUrlResponse` DTO

### Phase 2: Object Storage 연동 (2-3일) ✅ **완료**
**목표**: S3/MinIO 연동 및 Signed URL 발급

#### 2.1 의존성 추가
```toml
# Cargo.toml
[dependencies]
aws-sdk-s3 = "1.0"
tokio-util = { version = "0.7", features = ["codec"] }
```

#### 2.2 Object Storage 서비스 구현
- `ObjectStorageService` trait 정의
- `S3ObjectStorageService` 구현
- `MinIOObjectStorageService` 구현 (로컬 개발용)

#### 2.3 Signed URL 발급 로직
- PUT URL 생성 (업로드용)
- GET URL 생성 (다운로드용)
- TTL 설정 (기본 10분, 최대 1시간)

### Phase 3: API 엔드포인트 구현 (2-3일)
**목표**: 마스크 관련 REST API 완성

#### 3.1 마스크 그룹 관리 API
- `POST /api/annotations/{annotation_id}/mask-groups` - 그룹 생성
- `GET /api/annotations/{annotation_id}/mask-groups` - 그룹 목록 조회
- `GET /api/annotations/{annotation_id}/mask-groups/{group_id}` - 그룹 상세 조회
- `DELETE /api/annotations/{annotation_id}/mask-groups/{group_id}` - 그룹 삭제

#### 3.2 마스크 업로드 API
- `POST /api/annotations/{annotation_id}/mask-groups/{group_id}/signed-url` - Signed URL 발급
- `POST /api/annotations/{annotation_id}/mask-groups/{group_id}/complete` - 업로드 완료 처리

#### 3.3 마스크 조회 API
- `GET /api/annotations/{annotation_id}/mask-groups/{group_id}/masks` - 마스크 목록 조회
- `GET /api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}` - 마스크 상세 조회

### Phase 4: 서비스 레이어 구현 (1-2일) 🚧 **진행 중**
**목표**: 비즈니스 로직 및 유스케이스 구현

#### 4.1 Repository 구현 ✅ **완료**
- `MaskGroupRepository` trait ✅
- `MaskRepository` trait ✅
- PostgreSQL 구현체 ✅

#### 4.2 Use Case 구현
- `CreateMaskGroupUseCase`
- `GenerateSignedUrlUseCase`
- `CompleteUploadUseCase`
- `ListMaskGroupsUseCase`
- `DeleteMaskGroupUseCase`

#### 4.3 서비스 구현
- `MaskGroupService`
- `MaskService`

### Phase 5: 컨트롤러 구현 (1일)
**목표**: HTTP 요청/응답 처리

#### 5.1 Mask Group Controller
- 모든 마스크 그룹 관련 엔드포인트
- Swagger 문서화
- 에러 핸들링

#### 5.2 Mask Controller
- 마스크 조회 관련 엔드포인트
- 파일 메타데이터 처리

### Phase 6: 테스트 구현 (2-3일)
**목표**: 단위 테스트 및 통합 테스트

#### 6.1 단위 테스트
- Repository 테스트
- Service 테스트
- Use Case 테스트

#### 6.2 통합 테스트
- API 엔드포인트 테스트
- Object Storage 연동 테스트
- 전체 플로우 테스트

#### 6.3 성능 테스트
- 병렬 업로드 테스트
- 대용량 파일 업로드 테스트

### Phase 7: 설정 및 배포 (1일)
**목표**: 환경 설정 및 배포 준비

#### 7.1 설정 파일 업데이트
```toml
# config/default.toml
[object_storage]
provider = "s3" # or "minio"
bucket_name = "pacs-masks"
region = "us-east-1"
endpoint = "" # MinIO용
access_key = ""
secret_key = ""

[signed_url]
default_ttl = 600 # 10분
max_ttl = 3600 # 1시간
```

#### 7.2 환경 변수 설정
- AWS_ACCESS_KEY_ID
- AWS_SECRET_ACCESS_KEY
- S3_BUCKET_NAME
- S3_REGION

## 🔧 기술적 고려사항

### 1. 에러 처리
- Object Storage 연결 실패
- Signed URL 만료
- 파일 업로드 실패
- DB 트랜잭션 실패

### 2. 보안
- IAM 정책으로 prefix 제한
- HTTPS 통신 강제
- 파일명 검증 (개인정보 포함 금지)
- CORS 설정

### 3. 성능
- 병렬 업로드 지원
- 메모리 효율적인 파일 처리
- DB 인덱스 최적화

### 4. 모니터링
- 업로드 성공/실패 로그
- 저장소 사용량 모니터링
- API 응답 시간 측정

## 📅 예상 일정

| Phase | 작업 | 예상 기간 | 담당 |
|-------|------|-----------|------|
| 1 | DB 스키마 구현 | 1-2일 | Backend |
| 2 | Object Storage 연동 | 2-3일 | Backend |
| 3 | API 엔드포인트 구현 | 2-3일 | Backend |
| 4 | 서비스 레이어 구현 | 1-2일 | Backend |
| 5 | 컨트롤러 구현 | 1일 | Backend |
| 6 | 테스트 구현 | 2-3일 | Backend/QA |
| 7 | 설정 및 배포 | 1일 | DevOps |

**총 예상 기간**: 10-15일

## 🎯 성공 기준

### 기능적 요구사항
- [ ] 마스크 그룹 생성/조회/삭제 가능
- [ ] Signed URL을 통한 직접 업로드 가능
- [ ] 업로드 완료 후 메타데이터 저장
- [ ] 마스크 목록 조회 가능

### 비기능적 요구사항
- [ ] 100개 slice 병렬 업로드 1분 이내
- [ ] API 응답 시간 200ms 이하
- [ ] 99.9% 가용성
- [ ] 1TB 이상 저장 용량 지원

## 🚨 위험 요소 및 대응 방안

### 1. Object Storage 비용
**위험**: 대용량 파일 저장으로 인한 비용 증가
**대응**: Lifecycle Rule 설정, 압축 옵션 제공

### 2. 네트워크 대역폭
**위험**: 대용량 업로드 시 네트워크 병목
**대응**: 청크 업로드, 병렬 처리

### 3. DB 성능
**위험**: 대량 메타데이터로 인한 DB 부하
**대응**: 인덱스 최적화, 파티셔닝 고려

### 4. 보안 취약점
**위험**: Signed URL 악용, 파일 접근 권한 오류
**대응**: TTL 단축, IAM 정책 강화, 감사 로그

## 📚 참고 자료
- [AWS S3 Signed URL 가이드](https://docs.aws.amazon.com/AmazonS3/latest/userguide/PresignedUrlUploadObject.html)
- [MinIO Go SDK 문서](https://docs.min.io/docs/golang-client-quickstart-guide.html)
- [Actix Web 파일 업로드](https://actix.rs/docs/extractors/#multipart)
- [SQLx 마이그레이션](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)

---

**작성일**: 2025-10-07  
**작성자**: AI Assistant  
**버전**: 1.0
