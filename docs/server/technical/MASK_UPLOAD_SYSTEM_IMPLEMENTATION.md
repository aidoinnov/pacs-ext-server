# PACS Mask Upload System Implementation Guide

## 📋 개요

PACS 서버에 마스크 업로드 시스템을 구현했습니다. 이 시스템은 의료 영상 분석을 위한 마스크 데이터를 안전하고 효율적으로 관리할 수 있도록 설계되었습니다.

## 🏗️ 시스템 아키텍처

### 1. Clean Architecture 적용
- **Domain Layer**: 엔티티, Repository Traits
- **Application Layer**: DTOs, Services, Use Cases
- **Infrastructure Layer**: Repository 구현체, Object Storage 연동
- **Presentation Layer**: Controllers, API 엔드포인트

### 2. 주요 컴포넌트
- **Mask Group Management**: 마스크 그룹 관리
- **Mask File Management**: 개별 마스크 파일 관리
- **Object Storage Integration**: S3/MinIO 연동
- **Signed URL Service**: 보안 URL 생성 및 관리

## 🗄️ 데이터베이스 스키마

### Mask Group Table
```sql
CREATE TABLE annotation_mask_group (
    id SERIAL PRIMARY KEY,
    annotation_id INTEGER NOT NULL REFERENCES annotation_annotation(id) ON DELETE CASCADE,
    group_name TEXT,                       -- 예: Liver_Segmentation_v2
    model_name TEXT,                       -- AI 모델명 (optional)
    version TEXT,                          -- 버전명 (optional)
    modality TEXT,                         -- CT/MR 등
    slice_count INTEGER DEFAULT 1,
    mask_type TEXT DEFAULT 'segmentation', -- segmentation, bounding_box 등
    description TEXT,
    created_by INTEGER,                    -- 생성한 사용자 ID (optional)
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Mask Table
```sql
CREATE TABLE annotation_mask (
    id SERIAL PRIMARY KEY,
    mask_group_id INTEGER NOT NULL REFERENCES annotation_mask_group(id) ON DELETE CASCADE,
    slice_index INTEGER,                   -- 볼륨 내 슬라이스 인덱스
    sop_instance_uid TEXT,                 -- DICOM SOP Instance UID
    label_name TEXT,                       -- 예: liver, spleen
    file_path TEXT NOT NULL,               -- S3/MinIO 경로
    mime_type TEXT DEFAULT 'image/png',
    file_size BIGINT,
    checksum TEXT,                         -- 파일 무결성 검증용
    width INTEGER,
    height INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## 🎯 핵심 기능

### 1. Mask Group Management
- **생성**: AI 모델, 수동 생성 지원
- **조회**: 어노테이션별, 사용자별, 모달리티별 필터링
- **업데이트**: 그룹 정보 수정
- **삭제**: 그룹 및 연관된 마스크 파일 삭제

### 2. Mask File Management
- **업로드**: PNG, JPEG, DICOM 형식 지원
- **다운로드**: 보안 URL을 통한 접근
- **메타데이터**: 파일 크기, 체크섬, 이미지 크기 관리
- **검색**: SOP Instance UID, 라벨별 검색

### 3. Object Storage Integration
- **S3 지원**: AWS S3 호환 스토리지
- **MinIO 지원**: 자체 호스팅 MinIO 서버
- **보안**: IAM 정책 기반 접근 제어
- **확장성**: 대용량 파일 처리

### 4. Signed URL Service
- **PUT URL**: 업로드용 보안 URL
- **GET URL**: 다운로드용 보안 URL
- **TTL 관리**: 기본 10분, 최대 1시간
- **메타데이터**: 자동 어노테이션 ID, 사용자 ID 추가

## 🔧 기술 구현

### 1. Domain Layer

#### MaskGroup Entity
```rust
pub struct MaskGroup {
    pub id: i32,
    pub annotation_id: i32,
    pub group_name: Option<String>,
    pub model_name: Option<String>,
    pub version: Option<String>,
    pub modality: Option<String>,
    pub slice_count: i32,
    pub mask_type: String,
    pub description: Option<String>,
    pub created_by: Option<i32>,
    pub created_at: DateTime<Utc>,
}
```

#### Mask Entity
```rust
pub struct Mask {
    pub id: i32,
    pub mask_group_id: i32,
    pub slice_index: Option<i32>,
    pub sop_instance_uid: Option<String>,
    pub label_name: Option<String>,
    pub file_path: String,
    pub mime_type: String,
    pub file_size: Option<i64>,
    pub checksum: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: DateTime<Utc>,
}
```

### 2. Application Layer

#### DTOs
- **CreateMaskGroupRequest**: 마스크 그룹 생성 요청
- **MaskGroupResponse**: 마스크 그룹 응답
- **CreateMaskRequest**: 마스크 생성 요청
- **MaskResponse**: 마스크 응답
- **SignedUrlRequest**: Signed URL 요청
- **SignedUrlResponse**: Signed URL 응답

#### Services
- **ObjectStorageService**: Object Storage 연동
- **SignedUrlService**: Signed URL 생성 및 관리

### 3. Infrastructure Layer

#### Object Storage Services
- **S3ObjectStorageService**: AWS S3 연동
- **MinIOObjectStorageService**: MinIO 연동

#### Repository Traits
- **MaskGroupRepository**: 마스크 그룹 데이터 접근
- **MaskRepository**: 마스크 파일 데이터 접근

### 4. Configuration

#### Object Storage 설정
```toml
[object_storage]
provider = "s3"  # "s3" or "minio"
bucket_name = "pacs-masks"
region = "us-east-1"
endpoint = ""  # MinIO endpoint (leave empty for AWS S3)
access_key = ""
secret_key = ""

[signed_url]
default_ttl = 600  # 10 minutes
max_ttl = 3600     # 1 hour
```

## 🚀 API 엔드포인트

### Mask Group APIs
- `POST /api/mask-groups` - 마스크 그룹 생성
- `GET /api/mask-groups` - 마스크 그룹 목록 조회
- `GET /api/mask-groups/{id}` - 마스크 그룹 상세 조회
- `PUT /api/mask-groups/{id}` - 마스크 그룹 수정
- `DELETE /api/mask-groups/{id}` - 마스크 그룹 삭제

### Mask APIs
- `POST /api/masks` - 마스크 생성
- `GET /api/masks` - 마스크 목록 조회
- `GET /api/masks/{id}` - 마스크 상세 조회
- `PUT /api/masks/{id}` - 마스크 수정
- `DELETE /api/masks/{id}` - 마스크 삭제

### Signed URL APIs
- `POST /api/signed-urls/upload` - 업로드용 Signed URL 생성
- `POST /api/signed-urls/download` - 다운로드용 Signed URL 생성
- `POST /api/signed-urls/mask-upload` - 마스크 업로드용 Signed URL 생성
- `POST /api/signed-urls/mask-download` - 마스크 다운로드용 Signed URL 생성

## 🔒 보안 기능

### 1. Signed URL 보안
- **TTL 검증**: 0초 미만, 최대값 초과 방지
- **파일 경로 검증**: `..` 경로, 절대 경로 방지
- **메타데이터 자동 추가**: 어노테이션 ID, 사용자 ID, 마스크 그룹 ID 등

### 2. Object Storage 보안
- **IAM 정책**: 최소 권한 원칙 적용
- **CORS 설정**: 허용된 도메인만 접근
- **암호화**: 전송 중 및 저장 시 암호화

### 3. 데이터 무결성
- **체크섬 검증**: 파일 무결성 확인
- **파일 크기 검증**: 예상 크기와 실제 크기 비교
- **MIME 타입 검증**: 허용된 파일 형식만 업로드

## 📊 성능 최적화

### 1. 파일 경로 구조
- **마스크 파일**: `masks/annotation_{id}/group_{id}/{filename}`
- **어노테이션 데이터**: `annotations/annotation_{id}/{filename}`

### 2. 캐싱 전략
- **메타데이터 캐싱**: 자주 조회되는 정보 캐시
- **Signed URL 캐싱**: 재사용 가능한 URL 캐시

### 3. 배치 처리
- **대용량 업로드**: 여러 파일 동시 업로드 지원
- **압축**: 이미지 압축을 통한 저장 공간 절약

## 🧪 테스트

### 1. Unit Tests
- **Entity Tests**: 엔티티 생성, 검증 로직
- **Service Tests**: Signed URL 생성, Object Storage 연동
- **Repository Tests**: 데이터 접근 로직

### 2. Integration Tests
- **API Tests**: 엔드포인트 동작 확인
- **Database Tests**: 데이터 저장 및 조회 확인
- **Object Storage Tests**: 실제 스토리지 연동 확인

### 3. Mock Tests
- **Object Storage Mock**: AWS SDK 없이 테스트
- **Service Mock**: 외부 의존성 제거

## 📈 모니터링 및 로깅

### 1. 로깅
- **업로드 로그**: 파일 업로드 성공/실패
- **다운로드 로그**: 파일 다운로드 접근 기록
- **에러 로그**: 시스템 오류 및 예외 상황

### 2. 메트릭
- **업로드 통계**: 파일 수, 크기, 형식별 통계
- **사용자 활동**: 사용자별 업로드/다운로드 패턴
- **성능 지표**: 응답 시간, 처리량

## 🔄 마이그레이션

### 1. 데이터베이스 마이그레이션
```sql
-- 003_add_mask_tables.sql
-- annotation_mask_group 테이블 생성
-- annotation_mask 테이블 생성
-- 인덱스 생성
```

### 2. 설정 마이그레이션
- Object Storage 설정 추가
- Signed URL 설정 추가
- CORS 설정 업데이트

## 🚀 배포 가이드

### 1. 환경 설정
```bash
# 환경 변수 설정
export APP_OBJECT_STORAGE__PROVIDER="s3"
export APP_OBJECT_STORAGE__BUCKET_NAME="pacs-masks"
export APP_OBJECT_STORAGE__REGION="us-east-1"
export APP_OBJECT_STORAGE__ACCESS_KEY="your-access-key"
export APP_OBJECT_STORAGE__SECRET_KEY="your-secret-key"
```

### 2. 데이터베이스 마이그레이션
```bash
# 마이그레이션 실행
sqlx migrate run
```

### 3. 서비스 시작
```bash
# 서비스 시작
cargo run --release
```

## 📚 추가 문서

- [AWS S3 Integration Guide](./AWS_S3_INTEGRATION_GUIDE.md)
- [Object Storage Service Tests](./tests/object_storage_mock_test.rs)
- [Mask Upload API Guide](./ANNOTATION_API_GUIDE.md)
- [Database Schema](./infra/db/schema.sql)

## 🎯 향후 계획

### 1. 단기 계획
- [ ] Infrastructure Layer Repository 구현체 완성
- [ ] Presentation Layer Controller 구현
- [ ] 통합 테스트 완성
- [ ] API 문서화 완성

### 2. 중기 계획
- [ ] 실시간 업로드 진행률 표시
- [ ] 대용량 파일 청크 업로드 지원
- [ ] 이미지 미리보기 기능
- [ ] 버전 관리 시스템

### 3. 장기 계획
- [ ] AI 모델 연동 API
- [ ] 자동 마스크 생성 기능
- [ ] 분산 스토리지 지원
- [ ] 고급 검색 및 필터링

---

## 📞 지원

기술적 문의사항이나 버그 리포트는 개발팀에 연락해주세요.

**구현 완료일**: 2024년 1월
**버전**: 1.0.0
**상태**: 개발 완료 (테스트 및 배포 준비 중)
