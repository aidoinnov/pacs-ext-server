use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 프로젝트 데이터 정보
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ProjectDataInfo {
    /// 데이터 ID
    pub id: i32,
    /// Study UID
    pub study_uid: String,
    /// Study 설명
    pub study_description: Option<String>,
    /// 환자 ID
    pub patient_id: Option<String>,
    /// 환자 이름
    pub patient_name: Option<String>,
    /// Study 날짜
    pub study_date: Option<String>,
    /// 모달리티
    pub modality: Option<String>,
}

/// 사용자 정보
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UserInfo {
    /// 사용자 ID
    pub id: i32,
    /// 사용자명
    pub username: String,
    /// 이메일
    pub email: String,
    /// 실명
    pub full_name: Option<String>,
    /// 소속 기관
    pub organization: Option<String>,
}

/// 데이터 접근 정보
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DataAccessInfo {
    /// 프로젝트 데이터 ID
    pub project_data_id: i32,
    /// 사용자 ID
    pub user_id: i32,
    /// 접근 상태
    pub status: String, // "APPROVED", "DENIED", "PENDING"
    /// 검토 시각
    pub reviewed_at: Option<String>,
    /// 검토자 ID
    pub reviewed_by: Option<i32>,
}

/// 페이지네이션 정보
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct PaginationInfo {
    /// 현재 페이지
    pub page: i32,
    /// 페이지 크기
    pub page_size: i32,
    /// 전체 항목 수
    pub total_items: i64,
    /// 전체 페이지 수
    pub total_pages: i64,
}

/// 프로젝트 데이터 접근 매트릭스 응답
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProjectDataAccessMatrixResponse {
    /// 데이터 목록
    pub data_list: Vec<ProjectDataInfo>,
    /// 사용자 목록
    pub users: Vec<UserInfo>,
    /// 접근 매트릭스
    pub access_matrix: Vec<DataAccessInfo>,
    /// 페이지네이션 정보
    pub pagination: PaginationInfo,
}

/// 프로젝트 데이터 생성 요청
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CreateProjectDataRequest {
    /// Study UID
    pub study_uid: String,
    /// Study 설명
    pub study_description: Option<String>,
    /// 환자 ID
    pub patient_id: Option<String>,
    /// 환자 이름
    pub patient_name: Option<String>,
    /// Study 날짜
    pub study_date: Option<String>,
    /// 모달리티
    pub modality: Option<String>,
}

/// 프로젝트 데이터 생성 응답
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CreateProjectDataResponse {
    /// 성공 여부
    pub success: bool,
    /// 메시지
    pub message: String,
    /// 생성된 데이터 ID
    pub data_id: Option<i32>,
}

/// 접근 권한 수정 요청
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateDataAccessRequest {
    /// 접근 상태
    pub status: String, // "APPROVED", "DENIED", "PENDING"
    /// 검토 사유
    pub review_note: Option<String>,
}

/// 접근 권한 수정 응답
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateDataAccessResponse {
    /// 성공 여부
    pub success: bool,
    /// 메시지
    pub message: String,
}

/// 일괄 접근 권한 수정 요청
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct BatchUpdateDataAccessRequest {
    /// 사용자 ID 목록
    pub user_ids: Vec<i32>,
    /// 접근 상태
    pub status: String, // "APPROVED", "DENIED", "PENDING"
    /// 검토 사유
    pub review_note: Option<String>,
}

/// 일괄 접근 권한 수정 응답
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct BatchUpdateDataAccessResponse {
    /// 성공 여부
    pub success: bool,
    /// 메시지
    pub message: String,
    /// 업데이트된 항목 수
    pub updated_count: i32,
}

/// 접근 요청 응답
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RequestDataAccessResponse {
    /// 성공 여부
    pub success: bool,
    /// 메시지
    pub message: String,
}

/// 프로젝트 데이터 목록 조회 요청
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetProjectDataListRequest {
    /// 페이지
    pub page: Option<i32>,
    /// 페이지 크기
    pub page_size: Option<i32>,
    /// 검색어
    pub search: Option<String>,
    /// 상태 필터
    pub status: Option<String>,
    /// 사용자 ID 필터
    pub user_id: Option<i32>,
}

/// 프로젝트 데이터 목록 응답
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProjectDataListResponse {
    /// 데이터 목록
    pub data_list: Vec<ProjectDataInfo>,
    /// 페이지네이션 정보
    pub pagination: PaginationInfo,
}

