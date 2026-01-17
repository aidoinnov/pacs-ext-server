# 요약 (Executive Summary)

## 📌 한 줄 요약
Annotation Snapshot API 구현 후 Entity에 필드를 추가했으나, Repository의 SELECT 쿼리를 업데이트하지 않아 모든 Annotation 조회/삭제 기능이 실패하는 문제를 수정했습니다.

---

## 🔴 문제

### 증상
- ❌ Annotation 조회 실패 (HTTP 500)
- ❌ Annotation 삭제 실패 (HTTP 500)
- ❌ 모든 Annotation 관련 API 오류

### 원인
1. **DB Migration**: `annotation_annotation` 테이블에 snapshot 관련 컬럼 3개 추가
2. **Entity 업데이트**: `Annotation` 구조체에 snapshot 필드 3개 추가
3. **Repository 미업데이트**: SELECT 쿼리에 새 필드 누락 ⚠️

### 결과
```
Entity: 20개 필드
DB: 20개 컬럼
SELECT 쿼리: 17개 컬럼 ❌

→ sqlx FromRow 매핑 실패
→ MissingColumn("snapshot_image_key") 에러
→ HTTP 500 Internal Server Error
```

---

## ✅ 해결

### 수정 내용
**모든 SELECT 쿼리에 snapshot 필드 3개 추가**:
- `snapshot_image_key`
- `snapshot_status`
- `snapshot_uploaded_at`

### 수정 범위
- **파일**: `pacs-server/src/infrastructure/repositories/annotation_repository_impl.rs`
- **메서드**: 15개
- **쿼리**: 20개 (일부 메서드는 2개 이상의 쿼리 포함)

### 수정 패턴
```diff
  SELECT id, project_id, user_id, study_uid, series_uid, instance_uid,
-        tool_name, tool_version, data, is_shared, created_at, updated_at,
-        viewer_software, description, measurement_values, label, version
+        tool_name, tool_version, data, is_shared,
+        snapshot_image_key, snapshot_status, snapshot_uploaded_at,
+        created_at, updated_at, version,
+        viewer_software, description, measurement_values, label
  FROM annotation_annotation
```

---

## 📊 영향 분석

### 수정된 메서드 (15개)
1. ✅ `find_by_id()` - ID로 조회
2. ✅ `find_by_project_id()` - 프로젝트 ID로 조회
3. ✅ `find_by_user_id()` - 사용자 ID로 조회
4. ✅ `find_by_study_uid()` - Study UID로 조회
5. ✅ `find_by_series_uid()` - Series UID로 조회
6. ✅ `find_by_instance_uid()` - Instance UID로 조회
7. ✅ `find_by_project_and_study()` - 프로젝트 + Study로 조회
8. ✅ `find_by_project_and_series()` - 프로젝트 + Series로 조회
9. ✅ `find_by_project_and_instance()` - 프로젝트 + Instance로 조회
10. ✅ `create()` - 생성 (RETURNING 절)
11. ✅ `update()` - 업데이트 (기존 데이터 조회 + RETURNING 절)
12. ✅ `update_with_version_check()` - 버전 체크 업데이트 (기존 데이터 조회 + RETURNING 절)
13. ✅ `update_snapshot_info()` - Snapshot 정보 업데이트 (RETURNING 절)
14. ✅ `delete()` - 삭제 (기존 데이터 조회)
15. ✅ `find_by_project_and_series_paginated()` - 페이지네이션 조회

### 성능 영향
- **네트워크 전송량**: 미미한 증가 (대부분 NULL 값)
- **메모리 사용량**: 무시할 수 있는 수준
- **쿼리 실행 시간**: 변화 없음

---

## ✅ 검증

### 테스트 결과
```bash
✅ test_annotation_level_filtering_refactored.py
✅ test_annotation_version_conflict_refactored.py
✅ test_annotation_head_request_refactored.py
✅ test_annotation_snapshot_e2e_refactored.py
✅ test_annotation_permission_filtering_refactored.py
✅ test_annotation_permission_management_refactored.py
```

### 기능 검증
- ✅ Annotation 생성
- ✅ Annotation 조회 (ID, Project, User, Study, Series, Instance)
- ✅ Annotation 업데이트
- ✅ Annotation 삭제
- ✅ Annotation History 생성

---

## 🎯 핵심 교훈

### 1. Entity 변경 시 체크리스트
```
✅ Entity 구조체 수정
✅ DB Migration 작성
✅ Repository의 모든 SELECT 쿼리 업데이트 ⭐ 중요!
✅ RETURNING 절이 있는 INSERT/UPDATE 쿼리 업데이트
✅ 테스트 실행
```

### 2. sqlx의 타입 매핑
- `query_as::<_, T>` 사용 시 런타임 에러 발생 가능
- Entity 필드와 SELECT 쿼리 필드가 정확히 일치해야 함
- 필드 순서도 일치하는 것이 권장됨

### 3. 예방 방법
- **컴파일 타임 체크**: `query_as!` 매크로 사용 고려
- **통합 테스트**: Repository 레이어 테스트 강화
- **E2E 테스트**: API 엔드포인트 테스트 자동화
- **코드 리뷰**: Entity 변경 시 Repository 쿼리 확인

---

## 📁 문서 구조

```
issues/2026-01-14-annotation-조회-삭제-오류-수정/
├── README.md                 # 이슈 개요 및 전체 설명
├── SUMMARY.md               # 요약 (이 문서)
├── 수정-상세-내역.md         # 수정된 쿼리 목록 및 Before/After
├── 기술-분석.md             # sqlx 매핑 메커니즘 및 원인 분석
├── 체크리스트.md            # Entity 변경 시 체크리스트
└── 다이어그램.md            # Mermaid 다이어그램
```

---

## 🚀 다음 단계

### 단기
- [ ] 스테이징 환경 배포
- [ ] 프로덕션 환경 배포
- [ ] 모니터링 확인

### 중기
- [ ] `query_as!` 매크로 도입 검토
- [ ] Repository 통합 테스트 추가
- [ ] CI/CD 파이프라인에 E2E 테스트 추가

### 장기
- [ ] Entity 변경 자동화 도구 개발
- [ ] 코드 생성기 도입 검토
- [ ] 타입 안정성 강화

---

## 📖 관련 문서

- [README.md](./README.md) - 이슈 전체 설명
- [수정-상세-내역.md](./수정-상세-내역.md) - 수정된 쿼리 목록
- [기술-분석.md](./기술-분석.md) - 기술적 분석
- [체크리스트.md](./체크리스트.md) - Entity 변경 시 체크리스트
- [다이어그램.md](./다이어그램.md) - Mermaid 다이어그램

---

## 👥 작업자

- **작업자**: aido
- **날짜**: 2026-01-14
- **우선순위**: 🔴 High (기능 동작 불가)
- **상태**: ✅ 완료

---

## 📝 메모

이 이슈는 Entity 변경 시 Repository 쿼리를 빠짐없이 업데이트해야 한다는 중요한 교훈을 남겼습니다. 
향후 유사한 문제를 예방하기 위해 체크리스트를 작성하고, 자동화 도구 도입을 검토할 필요가 있습니다.

