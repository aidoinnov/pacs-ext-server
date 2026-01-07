use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Viewer Study Meta Batch API Request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ViewerStudyMetaRequest {
    /// StudyInstanceUID 목록
    #[schema(example = json!(["1.2.840.113619.2.55.3.604688433.1234", "1.2.840.113619.2.55.3.604688433.5678"]))]
    pub study_uids: Vec<String>,

    /// 페이지 번호 (1부터 시작, 기본값: 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 1)]
    pub page: Option<i32>,

    /// 페이지 크기 (기본값: 50, 최소: 1, 최대: 200)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 50)]
    pub page_size: Option<i32>,
}

/// Viewer Study Meta Response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ViewerStudyMetaResponse {
    /// Study 메타데이터 목록
    pub studies: Vec<ViewerStudyMeta>,

    /// 페이지네이션 정보
    pub pagination: ViewerPaginationInfo,
}

/// Viewer Study Meta DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ViewerStudyMeta {
    /// StudyInstanceUID
    #[schema(example = "1.2.840.113619.2.55.3.604688433.1234")]
    pub study_uid: String,

    /// StudyDate (YYYYMMDD)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "20240115")]
    pub study_date: Option<String>,

    /// StudyTime (HHMMSS)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "093012")]
    pub study_time: Option<String>,

    /// StudyDescription
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Chest CT")]
    pub study_description: Option<String>,

    /// PatientName
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "DOE^JOHN")]
    pub patient_name: Option<String>,

	    /// PatientID
	    #[serde(skip_serializing_if = "Option::is_none")]
	    #[schema(example = "P123456")]
	    pub patient_id: Option<String>,

	    /// PatientSex (0010,0040)
	    #[serde(skip_serializing_if = "Option::is_none")]
	    #[schema(example = "M")]
	    pub patient_sex: Option<String>,

	    /// PatientAge (0010,1010, e.g. "032Y")
	    #[serde(skip_serializing_if = "Option::is_none")]
	    #[schema(example = "032Y")]
	    pub patient_age: Option<String>,

	    /// PatientBirthDate (0010,0030, YYYYMMDD)
	    #[serde(skip_serializing_if = "Option::is_none")]
	    #[schema(example = "19851224")]
	    pub patient_birth_date: Option<String>,

	    /// Modality 목록
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = json!(["CT"]))]
    pub modalities_in_study: Option<Vec<String>>,

    /// Series 개수
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 3)]
    pub number_of_series: Option<i32>,

    /// Instance 개수
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 245)]
    pub number_of_instances: Option<i32>,
}

impl ViewerStudyMeta {
    /// DICOMweb JSON에서 ViewerStudyMeta로 변환
    pub fn from_dicomweb_json(json: &serde_json::Value) -> Self {
	        Self {
	            study_uid: extract_string_value(json, "0020000D").unwrap_or_default(),
	            study_date: extract_string_value(json, "00080020"),
	            study_time: extract_string_value(json, "00080030"),
	            study_description: extract_string_value(json, "00081030"),
	            patient_name: extract_patient_name(json),
	            patient_id: extract_string_value(json, "00100020"),
	            patient_sex: extract_string_value(json, "00100040"),
	            patient_age: extract_string_value(json, "00101010"),
	            patient_birth_date: extract_string_value(json, "00100030"),
	            modalities_in_study: extract_string_array(json, "00080061"),
	            number_of_series: extract_int_value(json, "00201206"),
	            number_of_instances: extract_int_value(json, "00201208"),
	        }
	    }
}

/// DICOM 태그에서 문자열 값 추출
fn extract_string_value(json: &serde_json::Value, tag: &str) -> Option<String> {
    json.get(tag)?
        .get("Value")?
        .get(0)?
        .as_str()
        .map(|s| s.to_string())
}

/// DICOM 태그에서 문자열 배열 추출
fn extract_string_array(json: &serde_json::Value, tag: &str) -> Option<Vec<String>> {
    let array = json.get(tag)?.get("Value")?.as_array()?;
    Some(
        array
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
    )
}

/// DICOM 태그에서 정수 값 추출
fn extract_int_value(json: &serde_json::Value, tag: &str) -> Option<i32> {
    let value_str = json.get(tag)?.get("Value")?.get(0)?.as_str()?;
    value_str.parse::<i32>().ok()
}

/// PatientName 추출 (PN VR 처리)
fn extract_patient_name(json: &serde_json::Value) -> Option<String> {
    // PatientName은 PN VR이므로 {"Alphabetic": "DOE^JOHN"} 형태일 수 있음
    if let Some(value) = json.get("00100010")?.get("Value")?.get(0) {
        if let Some(alphabetic) = value.get("Alphabetic") {
            return alphabetic.as_str().map(|s| s.to_string());
        }
        // 또는 단순 문자열일 수도 있음
        if let Some(s) = value.as_str() {
            return Some(s.to_string());
        }
    }
    None
}

// ============================================================================
// Series Meta DTOs
// ============================================================================

