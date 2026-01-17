# AI Assistant를 위한 컨텍스트

> 이 문서는 다음 세션의 AI Assistant가 이 이슈를 빠르게 이해할 수 있도록 작성되었습니다.

---

## 🎯 이슈 요약 (30초 버전)

**문제**: Annotation Entity에 snapshot 필드 3개를 추가했으나, Repository의 SELECT 쿼리를 업데이트하지 않아 모든 Annotation 조회/삭제 API가 실패했습니다.

**해결**: `annotation_repository_impl.rs`의 15개 메서드, 20개 쿼리에 snapshot 필드 3개를 추가했습니다.

**상태**: ✅ 수정 완료, 테스트 통과, 문서화 완료

---

## 📁 프로젝트 구조

```
pacs-ext-server/
├── pacs-server/
│   ├── src/
│   │   ├── domain/
│   │   │   ├── entities/
│   │   │   │   └── annotation.rs          # Annotation Entity (snapshot 필드 추가됨)
│   │   │   └── repositories/
│   │   │       └── annotation_repository.rs
│   │   └── infrastructure/
│   │       └── repositories/
│   │           └── annotation_repository_impl.rs  # ✅ 수정된 파일
│   ├── migrations/
│   │   ├── 036_add_snapshot_image_to_annotations.sql  # Snapshot 컬럼 추가
│   │   └── 037_fix_snapshot_uploaded_at_type.sql      # 타입 수정
│   └── e2e/
│       └── test_annotation_snapshot_e2e_refactored.py  # E2E 테스트
└── issues/
    └── 2026-01-14-annotation-조회-삭제-오류-수정/  # 이 폴더
        ├── INDEX.md                    # 📚 문서 인덱스
        ├── SUMMARY.md                  # 📌 요약
        ├── README.md                   # 📋 전체 설명
        ├── API-구현-안내.md            # 🔌 API 사용법
        ├── CONTEXT-FOR-AI.md           # 🤖 이 문서
        ├── 수정-상세-내역.md           # 📝 수정 내역
        ├── 기술-분석.md                # 🔍 기술 분석
        ├── 체크리스트.md               # ✅ 체크리스트
        └── 다이어그램.md               # 📊 다이어그램
```

---

## 🔑 핵심 파일

### 1. Annotation Entity
**위치**: `pacs-server/src/domain/entities/annotation.rs`

**추가된 필드**:
```rust
pub struct Annotation {
    // ... 기존 필드들
    pub snapshot_image_key: Option<String>,
    pub snapshot_status: Option<SnapshotUploadStatus>,
    pub snapshot_uploaded_at: Option<DateTime<Utc>>,
    // ... 나머지 필드들
}
```

### 2. Annotation Repository (수정됨)
**위치**: `pacs-server/src/infrastructure/repositories/annotation_repository_impl.rs`

**수정된 메서드**: 15개
- 모든 SELECT 쿼리에 snapshot 필드 3개 추가
- RETURNING 절에도 snapshot 필드 3개 추가

**패턴**:
```rust
// Before (❌ 오류)
"SELECT id, project_id, ..., is_shared, created_at, updated_at, ..."

// After (✅ 정상)
"SELECT id, project_id, ..., is_shared,
        snapshot_image_key, snapshot_status, snapshot_uploaded_at,
        created_at, updated_at, ..."
```

### 3. DB Migrations
**위치**: `pacs-server/migrations/`

- `036_add_snapshot_image_to_annotations.sql`: snapshot 컬럼 추가
- `037_fix_snapshot_uploaded_at_type.sql`: TIMESTAMP → TIMESTAMP WITH TIME ZONE

---

## 🔍 문제 발생 원인

### 1. sqlx의 타입 매핑
- 프로젝트는 `query_as::<_, T>` 사용 (런타임 체크)
- Entity 필드와 SELECT 쿼리 필드가 정확히 일치해야 함
- 불일치 시 `MissingColumn` 에러 발생

### 2. 에러 흐름
```
Entity: 20개 필드 (snapshot 3개 포함)
    ↓
SELECT: 17개 컬럼 (snapshot 누락)
    ↓
sqlx FromRow 매핑 시도
    ↓
MissingColumn("snapshot_image_key") 에러
    ↓
HTTP 500 Internal Server Error
```

---

## ✅ 수정 내용

### 수정된 메서드 (15개)

1. `find_by_id()` - ID로 조회
2. `find_by_project_id()` - 프로젝트 ID로 조회
3. `find_by_user_id()` - 사용자 ID로 조회
4. `find_by_study_uid()` - Study UID로 조회
5. `find_by_series_uid()` - Series UID로 조회
6. `find_by_instance_uid()` - Instance UID로 조회
7. `find_by_project_and_study()` - 프로젝트 + Study로 조회
8. `find_by_project_and_series()` - 프로젝트 + Series로 조회
9. `find_by_project_and_instance()` - 프로젝트 + Instance로 조회
10. `create()` - 생성 (RETURNING 절)
11. `update()` - 업데이트 (기존 데이터 조회 + RETURNING 절)
12. `update_with_version_check()` - 버전 체크 업데이트 (기존 데이터 조회 + RETURNING 절)
13. `update_snapshot_info()` - Snapshot 정보 업데이트 (RETURNING 절)
14. `delete()` - 삭제 (기존 데이터 조회)
15. `find_by_project_and_series_paginated()` - 페이지네이션 조회

