# User-Project Matrix API

## 📋 API 개요

### 목적
유저-프로젝트 역할 관계를 **매트릭스(행렬) 형태**로 조회하는 API입니다. 관리 UI에서 테이블 형태로 표시하여 한눈에 모든 유저의 프로젝트 역할을 확인할 수 있습니다.

### 매트릭스 구조

```
┌─────────────┬──────────────┬──────────────┬──────────────┐
│   User      │  Project 1   │  Project 2   │  Project 3   │
├─────────────┼──────────────┼──────────────┼──────────────┤
│ User 1      │ ADMIN        │ (no role)    │ VIEWER       │
│ User 2      │ (no role)    │ MEMBER       │ ADMIN        │
│ User 3      │ VIEWER       │ MEMBER       │ (no role)    │
└─────────────┴──────────────┴──────────────┴──────────────┘
```

- **행 (Row)**: 유저 목록 (User)
- **열 (Column)**: 프로젝트 목록 (Project)
- **셀 (Cell)**: 해당 유저가 해당 프로젝트에서 가진 역할 (Role)
  - 역할이 없으면 `role_id: null`, `role_name: null`

### 조회 대상

- **전체 유저**: 시스템에 등록된 모든 유저 (페이지네이션 적용)
- **전체 프로젝트**: 시스템에 등록된 모든 프로젝트 (페이지네이션 적용)
- **필터링 가능**:
  - 특정 유저 ID 목록 (`user_ids`)
  - 특정 프로젝트 ID 목록 (`project_ids`)
  - 유저 이름/이메일 검색 (`user_search`)
  - 유저 정렬 (`user_sort_by`, `user_sort_order`)

### 주요 특징

1. **이중 페이지네이션 (Dual Pagination)**
   - 유저 페이지네이션: `user_page`, `user_page_size`
   - 프로젝트 페이지네이션: `project_page`, `project_page_size`
   - 각각 독립적으로 페이지 이동 가능

2. **성능 최적화**
   - 병렬 조회: 유저와 프로젝트를 동시에 조회 (`tokio::try_join!`)
   - N+1 쿼리 방지: 멤버십을 일괄 조회 후 HashMap으로 O(1) 조회
   - 최대 페이지 크기 제한: 50개 (과도한 데이터 로드 방지)

3. **유연한 필터링**
   - 유저 검색, 정렬, ID 필터링
   - 프로젝트 ID 필터링
   - 역할 ID 필터링 (향후 지원 예정)

---

## 🚀 API 엔드포인트

### GET /api/user-project-matrix

유저-프로젝트 역할 매트릭스를 조회합니다.

#### 요청 예시

```http
GET /api/user-project-matrix?user_page=1&user_page_size=10&project_page=1&project_page_size=10&user_sort_by=username&user_sort_order=asc
```

#### 쿼리 파라미터

| 파라미터 | 타입 | 필수 | 기본값 | 설명 |
|---------|------|------|--------|------|
| `user_page` | integer | ❌ | 1 | 유저 페이지 번호 |
| `user_page_size` | integer | ❌ | 10 | 유저 페이지 크기 (최대 50) |
| `project_page` | integer | ❌ | 1 | 프로젝트 페이지 번호 |
| `project_page_size` | integer | ❌ | 10 | 프로젝트 페이지 크기 (최대 50) |
| `user_sort_by` | string | ❌ | username | 유저 정렬 기준 (`username`, `email`, `created_at`) |
| `user_sort_order` | string | ❌ | asc | 정렬 순서 (`asc`, `desc`) |
| `user_search` | string | ❌ | - | 유저 이름/이메일 검색 (부분 일치) |
| `user_ids` | array[integer] | ❌ | - | 특정 유저 ID 목록 (예: `1,2,3`) |
| `project_ids` | array[integer] | ❌ | - | 특정 프로젝트 ID 목록 (예: `1,2,3`) |
| `role_id` | integer | ❌ | - | 특정 역할 ID 필터 (향후 지원) |

#### 응답 예시

```json
{
  "matrix": [
    {
      "user_id": 1,
      "username": "iaid-pacs-admin",
      "email": "heeya8876@naver.com",
      "full_name": "iaid-pacs-admin1",
      "project_roles": [
        {
          "project_id": 2,
          "project_name": "AI Image Analysis Project",
          "role_id": 183,
          "role_name": "PROJECT_ADMIN"
        },
        {
          "project_id": 3,
          "project_name": "Medical Research Project",
          "role_id": null,
          "role_name": null
        }
      ]
    },
    {
      "user_id": 6,
      "username": "kukkuk989",
      "email": "kukkuk989@protonmail.com",
      "full_name": "정희수",
      "project_roles": [
        {
          "project_id": 2,
          "project_name": "AI Image Analysis Project",
          "role_id": 184,
          "role_name": "MEMBER"
        },
        {
          "project_id": 3,
          "project_name": "Medical Research Project",
          "role_id": 185,
          "role_name": "VIEWER"
        }
      ]
    }
  ],
  "projects": [
    {
      "project_id": 2,
      "project_name": "AI Image Analysis Project",
      "description": "MRI 영상 기반 병변 탐지 연구 프로젝트",
      "status": "InProgress"
    },
    {
      "project_id": 3,
      "project_name": "Medical Research Project",
      "description": "의료 영상 연구 프로젝트",
      "status": "Preparing"
    }
  ],
  "pagination": {
    "user_page": 1,
    "user_page_size": 10,
    "user_total_count": 3,
    "user_total_pages": 1,
    "project_page": 1,
    "project_page_size": 10,
    "project_total_count": 2,
    "project_total_pages": 1
  }
}
```

