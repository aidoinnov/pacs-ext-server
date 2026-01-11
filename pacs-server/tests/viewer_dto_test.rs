/**
 * Viewer DTO Test
 * 
 * Viewer BFF API DTO의 단위 테스트
 * - DICOMweb JSON 파싱
 * - DTO 변환 로직
 */

use serde_json::json;
use pacs_server::application::dto::viewer_dto::*;

#[test]
fn test_viewer_study_meta_from_dicomweb_json() {
    let dicomweb_json = json!({
        "0020000D": {
            "vr": "UI",
            "Value": ["1.2.840.113619.2.55.3.604688433.1234"]
        },
        "00080020": {
            "vr": "DA",
            "Value": ["20240115"]
        },
        "00080030": {
            "vr": "TM",
            "Value": ["093012"]
        },
        "00081030": {
            "vr": "LO",
            "Value": ["Chest CT"]
        },
        "00100010": {
            "vr": "PN",
            "Value": [{"Alphabetic": "DOE^JOHN"}]
        },
        "00100020": {
            "vr": "LO",
            "Value": ["P123456"]
        },
        "00080061": {
            "vr": "CS",
            "Value": ["CT"]
        },
        "00201206": {
            "vr": "IS",
            "Value": ["3"]
        },
        "00201208": {
            "vr": "IS",
            "Value": ["245"]
        }
    });

    let study_meta = ViewerStudyMeta::from_dicomweb_json(&dicomweb_json);

    assert_eq!(study_meta.study_uid, "1.2.840.113619.2.55.3.604688433.1234");
    assert_eq!(study_meta.study_date, Some("20240115".to_string()));
    assert_eq!(study_meta.study_time, Some("093012".to_string()));
    assert_eq!(study_meta.study_description, Some("Chest CT".to_string()));
    assert_eq!(study_meta.patient_name, Some("DOE^JOHN".to_string()));
    assert_eq!(study_meta.patient_id, Some("P123456".to_string()));
    assert_eq!(study_meta.modalities_in_study, Some(vec!["CT".to_string()]));
    assert_eq!(study_meta.number_of_series, Some(3));
    assert_eq!(study_meta.number_of_instances, Some(245));
}

#[test]
fn test_viewer_series_meta_from_dicomweb_json() {
    let dicomweb_json = json!({
        "0020000E": {
            "vr": "UI",
            "Value": ["1.2.840.113619.2.55.3.604688433.1234.1"]
        },
        "0020000D": {
            "vr": "UI",
            "Value": ["1.2.840.113619.2.55.3.604688433.1234"]
        },
        "00081030": {
            "vr": "LO",
            "Value": ["Brain MRI Study"]
        },
        "00200011": {
            "vr": "IS",
            "Value": ["1"]
        },
        "0008103E": {
            "vr": "LO",
            "Value": ["Axial T1"]
        },
        "00080060": {
            "vr": "CS",
            "Value": ["MR"]
        },
        "00201209": {
            "vr": "IS",
            "Value": ["120"]
        },
        "00080021": {
            "vr": "DA",
            "Value": ["20240115"]
        },
        "00080031": {
            "vr": "TM",
            "Value": ["093012"]
        },
        "00180015": {
            "vr": "CS",
            "Value": ["BRAIN"]
        },
        "00181030": {
            "vr": "LO",
            "Value": ["T1_MPRAGE"]
        }
    });

    let series_meta = ViewerSeriesMeta::from_dicomweb_json(&dicomweb_json);

    assert_eq!(series_meta.series_uid, "1.2.840.113619.2.55.3.604688433.1234.1");
    assert_eq!(series_meta.study_uid, Some("1.2.840.113619.2.55.3.604688433.1234".to_string()));
    assert_eq!(series_meta.study_description, Some("Brain MRI Study".to_string()));
    assert_eq!(series_meta.series_number, Some(1));
    assert_eq!(series_meta.series_description, Some("Axial T1".to_string()));
    assert_eq!(series_meta.modality, Some("MR".to_string()));
    assert_eq!(series_meta.number_of_instances, Some(120));
    assert_eq!(series_meta.series_date, Some("20240115".to_string()));
    assert_eq!(series_meta.series_time, Some("093012".to_string()));
    assert_eq!(series_meta.body_part_examined, Some("BRAIN".to_string()));
    assert_eq!(series_meta.protocol_name, Some("T1_MPRAGE".to_string()));
}

#[test]
fn test_viewer_study_meta_request_serialization() {
    let request = ViewerStudyMetaRequest {
        study_uids: vec![
            "1.2.840.113619.2.55.3.604688433.1234".to_string(),
            "1.2.840.113619.2.55.3.604688433.5678".to_string(),
        ],
        max_count: Some(20),
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["study_uids"].as_array().unwrap().len(), 2);
    assert_eq!(json["max_count"], 20);
}

#[test]
fn test_viewer_series_meta_request_serialization() {
    use pacs_server::application::dto::viewer_dto::SeriesQuery;

    let request = ViewerSeriesMetaRequest {
        series_queries: vec![
            SeriesQuery {
                study_uid: "1.2.840.113619.2.55.3.604688433.1234".to_string(),
                series_uid: "1.2.840.113619.2.55.3.604688433.1234.1".to_string(),
            },
            SeriesQuery {
                study_uid: "1.2.840.113619.2.55.3.604688433.1234".to_string(),
                series_uid: "1.2.840.113619.2.55.3.604688433.1234.2".to_string(),
            },
        ],
        max_count: Some(50),
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["series_queries"].as_array().unwrap().len(), 2);
    assert_eq!(json["max_count"], 50);

    // 첫 번째 쿼리 검증
    let first_query = &json["series_queries"][0];
    assert_eq!(first_query["study_uid"], "1.2.840.113619.2.55.3.604688433.1234");
    assert_eq!(first_query["series_uid"], "1.2.840.113619.2.55.3.604688433.1234.1");
}

