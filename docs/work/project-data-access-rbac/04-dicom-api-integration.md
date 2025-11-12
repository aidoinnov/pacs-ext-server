# [RBAC 기능 보완] DICOM QIDO API에 project_data_access 접근 제어 적용

## 📅 작업 일자
2025-11-12

## 🎯 작업 목표

DICOM QIDO API (Studies, Series, Instances 조회)에 `project_data_access` 테이블 기반 접근 제어를 적용하여, 프로젝트 멤버가 특정 데이터만 접근하도록 제한할 수 있게 함.

---

## 🔍 문제 정의

### 기존 상황

**DICOM QIDO API 접근 제어**:
1. ✅ 기존 RBAC (Role-Based Access Control) 적용
2. ✅ `DICOM_GLOBAL_ACCESS` 권한 확인
3. ❌ `project_data_access` 테이블 확인 **없음**

**문제점**:
- 프로젝트 멤버는 프로젝트의 모든 DICOM 데이터에 접근 가능
- `project_data_access` 테이블에 제약이 있어도 DICOM API에서는 무시됨
- 민감한 데이터 보호 불가능

---

## ✅ 해결 방법

### 1. 헬퍼 함수 추가

**파일**: `pacs-server/src/presentation/controllers/dicom_gateway_controller.rs`

#### `can_access_study()` 함수

```rust
/// 사용자가 특정 Study에 접근 가능한지 확인 (project_data_access 테이블 기반)
/// 
/// 로직:
/// 1. project_data_access 테이블에 레코드가 없으면 → 전체 접근 가능 (기본)
/// 2. 레코드가 있으면 → 해당 레코드의 status와 expires_at 확인
///    - status = 'APPROVED' AND (expires_at IS NULL OR expires_at > NOW()) → 접근 가능
///    - 그 외 → 접근 불가
async fn can_access_study(
    user_id: i32,
    project_id: i32,
    study_uid: &str,
    pool: &sqlx::PgPool,
) -> bool
```

**동작 방식**:

1. **제약 없음 (기본)**:
   ```sql
   -- project_data_access 테이블에 레코드 없음
   SELECT EXISTS(
       SELECT 1 FROM project_data_access
       WHERE user_id = $1 AND project_id = $2
   )
   -- 결과: false → 전체 접근 가능 ✅
   ```

2. **제약 있음 (특정 Study만)**:
   ```sql
   -- 승인된 Study만 접근 가능
   SELECT EXISTS(
       SELECT 1 FROM project_data_access pda
       INNER JOIN project_data_study pds ON pda.study_id = pds.id
       WHERE pda.user_id = $1
         AND pda.project_id = $2
         AND pds.study_uid = $3
         AND pda.status = 'APPROVED'
         AND (pda.expires_at IS NULL OR pda.expires_at > NOW())
   )
   -- 결과: true/false → 승인된 Study만 접근 ✅
   ```

---

### 2. Studies 조회 API 수정

**엔드포인트**: `GET /api/dicom/studies`

**변경 내용**:

```rust
// 기존 RBAC 평가
let result = evaluator
    .evaluate_study_uid(user_id, pid, &study_uid)
    .await;

// project_data_access 테이블 확인 (추가)
let has_data_access = can_access_study(
    user_id,
    pid,
    &study_uid,
    project_data_repo.pool(),
)
.await;

// 두 조건 모두 만족해야 접근 가능
if result.allowed && has_data_access {
    allowed_items.push(item.clone());
}
```

**로직**:
1. 기존 RBAC 평가 (`evaluator.evaluate_study_uid()`)
2. `project_data_access` 테이블 확인 (`can_access_study()`)
3. **두 조건 모두 만족**해야 Study 반환

---

### 3. Series 조회 API 수정

**엔드포인트**: `GET /api/dicom/studies/{study_uid}/series`

**변경 내용**:

