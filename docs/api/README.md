# PACS Extension Server API 문서

## 🚀 빠른 시작

### 서버 실행
```bash
cd pacs-server
cargo run &
```

### API 접근
- **Base URL**: `http://localhost:8080/api`
- **Swagger UI**: http://localhost:8080/swagger-ui/
- **OpenAPI 스펙**: http://localhost:8080/api-docs/openapi.json

## 📚 API 문서

### ⚡ 캐싱 가이드
- **[통합 캐싱 가이드](./caching-guide.md)** ⭐ 필독
  - 모든 캐싱 전략 비교 및 사용법
  - Redis 캐싱 vs HTTP 캐싱
  - 클라이언트 구현 가이드
- **[QIDO-RS 캐시 가이드](./qido-cache-client-guide.md)**
  - Redis 기반 DICOM 데이터 캐싱
  - 60초 TTL, 자동 캐싱
- **[Membership 캐시 가이드](./membership-cache-guide.md)** ⭐ NEW
  - Redis 기반 프로젝트 멤버십 캐싱
  - 180초 TTL, RBAC 성능 개선
  - DB 쿼리 80% 절감
- **[Capability 캐시 가이드](./capability-cache-client-guide.md)**
  - HTTP ETag 기반 캐싱
  - 60초 TTL, 조건부 요청
- **[Role Assignment 캐시 가이드](./role-assignment-caching-guide.md)**
  - HTTP ETag 기반 캐싱
  - 1초 TTL, 중복 요청 방지

### 🏷️ Annotation API
- **[Annotation 목록 조회 (Lesion 정보 포함)](./annotation/annotation-list-with-lesion.md)** ⭐ NEW
  - Lesion 정보가 포함된 Annotation 목록 조회
  - 프로젝트별, 사용자별, Study별 조회
- **[Lesion Assignment API](./annotation/lesion-assignment.md)** ⭐ NEW
  - RECIST 1.1 기준 Lesion 할당 (TARGET, NON_TARGET, TARGET_NEW, NON_TARGET_NEW)
  - Lesion 생성, 수정, 삭제, 조회

### 📊 TimePoint API
- **[TimePoints with Studies API (X축 API)](./timepoint/timepoints-with-studies.md)** ⭐ NEW
  - Subject의 TimePoints + Studies + Unassigned Studies 조회
  - RECIST Report X축 데이터
- **[Annotations by TimePoint API (Y축 API)](./timepoint/annotations-by-timepoint.md)** ⭐ NEW
  - TimePoint의 모든 Annotations 조회 (Lesion 정보 포함)
  - RECIST Report Y축 데이터

### 🩺 DICOM API
- **[DICOM Data Access Check API](./dicom/data-access-check-api.md)** ⭐ NEW
  - Study/Series 접근 권한 확인
  - 프로젝트별 접근 권한 확인
  - RBAC 기반 권한 평가
- **[시리즈/인스턴스 조회 API](./dicom/시리즈-인스턴스-조회-API.md)**
  - Series 및 Instance 조회
  - WADO-RS 썸네일 URL 포함
- **[DICOM Gateway API](./dicom/dicom-gateway-api.md)**
  - QIDO-RS 프록시
  - RBAC 필터링

### 🔐 Capability API (권한 관리)
- [Capability API 스펙 문서](./capability-api-specification.md)
- [UI 구현 가이드](../ui/capability-ui-implementation-guide.md)

**주요 엔드포인트**:
- `GET /api/roles/global/capabilities/matrix` - 전역 Role-Capability 매트릭스
- `GET /api/capabilities` - 모든 Capability 목록
- `GET /api/capabilities/{id}` - Capability 상세 조회
- `PUT /api/roles/{role_id}/capabilities/{capability_id}` - Capability 할당/제거

### 📋 기존 API들
- **Project User Matrix API**: `/api/projects/{id}/users/matrix`
- **Role Permission Matrix API**: `/api/roles/global/permissions/matrix`
- **Project Data Access API**: `/api/projects/{id}/data-access`
- **User Registration API**: `/api/auth/signup`
- **Token Refresh API**: `/api/auth/refresh`

## 🎯 Capability API 특징

### 3단계 권한 구조
```
Role → Capability → Permission
      (UI 표시)    (실제 권한)
```

### 카테고리별 그룹화
- **관리**: 시스템 관리, 사용자 관리, 역할 관리, 프로젝트 관리
- **프로젝트**: 프로젝트 생성, 편집, 할당
- **DICOM 데이터 관리**: 읽기, 쓰기, 삭제, 공유
- **어노테이션 관리**: 본인/모든 어노테이션 읽기, 작성, 삭제, 공유
- **마스크 관리**: 읽기, 작성, 삭제
- **행잉 프로토콜 관리**: 전체 관리