---

## 📊 응답 데이터 구조

### UserProjectMatrixResponse

| 필드 | 타입 | 설명 |
|------|------|------|
| `matrix` | Array<UserProjectMatrixRow> | 매트릭스 행 목록 (유저별) |
| `projects` | Array<ProjectInfo> | 프로젝트 정보 목록 (열 헤더용) |
| `pagination` | UserProjectMatrixPagination | 페이지네이션 정보 |

### UserProjectMatrixRow

| 필드 | 타입 | 설명 |
|------|------|------|
| `user_id` | integer | 유저 ID |
| `username` | string | 유저명 |
| `email` | string | 이메일 |
| `full_name` | string \| null | 실명 (선택사항) |
| `project_roles` | Array<ProjectRoleCell> | 해당 유저의 프로젝트 역할 목록 |

### ProjectRoleCell

| 필드 | 타입 | 설명 |
|------|------|------|
| `project_id` | integer | 프로젝트 ID |
| `project_name` | string | 프로젝트명 |
| `role_id` | integer \| null | 역할 ID (역할이 없으면 null) |
| `role_name` | string \| null | 역할명 (역할이 없으면 null) |

### ProjectInfo

| 필드 | 타입 | 설명 |
|------|------|------|
| `project_id` | integer | 프로젝트 ID |
| `project_name` | string | 프로젝트명 |
| `description` | string \| null | 프로젝트 설명 |
| `status` | string | 프로젝트 상태 (`InProgress`, `Preparing`, `Completed`, etc.) |

### UserProjectMatrixPagination

| 필드 | 타입 | 설명 |
|------|------|------|
| `user_page` | integer | 현재 유저 페이지 번호 |
| `user_page_size` | integer | 유저 페이지 크기 |
| `user_total_count` | integer | 유저 총 개수 |
| `user_total_pages` | integer | 유저 총 페이지 수 |
| `project_page` | integer | 현재 프로젝트 페이지 번호 |
| `project_page_size` | integer | 프로젝트 페이지 크기 |
| `project_total_count` | integer | 프로젝트 총 개수 |
| `project_total_pages` | integer | 프로젝트 총 페이지 수 |

---

## 💡 사용 예시

### 1. 기본 조회 (첫 페이지)

```bash
curl "https://extension.pacs.ai-do.kr/api/user-project-matrix?user_page=1&user_page_size=10&project_page=1&project_page_size=10"
```

### 2. 유저 검색

```bash
curl "https://extension.pacs.ai-do.kr/api/user-project-matrix?user_search=admin&user_page=1&user_page_size=10"
```

### 3. 특정 유저만 조회

```bash
curl "https://extension.pacs.ai-do.kr/api/user-project-matrix?user_ids=1,2,3&project_page=1&project_page_size=10"
```

### 4. 특정 프로젝트만 조회

```bash
curl "https://extension.pacs.ai-do.kr/api/user-project-matrix?project_ids=2,3&user_page=1&user_page_size=10"
```

### 5. 유저 정렬 (이메일 기준 내림차순)

```bash
curl "https://extension.pacs.ai-do.kr/api/user-project-matrix?user_sort_by=email&user_sort_order=desc&user_page=1&user_page_size=10"
```

---

## 🎯 활용 시나리오

### 1. 관리자 대시보드
- 모든 유저의 프로젝트 역할을 한눈에 확인
- 역할 할당 현황 파악
- 미할당 유저 식별 (role_id가 null인 셀)

### 2. 프로젝트 멤버 관리
- 특정 프로젝트의 멤버 목록 확인
- 역할 변경이 필요한 유저 식별

### 3. 유저 권한 감사
- 특정 유저가 어떤 프로젝트에 접근 권한이 있는지 확인
- 과도한 권한을 가진 유저 식별

### 4. 대량 역할 할당
- 여러 유저에게 동시에 역할 할당 필요 시 현황 파악
- 역할 할당 전후 비교

---

## 📚 관련 문서

- [아키텍처 다이어그램](./architecture-diagram.md)
- [데이터 구조 다이어그램](./data-structure-diagram.md)
- [처리 흐름 다이어그램](./sequence-diagram.md)
- [성능 최적화 전략](./performance-optimization.md)
- [데이터베이스 스키마](./database-schema.md)
- [클라이언트 가이드](./client-guide.md)

---

## ⚠️ 주의사항

1. **페이지 크기 제한**: 최대 50개로 제한됩니다. 과도한 데이터 로드를 방지하기 위함입니다.
2. **성능 고려**: 유저와 프로젝트가 많을 경우 적절한 페이지 크기를 사용하세요.
3. **역할 null 처리**: 역할이 할당되지 않은 경우 `role_id`와 `role_name`이 `null`입니다.
4. **정렬 옵션**: 현재 유저 정렬만 지원하며, 프로젝트 정렬은 향후 추가 예정입니다.

---

## 🔄 버전 히스토리

- **v0.1.28** (2025-11-10): 초기 API 문서 작성

