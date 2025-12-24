#[cfg(test)]
mod series_user_note_dto_tests {
    use chrono::{TimeZone, Utc};
    use pacs_server::application::dto::series_user_note_dto::*;
    use serde_json::json;

    #[test]
    fn test_create_or_update_series_note_request_serialization() {
        let request = CreateOrUpdateSeriesNoteRequest {
            note: "이 시리즈는 프로젝트 A에서 분석 중입니다".to_string(),
        };

        // Test serialization
        let json_str = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json_str.contains("note"));
        assert!(json_str.contains("이 시리즈는 프로젝트 A에서 분석 중입니다"));

        // Test deserialization
        let deserialized: CreateOrUpdateSeriesNoteRequest =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(
            deserialized.note,
            "이 시리즈는 프로젝트 A에서 분석 중입니다"
        );
    }

    #[test]
    fn test_create_or_update_series_note_request_empty_note() {
        let request = CreateOrUpdateSeriesNoteRequest {
            note: "".to_string(),
        };

        let json_str = serde_json::to_string(&request).expect("Failed to serialize");
        let deserialized: CreateOrUpdateSeriesNoteRequest =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.note, "");
    }

    #[test]
    fn test_series_note_response_serialization() {
        let response = SeriesNoteResponse {
            id: 1,
            series_id: 123,
            user_id: 456,
            project_id: Some(1),
            note: "이 시리즈는 프로젝트 A에서 분석 중입니다".to_string(),
            created_at: Utc.timestamp_opt(1704110400, 0).unwrap(),
            updated_at: Utc.timestamp_opt(1704112200, 0).unwrap(),
        };

        // Test serialization
        let json_str = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json_str.contains("id"));
        assert!(json_str.contains("series_id"));
        assert!(json_str.contains("user_id"));
        assert!(json_str.contains("project_id"));
        assert!(json_str.contains("note"));
        assert!(json_str.contains("created_at"));
        assert!(json_str.contains("updated_at"));

        // Test deserialization
        let deserialized: SeriesNoteResponse =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.series_id, 123);
        assert_eq!(deserialized.user_id, 456);
        assert_eq!(deserialized.project_id, Some(1));
        assert_eq!(
            deserialized.note,
            "이 시리즈는 프로젝트 A에서 분석 중입니다"
        );
    }

    #[test]
    fn test_series_note_response_with_global_note() {
        let response = SeriesNoteResponse {
            id: 2,
            series_id: 124,
            user_id: 457,
            project_id: None, // 전역 note
            note: "전역 메모입니다".to_string(),
            created_at: Utc.timestamp_opt(1704110400, 0).unwrap(),
            updated_at: Utc.timestamp_opt(1704112200, 0).unwrap(),
        };

        let json_str = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json_str.contains("project_id"));
        assert!(json_str.contains("null")); // None은 null로 직렬화됨

        let deserialized: SeriesNoteResponse =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.project_id, None);
    }

    #[test]
    fn test_series_note_with_user_response_serialization() {
        let response = SeriesNoteWithUserResponse {
            id: 1,
            series_id: 123,
            user: SeriesNoteUserInfo {
                id: 456,
                username: "user1".to_string(),
                email: "user1@example.com".to_string(),
                full_name: Some("홍길동".to_string()),
            },
            project_id: Some(1),
            note: "테스트 메모".to_string(),
            created_at: Utc.timestamp_opt(1704110400, 0).unwrap(),
            updated_at: Utc.timestamp_opt(1704112200, 0).unwrap(),
        };

        let json_str = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json_str.contains("user"));
        assert!(json_str.contains("username"));
        assert!(json_str.contains("email"));
        assert!(json_str.contains("full_name"));
        assert!(json_str.contains("홍길동"));

        let deserialized: SeriesNoteWithUserResponse =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.user.id, 456);
        assert_eq!(deserialized.user.username, "user1");
        assert_eq!(deserialized.user.email, "user1@example.com");
        assert_eq!(deserialized.user.full_name, Some("홍길동".to_string()));
    }

    #[test]
    fn test_series_note_list_response_serialization() {
        let response = SeriesNoteListResponse {
            success: true,
            notes: vec![
                SeriesNoteWithUserResponse {
                    id: 1,
                    series_id: 123,
                    user: SeriesNoteUserInfo {
                        id: 456,
                        username: "user1".to_string(),
                        email: "user1@example.com".to_string(),
                        full_name: Some("홍길동".to_string()),
                    },
                    project_id: Some(1),
                    note: "첫 번째 메모".to_string(),
                    created_at: Utc.timestamp_opt(1704110400, 0).unwrap(),
                    updated_at: Utc.timestamp_opt(1704112200, 0).unwrap(),
                },
                SeriesNoteWithUserResponse {
                    id: 2,
                    series_id: 123,
                    user: SeriesNoteUserInfo {
                        id: 457,
                        username: "user2".to_string(),
                        email: "user2@example.com".to_string(),
                        full_name: Some("김철수".to_string()),
                    },
                    project_id: Some(1),
                    note: "두 번째 메모".to_string(),
                    created_at: Utc.timestamp_opt(1704112400, 0).unwrap(),
                    updated_at: Utc.timestamp_opt(1704112400, 0).unwrap(),
                },
            ],
        };

        let json_str = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json_str.contains("success"));
        assert!(json_str.contains("notes"));
        assert!(json_str.contains("user1"));
        assert!(json_str.contains("user2"));

        let deserialized: SeriesNoteListResponse =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.success, true);
        assert_eq!(deserialized.notes.len(), 2);
        assert_eq!(deserialized.notes[0].user.username, "user1");
        assert_eq!(deserialized.notes[1].user.username, "user2");
    }

    #[test]
    fn test_series_note_single_response_serialization() {
        let response = SeriesNoteSingleResponse {
            success: true,
            note: Some(SeriesNoteResponse {
                id: 1,
                series_id: 123,
                user_id: 456,
                project_id: Some(1),
                note: "테스트 메모".to_string(),
                created_at: Utc.timestamp_opt(1704110400, 0).unwrap(),
                updated_at: Utc.timestamp_opt(1704112200, 0).unwrap(),
            }),
        };

        let json_str = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json_str.contains("success"));
        assert!(json_str.contains("note"));

        let deserialized: SeriesNoteSingleResponse =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.success, true);
        assert!(deserialized.note.is_some());
        assert_eq!(deserialized.note.unwrap().id, 1);
    }

    #[test]
    fn test_series_note_single_response_with_none() {
        let response = SeriesNoteSingleResponse {
            success: true,
            note: None,
        };

        let json_str = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json_str.contains("success"));
        assert!(json_str.contains("note"));
        assert!(json_str.contains("null"));

        let deserialized: SeriesNoteSingleResponse =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.success, true);
        assert!(deserialized.note.is_none());
    }

    #[test]
    fn test_series_note_user_info_serialization() {
        let user_info = SeriesNoteUserInfo {
            id: 456,
            username: "user1".to_string(),
            email: "user1@example.com".to_string(),
            full_name: Some("홍길동".to_string()),
        };

        let json_str = serde_json::to_string(&user_info).expect("Failed to serialize");
        assert!(json_str.contains("id"));
        assert!(json_str.contains("username"));
        assert!(json_str.contains("email"));
        assert!(json_str.contains("full_name"));

        let deserialized: SeriesNoteUserInfo =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.id, 456);
        assert_eq!(deserialized.username, "user1");
        assert_eq!(deserialized.email, "user1@example.com");
        assert_eq!(deserialized.full_name, Some("홍길동".to_string()));
    }

    #[test]
    fn test_series_note_user_info_without_full_name() {
        let user_info = SeriesNoteUserInfo {
            id: 457,
            username: "user2".to_string(),
            email: "user2@example.com".to_string(),
            full_name: None,
        };

        let json_str = serde_json::to_string(&user_info).expect("Failed to serialize");
        let deserialized: SeriesNoteUserInfo =
            serde_json::from_str(&json_str).expect("Failed to deserialize");
        assert_eq!(deserialized.full_name, None);
    }
}

