// Intentionally lightweight and ignored by default to avoid external deps

#[ignore]
#[actix_rt::test]
async fn rbac_rule_merge_priority_deny_overrides_allow() {
    // GIVEN: 역할 규칙 ALLOW(Modality=CT), 프로젝트 규칙 DENY(StudyDate out of range)
    // WHEN: evaluator 병합/평가 수행
    // THEN: DENY가 우선되어 거부되어야 한다
}

#[ignore]
#[actix_rt::test]
async fn rbac_rule_merge_limit_intersection() {
    // GIVEN: 역할 규칙 ALLOW(Modality=CT), 프로젝트 규칙 LIMIT(StudyDate=20240101-20241231)
    // WHEN: evaluator 병합/평가 수행
    // THEN: CT 이면서 2024년 기간에 해당하는 데이터만 허용된다
}

#[ignore]
#[actix_rt::test]
async fn rbac_explicit_access_overrides_rules_at_level() {
    // GIVEN: 해당 Study에 대해 명시 권한 존재 + 규칙이 존재하더라도
    // WHEN: evaluator 평가
    // THEN: 해당 레벨에서 명시 권한이 우선 허용된다
}


