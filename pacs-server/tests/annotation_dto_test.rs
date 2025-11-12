#[cfg(test)]
mod annotation_dto_tests {
    use chrono::{TimeZone, Utc};
    use pacs_server::application::dto::annotation_dto::{
        AnnotationResponse, CreateAnnotationRequest, UpdateAnnotationRequest,
    };
    use serde_json::json;

    #[test]
    fn test_create_annotation_request_serialization() {
        let request = CreateAnnotationRequest {
            project_id: Some(1),
            user_id: Some(1),
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: json!({
                "type": "circle",
                "x": 100,
                "y": 200,
                "radius": 50,
                "color": "#FF0000",
                "label": "Test Circle"
            }),
            viewer_software: Some("OHIF Viewer".to_string()),
            tool_name: Some("Circle Tool".to_string()),
            tool_version: Some("2.1.0".to_string()),
            description: Some("Test annotation with new fields".to_string()),
            measurement_values: None,
            label: Some("Tumor".to_string()),
        };

        // Test serialization
        let json_str = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json_str.contains("viewer_software"));
        assert!(json_str.contains("tool_name"));
        assert!(json_str.contains("tool_version"));
        assert!(json_str.contains("description"));
        assert!(json_str.contains("label"));
        assert!(json_str.contains("Tumor"));