### 기본 역할 (5개)
- **SUPER_ADMIN**: 모든 권한 (20개 Capability)
- **ADMIN**: 관리 권한 + 기본 권한 (15개 Capability)
- **PROJECT_ADMIN**: 프로젝트 관리 권한 (14개 Capability)
- **USER**: 기본 사용자 권한 (7개 Capability)
- **VIEWER**: 읽기 전용 권한 (4개 Capability)

## 📖 주요 API 사용 예시

### TimePoints with Studies (X축 API)

**Subject의 모든 TimePoints + Studies 조회:**
```bash
GET /api/subjects/{subject_id}/timepoints-with-studies?include_unassigned=true
```

**응답:**
```json
{
  "subject_id": 1,
  "subject_code": "SUBJ-001",
  "timepoints": [
    {
      "id": 1,
      "name": "Baseline",
      "timepoint_date": "2026-01-01",
      "studies": [...]
    }
  ],
  "unassigned_studies": [...]
}
```

**상세 문서:** [TimePoints with Studies API (X축 API)](./timepoint/timepoints-with-studies.md)

---

### Annotations by TimePoint (Y축 API)

**TimePoint의 모든 Annotations 조회:**
```bash
GET /api/timepoints/{timepoint_id}/annotations
```

**응답:**
```json
{
  "timepoint_id": 1,
  "timepoint_name": "Baseline",
  "annotations": [
    {
      "id": 123,
      "lesion_type": "TARGET",
      "lesion_number": 1,
      "description": "Liver lesion #1",
      "measurement_values": {"diameter": 25.5, "unit": "mm"}
    }
  ],
  "total": 1
}
```

**상세 문서:** [Annotations by TimePoint API (Y축 API)](./timepoint/annotations-by-timepoint.md)

---

## 🔧 개발자 도구

### API 테스트
```bash
# 헬스 체크
curl http://localhost:8080/health

# 전역 매트릭스 조회
curl http://localhost:8080/api/roles/global/capabilities/matrix | jq

# Capability 상세 조회
curl http://localhost:8080/api/capabilities/36 | jq

# 권한 할당
curl -X PUT http://localhost:8080/api/roles/2/capabilities/36 \
  -H "Content-Type: application/json" \
  -d '{"assign": true}'
```

### 데이터베이스 확인
```bash
# Capability 목록
psql "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension" \
  -c "SELECT id, name, display_name, category FROM security_capability ORDER BY category, display_name;"

# 매핑 관계 확인
psql "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension" \
  -c "SELECT c.name as capability, p.resource_type, p.action FROM security_capability c JOIN security_capability_mapping cm ON c.id = cm.capability_id JOIN security_permission p ON cm.permission_id = p.id ORDER BY c.name;"
```

## 📊 데이터 현황

- **Roles**: 5개
- **Permissions**: 43개 (6개 카테고리)
- **Capabilities**: 20개 (6개 카테고리)
- **Capability-Permission Mappings**: 90개
- **Role-Capability Mappings**: 60개
- **Role-Permission Mappings**: 115개 (하위 호환성)

## 🛠️ 기술 스택

- **Backend**: Rust + Actix-web + SQLx + PostgreSQL
- **API 문서**: Utoipa (OpenAPI 3.0)
- **인증**: JWT + Keycloak
- **데이터베이스**: PostgreSQL 15+
- **캐싱**: Redis (선택사항)

## 📝 추가 리소스

- [기술 문서](../technical/)
- [UI 구현 가이드](../ui/)
- [데이터베이스 스키마](../database/)
- [배포 가이드](../deployment/)

## 🐛 문제 해결

### 서버가 시작되지 않는 경우
1. 포트 8080이 사용 중인지 확인: `lsof -i :8080`
2. 데이터베이스 연결 확인: `.env` 파일의 `DATABASE_URL` 설정
3. 로그 확인: `cargo run` 실행 시 출력되는 로그

### API 응답이 느린 경우
1. 데이터베이스 인덱스 확인
2. 캐시 설정 확인
3. 네트워크 연결 상태 확인

### 권한 관련 오류
1. 데이터베이스의 Role-Capability 매핑 확인
2. JWT 토큰 유효성 확인
3. Keycloak 서버 상태 확인

## 📞 지원

문제가 발생하거나 질문이 있으시면:
1. 로그 파일 확인
2. Swagger UI에서 API 테스트
3. 데이터베이스 상태 확인
4. 개발팀에 문의