```rust
// 0. project_id가 있으면 Study 접근 권한 확인 (project_data_access)
if let Some(pid) = project_id_opt {
    let has_study_access = can_access_study(
        user_id,
        pid,
        &study_uid,
        project_data_repo.pool(),
    )
    .await;

    if !has_study_access {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied to this study"
        }));
    }
}
```

**로직**:
1. Series 조회 전에 **Study 접근 권한 확인**
2. 접근 불가 시 `403 Forbidden` 반환
3. 접근 가능 시 Series 목록 반환

---

### 4. Instances 조회 API 수정

**엔드포인트**: `GET /api/dicom/studies/{study_uid}/series/{series_uid}/instances`

**변경 내용**:

```rust
// 0. project_id가 있으면 Study 접근 권한 확인 (project_data_access)
if let Some(pid) = project_id_opt {
    let has_study_access = can_access_study(
        user_id,
        pid,
        &study_uid,
        project_data_repo.pool(),
    )
    .await;

    if !has_study_access {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied to this study"
        }));
    }
}
```

**로직**:
1. Instances 조회 전에 **Study 접근 권한 확인**
2. 접근 불가 시 `403 Forbidden` 반환
3. 접근 가능 시 Instances 목록 반환

---

## 🔐 접근 제어 흐름

### 전체 흐름도

```
사용자 요청
    ↓
1. 인증 확인 (JWT 토큰)
    ↓
2. DICOM_GLOBAL_ACCESS 권한 확인
    ├─ YES → 전체 접근 가능 (필터링 없음)
    └─ NO → 다음 단계
        ↓
3. project_id 확인
    ├─ 없음 → 403 Forbidden
    └─ 있음 → 다음 단계
        ↓
4. 기존 RBAC 평가 (evaluator)
    ├─ 거부 → 필터링
    └─ 허용 → 다음 단계
        ↓
5. project_data_access 테이블 확인 (NEW!)
    ├─ 레코드 없음 → 전체 접근 ✅
    └─ 레코드 있음 → 승인된 Study만 접근 ✅
        ↓
6. 최종 응답 반환
```

---

## 💡 사용 시나리오

### 시나리오 1: 일반 연구원 (제약 없음)

**설정**:
- 사용자 ID: 123
- 프로젝트 ID: 90
- `project_data_access` 테이블: 레코드 없음

**API 호출**:
```bash
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/dicom/studies?project_id=90"
```

**결과**:
```json
[
  { "0020000D": { "Value": ["1.2.840...study1"] } },
  { "0020000D": { "Value": ["1.2.840...study2"] } },
  { "0020000D": { "Value": ["1.2.840...study3"] } }
]
```
→ ✅ 모든 Study 반환 (제약 없음)

---

### 시나리오 2: 제한된 연구원 (특정 Study만)

**설정**:
- 사용자 ID: 456
- 프로젝트 ID: 90
- `project_data_access` 테이블:
  ```sql
  INSERT INTO project_data_access (user_id, project_id, study_id, status)
  VALUES (456, 90, 100, 'APPROVED');  -- study_uid = "1.2.840...study1"
  ```

**API 호출**:
```bash
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/dicom/studies?project_id=90"
```

**결과**:
```json
[
  { "0020000D": { "Value": ["1.2.840...study1"] } }
]
```
→ ✅ Study 1만 반환 (다른 Study는 필터링됨)

---

### 시나리오 3: Series 조회 (접근 거부)

**설정**:
- 사용자 ID: 456
- 프로젝트 ID: 90
- Study UID: "1.2.840...study2" (접근 불가)

**API 호출**:
```bash
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/dicom/studies/1.2.840...study2/series?project_id=90"
```

**결과**:
```json
{
  "error": "Access denied to this study"
}
```
→ ❌ 403 Forbidden (Study 접근 권한 없음)

---

## 🧪 테스트 방법

### 1. 제약 없는 사용자 테스트

