use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct RunE2ERequest {
    pub script: String,
}

#[derive(Debug, Serialize)]
pub struct RunE2EResponse {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// E2E 테스트 스크립트 실행
///
/// Python E2E 테스트 스크립트를 실행하고 결과를 반환합니다.
pub async fn run_e2e_test(req: web::Json<RunE2ERequest>) -> impl Responder {
    let script_name = &req.script;
    
    // 보안: 허용된 스크립트만 실행
    let allowed_scripts = vec![
        "test_annotation_snapshot_e2e.py",
        "test_me_studies.py",
        "test_keycloak_qido_direct.py",
        "test_all_studies_access.py",
        "test_includefield.py",
        "test_includefield_detailed.py",
        "test_keycloak_direct_login.py",
        "test_study_description_includefield.py",
        "series_all.py",
        "compare_studies_endpoints.py",
        "analyze_duplicates.py",
    ];
    
    if !allowed_scripts.contains(&script_name.as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Script not allowed",
            "allowed_scripts": allowed_scripts,
        }));
    }
    
    // e2e 디렉토리 경로 구성
    let mut script_path = PathBuf::from("e2e");
    script_path.push(script_name);
    
    // 스크립트 존재 확인
    if !script_path.exists() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Script not found: {}", script_path.display()),
        }));
    }
    
    // Python 스크립트 실행
    let output = match Command::new("python3")
        .arg(&script_path)
        .current_dir(".")
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to execute script: {}", e),
            }));
        }
    };
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();
    let exit_code = output.status.code();
    
    let response = RunE2EResponse {
        success,
        stdout,
        stderr,
        exit_code,
    };
    
    if success {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::Ok().json(response) // 스크립트 실행은 성공했지만 테스트가 실패한 경우도 200 반환
    }
}

/// 사용 가능한 E2E 테스트 목록 조회
pub async fn list_e2e_tests() -> impl Responder {
    let tests = vec![
        serde_json::json!({
            "name": "Annotation Snapshot Upload",
            "script": "test_annotation_snapshot_e2e.py",
            "description": "어노테이션 스냅샷 이미지 업로드 전체 워크플로우 테스트",
        }),
        serde_json::json!({
            "name": "Me Studies Endpoint",
            "script": "test_me_studies.py",
            "description": "/api/me/dicom/studies 엔드포인트 테스트",
        }),
        serde_json::json!({
            "name": "Keycloak QIDO Direct",
            "script": "test_keycloak_qido_direct.py",
            "description": "Keycloak 토큰으로 Dcm4chee QIDO 직접 요청 테스트",
        }),
        serde_json::json!({
            "name": "All Studies Access",
            "script": "test_all_studies_access.py",
            "description": "전체 Studies 접근 권한 테스트",
        }),
        serde_json::json!({
            "name": "Include Field Test",
            "script": "test_includefield.py",
            "description": "DICOM includefield 파라미터 테스트",
        }),
        serde_json::json!({
            "name": "Include Field Detailed",
            "script": "test_includefield_detailed.py",
            "description": "DICOM includefield 상세 테스트",
        }),
        serde_json::json!({
            "name": "Keycloak Direct Login",
            "script": "test_keycloak_direct_login.py",
            "description": "Keycloak 직접 로그인 테스트",
        }),
        serde_json::json!({
            "name": "Study Description Include Field",
            "script": "test_study_description_includefield.py",
            "description": "Study Description includefield 테스트",
        }),
    ];
    
    HttpResponse::Ok().json(tests)
}

/// 라우트 설정
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/e2e")
            .route("/run", web::post().to(run_e2e_test))
            .route("/list", web::get().to(list_e2e_tests)),
    );
}

