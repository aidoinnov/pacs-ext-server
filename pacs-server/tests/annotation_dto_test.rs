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
        };

        // Test serialization
        let json_str = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json_str.contains("viewer_software"));
        assert!(json_str.contains("tool_name"));
        assert!(json_str.contains("tool_version"));
        assert!(json_str.contains("description"));

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
        };

        // Test serialization with None values
        let json_str = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json_str.contains("viewer_software"));
        assert!(json_str.contains("tool_name"));
        assert!(json_str.contains("tool_version"));
        assert!(json_str.contains("description"));

        // Test deserialization
        let deserialized: CreateAnnotationRequest =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.viewer_software, None);
        assert_eq!(deserialized.tool_name, None);
        assert_eq!(deserialized.tool_version, None);
        assert_eq!(deserialized.description, None);
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
        };

        // Test serialization
        let json_str = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json_str.contains("viewer_software"));
        assert!(json_str.contains("tool_name"));
        assert!(json_str.contains("tool_version"));
        assert!(json_str.contains("description"));

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
    }

    #[test]
    fn test_annotation_response_serialization() {
        let response = AnnotationResponse {
            id: 123,
            user_id: 456,
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
            created_at: Utc.timestamp_opt(1704110400, 0).unwrap(),
            updated_at: Utc.timestamp_opt(1704112200, 0).unwrap(),
        };

        // Test serialization
        let json_str = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json_str.contains("viewer_software"));
        assert!(json_str.contains("tool_name"));
        assert!(json_str.contains("tool_version"));
        assert!(json_str.contains("description"));
        assert!(json_str.contains("created_at"));
        assert!(json_str.contains("updated_at"));

        // Test deserialization
        let deserialized: AnnotationResponse =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.id, 123);
        assert_eq!(deserialized.user_id, 456);
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
    }

    #[test]
    fn test_annotation_response_with_none_fields() {
        let response = AnnotationResponse {
            id: 789,
            user_id: 101,
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: json!({"type": "line", "x1": 0, "y1": 0, "x2": 100, "y2": 100}),
            viewer_software: None,
            tool_name: None,
            tool_version: None,
            description: None,
            measurement_values: None,
            created_at: Utc.timestamp_opt(1704196800, 0).unwrap(),
            updated_at: Utc.timestamp_opt(1704197700, 0).unwrap(),
        };

        // Test serialization with None values
        let json_str = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json_str.contains("viewer_software"));
        assert!(json_str.contains("tool_name"));
        assert!(json_str.contains("tool_version"));
        assert!(json_str.contains("description"));

        // Test deserialization
        let deserialized: AnnotationResponse =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.id, 789);
        assert_eq!(deserialized.user_id, 101);
        assert_eq!(deserialized.viewer_software, None);
        assert_eq!(deserialized.tool_name, None);
        assert_eq!(deserialized.tool_version, None);
        assert_eq!(deserialized.description, None);
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
        };

        let point_json = serde_json::to_string(&point_request).expect("Failed to serialize point");
        assert!(point_json.contains("point"));
        assert!(point_json.contains("Cornerstone.js"));
    }
}
