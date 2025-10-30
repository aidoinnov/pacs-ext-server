---

# 📘 PACS 마스크 업로드 설계 단계 문서 (Design Phase)

## 1️⃣ 개요

본 문서는 PACS(DICOM Viewer) 환경에서 AI 또는 사용자가 생성한 **Segmentation Mask** 데이터를

안정적이고 확장성 있게 저장·관리·조회하기 위한 v2 아키텍처 설계를 정의한다.

---

## 2️⃣ 시스템 목표

| 구분 | 목표 |
| --- | --- |
| 🎯 **기능적 목표** | Annotation에 연결된 마스크 데이터를 그룹 단위(3D/버전)로 관리 |
| ⚙️ **기술적 목표** | 대용량 마스크 파일을 클라이언트가 직접 Object Storage로 업로드 |
| 🧱 **데이터 구조 목표** | Annotation → Mask Group → Mask (1:N:N) 구조 확립 |
| 🔒 **보안 목표** | Signed URL 기반 단기 권한 부여, IAM Prefix 제한, HTTPS 통신 |
| 🌐 **확장 목표** | AI inference pipeline / DICOM SEG 변환 연동 가능 구조 |

---

## 3️⃣ 아키텍처 개요

### 📊 논리 구조

```
Viewer (OHIF / AI Client)
   ↓
Annotation Server (API)
   ↓
Object Storage (S3/MinIO)
   ↓
PostgreSQL (annotation, mask_group, mask)

```

### 📦 데이터 흐름

1. Viewer → 서버에 마스크 그룹 생성 요청
2. 서버 → S3 signed URL 발급
3. Viewer → S3로 직접 업로드 (PUT)
4. Viewer → 업로드 완료 후 서버에 완료 알림
5. 서버 → DB에 메타데이터 저장

---

## 4️⃣ 데이터 모델 설계

### 4.1 `annotation_mask_group` (Segmentation 단위 그룹)

```sql
CREATE TABLE annotation_mask_group (
    id SERIAL PRIMARY KEY,
    annotation_id INTEGER NOT NULL REFERENCES annotation(id) ON DELETE CASCADE,
    group_name TEXT,                       -- 예: Liver_Segmentation_v2
    model_name TEXT,                       -- AI 모델명 (optional)
    version TEXT,                          -- 버전명 (optional)
    modality TEXT,                         -- CT/MR 등
    slice_count INTEGER DEFAULT 1,
    mask_type TEXT DEFAULT 'segmentation',
    description TEXT,
    created_by INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_mask_group_annotation_id ON annotation_mask_group(annotation_id);

```

---

### 4.2 `annotation_mask` (각 slice 또는 label별 파일)

```sql
CREATE TABLE annotation_mask (
    id SERIAL PRIMARY KEY,
    mask_group_id INTEGER NOT NULL REFERENCES annotation_mask_group(id) ON DELETE CASCADE,
    slice_index INTEGER,                   -- 볼륨 내 slice index
    label_name TEXT,                       -- 예: liver, spleen
    file_path TEXT NOT NULL,               -- S3 경로
    mime_type TEXT DEFAULT 'image/png',
    file_size BIGINT,
    checksum TEXT,
    width INTEGER,
    height INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_mask_mask_group_id ON annotation_mask(mask_group_id);

```

---

## 5️⃣ 파일 저장 구조 설계

| 항목 | 규칙 |
| --- | --- |
| **버킷명** | `pacs-masks` |
| **경로 규칙** | `mask/{annotation_id}/{group_id}/{filename}` |
| **파일명 규칙** | `{slice_index}_{label_name}.png` |
| **예시** | `mask/123/17/0001_liver.png` |
| **메타파일** | `metadata.json` (slice_count, labels, model_name 등) |

---

## 6️⃣ API 설계

| 동작 | 올바른 REST 경로 | 설명 |
| --- | --- | --- |
| 그룹 생성 | `POST /api/annotations/:annotation_id/mask-groups` | 특정 annotation에 마스크 그룹 추가 |
| 그룹 목록 조회 | `GET /api/annotations/:annotation_id/mask-groups` | annotation에 속한 그룹 리스트 |
| 그룹 상세 조회 | `GET /api/annotations/:annotation_id/mask-groups/:group_id` | 그룹 상세 정보 |
| slice 목록 조회 | `GET /api/annotations/:annotation_id/mask-groups/:group_id/masks` | 그룹 내 slice mask 리스트 |
| signed URL 발급 | `POST /api/annotations/:annotation_id/mask-groups/:group_id/signed-url` | 특정 그룹 내 업로드 URL 발급 |
| 업로드 완료 등록 | `POST /api/annotations/:annotation_id/mask-groups/:group_id/complete` | 업로드 완료 처리 |
| 삭제 | `DELETE /api/annotations/:annotation_id/mask-groups/:group_id` | 그룹 전체 삭제 |

---

### 6.1 요청/응답 예시

**[그룹 생성]**

```json
POST /api/mask-group
{
  "annotation_id": 123,
  "group_name": "AI_Liver_v2",
  "model_name": "monai_unet",
  "version": "v2"
}
→ { "group_id": 17 }

```

