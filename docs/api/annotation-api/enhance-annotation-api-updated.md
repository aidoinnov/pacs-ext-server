# 🩻 Annotation REST API v1 — Technical Specification (Updated)

**Version:** 1.1  
**Scope:** DICOM Viewer 1차 개발 (REST 기반 Annotation 관리)  
**Author:** Backend Team  
**Date:** 2025-11-07  
**Status:** ✅ 현재 구현 상태 반영

---

## 📘 목적

본 문서는 DICOM Viewer의 1차 개발 범위인 **Annotation REST API 설계 명세**를 정의한다.  
현재 구현된 기능과 향후 개발 계획을 명시한다.

---

## ⚙️ API Endpoints (Query Parameter 기반)

### 1. Series 레벨 조회

**`GET /api/annotations?series_instance_uid={seriesUID}&project_id={projectID}`**

- **Query Parameters:**
  - `series_instance_uid` (required): Series UID
  - `project_id` (optional): Project ID
  - `user_id` (optional): User ID (권한 기반 필터링 활성화)
  - `level` (optional): 필터링 레벨 (`study`, `series`, `instance`)
  - `viewer_software` (optional): Viewer 소프트웨어 필터링

### 2. Study 레벨 조회

**`GET /api/annotations?study_instance_uid={studyUID}&project_id={projectID}`**

- **Query Parameters:** Series와 동일

### 3. Instance 단위 조회

**`GET /api/annotations?sop_instance_uid={instanceUID}`**

- **Query Parameters:**
  - `sop_instance_uid` (required): SOP Instance UID
  - `level` (optional): 필터링 레벨
  - `viewer_software` (optional): Viewer 필터링
  - `user_id` (optional): User ID 필터링

### 4. Annotation 생성

**`POST /api/annotations`**

- **Request Body:** 모든 필드 포함 (project_id, user_id, UIDs, tool_name, data 등)

### 5. Annotation 수정

**`PUT /api/annotations/{annotation_id}`**

- **Request Body:** 수정할 필드만 포함

### 6. Annotation 삭제

**`DELETE /api/annotations/{annotation_id}`**

---

## 🗄️ 데이터 모델 (현재 구현)

| 필드                  | 타입       | 설명                           |
| ------------------- | -------- | ----------------------------- |
| id                  | integer  | 고유 ID (Primary Key)          |
| project_id          | integer  | 프로젝트 ID                      |
| user_id             | integer  | 생성자 User ID                  |
| study_instance_uid  | string   | Study UID                     |
| series_instance_uid | string   | Series UID                    |
| sop_instance_uid    | string   | SOP Instance UID (Instance)   |
| tool_name           | string   | 도구명 (ROI, Mask, Note 등)     |
| tool_version        | string   | 도구 버전                        |
| data                | JSON     | annotation 데이터 (geometry 등) |
| is_shared           | boolean  | 공유 여부                        |
| created_at          | datetime | 생성 시간                        |
| updated_at          | datetime | 마지막 수정 시간                   |
| viewer_software     | string   | Viewer 소프트웨어명               |
| description         | string   | 설명/라벨                        |
| measurement_values  | JSON     | 측정값 (선택사항)                  |
| user_name           | string   | 생성자 이름 (응답에만 포함)          |

---

## ✅ 구현된 기능

1️⃣ **Query Parameter 기반 조회**
   - `series_instance_uid`, `study_instance_uid`, `sop_instance_uid` 지원
   - `project_id`, `user_id` 파라미터로 필터링
   - `level` 파라미터로 DICOM 계층 필터링
   - `viewer_software` 파라미터로 Viewer 필터링

2️⃣ **권한 기반 필터링 (RBAC)**
   - `user_id` + `project_id` 조합으로 권한 체크
   - `ANNOTATION:READ_ALL` 권한 있으면 모든 어노테이션 조회
   - 권한 없으면 본인 어노테이션만 조회

3️⃣ **CRUD 작업**
   - ✅ Create: `POST /api/annotations`
   - ✅ Read: `GET /api/annotations?...` (다양한 필터링)
   - ✅ Update: `PUT /api/annotations/{annotation_id}`
   - ✅ Delete: `DELETE /api/annotations/{annotation_id}`

4️⃣ **응답 포맷**
   - 모든 조회는 `{ annotations: [...], total: N }` 형식
   - 각 annotation에 `user_name` 포함 (N+1 쿼리 최적화됨)

---

## ❌ 미구현 기능 (향후 개발)

1️⃣ **Version Conflict 처리**
   - 현재: Last-Write-Wins (덮어쓰기)
   - 향후: Optimistic Locking 또는 Conflict Resolution

2️⃣ **WebSocket 실시간 동기화**
   - annotation_created / updated / deleted 이벤트 전파
   - 다중 사용자 동시 편집 지원

3️⃣ **Collaborative Lock**
   - 동일 annotation 동시 수정 방지
   - 편집자 실시간 표시 (Presence)

4️⃣ **History / Audit Trail**
   - annotation 변경 이력 저장
   - 버전별 스냅샷 관리

5️⃣ **HEAD 요청**
   - 버전 정보만 확인하는 경량 요청
   - 캐시 유효성 검증용

---

## 📝 주요 차이점 (원본 vs 현재 구현)

| 항목 | 원본 설계 | 현재 구현 |
|------|---------|---------|
| **URL 스타일** | Path Parameter (`/series/{id}`) | Query Parameter (`?series_instance_uid=`) |
| **버전 관리** | baseVersion 필수 (Optimistic Lock) | Last-Write-Wins (덮어쓰기) |
| **HEAD 요청** | 지원 | 미지원 |
| **권한 체크** | 미언급 | ✅ RBAC 기반 구현 |
| **응답 포맷** | 배열 | `{ annotations: [...], total: N }` |
| **필터링** | 기본 | ✅ level, viewer_software 등 다양한 필터 |