```bash
# 1. 토큰 획득
TOKEN=$(curl -s -X POST "http://localhost:8080/api/auth/keycloak-token" \
  -H "Content-Type: application/json" \
  -d '{"username": "test_super_admin", "password": "TestAdmin123!"}' \
  | jq -r '.access_token')

# 2. Studies 조회
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/dicom/studies?project_id=90" | jq '.'

# 예상 결과: 모든 Study 반환
```

### 2. 제약 있는 사용자 테스트

```bash
# 1. project_data_access 레코드 추가
psql -h localhost -p 5456 -U pacs_extension_admin -d pacs_extension <<EOF
INSERT INTO project_data_access (user_id, project_id, study_id, status)
SELECT 123, 90, pds.id, 'APPROVED'
FROM project_data_study pds
WHERE pds.study_uid = '1.2.840.113619.2.55.3.604688119.868.1234567890.1'
LIMIT 1;
EOF

# 2. Studies 조회
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/dicom/studies?project_id=90" | jq '.'

# 예상 결과: 승인된 Study만 반환
```

### 3. Series 접근 거부 테스트

```bash
# 접근 불가한 Study의 Series 조회
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/dicom/studies/1.2.840...other_study/series?project_id=90" \
  | jq '.'

# 예상 결과: {"error": "Access denied to this study"}
```

---

## 📊 변경 사항 요약

### 수정된 파일

1. **`pacs-server/src/presentation/controllers/dicom_gateway_controller.rs`**
   - `can_access_study()` 헬퍼 함수 추가 (90줄)
   - `get_studies()` 함수 수정 (project_data_access 확인 추가)
   - `get_series()` 함수 수정 (Study 접근 권한 확인 추가)
   - `get_instances()` 함수 수정 (Study 접근 권한 확인 추가)

### 추가된 기능

1. ✅ `project_data_access` 테이블 기반 접근 제어
2. ✅ Study 레벨 접근 권한 확인
3. ✅ 만료 시간 (expires_at) 확인
4. ✅ 상태 (status) 확인 (APPROVED만 허용)
5. ✅ 로깅 추가 (디버깅 용이)

---

## 🎯 핵심 원칙

### 기본 = 전체 접근 ✅
```
project_data_access 테이블에 레코드 없음
→ 프로젝트 멤버는 모든 데이터 접근 가능
```

### 제약 = 특정 데이터만 접근 🔒
```
project_data_access 테이블에 레코드 있음
→ 승인된 Study만 접근 가능
→ 다른 Study는 필터링됨
```

### 이중 확인 🔐
```
1. 기존 RBAC 평가 (evaluator)
2. project_data_access 테이블 확인
→ 두 조건 모두 만족해야 접근 가능
```

---

## 🔍 로그 예시

### 제약 없는 사용자

```
DEBUG Gateway: No access restrictions for user 123 in project 90 → Full access granted
```

### 제약 있는 사용자 (승인됨)

```
DEBUG Gateway: User 456 has approved access to study 1.2.840...study1 in project 90
```

### 제약 있는 사용자 (거부됨)

```
DEBUG Gateway: User 456 does NOT have access to study 1.2.840...study2 in project 90 (restricted)
DEBUG Gateway: Study 1.2.840...study2 filtered out by project_data_access restrictions
```

---

## 🎉 결론

DICOM QIDO API에 `project_data_access` 테이블 기반 접근 제어가 성공적으로 적용되었습니다.

**주요 성과**:
1. ✅ 프로젝트 멤버의 데이터 접근을 세밀하게 제어 가능
2. ✅ 민감한 데이터 보호 강화
3. ✅ 기존 RBAC와 통합되어 이중 보안 제공
4. ✅ 만료 시간 및 상태 관리 지원

**기본 원칙 유지**:
- 제약 없음 = 전체 접근 (기본)
- 제약 있음 = 특정 데이터만 접근 (예외)

