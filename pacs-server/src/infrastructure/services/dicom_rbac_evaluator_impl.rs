use crate::domain::services::dicom_rbac_evaluator::{DicomRbacEvaluator, RbacEvaluationResult};
use crate::domain::entities::access_condition::{AccessCondition, ConditionType};
use sqlx::PgPool;

pub struct DicomRbacEvaluatorImpl {
    pub pool: PgPool,
}

impl DicomRbacEvaluatorImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 사용자의 프로젝트 내 역할 ID 조회
    async fn get_user_role_id(&self, user_id: i32, project_id: i32) -> Option<i32> {
        sqlx::query_scalar(
            "SELECT role_id FROM security_user_project WHERE user_id = $1 AND project_id = $2",
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()
    }

    /// Study의 DICOM 태그 값 조회 (조건 평가용)
    async fn get_study_dicom_values(&self, study_id: i32) -> StudyDicomValues {
        // study_date는 DATE 타입이므로 DICOM 형식(YYYYMMDD)으로 변환 필요
        let result: Option<(Option<String>, Option<chrono::NaiveDate>, Option<String>)> = sqlx::query_as(
            "SELECT modality, study_date, patient_id 
             FROM project_data_study 
             WHERE id = $1"
        )
        .bind(study_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        if let Some((modality, study_date, patient_id)) = result {
            // NaiveDate를 DICOM 형식(YYYYMMDD)으로 변환
            let study_date_str = study_date.map(|d| d.format("%Y%m%d").to_string());
            StudyDicomValues {
                modality,
                study_date: study_date_str,
                patient_id,
            }
        } else {
            StudyDicomValues::default()
        }
    }

    /// Series의 DICOM 태그 값 조회 (조건 평가용)
    async fn get_series_dicom_values(&self, series_id: i32) -> SeriesDicomValues {
        // Series는 직접적인 DICOM 태그가 적으므로 상위 Study에서 상속받는 경우가 많음
        let study_id: Option<i32> = sqlx::query_scalar(
            "SELECT study_id FROM project_data_series WHERE id = $1"
        )
        .bind(series_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        if let Some(sid) = study_id {
            let study_values = self.get_study_dicom_values(sid).await;
            SeriesDicomValues {
                modality: study_values.modality,
                study_date: study_values.study_date,
                patient_id: study_values.patient_id,
            }
        } else {
            SeriesDicomValues::default()
        }
    }

    /// 조건 평가: DICOM 태그 값이 조건을 만족하는지 확인
    fn evaluate_condition(&self, condition: &AccessCondition, dicom_values: &StudyDicomValues) -> bool {
        if let Some(ref dicom_tag) = condition.dicom_tag {
            let actual_value = match dicom_tag.as_str() {
                "00080060" | "Modality" => dicom_values.modality.as_deref(),
                "00100020" | "PatientID" => dicom_values.patient_id.as_deref(),
                "00080020" | "StudyDate" => dicom_values.study_date.as_deref(),
                _ => None,
            };

            if let (Some(actual), Some(condition_value)) = (actual_value, &condition.value) {
                match condition.operator.as_str() {
                    "EQ" | "EQUALS" | "==" => actual == condition_value,
                    "NE" | "NOT_EQUALS" | "!=" => actual != condition_value,
                    "RANGE" | "BETWEEN" => {
                        // StudyDate 범위 비교 (YYYYMMDD-YYYYMMDD 형식)
                        if dicom_tag == "00080020" || dicom_tag == "StudyDate" {
                            self.check_date_range(actual, condition_value)
                        } else {
                            false
                        }
                    }
                    "CONTAINS" | "LIKE" => actual.contains(condition_value),
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// 날짜 범위 확인 (YYYYMMDD-YYYYMMDD 형식)
    fn check_date_range(&self, actual_date: &str, range_str: &str) -> bool {
        if let Some((start_str, end_str)) = range_str.split_once('-') {
            if let (Ok(actual), Ok(start), Ok(end)) = (
                actual_date.parse::<u32>(),
                start_str.trim().parse::<u32>(),
                end_str.trim().parse::<u32>(),
            ) {
                return actual >= start && actual <= end;
            }
        }
        false
    }

    /// 룰 기반 조건 평가 (프로젝트 및 역할 조건)
    async fn evaluate_rule_based_conditions(
        &self,
        user_id: i32,
        project_id: i32,
        dicom_values: &StudyDicomValues,
        resource_level: &str,
    ) -> RbacEvaluationResult {
        // 1) 프로젝트별 조건 조회 및 평가
        let project_conditions: Vec<AccessCondition> = sqlx::query_as::<_, AccessCondition>(
            "SELECT ac.id, ac.resource_type, ac.resource_level, ac.dicom_tag, ac.operator, ac.value, ac.condition_type, ac.created_at \
             FROM security_access_condition ac \
             JOIN security_project_dicom_condition pc ON pc.access_condition_id = ac.id \
             WHERE pc.project_id = $1 \
             ORDER BY pc.priority DESC, ac.id ASC"
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        if !project_conditions.is_empty() {
            for condition in project_conditions.iter() {
                // 리소스 레벨 일치 확인 (ResourceLevel은 SCREAMING_SNAKE_CASE로 저장됨)
                let condition_level_str = match condition.resource_level {
                    crate::domain::entities::access_condition::ResourceLevel::Study => "STUDY",
                    crate::domain::entities::access_condition::ResourceLevel::Series => "SERIES",
                    crate::domain::entities::access_condition::ResourceLevel::Instance => "INSTANCE",
                };
                if condition_level_str == resource_level.to_uppercase() {
                    let matches = self.evaluate_condition(condition, dicom_values);
                    
                    match condition.condition_type {
                        ConditionType::Allow => {
                            if matches {
                                return RbacEvaluationResult {
                                    allowed: true,
                                    reason: Some(format!("rule_based_allow: project_condition_{}", condition.id)),
                                };
                            }
                        }
                        ConditionType::Deny => {
                            if matches {
                                return RbacEvaluationResult {
                                    allowed: false,
                                    reason: Some(format!("rule_based_deny: project_condition_{}", condition.id)),
                                };
                            }
                        }
                        ConditionType::Limit => {
                            // Limit는 나중에 구현 가능 (예: 제한된 필드만 접근)
                            if matches {
                                return RbacEvaluationResult {
                                    allowed: true,
                                    reason: Some(format!("rule_based_limit: project_condition_{}", condition.id)),
                                };
                            }
                        }
                    }
                }
            }
        }

        // 2) 역할별 조건 조회 및 평가
        if let Some(role_id) = self.get_user_role_id(user_id, project_id).await {
            let role_conditions: Vec<AccessCondition> = sqlx::query_as::<_, AccessCondition>(
                "SELECT ac.id, ac.resource_type, ac.resource_level, ac.dicom_tag, ac.operator, ac.value, ac.condition_type, ac.created_at \
                 FROM security_access_condition ac \
                 JOIN security_role_dicom_condition rc ON rc.access_condition_id = ac.id \
                 WHERE rc.role_id = $1 \
                 ORDER BY rc.priority DESC, ac.id ASC"
            )
            .bind(role_id)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();

            if !role_conditions.is_empty() {
                for condition in role_conditions.iter() {
                    let condition_level_str = match condition.resource_level {
                        crate::domain::entities::access_condition::ResourceLevel::Study => "STUDY",
                        crate::domain::entities::access_condition::ResourceLevel::Series => "SERIES",
                        crate::domain::entities::access_condition::ResourceLevel::Instance => "INSTANCE",
                    };
                    if condition_level_str == resource_level.to_uppercase() {
                        let matches = self.evaluate_condition(condition, dicom_values);
                        
                        match condition.condition_type {
                            ConditionType::Allow => {
                                if matches {
                                    return RbacEvaluationResult {
                                        allowed: true,
                                        reason: Some(format!("rule_based_allow: role_condition_{}", condition.id)),
                                    };
                                }
                            }
                            ConditionType::Deny => {
                                if matches {
                                    return RbacEvaluationResult {
                                        allowed: false,
                                        reason: Some(format!("rule_based_deny: role_condition_{}", condition.id)),
                                    };
                                }
                            }
                            ConditionType::Limit => {
                                if matches {
                                    return RbacEvaluationResult {
                                        allowed: true,
                                        reason: Some(format!("rule_based_limit: role_condition_{}", condition.id)),
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }

        RbacEvaluationResult {
            allowed: false,
            reason: Some("no_matching_rule".to_string()),
        }
    }
}

/// Study의 DICOM 태그 값들을 담는 구조체
#[derive(Default)]
struct StudyDicomValues {
    modality: Option<String>,
    study_date: Option<String>,
    patient_id: Option<String>,
}

/// Series의 DICOM 태그 값들 (Study에서 상속)
#[derive(Default)]
struct SeriesDicomValues {
    modality: Option<String>,
    study_date: Option<String>,
    patient_id: Option<String>,
}

#[async_trait::async_trait]
impl DicomRbacEvaluator for DicomRbacEvaluatorImpl {
    async fn evaluate_study_access(&self, user_id: i32, project_id: i32, study_id: i32) -> RbacEvaluationResult {
        // 0) 프로젝트 멤버십 확인 (필수)
        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM security_user_project WHERE user_id = $1 AND project_id = $2)",
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false);

        if !is_member {
            return RbacEvaluationResult { 
                allowed: false, 
                reason: Some("user_not_project_member".to_string()) 
            };
        }

        // 1) 기관 기반 접근 (같은 기관 또는 기관 간 허용)
        let user_inst: Option<i32> = sqlx::query_scalar(
            "SELECT institution_id FROM security_user WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        let data_inst: Option<i32> = sqlx::query_scalar(
            "SELECT data_institution_id FROM project_data_study WHERE id = $1",
        )
        .bind(study_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        if let (Some(u), Some(d)) = (user_inst, data_inst) {
            if u == d {
                return RbacEvaluationResult { allowed: true, reason: Some("same_institution".to_string()) };
            }

            let has_cross_access: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM security_institution_data_access WHERE user_institution_id = $1 AND data_institution_id = $2 AND is_active = true)",
            )
            .bind(u)
            .bind(d)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(false);

            if has_cross_access {
                return RbacEvaluationResult { allowed: true, reason: Some("institution_cross_access".to_string()) };
            }
        }

        // 2) 명시적 접근 권한 (study 레벨)
        let has_explicit: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM project_data_access WHERE user_id = $1 AND project_id = $2 AND status = 'APPROVED' AND resource_level = 'STUDY' AND study_id = $3)",
        )
        .bind(user_id)
        .bind(project_id)
        .bind(study_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false);

        if has_explicit {
            return RbacEvaluationResult { allowed: true, reason: Some("explicit_study_access".to_string()) };
        }

        // 3) 룰 기반 조건 평가 (access_condition + role/project)
        let dicom_values = self.get_study_dicom_values(study_id).await;
        self.evaluate_rule_based_conditions(user_id, project_id, &dicom_values, "STUDY").await
    }

    async fn evaluate_series_access(&self, user_id: i32, project_id: i32, series_id: i32) -> RbacEvaluationResult {
        // 0) 프로젝트 멤버십 확인 (필수)
        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM security_user_project WHERE user_id = $1 AND project_id = $2)",
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false);

        if !is_member {
            return RbacEvaluationResult { 
                allowed: false, 
                reason: Some("user_not_project_member".to_string()) 
            };
        }

        // 1) series에 대한 명시적 권한
        let has_explicit_series: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM project_data_access WHERE user_id = $1 AND project_id = $2 AND status = 'APPROVED' AND resource_level = 'SERIES' AND series_id = $3)",
        )
        .bind(user_id)
        .bind(project_id)
        .bind(series_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false);

        if has_explicit_series {
            return RbacEvaluationResult { allowed: true, reason: Some("explicit_series_access".to_string()) };
        }

        // 2) series가 포함된 study에 대한 권한 상속
        let parent_study_id: Option<i32> = sqlx::query_scalar(
            "SELECT study_id FROM project_data_series WHERE id = $1",
        )
        .bind(series_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        if let Some(study_id) = parent_study_id {
            let study_result = self.evaluate_study_access(user_id, project_id, study_id).await;
            if study_result.allowed {
                return RbacEvaluationResult { allowed: true, reason: Some("inherited_from_study".to_string()) };
            }
        }

        // 3) 룰 기반 조건 평가 (series는 상위 study의 값으로 평가)
        let parent_study_id: Option<i32> = sqlx::query_scalar(
            "SELECT study_id FROM project_data_series WHERE id = $1",
        )
        .bind(series_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        if let Some(sid) = parent_study_id {
            let dicom_values = self.get_study_dicom_values(sid).await;
            self.evaluate_rule_based_conditions(user_id, project_id, &dicom_values, "SERIES").await
        } else {
            RbacEvaluationResult { allowed: false, reason: Some("series_parent_study_not_found".to_string()) }
        }
    }
    
    async fn evaluate_study_uid(&self, user_id: i32, project_id: i32, study_uid: &str) -> RbacEvaluationResult {
        let study_id: Option<i32> = sqlx::query_scalar(
            "SELECT id FROM project_data_study WHERE project_id = $1 AND study_uid = $2",
        )
        .bind(project_id)
        .bind(study_uid)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        if let Some(sid) = study_id {
            return self.evaluate_study_access(user_id, project_id, sid).await;
        }
        RbacEvaluationResult { allowed: false, reason: Some("study_not_found".to_string()) }
    }

    async fn evaluate_series_uid(&self, user_id: i32, project_id: i32, series_uid: &str) -> RbacEvaluationResult {
        // series_uid로 series 찾기 (project_id로 필터링하여 정확도 향상)
        let series_id: Option<i32> = sqlx::query_scalar(
            "SELECT pds.id FROM project_data_series pds
             JOIN project_data_study pdt ON pds.study_id = pdt.id
             WHERE pds.series_uid = $1 AND pdt.project_id = $2",
        )
        .bind(series_uid)
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        if let Some(series_id) = series_id {
            return self.evaluate_series_access(user_id, project_id, series_id).await;
        }
        RbacEvaluationResult { allowed: false, reason: Some("series_not_found".to_string()) }
    }

    async fn evaluate_instance_access(&self, user_id: i32, project_id: i32, instance_id: i32) -> RbacEvaluationResult {
        // 0) 프로젝트 멤버십 확인 (필수)
        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM security_user_project WHERE user_id = $1 AND project_id = $2)",
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false);

        if !is_member {
            return RbacEvaluationResult { 
                allowed: false, 
                reason: Some("user_not_project_member".to_string()) 
            };
        }

        // 1) instance에 대한 명시적 권한
        let has_explicit_instance: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM project_data_access WHERE user_id = $1 AND project_id = $2 AND status = 'APPROVED' AND resource_level = 'INSTANCE' AND instance_id = $3)",
        )
        .bind(user_id)
        .bind(project_id)
        .bind(instance_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false);

        if has_explicit_instance {
            return RbacEvaluationResult { allowed: true, reason: Some("explicit_instance_access".to_string()) };
        }

        // 2) instance가 포함된 series에 대한 권한 상속
        let parent_series_id: Option<i32> = sqlx::query_scalar(
            "SELECT series_id FROM project_data_instance WHERE id = $1",
        )
        .bind(instance_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        if let Some(series_id) = parent_series_id {
            let series_result = self.evaluate_series_access(user_id, project_id, series_id).await;
            if series_result.allowed {
                return RbacEvaluationResult { allowed: true, reason: Some("inherited_from_series".to_string()) };
            }
            
            // 3) 룰 기반 조건 평가 (instance는 상위 study의 값으로 평가)
            let parent_study_id: Option<i32> = sqlx::query_scalar(
                "SELECT study_id FROM project_data_series WHERE id = $1",
            )
            .bind(series_id)
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten();

            if let Some(sid) = parent_study_id {
                let dicom_values = self.get_study_dicom_values(sid).await;
                return self.evaluate_rule_based_conditions(user_id, project_id, &dicom_values, "INSTANCE").await;
            }
        }

        RbacEvaluationResult { allowed: false, reason: Some("instance_parent_study_not_found".to_string()) }
    }

    async fn evaluate_instance_uid(&self, user_id: i32, project_id: i32, instance_uid: &str) -> RbacEvaluationResult {
        // instance_uid로 instance 찾기 (project_id로 필터링하여 정확도 향상)
        let instance_id: Option<i32> = sqlx::query_scalar(
            "SELECT pdi.id FROM project_data_instance pdi
             JOIN project_data_series pds ON pdi.series_id = pds.id
             JOIN project_data_study pdt ON pds.study_id = pdt.id
             WHERE pdi.instance_uid = $1 AND pdt.project_id = $2",
        )
        .bind(instance_uid)
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        if let Some(instance_id) = instance_id {
            return self.evaluate_instance_access(user_id, project_id, instance_id).await;
        }
        RbacEvaluationResult { allowed: false, reason: Some("instance_not_found".to_string()) }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::access_condition::{ResourceLevel, ConditionType};
    use sqlx::postgres::PgConnectOptions;

    fn dummy_evaluator() -> DicomRbacEvaluatorImpl {
        // Lazy pool that won't actually connect unless used
        let opts = PgConnectOptions::new()
            .host("localhost")
            .port(5432)
            .database("dummy")
            .username("dummy")
            .password("dummy");
        let pool = sqlx::postgres::PgPoolOptions::new().connect_lazy_with(opts);
        DicomRbacEvaluatorImpl::new(pool)
    }

    fn make_cond(tag: &str, op: &str, val: &str) -> AccessCondition {
        AccessCondition {
            id: 1,
            resource_type: "DICOM".to_string(),
            resource_level: ResourceLevel::Study,
            dicom_tag: Some(tag.to_string()),
            operator: op.to_string(),
            value: Some(val.to_string()),
            condition_type: ConditionType::Allow,
            created_at: chrono::Utc::now(),
        }
    }

    fn values(modality: Option<&str>, study_date: Option<&str>, patient_id: Option<&str>) -> StudyDicomValues {
        StudyDicomValues {
            modality: modality.map(|s| s.to_string()),
            study_date: study_date.map(|s| s.to_string()),
            patient_id: patient_id.map(|s| s.to_string()),
        }
    }

    #[tokio::test]
    async fn test_eq_condition_modality_matches() {
        let eval = dummy_evaluator();
        let cond = make_cond("00080060", "EQ", "CT");
        let v = values(Some("CT"), None, None);
        assert!(eval.evaluate_condition(&cond, &v));
    }

    #[tokio::test]
    async fn test_ne_condition_patient_id_not_match() {
        let eval = dummy_evaluator();
        let cond = make_cond("00100020", "NE", "P123");
        let v = values(None, None, Some("P999"));
        assert!(eval.evaluate_condition(&cond, &v));
    }

    #[tokio::test]
    async fn test_range_condition_study_date_inclusive() {
        let eval = dummy_evaluator();
        let cond = make_cond("00080020", "RANGE", "20200101-20201231");
        let v = values(None, Some("20200615"), None);
        assert!(eval.evaluate_condition(&cond, &v));
    }

    #[tokio::test]
    async fn test_range_condition_study_date_outside() {
        let eval = dummy_evaluator();
        let cond = make_cond("StudyDate", "BETWEEN", "20200101-20201231");
        let v = values(None, Some("20191231"), None);
        assert!(!eval.evaluate_condition(&cond, &v));
    }

    #[tokio::test]
    async fn test_contains_operator() {
        let eval = dummy_evaluator();
        let cond = make_cond("Modality", "CONTAINS", "CT");
        let v = values(Some("CT+PT"), None, None);
        assert!(eval.evaluate_condition(&cond, &v));
    }

    #[tokio::test]
    async fn test_range_condition_inclusive_bounds() {
        let eval = dummy_evaluator();
        let cond = make_cond("00080020", "RANGE", "20200101-20200131");
        // start bound
        let v1 = values(None, Some("20200101"), None);
        assert!(eval.evaluate_condition(&cond, &v1));
        // end bound
        let v2 = values(None, Some("20200131"), None);
        assert!(eval.evaluate_condition(&cond, &v2));
    }

    #[tokio::test]
    async fn test_unknown_tag_returns_false() {
        let eval = dummy_evaluator();
        let cond = make_cond("99990000", "EQ", "X");
        let v = values(Some("X"), None, None);
        assert!(!eval.evaluate_condition(&cond, &v));
    }

    #[tokio::test]
    async fn test_like_not_matching_returns_false() {
        let eval = dummy_evaluator();
        let cond = make_cond("PatientID", "LIKE", "ABC");
        let v = values(None, None, Some("ZZZ123"));
        assert!(!eval.evaluate_condition(&cond, &v));
    }

    #[tokio::test]
    async fn test_priority_deny_overrides_allow_value_level() {
        let eval = dummy_evaluator();
        let allow = make_cond("00080060", "EQ", "CT");
        let mut deny = allow.clone();
        deny.condition_type = ConditionType::Deny;
        let v = values(Some("CT"), None, None);
        assert!(eval.evaluate_condition(&allow, &v));
        assert!(eval.evaluate_condition(&deny, &v));
        // 병합/우선순위는 상위 로직에서 수행되므로 여기서는 동시 매치 입력만 확인
    }

    #[tokio::test]
    async fn test_limit_intersection_semantics_value_level() {
        let eval = dummy_evaluator();
        let allow_ct = make_cond("00080060", "EQ", "CT");
        let limit_2024 = make_cond("00080020", "RANGE", "20240101-20241231");
        let v_ok = values(Some("CT"), Some("20240601"), None);
        let v_out = values(Some("CT"), Some("20231231"), None);
        assert!(eval.evaluate_condition(&allow_ct, &v_ok));
        assert!(eval.evaluate_condition(&limit_2024, &v_ok));
        assert!(eval.evaluate_condition(&allow_ct, &v_out));
        assert!(!eval.evaluate_condition(&limit_2024, &v_out));
    }
}

