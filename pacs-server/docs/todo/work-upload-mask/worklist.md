---

# 🧭 PACS 마스크 업로드 v2 작업 목록

## 🧩 1️⃣ 설계 단계

| 우선순위 | 작업 | 설명 | 담당 |
| --- | --- | --- | --- |
| 🟩 P1 | **DB 스키마 정의서 작성** | `annotation_mask_group`, `annotation_mask` 테이블 DDL 작성 (SQL + migration) | DB Engineer |
| 🟩 P1 | **S3 경로 규칙 정의** | `mask/{annotation_id}/{group_id}/{filename}` 패턴 확정 | Infra/DevOps |
| 🟨 P2 | **파일 메타데이터 표준화** | width, height, label, checksum, model_name 등 컬럼 확정 | Backend |
| 🟨 P2 | **API 명세서 정리** | `/api/mask-group/*` 엔드포인트 스펙 상세화 (request/response 예시 포함) | Backend |
| 🟦 P3 | **보안 정책 확정** | signed URL TTL, IAM Role, prefix policy | Infra |
| 🟦 P3 | **업로드 시나리오 정리** | ① Viewer 수동 업로드, ② AI inference 업로드 플로우 명세 | PM / System Architect |

---

## ⚙️ 2️⃣ 서버(백엔드) 구현 단계

| 우선순위 | 작업 | 설명 | 담당 |
| --- | --- | --- | --- |
| 🟩 P1 | **DB migration 스크립트 작성** | Flyway / Prisma / Diesel 등으로 테이블 추가 | DB Engineer |
| 🟩 P1 | **`POST /api/mask-group` 구현** | group 생성 (annotation_id, model_name, version 등) | Backend |
| 🟩 P1 | **`POST /api/mask-group/:group_id/signed-url` 구현** | signed PUT URL 발급 (AWS SDK or MinIO SDK) | Backend |
| 🟨 P2 | **`POST /api/mask-group/:group_id/complete`구현** | DB에 slice metadata 등록 | Backend |
| 🟨 P2 | **`GET /api/mask-group/:annotation_id` 구현** | group 목록 조회 API | Backend |
| 🟨 P2 | **`GET /api/mask/:group_id` 구현** | slice 목록 조회 API | Backend |
| 🟦 P3 | **`DELETE /api/mask-group/:group_id` 구현** | 그룹 삭제 (DB + S3 cleanup) | Backend |
| 🟦 P3 | **S3 에러 핸들링 추가** | URL 만료, 업로드 실패 처리 로직 | Backend |
| 🟦 P3 | **업로드 완료 후 audit log 기록** | 누가 언제 어떤 annotation에 업로드했는지 | Backend |

---

## ☁️ 3️⃣ 인프라 / 스토리지 설정

| 우선순위 | 작업 | 설명 | 담당 |
| --- | --- | --- | --- |
| 🟩 P1 | **S3 버킷 생성** | `pacs-masks` 버킷 생성 및 prefix 구조 설정 | Infra |
| 🟩 P1 | **IAM Role / Policy 설정** | PUT/GET 권한 prefix 단위로 제한 (`mask/*`) | Infra |
| 🟨 P2 | **Signed URL SDK 연동 테스트** | AWS SDK / boto3 / minio-go / aws-sdk-js | Infra |
| 🟨 P2 | **CORS 설정** | Viewer 도메인에서 PUT 허용 (method: PUT, GET) | Infra |
| 🟦 P3 | **Lifecycle Rule 적용** | 오래된 마스크 자동 삭제 (optional) | Infra |
| 🟦 P3 | **Storage Usage 모니터링 추가** | Grafana/Loki 로그 대시보드에 업로드 통계 표시 | DevOps |

---

## 🎨 4️⃣ 클라이언트(Viewer) 연동

| 우선순위 | 작업 | 설명 | 담당 |
| --- | --- | --- | --- |
| 🟩 P1 | **마스크 업로드 플로우 구현** | `POST /api/mask-group → signed-url → PUT → complete` 흐름 구현 | Frontend |
| 🟩 P1 | **진행률 표시 (progress bar)** | slice 단위 업로드 상태 표시 | Frontend |
| 🟨 P2 | **AI inference 연동 인터페이스 추가** | inference 결과를 자동으로 업로드할 수 있는 API 호출 | Frontend |
| 🟨 P2 | **마스크 리스트 뷰 구현** | group별 / slice별 mask 목록 표시 | Frontend |
| 🟦 P3 | **마스크 다운로드(보기) 기능 구현** | `/api/mask-group/:group_id/view` 프록시 스트리밍으로 표시 | Frontend |
| 🟦 P3 | **에러 재시도 로직 추가** | URL 만료 시 재발급 처리 | Frontend |

