use pacs_server::domain::view_selection::{ViewSelection, SelectedSeries};

#[tokio::test]
async fn test_view_selection_new() {
    let selection_id = "sel_test123".to_string();
    let series = vec![
        SelectedSeries {
            study_uid: "1.2.3".to_string(),
            series_uid: "1.2.3.4".to_string(),
        },
        SelectedSeries {
            study_uid: "2.3.4".to_string(),
            series_uid: "2.3.4.5".to_string(),
        },
    ];
    let user_id = 1;
    let ttl_sec = 1800; // 30 minutes

    let selection = ViewSelection::new(selection_id.clone(), series.clone(), user_id, ttl_sec);

    assert_eq!(selection.selection_id, selection_id);
    assert_eq!(selection.series.len(), 2);
    assert_eq!(selection.series, series);
    assert_eq!(selection.user_id, user_id);
    assert!(!selection.is_expired());
    
    // expires_at이 created_at보다 ttl_sec만큼 늦어야 함
    let diff = (selection.expires_at - selection.created_at).num_seconds();
    assert_eq!(diff, ttl_sec as i64);
}

#[tokio::test]
async fn test_view_selection_is_expired() {
    let selection_id = "sel_test123".to_string();
    let series = vec![SelectedSeries {
        study_uid: "1.2.3".to_string(),
        series_uid: "1.2.3.4".to_string(),
    }];
    let user_id = 1;
    
    // 만료된 Selection 생성 (TTL이 0)
    let mut selection = ViewSelection::new(selection_id, series, user_id, 0);
    
    // 시간이 지나도록 강제로 만료 시각을 과거로 설정
    use chrono::{Duration, Utc};
    selection.expires_at = Utc::now() - Duration::seconds(1);
    
    assert!(selection.is_expired());
}

#[tokio::test]
async fn test_view_selection_extend_ttl() {
    let selection_id = "sel_test123".to_string();
    let series = vec![SelectedSeries {
        study_uid: "1.2.3".to_string(),
        series_uid: "1.2.3.4".to_string(),
    }];
    let user_id = 1;
    let initial_ttl = 1800;
    
    let mut selection = ViewSelection::new(selection_id, series, user_id, initial_ttl);
    let original_expires_at = selection.expires_at;
    
    // TTL 연장
    let new_ttl = 3600;
    selection.extend_ttl(new_ttl);
    
    // expires_at이 업데이트되어야 함
    assert!(selection.expires_at > original_expires_at);
    assert!(!selection.is_expired());
    
    // 새로운 TTL이 적용되었는지 확인
    let diff = (selection.expires_at - selection.created_at).num_seconds();
    assert!(diff >= new_ttl as i64);
}

#[tokio::test]
async fn test_selected_series_equality() {
    let series1 = SelectedSeries {
        study_uid: "1.2.3".to_string(),
        series_uid: "1.2.3.4".to_string(),
    };
    
    let series2 = SelectedSeries {
        study_uid: "1.2.3".to_string(),
        series_uid: "1.2.3.4".to_string(),
    };
    
    let series3 = SelectedSeries {
        study_uid: "2.3.4".to_string(),
        series_uid: "2.3.4.5".to_string(),
    };
    
    assert_eq!(series1, series2);
    assert_ne!(series1, series3);
}


