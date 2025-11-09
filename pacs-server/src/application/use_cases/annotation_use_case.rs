//! # 어노테이션 Use Case 모듈
//!
//! 이 모듈은 어노테이션과 관련된 비즈니스 로직을 처리하는 Use Case를 정의합니다.
//! Use Case는 도메인 서비스를 조합하여 특정 비즈니스 흐름을 구현합니다.
//!
//! ## 주요 기능
//! - 어노테이션 생성, 조회, 수정, 삭제
//! - 어노테이션 목록 조회 및 필터링
//! - 어노테이션 공유 설정
//! - 어노테이션 히스토리 관리
//!
//! ## 아키텍처
//! - Clean Architecture의 Use Case 계층에 해당
//! - 도메인 서비스를 조합하여 비즈니스 로직 구현
//! - 프레젠테이션 계층과 도메인 계층 사이의 중재자 역할

// 애플리케이션 레이어의 DTO 모듈들
use crate::application::dto::{
    AnnotationListResponse, AnnotationResponse, CreateAnnotationRequest, UpdateAnnotationRequest,
};
// 도메인 레이어의 서비스 인터페이스
use crate::domain::services::{AnnotationService, AccessControlService};
// 도메인 레이어의 에러 타입
use crate::domain::ServiceError;
// 도메인 레이어의 엔티티
use crate::domain::entities::{Annotation, NewAnnotation};
// 도메인 레이어의 리포지토리
use crate::domain::repositories::UserRepository;
// 표준 라이브러리
use std::collections::HashMap;

/// 어노테이션 관리를 위한 Use Case
///
/// 이 구조체는 어노테이션과 관련된 모든 비즈니스 로직을 처리합니다.
/// 도메인 서비스를 조합하여 특정 비즈니스 흐름을 구현합니다.
///
/// # 제네릭 매개변수
/// - `A`: AnnotationService 트레이트를 구현하는 타입
/// - `U`: UserRepository 트레이트를 구현하는 타입
/// - `AC`: AccessControlService 트레이트를 구현하는 타입
///
/// # 필드
/// - `annotation_service`: 어노테이션 도메인 서비스
/// - `user_repository`: 사용자 리포지토리 (사용자 이름 조회용)
/// - `access_control_service`: 접근 제어 서비스 (권한 확인용)
///
/// # 예시
/// ```ignore
/// let annotation_use_case = AnnotationUseCase::new(annotation_service, user_repository, access_control_service);
/// let result = annotation_use_case.create_annotation(request, user_id, project_id).await;
/// ```
pub struct AnnotationUseCase<A: AnnotationService, U: UserRepository, AC: AccessControlService> {
    /// 어노테이션 도메인 서비스
    annotation_service: A,
    /// 사용자 리포지토리 (사용자 이름 조회용)
    user_repository: U,
    /// 접근 제어 서비스 (권한 확인용)
    access_control_service: AC,
}

impl<A: AnnotationService, U: UserRepository, AC: AccessControlService> AnnotationUseCase<A, U, AC> {
    /// 새로운 어노테이션 Use Case를 생성합니다.
    ///
    /// # 매개변수
    /// - `annotation_service`: 어노테이션 도메인 서비스
    /// - `user_repository`: 사용자 리포지토리
    /// - `access_control_service`: 접근 제어 서비스
    ///
    /// # 반환값
    /// 생성된 `AnnotationUseCase` 인스턴스
    pub fn new(annotation_service: A, user_repository: U, access_control_service: AC) -> Self {
        Self {
            annotation_service,
            user_repository,
            access_control_service,
        }
    }