### 수정 통계
- **수정된 파일**: 1개
- **수정된 메서드**: 15개
- **수정된 쿼리**: 20개
- **추가된 필드**: 3개

---

## 🔌 Annotation Snapshot API

### 엔드포인트

1. **업로드**: `POST /api/v1/annotations/{id}/snapshot`
   - Content-Type: `multipart/form-data`
   - 최대 크기: 10MB
   - 지원 형식: PNG, JPEG, WebP

2. **조회**: `GET /api/v1/annotations/{id}/snapshot`
   - 응답: 이미지 바이너리 데이터

3. **삭제**: `DELETE /api/v1/annotations/{id}/snapshot`
   - 응답: 204 No Content

### Snapshot 상태

```rust
pub enum SnapshotUploadStatus {
    Pending,    // 업로드 대기 중
    Uploaded,   // 업로드 완료
    Failed,     // 업로드 실패
}
```

---

## 🧪 테스트

### E2E 테스트 (모두 통과 ✅)
```bash
cd pacs-server/e2e
python test_annotation_level_filtering_refactored.py
python test_annotation_version_conflict_refactored.py
python test_annotation_head_request_refactored.py
python test_annotation_snapshot_e2e_refactored.py
python test_annotation_permission_filtering_refactored.py
python test_annotation_permission_management_refactored.py
```

---

## 📚 문서 읽기 가이드

### 빠른 이해 (15분)
1. `SUMMARY.md` - 요약 (5분)
2. `다이어그램.md` - 시각적 이해 (10분)

### 상세 이해 (45분)
1. `README.md` - 전체 개요 (10분)
2. `기술-분석.md` - 기술적 분석 (20분)
3. `수정-상세-내역.md` - 수정 내역 (15분)

### API 사용 (10분)
1. `API-구현-안내.md` - API 사용법

### Entity 변경 예정 (15분)
1. `체크리스트.md` - ⭐ 필수 읽기!

---

## 🎯 핵심 교훈

### Entity 변경 시 체크리스트

```
✅ 1. DB Migration 작성
✅ 2. Entity 구조체 수정
✅ 3. Repository의 모든 SELECT 쿼리 업데이트 ⭐ 가장 중요!
✅ 4. RETURNING 절 업데이트
✅ 5. Service/DTO 업데이트
✅ 6. API 응답 확인
✅ 7. 테스트 실행
✅ 8. 문서화
```

**가장 중요한 것**: Repository의 **모든** SELECT 쿼리를 빠짐없이 업데이트!

---

## 🚀 다음 작업

### 완료된 작업
- [x] 코드 수정
- [x] 로컬 테스트
- [x] E2E 테스트
- [x] 문서화

### 남은 작업
- [ ] 스테이징 배포
- [ ] 프로덕션 배포
- [ ] 모니터링 확인

---

## 💡 AI Assistant를 위한 팁

### 사용자가 다음과 같이 요청할 경우

1. **"Annotation API가 뭐야?"**
   → `API-구현-안내.md` 참고

2. **"Entity 변경하려는데 뭘 확인해야 해?"**
   → `체크리스트.md` 참고 (필수!)

3. **"이 이슈가 뭐였어?"**
   → `SUMMARY.md` 참고

4. **"왜 이런 문제가 발생했어?"**
   → `기술-분석.md` 참고

5. **"정확히 뭘 수정했어?"**
   → `수정-상세-내역.md` 참고

6. **"다이어그램으로 보여줘"**
   → `다이어그램.md` 참고

### 코드 위치

- **Entity**: `pacs-server/src/domain/entities/annotation.rs`
- **Repository**: `pacs-server/src/infrastructure/repositories/annotation_repository_impl.rs`
- **Migration**: `pacs-server/migrations/036_*.sql`, `037_*.sql`
- **E2E 테스트**: `pacs-server/e2e/test_annotation_snapshot_e2e_refactored.py`

### 검색 키워드

- `snapshot_image_key`
- `snapshot_status`
- `snapshot_uploaded_at`
- `SnapshotUploadStatus`
- `update_snapshot_info`

---

## 🔗 관련 링크

- [sqlx Documentation](https://docs.rs/sqlx/)
- [FromRow Derive Macro](https://docs.rs/sqlx/latest/sqlx/trait.FromRow.html)
- [PostgreSQL SELECT](https://www.postgresql.org/docs/current/sql-select.html)

---

## 📝 메타 정보

- **작성일**: 2026-01-14
- **작성자**: aido
- **상태**: ✅ 완료
- **우선순위**: 🔴 High (기능 동작 불가)
- **영향 범위**: Annotation CRUD 전체

