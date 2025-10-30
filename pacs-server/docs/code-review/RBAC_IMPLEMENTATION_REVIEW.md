# DICOM Gateway RBAC 구현 검토 보고서

**검토 일자**: 2025-01-29  
**검토 대상**: DICOM Gateway RBAC 필터링 구현

## ✅ 완료된 부분

### 1. 핵심 기능 구현
- ✅ **Keycloak Bearer 토큰 릴레이**: 완전 구현됨
  - Authorization 헤더에서 Bearer 토큰 추출
  - Keycloak JWT 토큰 디코딩 (`sub` 필드 추출)
  - DB에서 `keycloak_id`로 사용자 조회
  - Dcm4chee로 토큰 릴레이

- ✅ **사용자 ID 추출**: 완전 구현됨
  - 우리 JWT 서비스 토큰 검증
  - Keycloak 토큰 디코딩 및 DB 매핑

- ✅ **프로젝트 멤버십 확인**: ✅ **방금 추가됨**
  - 모든 evaluate 메서드에 프로젝트 멤버십 확인 추가
  - `security_user_project` 테이블 조회

- ✅ **명시적 접근 권한 확인**: 완전 구현됨
  - Study/Series/Instance 레벨 명시적 권한 확인
  - 계층적 상속 (Instance → Series → Study)

- ✅ **기관 기반 접근**: 완전 구현됨
  - 같은 기관 접근
  - 기관 간 교차 접근 (`security_institution_data_access`)

### 2. 게이트웨이 컨트롤러
- ✅ **QIDO 파라미터 병합**: 완전 구현됨
  - AccessCondition → QIDO 파라미터 매핑
  - 규칙 기반 사전 필터링

- ✅ **JSON 사후 필터링**: 완전 구현됨
  - Study/Series/Instance UID 추출
  - Evaluator 기반 필터링

### 3. Repository 구현
- ✅ **AccessConditionRepository**: 완전 구현됨
  - `list_by_project`: 프로젝트별 조건 조회
  - `list_by_role`: 역할별 조건 조회

- ✅ **UserRepository**: 완전 구현됨
  - `find_by_keycloak_id`: Keycloak ID로 사용자 조회

## ⚠️ 부분 구현 / 개선 필요

### 1. 룰 기반 조건 평가 (TODO)
**위치**: `dicom_rbac_evaluator_impl.rs` 
- `evaluate_study_access`: 라인 97
- `evaluate_series_access`: 라인 154
- `evaluate_instance_access`: 라인 233

**현재 상태**: 
```rust
// 3) TODO: 룰 기반 조건 평가
RbacEvaluationResult { allowed: false, reason: Some("no_matching_policy".to_string()) }
```

**필요한 구현**:
- `security_access_condition` 조회
- `security_project_dicom_condition` 조회 (프로젝트별)
- `security_role_dicom_condition` 조회 (역할별)
- DICOM 태그 값 비교 (Modality, PatientID, StudyDate 등)
- 조건 평가 (EQ, RANGE 등)

**중요도**: 중간 (QIDO 사전 필터링은 이미 작동)

### 2. 하드코딩된 기본값
**위치**: `dicom_gateway_controller.rs`
- 라인 92: `project_id.unwrap_or(1)`
- 라인 93: `user_id.unwrap_or(1)`

**문제**: 
- 프로젝트 ID가 없으면 기본값 1 사용
- 사용자 ID 추출 실패 시 기본값 1 사용

**권장 개선**:
```rust
let project_id = query.project_id.ok_or_else(|| {
    HttpResponse::BadRequest().json(json!({"error": "project_id is required"}))
})?;

let user_id = extract_user_id_from_token(&req, &jwt, &user_repo).await
    .ok_or_else(|| HttpResponse::Unauthorized().json(json!({"error": "Invalid or missing token"})))?;
```

**중요도**: 높음 (프로덕션에서 보안 이슈 가능)

### 3. 에러 처리 개선
**위치**: `dicom_rbac_evaluator_impl.rs`
- 여러 곳에서 `.unwrap_or(false)` 사용
- DB 에러를 무시하고 `false` 반환

**권장 개선**:
```rust
let is_member: bool = sqlx::query_scalar(...)
    .fetch_one(&self.pool)
    .await
    .unwrap_or_else(|e| {
        tracing::error!("Failed to check project membership: {}", e);
        false
    });
```

**중요도**: 중간 (에러 로깅 강화)

### 4. UID 조회 시 project_id 필터링 ✅ **방금 개선됨**
**위치**: `dicom_rbac_evaluator_impl.rs`
- ✅ `evaluate_series_uid`: project_id로 필터링 추가됨
- ✅ `evaluate_instance_uid`: project_id로 필터링 추가됨

**개선 전**:
```rust
SELECT id FROM project_data_series WHERE series_uid = $1
```

**개선 후**:
```rust
SELECT pds.id FROM project_data_series pds
JOIN project_data_study pdt ON pds.study_id = pdt.id
WHERE pds.series_uid = $1 AND pdt.project_id = $2
```

## 📋 구현 완성도 요약

| 기능 | 상태 | 완성도 |
|------|------|--------|
| Keycloak 토큰 릴레이 | ✅ 완료 | 100% |
| 사용자 ID 추출 | ✅ 완료 | 100% |
| 프로젝트 멤버십 확인 | ✅ 완료 | 100% |
| 명시적 접근 권한 | ✅ 완료 | 100% |
| 기관 기반 접근 | ✅ 완료 | 100% |
| 계층적 상속 | ✅ 완료 | 100% |
| QIDO 파라미터 병합 | ✅ 완료 | 100% |
| JSON 사후 필터링 | ✅ 완료 | 100% |
| 룰 기반 조건 평가 | ⚠️ TODO | 0% |
| 하드코딩 제거 | ⚠️ 개선 필요 | 50% |
| 에러 처리 개선 | ⚠️ 개선 필요 | 70% |

## 🎯 권장 사항

### 즉시 수정 권장
1. **프로젝트 ID 필수 검증**: `project_id`가 없으면 400 반환
2. **사용자 ID 추출 실패 시 401 반환**: 기본값 1 사용 제거

### 다음 단계 구현
1. **룰 기반 조건 평가**: AccessCondition 기반 DICOM 태그 필터링
2. **에러 로깅 강화**: DB 에러 시 로깅 추가

### 테스트 권장
1. **프로젝트 멤버십 테스트**: 비멤버 접근 차단 확인
2. **다양한 시나리오 테스트**: 명시적 권한, 기관 접근, 룰 기반 필터링