**[Signed URL 발급]**

```json
POST /api/mask-group/17/signed-url
{
  "filename": "0001_liver.png",
  "mime_type": "image/png"
}
→ {
  "upload_url": "https://s3.example.com/mask/123/17/0001_liver.png?...",
  "file_path": "s3://pacs-masks/mask/123/17/0001_liver.png",
  "expires_in": 600
}

```

**[업로드 완료]**

```json
POST /api/mask-group/17/complete
{
  "slice_count": 120,
  "labels": ["liver", "spleen"]
}
→ { "status": "ok" }

```

---

- 
    
    ## 7️⃣ 보안 설계
    
    | 항목 | 정책 |
    | --- | --- |
    | **Signed URL TTL** | 기본 10분, 최대 1시간 |
    | **IAM Policy** | prefix 단위(`mask/{annotation_id}/`) 권한 부여 |
    | **CORS** | Viewer 도메인 한정, PUT/GET 허용 |
    | **프로토콜** | HTTPS only |
    | **로그/Audit** | URL 발급, 업로드 완료 시점 기록 |
    | **파일명 정책** | 개인정보 포함 금지 (annotation_id 기반) |
    
    ## 8️⃣ 내부망 대응 설계 (Fallback)
    
    | 항목 | Signed URL 모드 | 내부 업로드 모드 |
    | --- | --- | --- |
    | 업로드 | PUT to S3 | POST `/api/mask/upload` |
    | 저장소 | S3 | 로컬 NAS / PVC |
    | 인증 | signed URL | 세션 / 토큰 인증 |
    | TTL | 10분 | 제한 없음 |
    | 구조 | 동일한 DB 스키마 유지 | 동일 |
    
    ## 9️⃣ 예외 처리 설계
    
    | 상황 | 코드 | 처리 방식 |
    | --- | --- | --- |
    | annotation_id 없음 | 404 | “annotation not found” |
    | S3 URL 만료 | 403 | signed URL 재발급 요청 |
    | 파일 누락 | 400 | missing file_path |
    | DB insert 실패 | 500 | rollback |
    | 삭제 중 파일 존재 오류 | 409 | retry after cleanup job |
    
    ---
    
    ## 11️⃣ 보관 / 삭제 정책
    
    | 항목 | 정책 |
    | --- | --- |
    | **삭제 정책** | group 삭제 시 모든 slice cascade 삭제 |
    | **보관 주기** | 기본 무기한, 필요 시 Lifecycle Rule로 90일 만료 |
    | **오브젝트 이름 규칙** | 변경 불가 (immutable naming) |
    | **Cleanup Job** | orphan mask (annotation 삭제 후 남은 파일) 주기적 정리 |
    
    ---
    
    ## 2️⃣ 아키텍처 다이어그램
    
    ```mermaid
    sequenceDiagram
        participant V as DICOM Viewer
        participant A as Annotation Server
        participant S3 as Object Storage
        participant DB as PostgreSQL
    
        V->>A: POST /api/mask-group (annotation_id)
        A-->>V: group_id
        loop for each slice
            V->>A: POST /api/mask-group/{group_id}/signed-url
            A-->>V: upload_url
            V->>S3: PUT slice_{n}.png
        end
        V->>A: POST /api/mask-group/{group_id}/complete
        A->>DB: INSERT metadata
        A-->>V: status OK
    
    ```
    
    ---
    
    ## 3️⃣ 비기능 요구사항 (NFR)
    
    | 항목 | 목표치 |
    | --- | --- |
    | **업로드 속도** | 병렬 100 slice 기준 1분 이내 |
    | **다운로드 지연** | 200ms 이하 (proxy streaming 기준) |
    | **서버 부하** | 파일 I/O 0 (직접 업로드 구조로 오프로딩) |
    | **URL TTL** | 최대 1시간, 자동 만료 |
    | **저장 용량** | 1TB 이상 확장 가능 (S3 기반) |

---

---

---

## ✅ 4️⃣ 설계 산출물 요약

| 산출물 | 설명 | 파일 경로 제안 |
| --- | --- | --- |
| ERD 다이어그램 | DB 관계도 | `/docs/erd/annotation_mask_v2.drawio` |
| API 스펙 문서 | OpenAPI 3.0 YAML | `/docs/api/pacs_mask_v2.yaml` |
| 설계 문서 (본 파일) | 시스템 구조, 정책 포함 | `/docs/specs/pacs_mask_upload_design_phase.md` |
| IAM 정책 예시 | AWS IAM Role JSON | `/infra/iam/pacs_mask_policy.json` |
| 테스트 케이스 | QA 시나리오 | `/tests/pacs_mask_upload_v2/` |

---

## 🧭 작업단계

| 구분 | 다음 단계 | 담당 |
| --- | --- | --- |
| 1️⃣ | DB migration 구현 | DB Engineer |
| 2️⃣ | Signed URL API 개발 | Backend |
| 3️⃣ | S3 버킷 정책 적용 | Infra |
| 4️⃣ | Viewer 업로드 연동 | Frontend |
| 5️⃣ | 대용량 볼륨 테스트 | QA |
| 6️⃣ | 문서화 + 운영 반영 | Architect |