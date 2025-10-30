# PACS Server DTO 작성 가이드

## 📋 목차
1. [DTO 개요](#dto-개요)
2. [기본 패턴](#기본-패턴)
3. [공통 구조](#공통-구조)
4. [네이밍 컨벤션](#네이밍-컨벤션)
5. [주요 키워드와 문법](#주요-키워드와-문법)
6. [예외 패턴](#예외-패턴)
7. [실제 예제 분석](#실제-예제-분석)
8. [연습 문제](#연습-문제)
9. [체크리스트](#체크리스트)

---

## DTO 개요

### DTO란?
**Data Transfer Object** - 계층 간 데이터 전송을 위한 객체로, API 요청/응답, 서비스 간 데이터 전달에 사용됩니다.

### 프로젝트에서의 역할
- **API 계층**: HTTP 요청/응답 데이터 구조 정의
- **서비스 계층**: 비즈니스 로직 간 데이터 전달
- **도메인 계층**: 엔티티와 DTO 간 변환

---

## 기본 패턴

### 1. 표준 DTO 구조
```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::NaiveDateTime;

/// [기능] [타입] DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct [Name]Request {
    pub field1: Type,
    pub field2: Option<Type>,
}

/// [기능] [타입] DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct [Name]Response {
    pub id: i32,
    pub field1: Type,
    pub created_at: NaiveDateTime,
}
```

### 2. 공통 패턴 분류

#### A. 요청 DTO 패턴
- `Create[Entity]Request` - 생성 요청
- `Update[Entity]Request` - 업데이트 요청
- `List[Entity]Request` - 목록 조회 요청
- `[Action][Entity]Request` - 특정 액션 요청

#### B. 응답 DTO 패턴
- `[Entity]Response` - 단일 엔티티 응답
- `[Entity]ListResponse` - 목록 응답
- `[Action]Response` - 액션 결과 응답

---

## 공통 구조

### 1. 필수 Import 패턴
```rust
use serde::{Deserialize, Serialize};  // 직렬화/역직렬화
use utoipa::ToSchema;                 // OpenAPI 스키마 생성
use uuid::Uuid;                       // UUID 타입
use chrono::NaiveDateTime;            // 날짜/시간 타입
```

### 2. 표준 Derive 매크로
```rust
#[derive(Debug, Deserialize, Serialize, ToSchema)]
```
- `Debug`: 디버깅용 출력
- `Deserialize`: JSON → 구조체 변환
- `Serialize`: 구조체 → JSON 변환
- `ToSchema`: OpenAPI 문서 생성

### 3. 공통 필드 패턴

#### ID 필드
```rust
pub id: i32,                    // 기본 ID
pub keycloak_id: Uuid,          // Keycloak 사용자 ID
pub user_id: i32,               // 사용자 ID
pub project_id: i32,            // 프로젝트 ID
```

#### 시간 필드
```rust
#[schema(value_type = String, example = "2024-01-01T00:00:00")]
pub created_at: NaiveDateTime,
pub updated_at: NaiveDateTime,
```

#### 옵셔널 필드
```rust
pub description: Option<String>,    // 설명
pub is_active: Option<bool>,        // 활성 상태
pub expires_in: Option<u64>,        // 만료 시간
```

---

## 네이밍 컨벤션

### 1. 구조체 명명
- **요청**: `[Action][Entity]Request`
- **응답**: `[Entity]Response` 또는 `[Action]Response`
- **목록**: `[Entity]ListResponse`

### 2. 필드 명명
- **snake_case** 사용
- **의미있는 이름** 사용
- **일관성** 유지

### 3. 예시
```rust
// ✅ 좋은 예
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
}

// ❌ 나쁜 예
pub struct UserCreateReq {
    pub user_name: String,
    pub e_mail: String,
}
```

---

## 주요 키워드와 문법

### 1. Serde 어노테이션
```rust
#[serde(rename = "fieldName")]        // JSON 필드명 변경
#[serde(skip_serializing_if = "Option::is_none")]  // None 값 직렬화 스킵
#[serde(default)]                     // 기본값 사용
```

### 2. Utoipa 어노테이션
```rust
#[schema(example = "example_value")]           // 예시 값
#[schema(value_type = String)]                 // 타입 명시
#[schema(description = "필드 설명")]            // 필드 설명
```

### 3. Option 타입 사용
```rust
pub field: Option<String>,           // 선택적 필드
pub field: Option<i32>,              // 선택적 숫자
pub field: Option<Vec<String>>,      // 선택적 배열
```

### 4. From 트레이트 구현
```rust
impl From<crate::domain::entities::user::User> for UserResponse {
    fn from(user: crate::domain::entities::user::User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}
```

---

## 예외 패턴

### 1. ToSchema 없는 DTO
```rust
// 내부용 DTO (API 문서에 노출되지 않음)
#[derive(Debug, Deserialize, Serialize)]
pub struct InternalDataTransfer {
    pub data: String,
}
```

### 2. Serialize만 있는 DTO
```rust
// 응답 전용 DTO (요청 받지 않음)
#[derive(Debug, Serialize, ToSchema)]
pub struct MaskResponse {
    pub id: i32,
    pub file_path: String,
}
```

### 3. Deserialize만 있는 DTO
```rust
// 요청 전용 DTO (응답하지 않음)
#[derive(Debug, Deserialize, ToSchema)]
pub struct DownloadUrlRequest {
    pub mask_id: i32,
    pub file_path: String,
}
```

### 4. 복잡한 JSON 필드
```rust
// JSON 데이터를 직접 처리
pub annotation_data: serde_json::Value,
```

### 5. HashMap 사용
```rust
use std::collections::HashMap;

pub masks_by_label: HashMap<String, i64>,
pub mime_type_distribution: HashMap<String, i64>,
```

---

## 실제 예제 분석

### 1. 사용자 DTO (user_dto.rs)
```rust
/// 사용자 생성 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateUserRequest {
    pub keycloak_id: Uuid,      // UUID 타입
    pub username: String,        // 필수 문자열
    pub email: String,          // 필수 문자열
}

/// 사용자 업데이트 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateUserRequest {
    pub email: Option<String>,  // 선택적 업데이트
}

/// 사용자 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i32,                // 데이터베이스 ID
    pub keycloak_id: Uuid,      // 외부 시스템 ID
    pub username: String,
    pub email: String,
    #[schema(value_type = String, example = "2024-01-01T00:00:00")]
    pub created_at: NaiveDateTime,  // 스키마 타입 명시
}
```

**패턴 분석:**
- ✅ 표준 네이밍 컨벤션
- ✅ 적절한 타입 선택 (Uuid, String, i32)
- ✅ 스키마 어노테이션 사용
- ✅ From 트레이트 구현

### 2. 어노테이션 DTO (annotation_dto.rs)
```rust
/// Annotation 생성 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateAnnotationRequest {
    /// Study Instance UID
    #[schema(example = "1.2.840.113619.2.55.3.604688119.868.1234567890.1")]
    pub study_instance_uid: String,

    /// Annotation 데이터 (JSON 형식)
    #[schema(example = json!({"type": "circle", "x": 100, "y": 200, "radius": 50}))]
    pub annotation_data: serde_json::Value,  // 복잡한 JSON 데이터

    /// 측정 도구 이름
    #[schema(example = "Circle Tool")]
    pub tool_name: Option<String>,           // 선택적 필드
}
```

**패턴 분석:**
- ✅ 상세한 문서화 주석
- ✅ 복잡한 JSON 데이터 처리
- ✅ DICOM 표준 필드명 사용
- ✅ 예시 값 제공

### 3. 마스크 DTO (mask_dto.rs)
```rust
/// 마스크 목록 조회 요청 DTO
#[derive(Debug, Deserialize, ToSchema)]  // Serialize 없음 (요청 전용)
pub struct ListMasksRequest {
    #[schema(example = 1)]
    pub page: Option<i32>,               // 페이지네이션
    
    #[schema(example = 20)]
    pub page_size: Option<i32>,
    
    #[schema(example = "liver")]
    pub label_name: Option<String>,      // 필터링 옵션
}

/// 마스크 목록 응답 DTO
#[derive(Debug, Serialize, ToSchema)]    // Deserialize 없음 (응답 전용)
pub struct MaskListResponse {
    pub masks: Vec<MaskResponse>,
    pub total_count: i64,
    pub current_page: i32,
    pub total_pages: i32,
}
```

**패턴 분석:**
- ✅ 요청/응답 분리
- ✅ 페이지네이션 구조
- ✅ 적절한 derive 매크로 선택

---

## 연습 문제

### 문제 1: 기본 DTO 작성
다음 요구사항에 맞는 DTO를 작성하세요:

**요구사항:**
- 병원 정보를 생성하는 API
- 필수 필드: name, address, phone
- 선택 필드: description, website
- 응답에는 id, created_at 포함

<details>
<summary>정답 보기</summary>

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::NaiveDateTime;

/// 병원 생성 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateHospitalRequest {
    pub name: String,
    pub address: String,
    pub phone: String,
    pub description: Option<String>,
    pub website: Option<String>,
}

/// 병원 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct HospitalResponse {
    pub id: i32,
    pub name: String,
    pub address: String,
    pub phone: String,
    pub description: Option<String>,
    pub website: Option<String>,
    #[schema(value_type = String, example = "2024-01-01T00:00:00")]
    pub created_at: NaiveDateTime,
}
```

</details>

### 문제 2: 복잡한 DTO 작성
다음 요구사항에 맞는 DTO를 작성하세요:

**요구사항:**
- 의료진 검색 API
- 필터: department, specialty, experience_years
- 정렬: name, experience_years
- 페이지네이션: page, page_size
- 응답: 의료진 목록 + 통계 정보

<details>
<summary>정답 보기</summary>

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::NaiveDateTime;
use std::collections::HashMap;

/// 의료진 검색 요청 DTO
#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchDoctorsRequest {
    #[schema(example = "cardiology")]
    pub department: Option<String>,
    
    #[schema(example = "interventional")]
    pub specialty: Option<String>,
    
    #[schema(example = 5)]
    pub min_experience_years: Option<i32>,
    
    #[schema(example = "name")]
    pub sort_by: Option<String>,
    
    #[schema(example = 1)]
    pub page: Option<i32>,
    
    #[schema(example = 20)]
    pub page_size: Option<i32>,
}

/// 의료진 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct DoctorResponse {
    pub id: i32,
    pub name: String,
    pub department: String,
    pub specialty: String,
    pub experience_years: i32,
    pub license_number: String,
    #[schema(value_type = String, example = "2024-01-01T00:00:00")]
    pub created_at: NaiveDateTime,
}

/// 의료진 검색 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct DoctorSearchResponse {
    pub doctors: Vec<DoctorResponse>,
    pub total_count: i64,
    pub current_page: i32,
    pub total_pages: i32,
    pub statistics: DoctorStatistics,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DoctorStatistics {
    pub total_doctors: i64,
    pub by_department: HashMap<String, i64>,
    pub by_specialty: HashMap<String, i64>,
    pub average_experience: f64,
}
```

</details>

### 문제 3: From 트레이트 구현
다음 엔티티에서 DTO로 변환하는 From 트레이트를 구현하세요:

```rust
// 엔티티
pub struct Patient {
    pub id: i32,
    pub name: String,
    pub birth_date: NaiveDateTime,
    pub gender: String,
    pub phone: Option<String>,
}

// DTO
pub struct PatientResponse {
    pub id: i32,
    pub name: String,
    pub age: i32,  // birth_date에서 계산
    pub gender: String,
    pub phone: Option<String>,
}
```

<details>
<summary>정답 보기</summary>

```rust
impl From<Patient> for PatientResponse {
    fn from(patient: Patient) -> Self {
        let age = chrono::Utc::now().year() - patient.birth_date.year();
        
        Self {
            id: patient.id,
            name: patient.name,
            age: age as i32,
            gender: patient.gender,
            phone: patient.phone,
        }
    }
}
```

</details>

### 문제 4: 예외 상황 처리
다음 상황에서 적절한 DTO를 작성하세요:

**상황:**
- 파일 업로드 API
- 요청: 파일 + 메타데이터
- 응답: 업로드 결과 + 다운로드 URL
- 에러: 파일 크기 제한, 형식 제한

<details>
<summary>정답 보기</summary>

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 파일 업로드 요청 DTO
#[derive(Debug, Deserialize, ToSchema)]
pub struct UploadFileRequest {
    #[schema(example = "document.pdf")]
    pub filename: String,
    
    #[schema(example = "application/pdf")]
    pub content_type: String,
    
    #[schema(example = "medical_report")]
    pub category: Option<String>,
    
    #[schema(example = "Patient medical report")]
    pub description: Option<String>,
}

/// 파일 업로드 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct UploadFileResponse {
    pub file_id: i32,
    pub filename: String,
    pub file_size: i64,
    pub download_url: String,
    pub expires_at: String,
    pub upload_status: UploadStatus,
}

#[derive(Debug, Serialize, ToSchema)]
pub enum UploadStatus {
    Success,
    Warning,
    Error,
}

/// 파일 업로드 에러 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct UploadErrorResponse {
    pub error_code: String,
    pub error_message: String,
    pub details: Option<serde_json::Value>,
}
```

</details>

---

## 체크리스트

### DTO 작성 전 체크리스트
- [ ] 요구사항이 명확한가?
- [ ] API 엔드포인트가 정의되었는가?
- [ ] 데이터베이스 스키마가 확인되었는가?

### DTO 작성 중 체크리스트
- [ ] 적절한 derive 매크로 사용
- [ ] 네이밍 컨벤션 준수
- [ ] 필수/선택 필드 구분
- [ ] 타입 선택이 적절한가?
- [ ] 스키마 어노테이션 추가
- [ ] 문서화 주석 작성

### DTO 작성 후 체크리스트
- [ ] From 트레이트 구현 (필요시)
- [ ] 테스트 작성
- [ ] API 문서 확인
- [ ] 코드 리뷰

---

## 추가 학습 자료

### 관련 문서
- [Serde 공식 문서](https://serde.rs/)
- [Utoipa 공식 문서](https://docs.rs/utoipa/latest/utoipa/)
- [Chrono 공식 문서](https://docs.rs/chrono/latest/chrono/)

### 프로젝트 내 관련 파일
- `src/application/dto/mod.rs` - DTO 모듈 정의
- `src/domain/entities/` - 도메인 엔티티
- `src/presentation/controllers/` - 컨트롤러에서 DTO 사용 예시

---

## 마무리

이 가이드를 통해 PACS Server 프로젝트의 DTO 작성 패턴을 익혔습니다. 
실제 개발 시에는 이 패턴을 참고하여 일관성 있는 DTO를 작성하고, 
예외 상황에서는 프로젝트의 기존 코드를 참고하여 적절한 해결책을 찾으세요.

**핵심 포인트:**
1. **일관성** - 프로젝트의 기존 패턴을 따르세요
2. **명확성** - 필드명과 타입을 명확하게 정의하세요
3. **문서화** - API 문서 생성을 위한 어노테이션을 활용하세요
4. **유연성** - Option 타입을 적절히 활용하세요