    /// Annotation 엔티티를 AnnotationResponse DTO로 변환합니다 (단일 조회용)
    ///
    /// # 매개변수
    /// - `annotation`: 변환할 Annotation 엔티티
    ///
    /// # 반환값
    /// - `Ok(AnnotationResponse)`: 변환된 응답 DTO
    /// - `Err(ServiceError)`: 사용자 조회 실패 시
    async fn to_response(&self, annotation: Annotation) -> Result<AnnotationResponse, ServiceError> {
        // 사용자 이름 조회
        let user_name = self
            .user_repository
            .find_by_id(annotation.user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .and_then(|user| user.full_name);

        Ok(AnnotationResponse {
            id: annotation.id,
            user_id: annotation.user_id,
            user_name,
            study_instance_uid: annotation.study_uid,
            series_instance_uid: annotation.series_uid.unwrap_or_default(),
            sop_instance_uid: annotation.instance_uid.unwrap_or_default(),
            annotation_data: annotation.data,
            tool_name: Some(annotation.tool_name),
            tool_version: annotation.tool_version,
            viewer_software: annotation.viewer_software,
            description: annotation.description,
            measurement_values: annotation.measurement_values,
            version: annotation.version,
            created_at: annotation.created_at,
            updated_at: annotation.updated_at,
        })
    }

    /// Annotation 엔티티 목록을 AnnotationResponse DTO 목록으로 변환합니다 (배치 최적화)
    ///
    /// 이 메서드는 N+1 문제를 방지하기 위해 모든 사용자 ID를 한 번에 조회합니다.
    ///
    /// # 매개변수
    /// - `annotations`: 변환할 Annotation 엔티티 목록
    ///
    /// # 반환값
    /// - `Ok(Vec<AnnotationResponse>)`: 변환된 응답 DTO 목록
    /// - `Err(ServiceError)`: 사용자 조회 실패 시
    async fn to_responses(
        &self,
        annotations: Vec<Annotation>,
    ) -> Result<Vec<AnnotationResponse>, ServiceError> {
        if annotations.is_empty() {
            return Ok(Vec::new());
        }

        // 1. 모든 고유한 user_id 수집
        let user_ids: Vec<i32> = annotations
            .iter()
            .map(|a| a.user_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // 2. 사용자 정보 배치 조회 (단 한 번의 쿼리!)
        let users = self
            .user_repository
            .find_by_ids(&user_ids)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 3. user_id -> full_name 매핑 생성
        let user_name_map: HashMap<i32, Option<String>> = users
            .into_iter()
            .map(|user| (user.id, user.full_name))
            .collect();

        // 4. Annotation -> AnnotationResponse 변환
        let responses = annotations
            .into_iter()
            .map(|annotation| {
                let user_name = user_name_map
                    .get(&annotation.user_id)
                    .cloned()
                    .flatten();

                AnnotationResponse {
                    id: annotation.id,
                    user_id: annotation.user_id,
                    user_name,
                    study_instance_uid: annotation.study_uid,
                    series_instance_uid: annotation.series_uid.unwrap_or_default(),
                    sop_instance_uid: annotation.instance_uid.unwrap_or_default(),
                    annotation_data: annotation.data,
                    tool_name: Some(annotation.tool_name),
                    tool_version: annotation.tool_version,
                    viewer_software: annotation.viewer_software,
                    description: annotation.description,
                    measurement_values: annotation.measurement_values,
                    version: annotation.version,
                    created_at: annotation.created_at,
                    updated_at: annotation.updated_at,
                }
            })
            .collect();

        Ok(responses)
    }

    /// 새로운 어노테이션을 생성합니다.
    ///
    /// 이 메서드는 사용자가 요청한 어노테이션을 생성하고, 필요한 검증을 수행합니다.
    /// DICOM UID들의 유효성을 검사하고, 기본값을 설정한 후 도메인 서비스를 호출합니다.
    ///
    /// # 매개변수
    /// - `request`: 어노테이션 생성 요청 데이터
    /// - `user_id`: 어노테이션을 생성하는 사용자의 ID
    /// - `project_id`: 어노테이션이 속할 프로젝트의 ID
    ///
    /// # 반환값
    /// - `Ok(AnnotationResponse)`: 생성된 어노테이션 정보
    /// - `Err(ServiceError)`: 검증 실패 또는 서비스 오류
    ///
    /// # 검증 규칙
    /// - Study Instance UID는 비어있을 수 없음
    /// - Series Instance UID는 비어있을 수 없음
    /// - SOP Instance UID는 비어있을 수 없음
    ///
    /// # 예시
    /// ```ignore
    /// let request = CreateAnnotationRequest {
    ///     study_instance_uid: "1.2.3.4.5.6.7.8.9.10".to_string(),
    ///     series_instance_uid: "1.2.3.4.5.6.7.8.9.11".to_string(),
    ///     sop_instance_uid: "1.2.3.4.5.6.7.8.9.12".to_string(),
    ///     tool_name: Some("manual".to_string()),
    ///     tool_version: Some("1.0.0".to_string()),
    ///     viewer_software: Some("OHIF".to_string()),
    ///     description: Some("간 분할 어노테이션".to_string()),
    ///     annotation_data: serde_json::json!({"type": "polygon", "points": []}),
    /// };
    ///
    /// let result = annotation_use_case.create_annotation(request, 1, 1).await?;
    /// ```
    pub async fn create_annotation(
        &self,
        request: CreateAnnotationRequest,
        user_id: i32,
        project_id: i32,
    ) -> Result<AnnotationResponse, ServiceError> {
        // Validation
        if request.study_instance_uid.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                "Study Instance UID cannot be empty".to_string(),
            ));
        }
        if request.series_instance_uid.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                "Series Instance UID cannot be empty".to_string(),
            ));
        }
        if request.sop_instance_uid.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                "SOP Instance UID cannot be empty".to_string(),
            ));
        }

        let new_annotation = NewAnnotation {
            project_id,
            user_id,
            study_uid: request.study_instance_uid,
            series_uid: Some(request.series_instance_uid),
            instance_uid: Some(request.sop_instance_uid),
            tool_name: request.tool_name.unwrap_or_else(|| "manual".to_string()),
            tool_version: request.tool_version,
            viewer_software: request.viewer_software,
            description: request.description,
            measurement_values: request.measurement_values,
            data: request.annotation_data,
            is_shared: false, // 기본값은 비공유
        };

        let annotation = self
            .annotation_service
            .create_annotation(new_annotation)
            .await?;

        self.to_response(annotation).await
    }

    /// ID로 어노테이션을 조회합니다.
    ///
    /// 이 메서드는 지정된 ID를 가진 어노테이션을 조회합니다.
    /// 어노테이션이 존재하지 않으면 NotFound 에러를 반환합니다.
    ///
    /// # 매개변수
    /// - `annotation_id`: 조회할 어노테이션의 ID
    ///
    /// # 반환값
    /// - `Ok(AnnotationResponse)`: 조회된 어노테이션 정보
    /// - `Err(ServiceError)`: 어노테이션을 찾을 수 없거나 서비스 오류
    ///
    /// # 예시
    /// ```ignore
    /// let annotation = annotation_use_case.get_annotation_by_id(123).await?;
    /// println!("어노테이션 ID: {}", annotation.id);
    /// ```
    pub async fn get_annotation_by_id(
        &self,
        annotation_id: i32,
    ) -> Result<AnnotationResponse, ServiceError> {
        let annotation = self
            .annotation_service
            .get_annotation_by_id(annotation_id)
            .await?;

        self.to_response(annotation).await
    }

    /// 프로젝트의 어노테이션 목록을 조회합니다.
    ///
    /// 이 메서드는 지정된 프로젝트에 속한 모든 어노테이션을 조회합니다.
    /// 결과는 AnnotationListResponse 형태로 반환되며, 총 개수와 어노테이션 목록을 포함합니다.
    ///
    /// # 매개변수
    /// - `project_id`: 어노테이션을 조회할 프로젝트의 ID
    ///
    /// # 반환값
    /// - `Ok(AnnotationListResponse)`: 어노테이션 목록과 총 개수
    /// - `Err(ServiceError)`: 서비스 오류
    ///
    /// # 예시
    /// ```ignore
    /// let response = annotation_use_case.get_annotations_by_project(1).await?;
    /// println!("총 어노테이션 수: {}", response.total);
    /// for annotation in response.annotations {
    ///     println!("어노테이션 ID: {}", annotation.id);
    /// }
    /// ```
    pub async fn get_annotations_by_project(
        &self,
        project_id: i32,
    ) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self
            .annotation_service
            .get_annotations_by_project(project_id)
            .await?;

        let total = annotations.len();
        let annotation_responses = self.to_responses(annotations).await?;

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
            page: 1,
            limit: total as i32,
            total_pages: 1,
            has_next: false,
            list_version: None,
        })
    }

    /// 사용자의 어노테이션 목록을 조회합니다.
    ///
    /// 이 메서드는 지정된 사용자가 생성한 모든 어노테이션을 조회합니다.
    /// 결과는 AnnotationListResponse 형태로 반환되며, 총 개수와 어노테이션 목록을 포함합니다.
    ///
    /// # 매개변수
    /// - `user_id`: 어노테이션을 조회할 사용자의 ID
    ///
    /// # 반환값
    /// - `Ok(AnnotationListResponse)`: 어노테이션 목록과 총 개수
    /// - `Err(ServiceError)`: 서비스 오류
    ///
    /// # 예시
    /// ```ignore
    /// let response = annotation_use_case.get_annotations_by_user(1).await?;
    /// println!("사용자의 어노테이션 수: {}", response.total);
    /// ```
    pub async fn get_annotations_by_user(
        &self,
        user_id: i32,
    ) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self
            .annotation_service
            .get_annotations_by_user(user_id)
            .await?;

        let total = annotations.len();
        let annotation_responses = self.to_responses(annotations).await?;

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
            page: 1,
            limit: total as i32,
            total_pages: 1,
            has_next: false,
            list_version: None,
        })
    }

    /// Study UID로 어노테이션 목록을 조회합니다.
    ///
    /// 이 메서드는 지정된 Study UID와 관련된 모든 어노테이션을 조회합니다.
    /// DICOM Study 단위로 어노테이션을 그룹화하여 조회할 때 사용됩니다.
    ///
    /// # 매개변수
    /// - `study_uid`: 어노테이션을 조회할 Study의 UID
    ///
    /// # 반환값
    /// - `Ok(AnnotationListResponse)`: 어노테이션 목록과 총 개수
    /// - `Err(ServiceError)`: 서비스 오류
    ///
    /// # 예시
    /// ```ignore
    /// let study_uid = "1.2.3.4.5.6.7.8.9.10";
    /// let response = annotation_use_case.get_annotations_by_study(study_uid).await?;
    /// println!("Study의 어노테이션 수: {}", response.total);
    /// ```
    pub async fn get_annotations_by_study(
        &self,
        study_uid: &str,
    ) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self
            .annotation_service
            .get_annotations_by_study(study_uid)
            .await?;

        let total = annotations.len();
        let annotation_responses = self.to_responses(annotations).await?;

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
            page: 1,
            limit: total as i32,
            total_pages: 1,
            has_next: false,
            list_version: None,
        })
    }

    /// Series UID로 어노테이션 목록을 조회합니다.
    ///
    /// 이 메서드는 지정된 Series UID와 관련된 모든 어노테이션을 조회합니다.
    /// DICOM Series 단위로 어노테이션을 그룹화하여 조회할 때 사용됩니다.
    ///
    /// # 매개변수
    /// - `series_uid`: 어노테이션을 조회할 Series의 UID
    ///
    /// # 반환값
    /// - `Ok(AnnotationListResponse)`: 어노테이션 목록과 총 개수
    /// - `Err(ServiceError)`: 서비스 오류
    ///
    /// # 예시
    /// ```ignore
    /// let series_uid = "1.2.3.4.5.6.7.8.9.11";
    /// let response = annotation_use_case.get_annotations_by_series(series_uid).await?;
    /// println!("Series의 어노테이션 수: {}", response.total);
    /// ```
    pub async fn get_annotations_by_series(
        &self,
        series_uid: &str,
    ) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self
            .annotation_service
            .get_annotations_by_series(series_uid)
            .await?;

        let total = annotations.len();
        let annotation_responses = self.to_responses(annotations).await?;

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
            page: 1,
            limit: total as i32,
            total_pages: 1,
            has_next: false,
            list_version: None,
        })
    }

    /// Instance UID로 어노테이션 목록을 조회합니다.
    ///
    /// 이 메서드는 지정된 Instance UID와 관련된 모든 어노테이션을 조회합니다.
    /// DICOM Instance 단위로 어노테이션을 그룹화하여 조회할 때 사용됩니다.
    ///
    /// # 매개변수
    /// - `instance_uid`: 어노테이션을 조회할 Instance의 UID
    ///
    /// # 반환값
    /// - `Ok(AnnotationListResponse)`: 어노테이션 목록과 총 개수
    /// - `Err(ServiceError)`: 서비스 오류
    ///
    /// # 예시
    /// ```ignore
    /// let instance_uid = "1.2.3.4.5.6.7.8.9.12";
    /// let response = annotation_use_case.get_annotations_by_instance(instance_uid).await?;
    /// println!("Instance의 어노테이션 수: {}", response.total);
    /// ```
    pub async fn get_annotations_by_instance(
        &self,
        instance_uid: &str,
    ) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self
            .annotation_service
            .get_annotations_by_instance(instance_uid)
            .await?;

        let total = annotations.len();
        let annotation_responses = self.to_responses(annotations).await?;

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
            page: 1,
            limit: total as i32,
            total_pages: 1,
            has_next: false,
            list_version: None,
        })
    }

    /// 프로젝트와 Study UID로 어노테이션 목록을 조회합니다.
    ///
    /// 이 메서드는 지정된 프로젝트와 Study UID에 해당하는 어노테이션을 조회합니다.
    /// 프로젝트와 Study를 모두 고려하여 더 정확한 필터링을 수행합니다.
    ///
    /// # 매개변수
    /// - `project_id`: 어노테이션을 조회할 프로젝트의 ID
    /// - `study_uid`: 어노테이션을 조회할 Study의 UID
    ///
    /// # 반환값
    /// - `Ok(AnnotationListResponse)`: 어노테이션 목록과 총 개수
    /// - `Err(ServiceError)`: 서비스 오류
    ///
    /// # 예시
    /// ```ignore
    /// let study_uid = "1.2.3.4.5.6.7.8.9.10";
    /// let response = annotation_use_case.get_annotations_by_project_and_study(1, study_uid).await?;
    /// println!("프로젝트 Study의 어노테이션 수: {}", response.total);
    /// ```
    pub async fn get_annotations_by_project_and_study(
        &self,
        project_id: i32,
        study_uid: &str,
    ) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self
            .annotation_service
            .get_annotations_by_project_and_study(project_id, study_uid)
            .await?;

        let total = annotations.len();
        let annotation_responses = self.to_responses(annotations).await?;

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
            page: 1,
            limit: total as i32,
            total_pages: 1,
            has_next: false,
            list_version: None,
        })
    }

    /// 공유 어노테이션 목록을 조회합니다.
    ///
    /// 이 메서드는 지정된 프로젝트에서 공유 설정된 어노테이션들을 조회합니다.
    /// 공유 어노테이션은 프로젝트의 모든 멤버가 볼 수 있습니다.
    ///
    /// # 매개변수
    /// - `project_id`: 공유 어노테이션을 조회할 프로젝트의 ID
    ///
    /// # 반환값
    /// - `Ok(AnnotationListResponse)`: 공유 어노테이션 목록과 총 개수
    /// - `Err(ServiceError)`: 서비스 오류
    ///
    /// # 예시
    /// ```ignore
    /// let response = annotation_use_case.get_shared_annotations(1).await?;
    /// println!("공유 어노테이션 수: {}", response.total);
    /// ```
    pub async fn get_shared_annotations(
        &self,
        project_id: i32,
    ) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self
            .annotation_service
            .get_shared_annotations(project_id)
            .await?;

        let total = annotations.len();
        let annotation_responses = self.to_responses(annotations).await?;

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
            page: 1,
            limit: total as i32,
            total_pages: 1,
            has_next: false,
            list_version: None,
        })
    }

    /// 어노테이션을 업데이트합니다.
    ///
    /// 이 메서드는 기존 어노테이션의 데이터를 업데이트합니다.
    /// 현재는 어노테이션 데이터만 업데이트할 수 있으며, 공유 설정은 변경할 수 없습니다.
    ///
    /// # 매개변수
    /// - `annotation_id`: 업데이트할 어노테이션의 ID
    /// - `request`: 업데이트 요청 데이터
    ///
    /// # 반환값
    /// - `Ok(AnnotationResponse)`: 업데이트된 어노테이션 정보
    /// - `Err(ServiceError)`: 어노테이션을 찾을 수 없거나 서비스 오류
    ///
    /// # 예시
    /// ```ignore
    /// let request = UpdateAnnotationRequest {
    ///     annotation_data: Some(serde_json::json!({"type": "polygon", "points": [[0, 0], [100, 100]]})),
    /// };
    /// let updated = annotation_use_case.update_annotation(123, request).await?;
    /// println!("업데이트된 어노테이션 ID: {}", updated.id);
    /// ```
    pub async fn update_annotation(
        &self,
        annotation_id: i32,
        request: UpdateAnnotationRequest,
    ) -> Result<AnnotationResponse, ServiceError> {
        // 현재 annotation 조회
        let current_annotation = self
            .annotation_service
            .get_annotation_by_id(annotation_id)
            .await?;

        // 버전 검증 (Optimistic Locking)
        // base_version이 제공된 경우, 현재 버전과 일치하는지 확인
        if let Some(base_version) = request.base_version {
            if current_annotation.version != base_version {
                return Err(ServiceError::VersionConflict {
                    current_version: current_annotation.version,
                    client_version: base_version,
                });
            }
        }

        // 업데이트할 데이터 결정
        let new_data = request.annotation_data.unwrap_or(current_annotation.data);
        let is_shared = false; // 현재는 is_shared 업데이트 기능 없음
        let measurement_values = request
            .measurement_values
            .or(current_annotation.measurement_values);

        let updated_annotation = self
            .annotation_service
            .update_annotation_with_measurements(
                annotation_id,
                new_data,
                is_shared,
                measurement_values,
            )
            .await?;

        self.to_response(updated_annotation).await
    }

    /// 어노테이션을 삭제합니다.
    ///
    /// 이 메서드는 지정된 ID를 가진 어노테이션을 삭제합니다.
    /// 어노테이션과 관련된 히스토리도 함께 삭제됩니다.
    ///
    /// # 매개변수
    /// - `annotation_id`: 삭제할 어노테이션의 ID
    ///
    /// # 반환값
    /// - `Ok(())`: 삭제 성공
    /// - `Err(ServiceError)`: 어노테이션을 찾을 수 없거나 서비스 오류
    ///
    /// # 예시
    /// ```ignore
    /// annotation_use_case.delete_annotation(123).await?;
    /// println!("어노테이션이 삭제되었습니다.");
    /// ```
    pub async fn delete_annotation(&self, annotation_id: i32) -> Result<(), ServiceError> {
        self.annotation_service
            .delete_annotation(annotation_id)
            .await
    }

    /// 사용자의 어노테이션 접근 권한을 확인합니다.
    ///
    /// 이 메서드는 지정된 사용자가 특정 어노테이션에 접근할 수 있는지 확인합니다.
    /// 사용자가 어노테이션이 속한 프로젝트의 멤버인 경우 접근이 허용됩니다.
    ///
    /// # 매개변수
    /// - `user_id`: 권한을 확인할 사용자의 ID
    /// - `annotation_id`: 접근하려는 어노테이션의 ID
    ///
    /// # 반환값
    /// - `Ok(true)`: 접근 권한이 있음
    /// - `Ok(false)`: 접근 권한이 없음
    /// - `Err(ServiceError)`: 서비스 오류
    ///
    /// # 예시
    /// ```ignore
    /// let can_access = annotation_use_case.can_access_annotation(1, 123).await?;
    /// if can_access {
    ///     println!("사용자는 이 어노테이션에 접근할 수 있습니다.");
    /// } else {
    ///     println!("접근 권한이 없습니다.");
    /// }
    /// ```
    pub async fn can_access_annotation(
        &self,
        user_id: i32,
        annotation_id: i32,
    ) -> Result<bool, ServiceError> {
        self.annotation_service
            .can_access_annotation(user_id, annotation_id)
            .await
    }

    // viewer_software 필터링 메서드들
    /// 사용자의 어노테이션 목록 조회 (viewer_software 필터링)
    ///
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// * `viewer_software` - 뷰어 소프트웨어 (옵션)
    ///
    /// # Returns
    /// * `Result<AnnotationListResponse, ServiceError>` - 어노테이션 목록 응답
    pub async fn get_annotations_by_user_with_viewer(
        &self,
        user_id: i32,
        viewer_software: Option<&str>,
    ) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self
            .annotation_service
            .get_annotations_by_user_with_viewer(user_id, viewer_software)
            .await?;

        let total = annotations.len();
        let annotation_responses = self.to_responses(annotations).await?;

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
            page: 1,
            limit: total as i32,
            total_pages: 1,
            has_next: false,
            list_version: None,
        })
    }

    /// 프로젝트의 어노테이션 목록 조회 (viewer_software 필터링)
    ///
    /// # Arguments
    /// * `project_id` - 프로젝트 ID
    /// * `viewer_software` - 뷰어 소프트웨어 (옵션)
    ///
    /// # Returns
    /// * `Result<AnnotationListResponse, ServiceError>` - 어노테이션 목록 응답
    pub async fn get_annotations_by_project_with_viewer(
        &self,
        project_id: i32,
        viewer_software: Option<&str>,
    ) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self
            .annotation_service
            .get_annotations_by_project_with_viewer(project_id, viewer_software)
            .await?;

        let total = annotations.len();
        let annotation_responses = self.to_responses(annotations).await?;

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
            page: 1,
            limit: total as i32,
            total_pages: 1,
            has_next: false,
            list_version: None,
        })
    }

    /// Study UID로 어노테이션 목록 조회 (viewer_software 필터링)
    ///
    /// # Arguments
    /// * `study_uid` - Study Instance UID
    /// * `viewer_software` - 뷰어 소프트웨어 (옵션)
    ///
    /// # Returns
    /// * `Result<AnnotationListResponse, ServiceError>` - 어노테이션 목록 응답
    pub async fn get_annotations_by_study_with_viewer(
        &self,
        study_uid: &str,
        viewer_software: Option<&str>,
    ) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self
            .annotation_service
            .get_annotations_by_study_with_viewer(study_uid, viewer_software)
            .await?;

        let total = annotations.len();
        let annotation_responses = self.to_responses(annotations).await?;

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
            page: 1,
            limit: total as i32,
            total_pages: 1,
            has_next: false,
            list_version: None,
        })
    }

    /// 권한 기반 프로젝트 어노테이션 조회
    ///
    /// 사용자의 권한에 따라 다른 어노테이션을 반환합니다:
    /// - `ANNOTATION:READ_ALL` 권한이 있으면: 프로젝트의 모든 어노테이션 반환
    /// - `ANNOTATION:READ_ALL` 권한이 없으면: 본인의 어노테이션만 반환
    ///
    /// # Arguments
    /// * `requesting_user_id` - 요청하는 사용자 ID
    /// * `project_id` - 프로젝트 ID
    /// * `viewer_software` - 뷰어 소프트웨어 (옵션)
    ///
    /// # Returns
    /// * `Result<AnnotationListResponse, ServiceError>` - 어노테이션 목록 응답
    ///
    /// # Errors
    /// * `ServiceError::Unauthorized` - 사용자가 프로젝트 멤버가 아닌 경우
    /// * `ServiceError::NotFound` - 사용자 또는 프로젝트를 찾을 수 없는 경우
    pub async fn get_annotations_by_project_with_permission(
        &self,
        requesting_user_id: i32,
        project_id: i32,
        viewer_software: Option<&str>,
    ) -> Result<AnnotationListResponse, ServiceError> {
        // 1. 사용자가 프로젝트 멤버인지 확인
        let is_member = self
            .access_control_service
            .is_project_member(requesting_user_id, project_id)
            .await?;

        if !is_member {
            return Err(ServiceError::Unauthorized(
                "User is not a member of this project".into(),
            ));
        }

        // 2. READ_ALL 권한 확인
        let has_read_all = self
            .access_control_service
            .check_permission(requesting_user_id, project_id, "ANNOTATION", "READ_ALL")
            .await?;

        // 3. 권한에 따라 다른 쿼리 실행
        if has_read_all {
            // READ_ALL 권한 있음: 프로젝트의 모든 어노테이션
            self.get_annotations_by_project_with_viewer(project_id, viewer_software)
                .await
        } else {
            // READ_ALL 권한 없음: 본인 어노테이션만
            self.get_annotations_by_user_with_viewer(requesting_user_id, viewer_software)
                .await
        }
    }

    /// Series UID와 프로젝트 ID로 어노테이션 목록을 조회합니다 (권한 기반 필터링).
    ///
    /// 이 메서드는 지정된 Series UID와 프로젝트에 해당하는 어노테이션을 조회합니다.
    /// READ_ALL 권한이 있으면 모든 어노테이션을 반환하고, 없으면 사용자 본인의 어노테이션만 반환합니다.
    ///
    /// # 매개변수
    /// - `requesting_user_id`: 요청하는 사용자의 ID
    /// - `series_uid`: 어노테이션을 조회할 Series의 UID
    /// - `project_id`: 어노테이션을 조회할 프로젝트의 ID
    ///
    /// # 반환값
    /// - `Ok(AnnotationListResponse)`: 어노테이션 목록과 총 개수
    /// - `Err(ServiceError)`: 서비스 오류
    ///
    /// # 예시
    /// ```ignore
    /// let series_uid = "1.2.3.4.5.6.7.8.9.11";
    /// let response = annotation_use_case
    ///     .get_annotations_by_series_and_project_with_user(1, series_uid, 2)
    ///     .await?;
    /// println!("Series의 어노테이션 수: {}", response.total);
    /// ```
    pub async fn get_annotations_by_series_and_project_with_user(
        &self,
        requesting_user_id: i32,
        series_uid: &str,
        project_id: i32,
    ) -> Result<AnnotationListResponse, ServiceError> {
        // 1. 사용자가 프로젝트 멤버인지 확인
        let is_member = self
            .access_control_service
            .is_project_member(requesting_user_id, project_id)
            .await?;

        if !is_member {
            return Err(ServiceError::Unauthorized(
                "User is not a member of this project".into(),
            ));
        }

        // 2. READ_ALL 권한 확인
        let has_read_all = self
            .access_control_service
            .check_permission(requesting_user_id, project_id, "ANNOTATION", "READ_ALL")
            .await?;

        // 3. 프로젝트와 Series로 어노테이션 조회
        let annotations = self
            .annotation_service
            .get_annotations_by_project_and_series(project_id, series_uid)
            .await?;

        // 4. 권한에 따라 필터링
        let filtered_annotations = if has_read_all {
            // READ_ALL 권한 있음: 모든 어노테이션 반환
            annotations
        } else {
            // READ_ALL 권한 없음: 본인 어노테이션만 반환
            annotations
                .into_iter()
                .filter(|ann| ann.user_id == requesting_user_id)
                .collect()
        };

        let total = filtered_annotations.len();
        let annotation_responses = self.to_responses(filtered_annotations).await?;

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
            page: 1,
            limit: total as i32,
            total_pages: 1,
            has_next: false,
            list_version: None,
        })
    }

    /// Study UID와 프로젝트 ID로 어노테이션 목록을 조회합니다 (권한 기반 필터링).
    ///
    /// 이 메서드는 지정된 Study UID와 프로젝트에 해당하는 어노테이션을 조회합니다.
    /// READ_ALL 권한이 있으면 모든 어노테이션을 반환하고, 없으면 사용자 본인의 어노테이션만 반환합니다.
    pub async fn get_annotations_by_project_and_study_with_user(
        &self,
        requesting_user_id: i32,
        project_id: i32,
        study_uid: &str,
    ) -> Result<AnnotationListResponse, ServiceError> {
        // 1. 사용자가 프로젝트 멤버인지 확인
        let is_member = self
            .access_control_service
            .is_project_member(requesting_user_id, project_id)
            .await?;

        if !is_member {
            return Err(ServiceError::Unauthorized(
                "User is not a member of this project".into(),
            ));
        }

        // 2. READ_ALL 권한 확인
        let has_read_all = self
            .access_control_service
            .check_permission(requesting_user_id, project_id, "ANNOTATION", "READ_ALL")
            .await?;

        // 3. 프로젝트와 Study로 어노테이션 조회
        let annotations = self
            .annotation_service
            .get_annotations_by_project_and_study(project_id, study_uid)
            .await?;

        // 4. 권한에 따라 필터링
        let filtered_annotations = if has_read_all {
            // READ_ALL 권한 있음: 모든 어노테이션 반환
            annotations
        } else {
            // READ_ALL 권한 없음: 본인 어노테이션만 반환
            annotations
                .into_iter()
                .filter(|ann| ann.user_id == requesting_user_id)
                .collect()
        };

        let total = filtered_annotations.len();
        let annotation_responses = self.to_responses(filtered_annotations).await?;

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
            page: 1,
            limit: total as i32,
            total_pages: 1,
            has_next: false,
            list_version: None,
        })
    }

    /// 프로젝트와 Series UID로 어노테이션 목록을 조회합니다 (권한 체크 없음).
    ///
    /// 이 메서드는 권한 체크 없이 지정된 Series UID와 프로젝트에 해당하는 모든 어노테이션을 반환합니다.
    pub async fn get_annotations_by_project_and_series(
        &self,
        project_id: i32,
        series_uid: &str,
    ) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self
            .annotation_service
            .get_annotations_by_project_and_series(project_id, series_uid)
            .await?;

        let total = annotations.len();
        let annotation_responses = self.to_responses(annotations).await?;

        // 리스트 버전 계산 (최신 수정 시간)
        let list_version = self
            .annotation_service
            .get_max_updated_at_by_project_and_series(project_id, series_uid)
            .await
            .ok()
            .flatten();

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total: total as usize,
            page: 1,
            limit: total as i32,
            total_pages: 1,
            has_next: false,
            list_version,
        })
    }

    /// 페이지네이션을 포함한 어노테이션 조회
    pub async fn get_annotations_by_project_and_series_paginated(
        &self,
        project_id: i32,
        series_uid: &str,
        page: i32,
        limit: i32,
    ) -> Result<AnnotationListResponse, ServiceError> {
        // 전체 개수 조회
        let all_annotations = self
            .annotation_service
            .get_annotations_by_project_and_series(project_id, series_uid)
            .await?;
        let total = all_annotations.len() as i32;

        // 페이지네이션된 데이터 조회
        let annotations = self
            .annotation_service
            .get_annotations_by_project_and_series_paginated(project_id, series_uid, page, limit)
            .await?;

        let annotation_responses = self.to_responses(annotations).await?;

        // 전체 페이지 수 계산
        let total_pages = (total + limit - 1) / limit;
        let has_next = page < total_pages;

        // 리스트 버전 계산 (최신 수정 시간)
        let list_version = self
            .annotation_service
            .get_max_updated_at_by_project_and_series(project_id, series_uid)
            .await
            .ok()
            .flatten();

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total: total as usize,
            page,
            limit,
            total_pages,
            has_next,
            list_version,
        })
    }
}
