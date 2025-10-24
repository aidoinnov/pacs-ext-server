pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;

// ServiceError를 직접 export
pub use domain::ServiceError;

// 테스트 모듈들
#[cfg(test)]
mod tests {
    pub mod user_registration_service_unit_test;
    pub mod user_registration_use_case_unit_test;
    pub mod user_registration_controller_unit_test;
}

// 통합 테스트
#[cfg(test)]
mod integration_tests {
    pub mod user_registration_integration_test;
}
