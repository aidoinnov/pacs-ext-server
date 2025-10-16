/// OpenAPI 서버 URL 테스트
/// openapi.rs에서 추가된 프로덕션 서버 URL을 테스트합니다.
#[cfg(test)]
mod tests {
    use utoipa::OpenApi;
    use pacs_server::presentation::openapi::ApiDoc;

    #[test]
    fn test_openapi_doc_compilation() {
        // ApiDoc이 정상적으로 컴파일되는지 확인
        let _openapi = ApiDoc::openapi();
        assert!(true); // 컴파일이 성공하면 테스트 통과
    }

    #[test]
    fn test_openapi_servers_include_production() {
        let openapi = ApiDoc::openapi();
        let servers = openapi.servers.as_ref().unwrap();
        
        // 서버 목록이 존재하는지 확인
        assert!(!servers.is_empty(), "서버 목록이 비어있으면 안됩니다");
        
        // 최소 3개의 서버가 있는지 확인 (localhost, 0.0.0.0, production)
        assert!(servers.len() >= 3, "최소 3개의 서버 URL이 있어야 합니다");
        
        // 각 서버 URL이 유효한 형식인지 확인
        for server in servers {
            assert!(!server.url.is_empty(), "서버 URL은 비어있으면 안됩니다");
            assert!(
                server.url.starts_with("http://") || server.url.starts_with("https://"),
                "서버 URL은 http:// 또는 https://로 시작해야 합니다: {}",
                server.url
            );
            if let Some(description) = &server.description {
                assert!(
                    !description.is_empty(),
                    "서버 설명은 비어있으면 안됩니다"
                );
            }
        }
    }

    #[test]
    fn test_openapi_servers_include_expected_urls() {
        let openapi = ApiDoc::openapi();
        let servers = openapi.servers.as_ref().unwrap();
        
        let server_urls: Vec<&str> = servers.iter().map(|s| s.url.as_str()).collect();
        
        // 예상되는 서버 URL들이 포함되어 있는지 확인
        assert!(
            server_urls.contains(&"http://localhost:8080"),
            "localhost:8080 서버 URL이 포함되어야 합니다"
        );
        
        assert!(
            server_urls.contains(&"http://0.0.0.0:8080"),
            "0.0.0.0:8080 서버 URL이 포함되어야 합니다"
        );
        
        assert!(
            server_urls.contains(&"https://api.pacs-server.com"),
            "프로덕션 서버 URL이 포함되어야 합니다"
        );
    }

    #[test]
    fn test_openapi_servers_descriptions() {
        let openapi = ApiDoc::openapi();
        let servers = openapi.servers.as_ref().unwrap();
        
        let descriptions: Vec<&str> = servers.iter()
            .filter_map(|s| s.description.as_ref().map(|d| d.as_str()))
            .collect();
        
        // 각 서버에 적절한 설명이 있는지 확인
        assert!(
            descriptions.iter().any(|desc| desc.contains("Local development")),
            "로컬 개발 서버 설명이 포함되어야 합니다"
        );
        
        assert!(
            descriptions.iter().any(|desc| desc.contains("all interfaces")),
            "모든 인터페이스 서버 설명이 포함되어야 합니다"
        );
        
        assert!(
            descriptions.iter().any(|desc| desc.contains("Production")),
            "프로덕션 서버 설명이 포함되어야 합니다"
        );
    }

    #[test]
    fn test_openapi_servers_url_format_validation() {
        let openapi = ApiDoc::openapi();
        let servers = openapi.servers.as_ref().unwrap();
        
        for server in servers {
            let url = &server.url;
            
            // URL 형식 검증
            assert!(
                url.starts_with("http://") || url.starts_with("https://"),
                "URL은 http:// 또는 https://로 시작해야 합니다: {}",
                url
            );
            
            // 포트 번호가 포함되어 있는지 확인 (프로덕션 제외)
            if url.contains("localhost") || url.contains("0.0.0.0") {
                assert!(
                    url.contains(":8080"),
                    "로컬 서버는 포트 8080을 포함해야 합니다: {}",
                    url
                );
            }
            
            // 프로덕션 URL은 https를 사용해야 함
            if url.contains("api.pacs-server.com") {
                assert!(
                    url.starts_with("https://"),
                    "프로덕션 서버는 https를 사용해야 합니다: {}",
                    url
                );
            }
        }
    }

    #[test]
    fn test_openapi_servers_no_duplicates() {
        let openapi = ApiDoc::openapi();
        let servers = openapi.servers.as_ref().unwrap();
        
        let mut urls = Vec::new();
        for server in servers {
            assert!(
                !urls.contains(&server.url),
                "중복된 서버 URL이 있습니다: {}",
                server.url
            );
            urls.push(server.url.clone());
        }
    }

    #[test]
    fn test_openapi_servers_environment_coverage() {
        let openapi = ApiDoc::openapi();
        let servers = openapi.servers.as_ref().unwrap();
        
        let server_urls: Vec<&str> = servers.iter().map(|s| s.url.as_str()).collect();
        
        // 개발 환경 커버리지
        let has_localhost = server_urls.iter().any(|url| url.contains("localhost"));
        let has_all_interfaces = server_urls.iter().any(|url| url.contains("0.0.0.0"));
        
        // 프로덕션 환경 커버리지
        let has_production = server_urls.iter().any(|url| url.contains("api.pacs-server.com"));
        
        assert!(has_localhost, "로컬호스트 서버가 포함되어야 합니다");
        assert!(has_all_interfaces, "모든 인터페이스 서버가 포함되어야 합니다");
        assert!(has_production, "프로덕션 서버가 포함되어야 합니다");
    }
}
