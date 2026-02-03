//! # Series User Report 엔티티 모듈
//!
//! 이 모듈은 사용자별 DICOM Series 리포트를 나타내는 엔티티들을 정의합니다.
//! Report는 프로젝트 종속 또는 전역으로 저장될 수 있으며, status 관리, 오디오 파일, 가이드 템플릿을 지원합니다.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 리포트 상태 열거형
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum ReportStatus {
    Unread,
    Approval,
    Unapproval,
}

impl ReportStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ReportStatus::Unread => "unread",
            ReportStatus::Approval => "approval",
            ReportStatus::Unapproval => "unapproval",
        }
    }
}

/// 사용자별 Series 리포트를 나타내는 엔티티
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SeriesUserReport {
    pub id: i32,
    pub series_id: i32,
    pub user_id: i32,
    pub project_id: Option<i32>,
    pub status: String, // 'unread', 'approval', 'unapproval'
    /// 적용된 원본 템플릿 ID (출처용, nullable)
    pub template_id: Option<i32>,
    /// 적용된 커스텀 템플릿 ID (출처용, nullable)
    pub custom_template_id: Option<i32>,
    pub dictate_file_path: Option<String>,
    pub dictate_file_size: Option<i64>,
    pub dictate_mime_type: Option<String>,
    pub description: String,
    pub conclusion: String,
    pub bodypart: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 새로운 Series User Report 생성용 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSeriesUserReport {
    pub series_id: i32,
    pub user_id: i32,
    pub project_id: Option<i32>,
    pub status: String,
    pub dictate_file_path: Option<String>,
    pub dictate_file_size: Option<i64>,
    pub dictate_mime_type: Option<String>,
    pub description: String,
    pub conclusion: String,
    pub bodypart: Option<String>,
}

impl NewSeriesUserReport {
    pub fn new(series_id: i32, user_id: i32, description: String, conclusion: String) -> Self {
        Self {
            series_id,
            user_id,
            project_id: None,
            status: "unread".to_string(),
            dictate_file_path: None,
            dictate_file_size: None,
            dictate_mime_type: None,
            description,
            conclusion,
            bodypart: None,
        }
    }

    pub fn with_project_id(mut self, project_id: i32) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub fn with_bodypart(mut self, bodypart: String) -> Self {
        self.bodypart = Some(bodypart);
        self
    }
}

/// Series User Report 업데이트용 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSeriesUserReport {
    pub status: Option<String>,
    pub template_id: Option<i32>,
    pub custom_template_id: Option<i32>,
    pub dictate_file_path: Option<String>,
    pub dictate_file_size: Option<i64>,
    pub dictate_mime_type: Option<String>,
    pub description: Option<String>,
    pub conclusion: Option<String>,
    pub bodypart: Option<String>,
}

impl UpdateSeriesUserReport {
    pub fn new() -> Self {
        Self {
            status: None,
            template_id: None,
            custom_template_id: None,
            dictate_file_path: None,
            dictate_file_size: None,
            dictate_mime_type: None,
            description: None,
            conclusion: None,
            bodypart: None,
        }
    }

    pub fn with_status(mut self, status: String) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_conclusion(mut self, conclusion: String) -> Self {
        self.conclusion = Some(conclusion);
        self
    }
}

impl Default for UpdateSeriesUserReport {
    fn default() -> Self {
        Self::new()
    }
}