        // Test deserialization
        let deserialized: CreateAnnotationRequest =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(
            deserialized.viewer_software,
            Some("OHIF Viewer".to_string())
        );
        assert_eq!(deserialized.tool_name, Some("Circle Tool".to_string()));
        assert_eq!(deserialized.tool_version, Some("2.1.0".to_string()));
        assert_eq!(
            deserialized.description,
            Some("Test annotation with new fields".to_string())
        );
        assert_eq!(deserialized.label, Some("Tumor".to_string()));
    }

    #[test]
    fn test_create_annotation_request_with_none_fields() {
        let request = CreateAnnotationRequest {
            project_id: Some(1),
            user_id: Some(1),
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: json!({"type": "point", "x": 150, "y": 150}),
            viewer_software: None,
            tool_name: None,
            tool_version: None,
            description: None,
            measurement_values: None,
            label: None,
        };

        // Test serialization with None values
        let json_str = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json_str.contains("viewer_software"));
        assert!(json_str.contains("tool_name"));
        assert!(json_str.contains("tool_version"));
        assert!(json_str.contains("description"));
        assert!(json_str.contains("label"));

        // Test deserialization
        let deserialized: CreateAnnotationRequest =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.viewer_software, None);
        assert_eq!(deserialized.tool_name, None);
        assert_eq!(deserialized.tool_version, None);
        assert_eq!(deserialized.description, None);
        assert_eq!(deserialized.label, None);
    }

    #[test]
    fn test_update_annotation_request_serialization() {
        let request = UpdateAnnotationRequest {
            annotation_data: Some(json!({
                "type": "rectangle",
                "x": 50,
                "y": 50,
                "width": 200,
                "height": 100,
                "color": "#00FF00"
            })),
            viewer_software: Some("Updated OHIF Viewer".to_string()),
            tool_name: Some("Updated Rectangle Tool".to_string()),
            tool_version: Some("3.0.0".to_string()),
            description: Some("Updated description".to_string()),
            measurement_values: None,
            base_version: Some(1),
            label: Some("Lesion".to_string()),
        };

        // Test serialization
        let json_str = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json_str.contains("viewer_software"));
        assert!(json_str.contains("tool_name"));
        assert!(json_str.contains("tool_version"));
        assert!(json_str.contains("description"));
        assert!(json_str.contains("label"));
        assert!(json_str.contains("Lesion"));

        // Test deserialization
        let deserialized: UpdateAnnotationRequest =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(
            deserialized.viewer_software,
            Some("Updated OHIF Viewer".to_string())
        );
        assert_eq!(
            deserialized.tool_name,
            Some("Updated Rectangle Tool".to_string())
        );
        assert_eq!(deserialized.tool_version, Some("3.0.0".to_string()));
        assert_eq!(
            deserialized.description,
            Some("Updated description".to_string())
        );
        assert_eq!(deserialized.label, Some("Lesion".to_string()));
    }

    #[test]
    fn test_annotation_response_serialization() {
        let response = AnnotationResponse {
            id: 123,
            user_id: 456,
            user_name: Some("홍길동".to_string()),
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: json!({
                "type": "polygon",
                "points": [[100, 100], [200, 100], [200, 200], [100, 200]],
                "color": "#0000FF"
            }),
            viewer_software: Some("OHIF Viewer".to_string()),
            tool_name: Some("Polygon Tool".to_string()),
            tool_version: Some("2.5.0".to_string()),
            description: Some("Polygon annotation".to_string()),
            measurement_values: None,
            label: Some("Normal".to_string()),
            version: 1,
            created_at: Utc.timestamp_opt(1704110400, 0).unwrap(),
            updated_at: Utc.timestamp_opt(1704112200, 0).unwrap(),
        };

        // Test serialization
        let json_str = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json_str.contains("user_name"));
        assert!(json_str.contains("홍길동"));
        assert!(json_str.contains("viewer_software"));
        assert!(json_str.contains("tool_name"));
        assert!(json_str.contains("tool_version"));
        assert!(json_str.contains("description"));
        assert!(json_str.contains("label"));
        assert!(json_str.contains("Normal"));
        assert!(json_str.contains("created_at"));
        assert!(json_str.contains("updated_at"));

        // Test deserialization
        let deserialized: AnnotationResponse =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.id, 123);
        assert_eq!(deserialized.user_id, 456);
        assert_eq!(deserialized.user_name, Some("홍길동".to_string()));
        assert_eq!(
            deserialized.viewer_software,
            Some("OHIF Viewer".to_string())
        );
        assert_eq!(deserialized.tool_name, Some("Polygon Tool".to_string()));
        assert_eq!(deserialized.tool_version, Some("2.5.0".to_string()));
        assert_eq!(
            deserialized.description,
            Some("Polygon annotation".to_string())
        );
        assert_eq!(deserialized.label, Some("Normal".to_string()));
    }

    #[test]
    fn test_annotation_response_with_none_fields() {
        let response = AnnotationResponse {
            id: 789,
            user_id: 101,
            user_name: None,
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: json!({"type": "line", "x1": 0, "y1": 0, "x2": 100, "y2": 100}),
            viewer_software: None,
            tool_name: None,
            tool_version: None,
            description: None,
            measurement_values: None,
            label: None,
            version: 1,
            created_at: Utc.timestamp_opt(1704196800, 0).unwrap(),
            updated_at: Utc.timestamp_opt(1704197700, 0).unwrap(),
        };

        // Test serialization with None values
        let json_str = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json_str.contains("user_name"));
        assert!(json_str.contains("viewer_software"));
        assert!(json_str.contains("tool_name"));
        assert!(json_str.contains("tool_version"));
        assert!(json_str.contains("description"));
        assert!(json_str.contains("label"));

        // Test deserialization
        let deserialized: AnnotationResponse =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.id, 789);
        assert_eq!(deserialized.user_id, 101);
        assert_eq!(deserialized.user_name, None);
        assert_eq!(deserialized.viewer_software, None);
        assert_eq!(deserialized.tool_name, None);
        assert_eq!(deserialized.tool_version, None);
        assert_eq!(deserialized.description, None);
        assert_eq!(deserialized.label, None);
    }

    #[test]
    fn test_annotation_data_various_types() {
        // Test circle annotation
        let circle_data = json!({
            "type": "circle",
            "x": 100,
            "y": 200,
            "radius": 50,
            "color": "#FF0000",
            "label": "Circle Annotation"
        });

        let circle_request = CreateAnnotationRequest {
            project_id: Some(1),
            user_id: Some(1),
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: circle_data,
            viewer_software: Some("OHIF Viewer".to_string()),
            tool_name: Some("Circle Tool".to_string()),
            tool_version: Some("2.1.0".to_string()),
            description: Some("Circle annotation test".to_string()),
            measurement_values: None,
            label: Some("Tumor".to_string()),
        };

        let circle_json =
            serde_json::to_string(&circle_request).expect("Failed to serialize circle");
        assert!(circle_json.contains("circle"));
        assert!(circle_json.contains("radius"));

        // Test rectangle annotation
        let rectangle_data = json!({
            "type": "rectangle",
            "x": 50,
            "y": 50,
            "width": 200,
            "height": 100,
            "color": "#00FF00",
            "label": "Rectangle Annotation"
        });

        let rectangle_request = CreateAnnotationRequest {
            project_id: Some(1),
            user_id: Some(1),
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: rectangle_data,
            viewer_software: Some("DICOM.js Viewer".to_string()),
            tool_name: Some("Rectangle Tool".to_string()),
            tool_version: Some("1.8.0".to_string()),
            description: Some("Rectangle annotation test".to_string()),
            measurement_values: None,
            label: Some("Lesion".to_string()),
        };

        let rectangle_json =
            serde_json::to_string(&rectangle_request).expect("Failed to serialize rectangle");
        assert!(rectangle_json.contains("rectangle"));
        assert!(rectangle_json.contains("width"));
        assert!(rectangle_json.contains("height"));

        // Test point annotation
        let point_data = json!({
            "type": "point",
            "x": 150,
            "y": 150,
            "color": "#0000FF",
            "label": "Point Annotation"
        });

        let point_request = CreateAnnotationRequest {
            project_id: Some(1),
            user_id: Some(1),
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: point_data,
            viewer_software: Some("Cornerstone.js".to_string()),
            tool_name: Some("Point Tool".to_string()),
            tool_version: Some("3.2.1".to_string()),
            description: Some("Point annotation test".to_string()),
            measurement_values: None,
            label: Some("Normal".to_string()),
        };

        let point_json = serde_json::to_string(&point_request).expect("Failed to serialize point");
        assert!(point_json.contains("point"));
        assert!(point_json.contains("Cornerstone.js"));
    }

    #[test]
    fn test_annotation_response_with_user_name() {
        // Test with user_name present
        let response_with_name = AnnotationResponse {
            id: 1,
            user_id: 5,
            user_name: Some("김철수".to_string()),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.5.6".to_string(),
            sop_instance_uid: "1.2.3.4.5.6.7".to_string(),
            annotation_data: json!({"type": "point", "x": 100, "y": 100}),
            viewer_software: Some("TI-DicomViewer".to_string()),
            tool_name: Some("Point Tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            description: Some("Test".to_string()),
            measurement_values: None,
            label: Some("Tumor".to_string()),
            version: 1,
            created_at: Utc.timestamp_opt(1704110400, 0).unwrap(),
            updated_at: Utc.timestamp_opt(1704112200, 0).unwrap(),
        };

        let json_str = serde_json::to_string(&response_with_name).expect("Failed to serialize");
        assert!(json_str.contains("user_name"));
        assert!(json_str.contains("김철수"));
        assert!(json_str.contains("label"));
        assert!(json_str.contains("Tumor"));

        let deserialized: AnnotationResponse =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.user_name, Some("김철수".to_string()));
        assert_eq!(deserialized.label, Some("Tumor".to_string()));

        // Test with user_name as None
        let response_without_name = AnnotationResponse {
            id: 2,
            user_id: 10,
            user_name: None,
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.5.6".to_string(),
            sop_instance_uid: "1.2.3.4.5.6.7".to_string(),
            annotation_data: json!({"type": "point", "x": 100, "y": 100}),
            viewer_software: None,
            tool_name: None,
            tool_version: None,
            description: None,
            measurement_values: None,
            label: None,
            version: 1,
            created_at: Utc.timestamp_opt(1704110400, 0).unwrap(),
            updated_at: Utc.timestamp_opt(1704112200, 0).unwrap(),
        };

        let json_str2 = serde_json::to_string(&response_without_name).expect("Failed to serialize");
        assert!(json_str2.contains("user_name"));
        assert!(json_str2.contains("label"));

        let deserialized2: AnnotationResponse =
            serde_json::from_str(&json_str2).expect("Failed to deserialize");
        assert_eq!(deserialized2.user_name, None);
        assert_eq!(deserialized2.label, None);
    }

    #[test]
    fn test_annotation_level_detection() {
        // Study level: series_uid and instance_uid are empty
        let study_level = AnnotationResponse {
            id: 1,
            user_id: 1,
            user_name: Some("Test User".to_string()),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "".to_string(),
            sop_instance_uid: "".to_string(),
            annotation_data: json!({"type": "study_note", "text": "Study level annotation"}),
            viewer_software: Some("TI-DicomViewer".to_string()),
            tool_name: Some("Note Tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            description: Some("Study level".to_string()),
            measurement_values: None,
            label: Some("Study Annotation".to_string()),
            version: 1,
            created_at: Utc.timestamp_opt(1704110400, 0).unwrap(),
            updated_at: Utc.timestamp_opt(1704112200, 0).unwrap(),
        };

        assert!(study_level.series_instance_uid.is_empty());
        assert!(study_level.sop_instance_uid.is_empty());

        // Series level: series_uid present, instance_uid empty
        let series_level = AnnotationResponse {
            id: 2,
            user_id: 1,
            user_name: Some("Test User".to_string()),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.5.6".to_string(),
            sop_instance_uid: "".to_string(),
            annotation_data: json!({"type": "series_note", "text": "Series level annotation"}),
            viewer_software: Some("TI-DicomViewer".to_string()),
            tool_name: Some("Note Tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            description: Some("Series level".to_string()),
            measurement_values: None,
            label: Some("Series Annotation".to_string()),
            version: 1,
            created_at: Utc.timestamp_opt(1704110400, 0).unwrap(),
            updated_at: Utc.timestamp_opt(1704112200, 0).unwrap(),
        };

        assert!(!series_level.series_instance_uid.is_empty());
        assert!(series_level.sop_instance_uid.is_empty());

        // Instance level: both series_uid and instance_uid present
        let instance_level = AnnotationResponse {
            id: 3,
            user_id: 1,
            user_name: Some("Test User".to_string()),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.5.6".to_string(),
            sop_instance_uid: "1.2.3.4.5.6.7".to_string(),
            annotation_data: json!({"type": "measurement", "value": 10.5}),
            viewer_software: Some("TI-DicomViewer".to_string()),
            tool_name: Some("Measurement Tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            description: Some("Instance level".to_string()),
            measurement_values: None,
            label: Some("Instance".to_string()),
            version: 1,
            created_at: Utc.timestamp_opt(1704110400, 0).unwrap(),
            updated_at: Utc.timestamp_opt(1704112200, 0).unwrap(),
        };

        assert!(!instance_level.series_instance_uid.is_empty());
        assert!(!instance_level.sop_instance_uid.is_empty());
    }
}