---

## 🧠 5️⃣ AI Integration (선택)

| 우선순위 | 작업 | 설명 | 담당 |
| --- | --- | --- | --- |
| 🟨 P2 | **Inference server → Mask Upload 자동화** | AI 결과를 signed URL로 업로드하는 worker 구현 | AI Engineer |
| 🟦 P3 | **Inference metadata 등록** | model_name, version, confidence score 등 DB에 저장 | Backend |
| 🟦 P3 | **Inference validation job** | AI 결과 검증 (slice 누락, 파일 무결성 등) | AI Engineer |

---

## 🧪 6️⃣ 테스트 / QA 단계

| 우선순위 | 작업 | 설명 | 담당 |
| --- | --- | --- | --- |
| 🟩 P1 | **단일 slice 업로드 테스트** | 정상 PUT + DB 등록 확인 | QA |
| 🟩 P1 | **다중 slice 병렬 업로드 테스트** | 100개 slice 동시에 업로드 성능 측정 | QA |
| 🟨 P2 | **URL 만료 처리 테스트** | TTL 초과 후 업로드 시 실패 처리 확인 | QA |
| 🟨 P2 | **파일 무결성 검사** | checksum 비교 | QA |
| 🟦 P3 | **삭제/cleanup 테스트** | mask_group 삭제 시 S3 파일 삭제 여부 확인 | QA |
| 🟦 P3 | **대용량 볼륨(>1000 slice)** | 성능, 타임아웃 검증 | QA |

---

## 🧩 7️⃣ 문서화 / 운영

| 우선순위 | 작업 | 설명 | 담당 |
| --- | --- | --- | --- |
| 🟩 P1 | **README 작성** | API 예제, 업로드 순서, 환경변수 정리 | Backend |
| 🟨 P2 | **`docs/specs/pacs_mask_upload_v2.md` 생성** | 아키텍처 + API + 플로우 다이어그램 포함 | Architect |
| 🟨 P2 | **운영 매뉴얼 작성** | 장애 처리, 로그 확인, S3 정책 변경 절차 | DevOps |
| 🟦 P3 | **Notion/Confluence 반영** | 팀 내부 공유용 문서화 | PM |

---

## 🧭 8️⃣ 실행 순서 (추천 로드맵)

1️⃣ **DB + Infra 세팅 (P1)**

→ DB migration, S3 버킷/IAM/CORS 설정

2️⃣ **백엔드 기본 API 구현 (P1~P2)**

→ signed-url, complete, get/list

3️⃣ **Viewer 업로드 연동 (P1)**

→ 업로드/다운로드 흐름 동작 확인

4️⃣ **AI integration + 볼륨 테스트 (P2~P3)**

→ 자동 업로드, 3D volume validation

5️⃣ **운영 모니터링 + QA (P3)**

→ Storage usage, TTL handling, 삭제 테스트

---

## 📦 9️⃣ AI Agent별 분담 예시

| Agent | 담당 범위 |
| --- | --- |
| 🧩 **DB Agent** | migration 스크립트, 테이블 설계 |
| ⚙️ **Backend Agent** | API 구현(`/mask-group`, `/signed-url`, `/complete`) |
| ☁️ **Infra Agent** | S3 / MinIO 설정, IAM 정책, CORS |
| 🧭 **Frontend Agent** | Viewer 업로드/다운로드 UI, progress 상태 |
| 🧠 **AI Agent** | inference 결과 자동 업로드 |
| 🔍 **QA Agent** | 병렬 업로드, TTL, 삭제, 무결성 테스트 |

---

## ✅ 10 최종 체크리스트

- [ ]  DB migration 완료
- [ ]  S3 버킷 및 IAM 정책 적용
- [ ]  signed URL 발급 테스트 성공
- [ ]  Viewer 업로드 → PUT → complete 플로우 동작
- [ ]  AI inference 서버 연동 확인
- [ ]  cleanup job 정상 동작
- [ ]  문서화 완료 (`docs/specs/pacs_mask_upload_v2.md`)

---