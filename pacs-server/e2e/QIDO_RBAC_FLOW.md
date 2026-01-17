# QIDO RBAC 로직 순서

## 📋 개요

QIDO (Query based on ID for DICOM Objects) 엔드포인트에서 RBAC (Role-Based Access Control) 로직이 어떻게 동작하는지 설명합니다.

---

## 🔄 전체 흐름 (Studies 조회 기준)

### 1️⃣ **인증 (Authentication)**
```rust
// 사용자 ID 추출
let user_id = extract_user_id_from_request(&req, &jwt, &user_repo).await;
```

**위치**: `dicom_gateway_controller.rs:295`

**동작**:
- JWT 토큰에서 사용자 ID 추출
- 실패 시 → `401 Unauthorized` 반환

---

### 2️⃣ **전역 접근 권한 확인**
```rust
// 전체 데이터 조회 권한 확인
let has_global_access = has_global_dicom_access(user_id, pool).await;
```

**위치**: `dicom_gateway_controller.rs:306`

**동작**:
- `DICOM_GLOBAL_ACCESS` capability 확인
- 이 권한이 있으면 → 모든 데이터 접근 가능 (RBAC 필터링 스킵 가능)

---

### 3️⃣ **프로젝트 ID 검증**
```rust
let project_id_opt = query.project_id;

// 전체 데이터 조회 권한이 없으면 project_id 필수
if !has_global_access && project_id_opt.is_none() {
    return HttpResponse::BadRequest().json({
        "error": "project_id is required (no global access permission)"
    });
}
```

**위치**: `dicom_gateway_controller.rs:309-332`

**동작**:
- `has_global_access = false` → `project_id` 필수
- `has_global_access = true` → `project_id` 선택적

---

### 4️⃣ **QIDO 파라미터 구성**
```rust
// 1. 사용자 입력 파라미터 파싱
let user_params = build_qido_params_from_user_query(&extra_for_qido)?;

// 2. Access Condition 규칙 적용 (project_id가 있을 때만)
let qido_params = if let Some(pid) = project_id_opt {
    let conditions = access_condition_repo.list_by_project(pid).await?;
    let rule_params = build_qido_params_from_conditions(&conditions);
    merge_qido_params(rule_params, user_params) // 사용자 입력 우선
} else {
    user_params
};
```

**위치**: `dicom_gateway_controller.rs:353-382`

**동작**:
- 사용자 쿼리 파라미터 파싱 (limit, offset, 필터 등)
- 프로젝트별 Access Condition 규칙 적용
- 사용자 입력이 규칙보다 우선

---

### 5️⃣ **DCM4CHEE QIDO 호출**
```rust
let qido_response = qido
    .qido_studies_with_bearer(bearer_token, qido_params)
    .await?;
```

**위치**: `dicom_gateway_controller.rs:408-416`

**동작**:
- DCM4CHEE 서버에 QIDO-RS 요청
- Bearer 토큰 전달 (Keycloak 인증)
- 실패 시 → `502 Bad Gateway` 반환

---

### 6️⃣ **RBAC 필터링**

#### 6-1. 전역 접근 권한이 있고 project_id가 없는 경우
```rust
if has_global_access && project_id_opt.is_none() {
    // 필터링 없이 모든 데이터 반환
    return qido_response;
}
```

**위치**: `dicom_gateway_controller.rs:419-422`

**동작**: RBAC 필터링 스킵

---

#### 6-2. project_id가 있는 경우 (일반적인 경우)
```rust
else if let Some(pid) = project_id_opt {
    for item in qido_response.as_array() {
        let study_uid = extract_study_uid(item);
        
        // A. 기존 RBAC 평가
        let rbac_result = evaluator.evaluate_study_uid(user_id, pid, &study_uid).await;
        
        // B. project_data_access 테이블 확인
        let has_data_access = can_access_study(user_id, pid, &study_uid, pool).await;
        
        // C. 두 조건 모두 만족해야 접근 가능
        if rbac_result.allowed && has_data_access {
            allowed_items.push(item);
        }
    }
}
```

**위치**: `dicom_gateway_controller.rs:423-460`

**동작**:
1. **기존 RBAC 평가** (`evaluate_study_uid`)
2. **project_data_access 테이블 확인** (`can_access_study`)
3. **두 조건 모두 만족** → 접근 허용

---

## 🔍 상세: `evaluate_study_uid` (기존 RBAC)

**위치**: `dicom_rbac_evaluator_impl.rs:526-554`

### 단계별 평가 순서

#### 1. Study가 프로젝트에 속하는지 확인
```sql
SELECT pds.id
FROM project_data_study pds
INNER JOIN project_data pd ON pd.study_id = pds.id
WHERE pd.project_id = $1 AND pds.study_uid = $2
```

**결과**:
- Study가 프로젝트에 없으면 → `study_not_found_in_project` (거부)
- Study가 있으면 → `evaluate_study_access` 호출

---

#### 2. `evaluate_study_access` 평가

**위치**: `dicom_rbac_evaluator_impl.rs:306-430`

##### 2-1. 프로젝트 멤버십 확인 (필수)
```sql
SELECT EXISTS(
    SELECT 1 FROM security_user_project
    WHERE user_id = $1 AND project_id = $2
)
```

**결과**:
- 멤버가 아니면 → `user_not_project_member` (거부)
- 멤버이면 → 다음 단계

---

