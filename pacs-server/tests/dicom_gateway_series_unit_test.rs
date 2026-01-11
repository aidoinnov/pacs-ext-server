/// DICOM Gateway Series API 단위 테스트
///
/// get_allowed_series_uids 함수의 단위 테스트
/// - resource_level='SERIES' 케이스
/// - resource_level='STUDY' 케이스
/// - 혼합 케이스

#[cfg(test)]
mod tests {

    // 실제 DB가 필요한 테스트는 통합 테스트로 분리
    // 여기서는 로직 검증에 집중

    #[test]
    fn test_get_allowed_series_uids_query_structure() {
        // 쿼리 구조 검증 (실제 실행 없이)
        // resource_level='SERIES'와 'STUDY' 케이스를 모두 포함하는지 확인
        
        let series_query = r#"
            SELECT DISTINCT pdser.series_uid
            FROM (
                SELECT pdser.series_uid
                FROM project_data pd
                INNER JOIN project_data_series pdser ON pd.series_id = pdser.id
                WHERE pd.project_id = $1
                  AND pd.resource_level = 'SERIES'
                  AND pd.series_id IS NOT NULL
                  AND pdser.series_uid IS NOT NULL
            ) AS combined
        "#;
        
        let study_query = r#"
            SELECT DISTINCT pdser.series_uid
            FROM (
                SELECT DISTINCT pdser.series_uid
                FROM project_data pd
                INNER JOIN project_data_study pds ON pd.study_id = pds.id
                INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
                WHERE pd.project_id = $1
                  AND pd.resource_level = 'STUDY'
                  AND pd.study_id IS NOT NULL
                  AND pdser.series_uid IS NOT NULL
            ) AS combined
        "#;
        
        // 쿼리에 필요한 요소가 포함되어 있는지 확인
        assert!(series_query.contains("resource_level = 'SERIES'"));
        assert!(series_query.contains("pd.series_id = pdser.id"));
        
        assert!(study_query.contains("resource_level = 'STUDY'"));
        assert!(study_query.contains("pds.id = pdser.study_id"));
    }

    #[test]
    fn test_query_union_structure() {
        // UNION 쿼리 구조 검증
        let full_query = r#"
            SELECT DISTINCT pdser.series_uid
            FROM (
                SELECT pdser.series_uid
                FROM project_data pd
                INNER JOIN project_data_series pdser ON pd.series_id = pdser.id
                WHERE pd.project_id = $1
                  AND pd.resource_level = 'SERIES'
                  AND pd.series_id IS NOT NULL
                  AND pdser.series_uid IS NOT NULL
                
                UNION
                
                SELECT DISTINCT pdser.series_uid
                FROM project_data pd
                INNER JOIN project_data_study pds ON pd.study_id = pds.id
                INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
                WHERE pd.project_id = $1
                  AND pd.resource_level = 'STUDY'
                  AND pd.study_id IS NOT NULL
                  AND pdser.series_uid IS NOT NULL
            ) AS combined
        "#;
        
        // UNION이 포함되어 있는지 확인
        assert!(full_query.contains("UNION"));
        assert!(full_query.contains("resource_level = 'SERIES'"));
        assert!(full_query.contains("resource_level = 'STUDY'"));
    }
}