/// Study-Series 쌍 조회 요청
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SeriesQuery {
    /// StudyInstanceUID
    #[schema(example = "1.2.840.113619.2.55.3.604688433.1234")]
    pub study_uid: String,

    /// SeriesInstanceUID
    #[schema(example = "1.2.840.113619.2.55.3.604688433.1234.1")]
    pub series_uid: String,

    /// StudyDescription (선택사항, 클라이언트가 이미 알고 있다면 전달)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Chest CT")]
    pub study_description: Option<String>,
}

/// Viewer Series Meta Batch API Request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ViewerSeriesMetaRequest {
    /// Study-Series 쌍 목록
    #[schema(example = json!([
        {"study_uid": "1.2.840.113619.2.55.3.604688433.1234", "series_uid": "1.2.840.113619.2.55.3.604688433.1234.1"},
        {"study_uid": "1.2.840.113619.2.55.3.604688433.1234", "series_uid": "1.2.840.113619.2.55.3.604688433.1234.2"}
    ]))]
    pub series_queries: Vec<SeriesQuery>,

    /// 페이지 번호 (1부터 시작, 기본값: 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 1)]
    pub page: Option<i32>,

    /// 페이지 크기 (기본값: 50, 최소: 1, 최대: 200)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 50)]
    pub page_size: Option<i32>,
}

/// Viewer Series Meta Response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ViewerSeriesMetaResponse {
    /// Series 메타데이터 목록
    pub series: Vec<ViewerSeriesMeta>,

    /// 페이지네이션 정보
    pub pagination: ViewerPaginationInfo,
}

    /// Study의 모든 Series Meta 조회 요청 (Study 내 특정 Series만 선택 조회 가능)
    #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
    pub struct ViewerStudySeriesMetaRequest {
        /// 선택적으로 특정 SeriesInstanceUID 목록을 지정
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schema(example = json!(["1.2.840.113619.2.55.3.604688433.1234.1"]))]
        pub series_uids: Option<Vec<String>>,

        /// 페이지 번호 (1부터 시작, 기본값: 1)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schema(example = 1)]
        pub page: Option<i32>,

        /// 페이지 크기 (기본값: 50, 최대: 200)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schema(example = 50)]
        pub page_size: Option<i32>,
    }

/// Study의 모든 Series Meta 조회 응답
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ViewerStudySeriesMetaResponse {
    /// Study UID
    #[schema(example = "1.2.840.113619.2.55.3.604688433.1234")]
    pub study_uid: String,

    /// Study Description
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Chest CT")]
    pub study_description: Option<String>,

    /// Series 메타데이터 목록 (페이지네이션 적용)
    pub series: Vec<ViewerSeriesMeta>,

    /// 페이지네이션 정보
    pub pagination: ViewerPaginationInfo,
}

/// 페이지네이션 정보
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ViewerPaginationInfo {
    /// 현재 페이지 (1부터 시작)
    #[schema(example = 1)]
    pub page: i32,

    /// 페이지 크기
    #[schema(example = 50)]
    pub page_size: i32,

    /// 총 항목 수
    #[schema(example = 245)]
    pub total_items: i32,

    /// 총 페이지 수
    #[schema(example = 5)]
    pub total_pages: i32,

    /// 다음 페이지 존재 여부
    #[schema(example = true)]
    pub has_next: bool,

    /// 이전 페이지 존재 여부
    #[schema(example = false)]
    pub has_previous: bool,
}

/// Viewer Series Meta DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ViewerSeriesMeta {
    /// SeriesInstanceUID
    #[schema(example = "1.2.840.113619.2.55.3.604688433.1234.1")]
    pub series_uid: String,

    /// StudyInstanceUID (부모)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "1.2.840.113619.2.55.3.604688433.1234")]
    pub study_uid: Option<String>,

    /// StudyDescription (부모 Study의 설명)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Chest CT")]
    pub study_description: Option<String>,

    /// SeriesNumber
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 1)]
    pub series_number: Option<i32>,

    /// SeriesDescription
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Axial T1")]
    pub series_description: Option<String>,

    /// Modality
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "MR")]
    pub modality: Option<String>,

    /// Instance 개수
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 120)]
    pub number_of_instances: Option<i32>,

    /// SeriesDate (YYYYMMDD)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "20240115")]
    pub series_date: Option<String>,

    /// SeriesTime (HHMMSS)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "093012")]
    pub series_time: Option<String>,

    /// BodyPartExamined
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "BRAIN")]
    pub body_part_examined: Option<String>,

    /// ProtocolName
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "T1_MPRAGE")]
    pub protocol_name: Option<String>,
}

impl ViewerSeriesMeta {
    /// DICOMweb JSON에서 ViewerSeriesMeta로 변환
    pub fn from_dicomweb_json(json: &serde_json::Value) -> Self {
        Self {
            series_uid: extract_string_value(json, "0020000E").unwrap_or_default(),
            study_uid: extract_string_value(json, "0020000D"),
            study_description: extract_string_value(json, "00081030"), // StudyDescription
            series_number: extract_int_value(json, "00200011"),
            series_description: extract_string_value(json, "0008103E"),
            modality: extract_string_value(json, "00080060"),
            number_of_instances: extract_int_value(json, "00201209"),
            series_date: extract_string_value(json, "00080021"),
            series_time: extract_string_value(json, "00080031"),
            body_part_examined: extract_string_value(json, "00180015"),
            protocol_name: extract_string_value(json, "00181030"),
        }
    }
}
