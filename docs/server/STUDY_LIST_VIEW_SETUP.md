# Study List View 컬럼 구성 가이드

## 개요

Study List에 표시되는 컬럼은 서버의 `study_list_view` / `study_list_view_field` 테이블에서 관리된다.
클라이언트는 View를 선택하면 해당 View에 정의된 컬럼 순서와 구성을 그대로 렌더링한다.

---

## 관련 마이그레이션

| 마이그레이션 | 내용 |
|---|---|
| `032_create_study_list_view.sql` | 테이블 생성, DICOM/Extension 필드 정의, `default` View 생성 |
| `033_add_extension_fields.sql` | 프로젝트 관련 Extension 필드 정의 추가 (project, timePoint, visitType 등) |
| `035_add_report_review_ext_fields.sql` | report_status, review Extension 필드 정의 추가 |
| `20260413_02_add_missing_ext_fields_to_default_view.sql` | default View 컬럼을 표준 순서로 재구성 |

---

## Default View 표준 컬럼 순서

| display_order | source | field_key | 표시명 |
|---|---|---|---|
| 0 | extension | project | Project |
| 1 | extension | subjectNo | Subject No |
| 2 | dicom | PatientAge | Age |
| 3 | dicom | PatientSex | Gender |
| 4 | dicom | StudyDescription | Study Description |
| 5 | dicom | ModalitiesInStudy | Modality |
| 6 | dicom | StudyDate | Study Date |
| 7 | extension | timePoint | Time Point |
| 8 | extension | visitType | Visit Type |
| 9 | extension | visitNumber | Visit Number |
| 10 | extension | annotationCount | Annotation |
| 11 | extension | status | Status |

---

## 필드 정의 테이블

### DICOM 필드 (`dicom_field_def`)

032, 033에서 seed된 DICOM 표준 필드. `field_key`가 DICOM keyword에 해당한다.

주요 필드: `PatientName`, `PatientID`, `StudyDate`, `StudyDescription`, `ModalitiesInStudy`, `AccessionNumber`, `PatientAge`, `PatientSex` 등

### Extension 필드 (`ext_field_def`)

DICOM 외 확장 메타데이터. `source_config`에 데이터 조회 설정이 JSON으로 저장된다.

| field_key | label | source_system | 데이터 출처 |
|---|---|---|---|
| project | Project | internal | `project_data.project_name` |
| projectId | Project ID | internal | `project_data.project_id` |
| subjectNo | Subject No | internal | `project_data.subject_no` |
| visitName | Visit Name | internal | `project_data.visit_name` |
| scanDate | Scan Date | internal | `project_data.scan_date` |
| timePoint | Time Point | internal | `project_data.time_point` |
| visitType | Visit Type | internal | `project_data.visit_type` |
| visitNumber | Visit Number | internal | `project_data.visit_number` |
| annotationCount | Annotation | annotation | `annotations` (count) |
| status | Status | workflow | `study_workflow.status` |
| report_status | Report Status | internal | `series_user_report.status` |
| review | Review | annotation | computed (reviewStage, availableStages 등) |

---

## API

### 컬럼 구성 조회

```
GET /api/study-list-views/default
```

View의 필드 목록을 `display_order` 순서로 반환한다.

### 사용 가능한 필드 정의 조회

```
GET /api/study-list-views/field-defs
GET /api/study-list-views/field-defs?source=extension
GET /api/study-list-views/field-defs?source=dicom&level=study
```

### View CRUD

```
POST   /api/study-list-views              # 생성
GET    /api/study-list-views              # 목록
GET    /api/study-list-views/{view_id}    # 상세
PUT    /api/study-list-views/{view_id}    # 수정
DELETE /api/study-list-views/{view_id}    # 삭제
```

---

## 수동 마이그레이션 (EKS)

서버의 `RUN_MIGRATIONS=true` 설정 시 자동 실행되지만, 수동 적용이 필요한 경우:

```bash
# Deploy 서버 접속
ssh dl-server102@192.168.0.202

# DB Pod에서 직접 SQL 실행
kubectl exec -n pacs db-postgresql-0 -- psql -U postgres -c "SQL_HERE"

# 현재 View 필드 확인
kubectl exec -n pacs db-postgresql-0 -- psql -U postgres -c \
  "SELECT display_order, field_source, field_key FROM study_list_view_field WHERE view_id='default' ORDER BY display_order;"
```
