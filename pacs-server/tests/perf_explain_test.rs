use std::env;
use std::process::Command;

fn run_explain(sql: &str, outfile: &str) -> Result<String, String> {
    let app_root = env::current_dir().map_err(|e| e.to_string())?;
    let script_path = app_root.join("scripts/explain.sh");
    let db_url = env::var("APP_DATABASE_URL").map_err(|e| e.to_string())?;

    let output = Command::new(script_path)
        .env("APP_DATABASE_URL", db_url)
        .arg(sql)
        .arg(outfile)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("explain.sh failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}

fn should_run() -> bool {
    env::var("APP_DATABASE_URL").is_ok()
}

#[test]
fn explain_study_uid_mapping() {
    if !should_run() {
        eprintln!("SKIP: APP_DATABASE_URL not set");
        return;
    }
    let sql = "SELECT id FROM project_data_study WHERE study_uid = '1.2.3.4.5' AND project_id = 1;";
    let out =
        run_explain(sql, "scripts/results/explain_study_uid_auto.txt").expect("explain failure");
    assert!(out.contains("EXPLAIN"));
}

#[test]
fn explain_project_membership() {
    if !should_run() {
        eprintln!("SKIP: APP_DATABASE_URL not set");
        return;
    }
    let sql = "SELECT 1 FROM security_user_project WHERE user_id = 10 AND project_id = 1;";
    let out = run_explain(sql, "scripts/results/explain_project_membership_auto.txt")
        .expect("explain failure");
    assert!(out.contains("EXPLAIN"));
}

#[test]
fn explain_explicit_access() {
    if !should_run() {
        eprintln!("SKIP: APP_DATABASE_URL not set");
        return;
    }
    let sql = "SELECT 1 FROM project_data_access WHERE user_id = 10 AND project_id = 1 AND resource_level = 'STUDY' AND study_id = 123 AND status = 'APPROVED' LIMIT 1;";
    let out = run_explain(sql, "scripts/results/explain_explicit_access_auto.txt")
        .expect("explain failure");
    assert!(out.contains("EXPLAIN"));
}

#[test]
fn explain_project_rules() {
    if !should_run() {
        eprintln!("SKIP: APP_DATABASE_URL not set");
        return;
    }
    let sql = "SELECT ac.* FROM security_access_condition ac JOIN security_project_dicom_condition pc ON pc.access_condition_id = ac.id WHERE pc.project_id = 1 ORDER BY pc.priority DESC, ac.id ASC;";
    let out = run_explain(sql, "scripts/results/explain_project_rules_auto.txt")
        .expect("explain failure");
    assert!(out.contains("EXPLAIN"));
}

#[test]
fn explain_study_list() {
    if !should_run() {
        eprintln!("SKIP: APP_DATABASE_URL not set");
        return;
    }
    let sql = "SELECT study_uid, modality, study_date, patient_id FROM project_data_study WHERE project_id = 1 AND study_date BETWEEN '2024-01-01' AND '2024-12-31' AND modality = 'CT';";
    let out =
        run_explain(sql, "scripts/results/explain_study_list_auto.txt").expect("explain failure");
    assert!(out.contains("EXPLAIN"));
}