// ========== 새로운 계층 구조 DTO (향후 구현) ==========

/// 사용자 접근 셀
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UserAccessCell {
    /// 사용자 ID
    pub user_id: i32,
    /// 사용자명
    pub username: String,
    /// 접근 상태
    pub status: String, // "APPROVED", "DENIED", "PENDING"
}

/// 데이터 접근 매트릭스 행
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DataAccessMatrixRow {
    /// 데이터 ID (Study ID 또는 Series ID)
    pub data_id: String,
    /// 리소스 레벨
    pub resource_level: String, // "STUDY", "SERIES"
    /// Study UID
    pub study_uid: String,
    /// Series UID (Series 레벨인 경우에만)
    pub series_uid: Option<String>,
    /// Modality (Series 레벨)
    pub modality: Option<String>,
    /// 환자 ID
    pub patient_id: Option<String>,
    /// 환자 이름
    pub patient_name: Option<String>,
    /// Study 날짜
    pub study_date: Option<String>,
    /// 사용자별 접근 상태
    pub user_access: Vec<UserAccessCell>,
}

/// 데이터 접근 매트릭스 응답 (새 구조)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HierarchicalDataAccessMatrixResponse {
    /// 행 목록 (데이터별 사용자 접근 상태)
    pub rows: Vec<DataAccessMatrixRow>,
    /// 사용자 목록 (열 헤더용)
    pub users: Vec<UserInfo>,
    /// 데이터 페이지네이션 정보
    pub data_pagination: PaginationInfo,
    /// 사용자 페이지네이션 정보
    pub user_pagination: PaginationInfo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_data_info_serialization() {
        let data_info = ProjectDataInfo {
            id: 1,
            study_uid: "1.2.840.113619.2.1.1.322987881.621.736169244.616".to_string(),
            study_description: Some("CT Chest".to_string()),
            patient_id: Some("P001".to_string()),
            patient_name: Some("John Doe".to_string()),
            study_date: Some("2024-01-15".to_string()),
            modality: Some("CT".to_string()),
        };

        let json = serde_json::to_string(&data_info).unwrap();
        let deserialized: ProjectDataInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(data_info, deserialized);
    }

    #[test]
    fn test_user_info_serialization() {
        let user_info = UserInfo {
            id: 1,
            username: "user1".to_string(),
            email: "user1@example.com".to_string(),
            full_name: Some("홍길동".to_string()),
            organization: Some("서울대학교병원".to_string()),
        };

        let json = serde_json::to_string(&user_info).unwrap();
        let deserialized: UserInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(user_info, deserialized);
    }

    #[test]
    fn test_data_access_info_serialization() {
        let access_info = DataAccessInfo {
            project_data_id: 1,
            user_id: 1,
            status: "APPROVED".to_string(),
            reviewed_at: Some("2024-01-16T10:00:00Z".to_string()),
            reviewed_by: Some(2),
        };

        let json = serde_json::to_string(&access_info).unwrap();
        let deserialized: DataAccessInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(access_info, deserialized);
    }

    #[test]
    fn test_pagination_info_serialization() {
        let pagination = PaginationInfo {
            page: 1,
            page_size: 20,
            total_items: 50,
            total_pages: 3,
        };

        let json = serde_json::to_string(&pagination).unwrap();
        let deserialized: PaginationInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(pagination, deserialized);
    }

    #[test]
    fn test_create_project_data_request_serialization() {
        let request = CreateProjectDataRequest {
            study_uid: "1.2.840.113619.2.1.1.322987881.621.736169244.616".to_string(),
            study_description: Some("CT Chest".to_string()),
            patient_id: Some("P001".to_string()),
            patient_name: Some("John Doe".to_string()),
            study_date: Some("2024-01-15".to_string()),
            modality: Some("CT".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CreateProjectDataRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request, deserialized);
    }

    #[test]
    fn test_update_data_access_request_serialization() {
        let request = UpdateDataAccessRequest {
            status: "APPROVED".to_string(),
            review_note: Some("승인됨".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: UpdateDataAccessRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request, deserialized);
    }

    #[test]
    fn test_batch_update_data_access_request_serialization() {
        let request = BatchUpdateDataAccessRequest {
            user_ids: vec![1, 2, 3],
            status: "APPROVED".to_string(),
            review_note: Some("일괄 승인".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: BatchUpdateDataAccessRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request, deserialized);
    }
}
