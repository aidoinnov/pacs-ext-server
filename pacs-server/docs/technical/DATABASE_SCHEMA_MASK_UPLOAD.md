# 🗄️ PACS 마스크 업로드 데이터베이스 스키마

## 📋 개요
PACS 마스크 업로드 시스템을 위한 데이터베이스 스키마 설계 및 구현 문서입니다.

## 🏗️ 테이블 구조

### 1. annotation_mask_group 테이블
마스크 그룹 정보를 저장하는 메인 테이블입니다.

```sql
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
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### 필드 설명
- `id`: 마스크 그룹 고유 식별자 (Primary Key)
- `annotation_id`: 연결된 어노테이션 ID (Foreign Key)
- `group_name`: 마스크 그룹 이름
- `model_name`: AI 모델 이름
- `version`: 모델 버전
- `modality`: 의료 영상 모달리티 (CT, MRI 등)
- `slice_count`: 슬라이스 개수 (기본값: 1)
- `mask_type`: 마스크 타입 (기본값: 'segmentation')
- `description`: 그룹 설명
- `created_by`: 생성자 ID
- `created_at`: 생성 시간
- `updated_at`: 수정 시간

### 2. annotation_mask 테이블
개별 마스크 파일 정보를 저장하는 테이블입니다.

```sql
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
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### 필드 설명
- `id`: 마스크 고유 식별자 (Primary Key)
- `mask_group_id`: 연결된 마스크 그룹 ID (Foreign Key)
- `slice_index`: 슬라이스 인덱스
- `sop_instance_uid`: DICOM SOP Instance UID
- `label_name`: 라벨 이름
- `file_path`: Object Storage 파일 경로
- `mime_type`: 파일 MIME 타입 (기본값: 'image/png')
- `file_size`: 파일 크기 (바이트)
- `checksum`: 파일 체크섬
- `width`: 이미지 너비
- `height`: 이미지 높이
- `created_at`: 생성 시간
- `updated_at`: 수정 시간

## 🔍 인덱스 설계

### 성능 최적화를 위한 인덱스
```sql
-- 마스크 그룹 조회 최적화
CREATE INDEX idx_mask_group_annotation_id ON annotation_mask_group(annotation_id);
CREATE INDEX idx_mask_group_created_by ON annotation_mask_group(created_by);
CREATE INDEX idx_mask_group_modality ON annotation_mask_group(modality);
CREATE INDEX idx_mask_group_mask_type ON annotation_mask_group(mask_type);

-- 마스크 조회 최적화
CREATE INDEX idx_mask_mask_group_id ON annotation_mask(mask_group_id);
CREATE INDEX idx_mask_sop_instance_uid ON annotation_mask(sop_instance_uid);
CREATE INDEX idx_mask_label_name ON annotation_mask(label_name);
CREATE INDEX idx_mask_mime_type ON annotation_mask(mime_type);
```

## 🔗 관계 설계

### 1. 어노테이션과 마스크 그룹 관계
- **1:N 관계**: 하나의 어노테이션은 여러 마스크 그룹을 가질 수 있음
- **CASCADE 삭제**: 어노테이션이 삭제되면 관련 마스크 그룹도 자동 삭제

### 2. 마스크 그룹과 마스크 관계
- **1:N 관계**: 하나의 마스크 그룹은 여러 마스크를 가질 수 있음
- **CASCADE 삭제**: 마스크 그룹이 삭제되면 관련 마스크도 자동 삭제

## 📊 데이터 타입 고려사항

### 1. BIGINT 사용
- `file_size`: 대용량 파일 지원을 위해 BIGINT 사용
- 최대 9,223,372,036,854,775,807 바이트 (약 8EB) 지원

### 2. TEXT vs VARCHAR
- `group_name`, `model_name`, `version` 등: 가변 길이 문자열
- `description`: 긴 설명 텍스트 지원
- `file_path`: 긴 파일 경로 지원

### 3. NULL 허용 정책
- 필수 필드: `annotation_id`, `mask_group_id`, `file_path`
- 선택 필드: `group_name`, `model_name`, `version`, `modality` 등
- 메타데이터: `slice_count`, `mask_type`, `mime_type` (기본값 제공)

## 🚀 마이그레이션 이력

### Migration 003: Add mask tables
```sql
-- 파일: migrations/003_add_mask_tables.sql
-- 생성일: 2025-10-07
-- 설명: 마스크 업로드 기능을 위한 테이블 생성
```

### Migration 004: Add updated_at columns
```sql
-- 파일: migrations/004_add_updated_at_columns.sql
-- 생성일: 2025-10-07
-- 설명: updated_at 컬럼 추가
```

## 🔧 Rust 엔티티 매핑

### MaskGroup 엔티티
```rust
pub struct MaskGroup {
    pub id: i32,
    pub annotation_id: i32,
    pub group_name: Option<String>,
    pub model_name: Option<String>,
    pub version: Option<String>,
    pub modality: Option<String>,
    pub slice_count: Option<i32>,
    pub mask_type: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Mask 엔티티
```rust
pub struct Mask {
    pub id: i32,
    pub mask_group_id: i32,
    pub slice_index: Option<i32>,
    pub sop_instance_uid: Option<String>,
    pub label_name: Option<String>,
    pub file_path: String,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub checksum: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## 📈 성능 고려사항

### 1. 쿼리 최적화
- 인덱스를 활용한 빠른 조회
- 적절한 WHERE 절 사용
- LIMIT/OFFSET을 통한 페이징

### 2. 저장 공간 최적화
- TEXT 타입의 적절한 사용
- 인덱스 크기 최적화
- 파티셔닝 고려 (향후 대용량 데이터)

### 3. 동시성 처리
- 트랜잭션 격리 수준 고려
- 락 경합 최소화
- 배치 처리 최적화

## 🔒 보안 고려사항

### 1. 데이터 무결성
- Foreign Key 제약조건
- CASCADE 삭제 정책
- NOT NULL 제약조건

### 2. 접근 제어
- 사용자별 데이터 격리
- 어노테이션 기반 권한 관리
- 감사 로그 고려

## 📚 참고 자료
- [PostgreSQL 데이터 타입 문서](https://www.postgresql.org/docs/current/datatype.html)
- [SQLx 마이그레이션 가이드](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)
- [Rust SQLx 문서](https://docs.rs/sqlx/latest/sqlx/)

---
**작성일**: 2025-10-07  
**작성자**: AI Assistant  
**버전**: 1.0
