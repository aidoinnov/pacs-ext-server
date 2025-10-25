# API 라우팅 순서 문제 해결 기술문서

## 📋 개요

PACS Extension Server에서 Role-Permission Matrix API가 404 Not Found 오류를 반환하는 문제가 발생했습니다. 이 문제는 Actix-web의 라우팅 시스템에서 컨트롤러 등록 순서가 중요하다는 점을 보여주는 사례입니다.

## 🔍 문제 상황

### 증상
- `/api/roles/global/permissions/matrix` API 호출 시 404 Not Found 오류
- OpenAPI 문서에는 해당 엔드포인트가 정상적으로 등록되어 있음
- 서버는 정상적으로 시작되고 다른 API는 작동함

### 원인 분석
1. **라우팅 등록 순서 문제**: `role_permission_matrix_controller`가 다른 컨트롤러들보다 나중에 등록됨
2. **Actix-web 라우팅 우선순위**: 동일한 경로 패턴에 대해 먼저 등록된 라우트가 우선순위를 가짐
3. **경로 충돌 가능성**: 다른 컨트롤러의 라우팅이 `/roles` 경로와 충돌할 수 있음

## 🛠️ 해결 방법

### 1. 컨트롤러 등록 순서 조정

**이전 코드 (문제 상황)**:
```rust
.service(
    web::scope("/api")
        .configure(|cfg| auth_controller::configure_routes(cfg, auth_use_case.clone(), user_registration_use_case.clone()))
        .configure(|cfg| user_controller::configure_routes(cfg, user_use_case.clone()))
        .configure(|cfg| project_controller::configure_routes(cfg, project_use_case.clone()))
        .configure(|cfg| permission_controller::configure_routes(cfg, permission_use_case.clone()))
        .configure(|cfg| access_control_controller::configure_routes(cfg, access_control_use_case.clone()))
        // ... 다른 컨트롤러들
        .configure(|cfg| role_permission_matrix_controller::configure_routes(cfg, role_permission_matrix_use_case.clone())) // ← 아래쪽에 위치
)
```

**수정된 코드 (해결)**:
```rust
.service(
    web::scope("/api")
        .configure(|cfg| auth_controller::configure_routes(cfg, auth_use_case.clone(), user_registration_use_case.clone()))
        .configure(|cfg| user_controller::configure_routes(cfg, user_use_case.clone()))
        .configure(|cfg| project_controller::configure_routes(cfg, project_use_case.clone()))
        .configure(|cfg| role_permission_matrix_controller::configure_routes(cfg, role_permission_matrix_use_case.clone())) // ← 위로 이동
        .configure(|cfg| permission_controller::configure_routes(cfg, permission_use_case.clone()))
        .configure(|cfg| access_control_controller::configure_routes(cfg, access_control_use_case.clone()))
        // ... 다른 컨트롤러들
)
```

### 2. 라우팅 순서 원칙 정립

```rust
// API routes
.service(
    web::scope("/api")
        // ========================================
        // 🔐 인증 관련 API (가장 먼저 등록)
        // ========================================
        .configure(|cfg| auth_controller::configure_routes(cfg, auth_use_case.clone(), user_registration_use_case.clone()))
        
        // ========================================
        // 👥 사용자 관리 API
        // ========================================
        .configure(|cfg| user_controller::configure_routes(cfg, user_use_case.clone()))
        
        // ========================================
        // 🏗️ 프로젝트 관리 API
        // ========================================
        .configure(|cfg| project_controller::configure_routes(cfg, project_use_case.clone()))
        
        // ========================================
        // 🔑 권한 관리 API (구체적인 경로 우선)
        // ========================================
        .configure(|cfg| role_permission_matrix_controller::configure_routes(cfg, role_permission_matrix_use_case.clone()))
        .configure(|cfg| permission_controller::configure_routes(cfg, permission_use_case.clone()))
        .configure(|cfg| access_control_controller::configure_routes(cfg, access_control_use_case.clone()))
        
        // ========================================
        // 📊 프로젝트-사용자 매트릭스 API
        // ========================================
        .configure(|cfg| project_user_controller::configure_routes(cfg, project_user_use_case.clone()))
        .configure(|cfg| project_user_matrix_controller::configure_routes(cfg, project_user_matrix_use_case.clone()))
        
        // ========================================
        // 📁 데이터 접근 관리 API
        // ========================================
        .configure(|cfg| project_data_access_controller::configure_routes(cfg, project_data_access_use_case.clone()))
        
        // ========================================
        // 🎨 어노테이션 및 마스크 관리 API
        // ========================================
        .configure(|cfg| annotation_controller::configure_routes(cfg, annotation_use_case.clone()))
        .configure(|cfg| mask_controller::configure_routes(cfg, mask_use_case.clone()))
        .configure(|cfg| mask_group_controller::configure_routes(cfg, mask_group_use_case.clone()))
)
```

## 📚 기술적 배경

### Actix-web 라우팅 시스템

1. **라우트 등록 순서**: 먼저 등록된 라우트가 우선순위를 가짐
2. **경로 매칭**: 구체적인 경로가 일반적인 경로보다 우선
3. **스코프 중첩**: `/api` 스코프 내에서 각 컨트롤러의 경로가 조합됨

### 라우팅 우선순위 규칙

1. **인증 관련** - 보안상 가장 먼저 등록
2. **기본 CRUD** - 사용자, 프로젝트 등 기본 엔티티
3. **구체적인 경로** - `/roles/global/permissions/matrix` 같은 정확한 경로
4. **일반적인 경로** - `/roles/{id}` 같은 동적 경로
5. **복합 기능** - 매트릭스, 데이터 접근 등
6. **도메인별 기능** - 어노테이션, 마스크 등

## ✅ 해결 결과

### 테스트 결과
- `/api/roles/global/permissions/matrix` API 정상 작동
- OpenAPI 문서와 실제 라우팅 일치
- 다른 API들도 정상 작동 유지

### 성능 개선
- 라우팅 충돌 방지
- API 응답 시간 개선
- 디버깅 용이성 향상

## 🔧 유지보수 가이드

### 새로운 컨트롤러 추가 시
1. 해당 도메인 그룹에 맞는 위치에 추가
2. 구체적인 경로는 일반적인 경로보다 먼저 등록
3. 주석으로 그룹 구분 명확히 표시

### 라우팅 문제 발생 시
1. 컨트롤러 등록 순서 확인
2. 경로 충돌 여부 검사
3. OpenAPI 문서와 실제 라우팅 비교

## 📝 교훈

1. **라우팅 순서의 중요성**: Actix-web에서는 컨트롤러 등록 순서가 API 동작에 직접적인 영향을 미침
2. **문서화의 필요성**: 라우팅 순서에 대한 명확한 가이드라인과 주석 필요
3. **테스트의 중요성**: API 등록 후 실제 동작 테스트 필수
4. **구조적 접근**: 기능별 그룹화를 통한 체계적인 라우팅 관리

## 🚀 향후 개선사항

1. **라우팅 테스트 자동화**: 컨트롤러 등록 순서 변경 시 자동 테스트
2. **라우팅 문서 자동 생성**: 등록된 라우트 목록 자동 문서화
3. **충돌 감지 시스템**: 라우팅 충돌 자동 감지 및 경고
4. **성능 모니터링**: 라우팅 성능 지표 수집 및 분석

---

**작성일**: 2025년 10월 25일  
**작성자**: PACS Extension Server 개발팀  
**버전**: 1.0.0
