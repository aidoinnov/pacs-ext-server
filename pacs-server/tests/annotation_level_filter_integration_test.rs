#[cfg(test)]
mod annotation_level_filter_tests {
    use pacs_server::application::dto::annotation_dto::{AnnotationListResponse, AnnotationResponse};
    use serde_json::json;

    /// Helper function to create mock annotation responses
    fn create_mock_annotations() -> Vec<AnnotationResponse> {
        use chrono::Utc;

        vec![
            // Study level annotation (series_uid and instance_uid are empty)
            AnnotationResponse {
                id: 1,
                user_id: 1,
                user_name: Some("김철수".to_string()),
                study_instance_uid: "1.2.3.4.5".to_string(),
                series_instance_uid: "".to_string(),
                sop_instance_uid: "".to_string(),
                annotation_data: json!({"type": "study_note", "text": "Study level"}),
                viewer_software: Some("TI-DicomViewer".to_string()),
                tool_name: Some("Note Tool".to_string()),
                tool_version: Some("1.0.0".to_string()),
                description: Some("Study level annotation".to_string()),
                measurement_values: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            // Series level annotation (series_uid present, instance_uid empty)
            AnnotationResponse {
                id: 2,
                user_id: 1,
                user_name: Some("김철수".to_string()),
                study_instance_uid: "1.2.3.4.5".to_string(),
                series_instance_uid: "1.2.3.4.5.6".to_string(),
                sop_instance_uid: "".to_string(),
                annotation_data: json!({"type": "series_note", "text": "Series level"}),
                viewer_software: Some("TI-DicomViewer".to_string()),
                tool_name: Some("Note Tool".to_string()),
                tool_version: Some("1.0.0".to_string()),
                description: Some("Series level annotation".to_string()),
                measurement_values: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            // Instance level annotation (both series_uid and instance_uid present)
            AnnotationResponse {
                id: 3,
                user_id: 1,
                user_name: Some("김철수".to_string()),
                study_instance_uid: "1.2.3.4.5".to_string(),
                series_instance_uid: "1.2.3.4.5.6".to_string(),
                sop_instance_uid: "1.2.3.4.5.6.7".to_string(),
                annotation_data: json!({"type": "measurement", "value": 10.5}),
                viewer_software: Some("TI-DicomViewer".to_string()),
                tool_name: Some("Measurement Tool".to_string()),
                tool_version: Some("1.0.0".to_string()),
                description: Some("Instance level annotation".to_string()),
                measurement_values: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            // Another instance level annotation
            AnnotationResponse {
                id: 4,
                user_id: 2,
                user_name: Some("이영희".to_string()),
                study_instance_uid: "1.2.3.4.5".to_string(),
                series_instance_uid: "1.2.3.4.5.6".to_string(),
                sop_instance_uid: "1.2.3.4.5.6.8".to_string(),
                annotation_data: json!({"type": "circle", "x": 100, "y": 100, "radius": 50}),
                viewer_software: Some("TI-DicomViewer".to_string()),
                tool_name: Some("Circle Tool".to_string()),
                tool_version: Some("1.0.0".to_string()),
                description: Some("Another instance level".to_string()),
                measurement_values: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ]
    }

    #[test]
    fn test_level_filter_study() {
        let annotations = create_mock_annotations();
        
        // Filter for study level
        let study_level: Vec<_> = annotations
            .iter()
            .filter(|ann| ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty())
            .collect();

        assert_eq!(study_level.len(), 1);
        assert_eq!(study_level[0].id, 1);
        assert_eq!(study_level[0].description, Some("Study level annotation".to_string()));
    }

    #[test]
    fn test_level_filter_series() {
        let annotations = create_mock_annotations();
        
        // Filter for series level
        let series_level: Vec<_> = annotations
            .iter()
            .filter(|ann| !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty())
            .collect();

        assert_eq!(series_level.len(), 1);
        assert_eq!(series_level[0].id, 2);
        assert_eq!(series_level[0].description, Some("Series level annotation".to_string()));
    }

    #[test]
    fn test_level_filter_instance() {
        let annotations = create_mock_annotations();
        
        // Filter for instance level
        let instance_level: Vec<_> = annotations
            .iter()
            .filter(|ann| !ann.sop_instance_uid.is_empty())
            .collect();

        assert_eq!(instance_level.len(), 2);
        assert_eq!(instance_level[0].id, 3);
        assert_eq!(instance_level[1].id, 4);
    }

    #[test]
    fn test_level_filter_all_levels() {
        let annotations = create_mock_annotations();
        
        // No filter - should return all
        assert_eq!(annotations.len(), 4);

        // Verify each level exists
        let study_count = annotations
            .iter()
            .filter(|ann| ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty())
            .count();
        let series_count = annotations
            .iter()
            .filter(|ann| !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty())
            .count();
        let instance_count = annotations
            .iter()
            .filter(|ann| !ann.sop_instance_uid.is_empty())
            .count();

        assert_eq!(study_count, 1);
        assert_eq!(series_count, 1);
        assert_eq!(instance_count, 2);
        assert_eq!(study_count + series_count + instance_count, 4);
    }

    #[test]
    fn test_user_name_field_present() {
        let annotations = create_mock_annotations();
        
        // All annotations should have user_name
        for ann in &annotations {
            assert!(ann.user_name.is_some());
        }

        // Check specific user names
        assert_eq!(annotations[0].user_name, Some("김철수".to_string()));
        assert_eq!(annotations[1].user_name, Some("김철수".to_string()));
        assert_eq!(annotations[2].user_name, Some("김철수".to_string()));
        assert_eq!(annotations[3].user_name, Some("이영희".to_string()));
    }

    #[test]
    fn test_level_filter_with_viewer_software() {
        let annotations = create_mock_annotations();
        
        // Filter by level and viewer_software
        let filtered: Vec<_> = annotations
            .iter()
            .filter(|ann| {
                !ann.sop_instance_uid.is_empty()
                    && ann.viewer_software.as_ref().map(|v| v.as_str()) == Some("TI-DicomViewer")
            })
            .collect();

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|ann| ann.viewer_software == Some("TI-DicomViewer".to_string())));
    }

    #[test]
    fn test_level_filter_with_user_id() {
        let annotations = create_mock_annotations();
        
        // Filter by level and user_id
        let user1_instance_level: Vec<_> = annotations
            .iter()
            .filter(|ann| !ann.sop_instance_uid.is_empty() && ann.user_id == 1)
            .collect();

        assert_eq!(user1_instance_level.len(), 1);
        assert_eq!(user1_instance_level[0].id, 3);
        assert_eq!(user1_instance_level[0].user_name, Some("김철수".to_string()));

        let user2_instance_level: Vec<_> = annotations
            .iter()
            .filter(|ann| !ann.sop_instance_uid.is_empty() && ann.user_id == 2)
            .collect();

        assert_eq!(user2_instance_level.len(), 1);
        assert_eq!(user2_instance_level[0].id, 4);
        assert_eq!(user2_instance_level[0].user_name, Some("이영희".to_string()));
    }

    #[test]
    fn test_annotation_response_list_structure() {
        let annotations = create_mock_annotations();

        let response = AnnotationListResponse {
            annotations: annotations.clone(),
            total: annotations.len(),
            list_version: None,
        };

        assert_eq!(response.total, 4);
        assert_eq!(response.annotations.len(), 4);

        // Test JSON serialization
        let json_str = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json_str.contains("annotations"));
        assert!(json_str.contains("total"));
        assert!(json_str.contains("user_name"));
        assert!(json_str.contains("김철수"));
        assert!(json_str.contains("이영희"));
    }

    #[test]
    fn test_empty_string_vs_none_for_uids() {
        // Test that empty strings are used for study/series level annotations
        let study_level = AnnotationResponse {
            id: 1,
            user_id: 1,
            user_name: Some("Test".to_string()),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "".to_string(),
            sop_instance_uid: "".to_string(),
            annotation_data: json!({}),
            viewer_software: None,
            tool_name: None,
            tool_version: None,
            description: None,
            measurement_values: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Empty strings should be used, not None
        assert_eq!(study_level.series_instance_uid, "");
        assert_eq!(study_level.sop_instance_uid, "");
        assert!(study_level.series_instance_uid.is_empty());
        assert!(study_level.sop_instance_uid.is_empty());
    }
}