##### 2-2. 명시적 거부 확인 (최우선)
```sql
SELECT EXISTS(
    SELECT 1 FROM project_data_access
    WHERE user_id = $1 AND project_id = $2
    AND status = 'DENIED' AND resource_level = 'STUDY' AND study_id = $3
)
```

**결과**:
- `DENIED` 레코드가 있으면 → `explicitly_denied` (거부)
- 없으면 → 다음 단계

---

##### 2-3. 명시적 승인 확인
```sql
SELECT EXISTS(
    SELECT 1 FROM project_data_access
    WHERE user_id = $1 AND project_id = $2
    AND status = 'APPROVED' AND resource_level = 'STUDY' AND study_id = $3
    AND (expires_at IS NULL OR expires_at > NOW())
)
```

**결과**:
- `APPROVED` 레코드가 있으면 → `explicitly_approved` (허용)
- 없으면 → 다음 단계

---

##### 2-4. 기관 간 접근 권한 확인
```rust
// 사용자 기관
let user_institution_id = get_user_institution(user_id).await;

// 데이터 기관
let data_institution_id = get_study_institution(study_id).await;

// 같은 기관이면 허용
if user_institution_id == data_institution_id {
    return "same_institution" (허용);
}

// 다른 기관이면 cross-access 확인
SELECT EXISTS(
    SELECT 1 FROM security_institution_data_access
    WHERE user_institution_id = $1 AND data_institution_id = $2
    AND is_active = true
)
```

**결과**:
- 같은 기관 → `same_institution` (허용)
- Cross-access 권한 있음 → `institution_cross_access` (허용)
- 없으면 → 다음 단계

---

##### 2-5. 룰 기반 조건 평가 (Access Condition)
```rust
let dicom_values = get_study_dicom_values(study_id).await;
let rule_result = evaluate_rule_based_conditions(user_id, project_id, &dicom_values, "STUDY").await;
```

**동작**:
- Study의 DICOM 태그 값 추출 (Modality, BodyPart 등)
- Access Condition 규칙과 비교
- 규칙이 명시적으로 거부하면 → `rule_denied` (거부)
- 규칙이 허용하면 → `rule_allowed` (허용)

---

##### 2-6. 기본값: 프로젝트 멤버 접근 허용
```rust
// 명시적 DENIED가 없고, 다른 제약도 없으면 허용
return "project_member_default_access" (허용);
```

---

## 🔍 상세: `can_access_study` (project_data_access 확인)

**위치**: `dicom_gateway_controller.rs:195-264`

### 단계별 확인 순서

#### 1. project_data_access 레코드 존재 확인
```sql
SELECT EXISTS(
    SELECT 1 FROM project_data_access pda
    INNER JOIN project_data_study pds ON pda.study_id = pds.id
    WHERE pda.user_id = $1 AND pda.project_id = $2 AND pds.study_uid = $3
)
```

**결과**:
- 레코드가 **없으면** → **접근 허용** (기본값)
- 레코드가 **있으면** → 다음 단계

---

#### 2. APPROVED 상태 확인
```sql
SELECT EXISTS(
    SELECT 1 FROM project_data_access pda
    INNER JOIN project_data_study pds ON pda.study_id = pds.id
    WHERE pda.user_id = $1 AND pda.project_id = $2 AND pds.study_uid = $3
    AND pda.status = 'APPROVED'
    AND (pda.expires_at IS NULL OR pda.expires_at > NOW())
)
```

**결과**:
- `APPROVED` 상태이고 만료되지 않음 → **접근 허용**
- 그 외 → **접근 거부**

---

## 📊 최종 접근 결정

```
접근 허용 = (기존 RBAC 허용) AND (project_data_access 허용)
```

### 예시

| 기존 RBAC | project_data_access | 최종 결과 |
|-----------|---------------------|-----------|
| ✅ 허용    | ✅ 허용 (또는 레코드 없음) | ✅ **허용** |
| ✅ 허용    | ❌ 거부              | ❌ **거부** |
| ❌ 거부    | ✅ 허용              | ❌ **거부** |
| ❌ 거부    | ❌ 거부              | ❌ **거부** |

---

## 🎯 요약

### QIDO RBAC 로직 순서 (한눈에 보기)

1. **인증** → JWT 토큰에서 user_id 추출
2. **전역 권한 확인** → `DICOM_GLOBAL_ACCESS` capability
3. **프로젝트 ID 검증** → 전역 권한 없으면 필수
4. **QIDO 파라미터 구성** → Access Condition 규칙 + 사용자 입력
5. **DCM4CHEE 호출** → QIDO-RS 요청
6. **RBAC 필터링**:
   - **6-1. 기존 RBAC 평가** (`evaluate_study_uid`):
     1. 프로젝트 멤버십 확인
     2. 명시적 거부 확인
     3. 명시적 승인 확인
     4. 기관 간 접근 권한 확인
     5. 룰 기반 조건 평가
     6. 기본값: 프로젝트 멤버 허용
   - **6-2. project_data_access 확인** (`can_access_study`):
     1. 레코드 존재 확인 (없으면 허용)
     2. APPROVED 상태 확인
   - **6-3. 최종 결정**: 두 조건 모두 만족해야 허용

---

## 📖 관련 파일

- `pacs-server/src/presentation/controllers/dicom_gateway_controller.rs` - QIDO 엔드포인트
- `pacs-server/src/infrastructure/services/dicom_rbac_evaluator_impl.rs` - RBAC 평가 로직
- `pacs-server/src/domain/services/dicom_rbac_evaluator.rs` - RBAC 인터페이스

