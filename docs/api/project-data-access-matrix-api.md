# Project Data Access Matrix API

## 개요

프로젝트별 데이터 접근 관리 매트릭스 API입니다. DICOM 데이터 (Study → Series → Modality)에 대한 사용자별 접근 권한을 관리합니다.

## 데이터 구조

### 테이블 구조

- **project_data_study**: DICOM Study 레벨 데이터
- **project_data_series**: DICOM Series 레벨 데이터  
- **project_data_access**: 사용자별 접근 권한 (Study, Series 레벨 지원)

### 접근 권한 레벨

- **STUDY**: Study 전체 접근 권한
- **SERIES**: 특정 Series 접근 권한
- **INSTANCE**: (향후 지원 예정)

### 접근 상태

- **APPROVED**: 접근 승인
- **DENIED**: 접근 거부
- **PENDING**: 접근 대기

## API 엔드포인트

### 1. 프로젝트 데이터 접근 매트릭스 조회

**GET** `/api/projects/{project_id}/data-access/matrix`

프로젝트별 데이터-사용자 접근 매트릭스를 조회합니다.

#### Path Parameters

- `project_id` (required): 프로젝트 ID

#### Query Parameters

- `page` (optional): 페이지 번호 (기본값: 1)
- `page_size` (optional): 페이지 크기 (기본값: 20)
- `search` (optional): 검색어 (Study UID, Patient ID, Patient Name)
- `status` (optional): 상태 필터 (APPROVED, DENIED, PENDING)
- `user_id` (optional): 사용자 ID 필터

#### Response

```json
{
  "data_list": [
    {
      "id": 1,
      "study_uid": "1.2.840.113619.2.1.1.322987881",
      "study_description": "CT Chest",
      "patient_id": "P001",
      "patient_name": "John Doe",
      "study_date": "2024-01-15",
      "modality": "CT"
    }
  ],
  "users": [
    {
      "id": 1,
      "username": "user1",
      "email": "user1@example.com",
      "full_name": "홍길동",
      "organization": "서울대학교병원"
    }
  ],
  "access_matrix": [
    {
      "project_data_id": 1,
      "user_id": 1,
      "status": "APPROVED",
      "reviewed_at": "2024-01-16T10:00:00Z",
      "reviewed_by": 2
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total_items": 50,
    "total_pages": 3
  }
}
```

### 2. 데이터 생성

**POST** `/api/projects/{project_id}/data`

새로운 DICOM Study 데이터를 생성합니다.

#### Request

```json
{
  "study_uid": "1.2.840.113619.2.1.1.322987881",
  "study_description": "CT Chest",
  "patient_id": "P001",
  "patient_name": "John Doe",
  "study_date": "2024-01-15",
  "modality": "CT"
}
```

#### Response

```json
{
  "success": true,
  "message": "Data created successfully",
  "data_id": 1
}
```

### 3. 개별 접근 권한 수정

**PUT** `/api/projects/{project_id}/data/{data_id}/access/{user_id}`

특정 사용자의 특정 데이터 접근 권한을 수정합니다.

#### Path Parameters

- `project_id`: 프로젝트 ID
- `data_id`: 데이터 ID
- `user_id`: 사용자 ID

#### Request

```json
{
  "status": "APPROVED",
  "review_note": "승인됨"
}
```

#### Response

```json
{
  "success": true,
  "message": "Access updated successfully"
}
```

### 4. 일괄 접근 권한 수정

**PUT** `/api/projects/{project_id}/data/{data_id}/access/batch`

특정 데이터에 대해 여러 사용자의 접근 권한을 일괄 수정합니다.

#### Path Parameters

- `project_id`: 프로젝트 ID
- `data_id`: 데이터 ID

#### Request

```json
{
  "user_ids": [1, 2, 3],
  "status": "APPROVED",
  "review_note": "일괄 승인"
}
```

#### Response

```json
{
  "success": true,
  "message": "Batch access updated successfully",
  "updated_count": 3
}
```

### 5. 접근 요청

**POST** `/api/projects/{project_id}/data/{data_id}/access/request`

새로운 접근 요청을 생성합니다.

#### Path Parameters

- `project_id`: 프로젝트 ID
- `data_id`: 데이터 ID

#### Response

```json
{
  "success": true,
  "message": "Access request submitted successfully"
}
```

### 6. 상태별 접근 권한 조회

**GET** `/api/data-access/status/{status}`

특정 상태의 접근 권한만 조회합니다.

#### Path Parameters

- `status`: 접근 상태 (APPROVED, DENIED, PENDING)

#### Query Parameters

- `page` (optional): 페이지 번호
- `page_size` (optional): 페이지 크기

#### Response

```json
{
  "data_list": [
    {
      "project_data_id": 1,
      "user_id": 1,
      "status": "APPROVED",
      "reviewed_at": "2024-01-16T10:00:00Z",
      "reviewed_by": 2
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total_items": 10,
    "total_pages": 1
  }
}
```

### 7. 사용자별 접근 권한 조회

**GET** `/api/users/{user_id}/data-access`

특정 사용자의 모든 접근 권한을 조회합니다.

#### Path Parameters

- `user_id`: 사용자 ID

#### Query Parameters

- `page` (optional): 페이지 번호
- `page_size` (optional): 페이지 크기

## UI 구현 가이드

### 표 구조

```
행: 데이터 (Study UID, Patient ID, Patient Name, Modality 등)
열: 사용자 (Username, Email 등)
셀: 접근 상태 (APPROVED/DENIED/PENDING)
```

### 필터링

1. **데이터 검색**: `search` 파라미터로 Study UID, Patient ID, Patient Name 검색
2. **상태 필터**: `status` 파라미터로 전체 표 필터링
3. **사용자 필터**: `user_id` 파라미터로 특정 사용자 상태만 표시

### 페이지네이션

현재는 데이터 행에 대한 페이지네이션만 지원됩니다.  
향후 사용자 컬럼 페이지네이션이 추가될 예정입니다.

### 일괄 작업

1. 여러 데이터 선택 → `batch_update` API로 일괄 승인/거부
2. 특정 데이터 선택 → 여러 사용자 일괄 승인/거부

## 향후 계획

### 계층 구조 지원

- Study → Series → Modality 3단계 계층 구조
- 각 레벨별 세밀한 접근 권한 제어

### 고급 기능

- 양방향 페이지네이션 (데이터 행 + 사용자 열)
- 데이터 필터링 강화 (Modality, Study Date 범위 등)
- 접근 패턴 분석 (자주 접근하는 데이터, 사용자 패턴 등)

### 성능 최적화

- N+1 쿼리 방지 (배치 조회)
- 인덱스 최적화
- 캐싱 전략

## 에러 코드

- `404`: 프로젝트, 데이터, 사용자를 찾을 수 없음
- `400`: 잘못된 요청 (검증 실패)
- `409`: 이미 존재하는 데이터
- `500`: 서버 내부 오류

