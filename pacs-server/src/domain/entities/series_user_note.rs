//! # Series User Note 엔티티 모듈
//!
//! 이 모듈은 사용자별 DICOM Series 메모를 나타내는 엔티티들을 정의합니다.
//! Series note는 프로젝트 종속 또는 전역으로 저장될 수 있습니다.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 사용자별 Series 메모를 나타내는 엔티티
///
/// 이 구조체는 데이터베이스의 `series_user_note` 테이블과 매핑되며,
/// 사용자가 특정 Series에 대해 작성한 텍스트 메모를 저장합니다.
///
/// # 필드
/// - `id`: 데이터베이스에서 자동 생성되는 고유 식별자
/// - `series_id`: 메모가 속한 Series의 ID
/// - `user_id`: 메모를 작성한 사용자의 ID
/// - `project_id`: 프로젝트 ID (NULL이면 전역 note, 값이 있으면 프로젝트별 note)
/// - `note`: 사용자가 작성한 텍스트 메모
/// - `created_at`: 메모가 생성된 시각
/// - `updated_at`: 메모가 마지막으로 수정된 시각
///
/// # 예시
/// ```ignore
/// let note = SeriesUserNote {
///     id: 1,
///     series_id: 123,
///     user_id: 456,
///     project_id: Some(1),
///     note: "이 시리즈는 프로젝트 A에서 분석 중입니다".to_string(),
///     created_at: DateTime::from_timestamp(1640995200, 0).unwrap(),
///     updated_at: DateTime::from_timestamp(1640995200, 0).unwrap(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SeriesUserNote {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// 메모가 속한 Series의 ID
    pub series_id: i32,
    /// 메모를 작성한 사용자의 ID
    pub user_id: i32,
    /// 프로젝트 ID (NULL이면 전역 note, 값이 있으면 프로젝트별 note)
    pub project_id: Option<i32>,
    /// 사용자가 작성한 텍스트 메모
    pub note: String,
    /// 메모가 생성된 시각
    pub created_at: DateTime<Utc>,
    /// 메모가 마지막으로 수정된 시각
    pub updated_at: DateTime<Utc>,
}

/// 새로운 Series User Note 생성용 구조체
///
/// 이 구조체는 Series note를 생성할 때 사용되며,
/// 빌더 패턴을 통해 필드를 설정할 수 있습니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSeriesUserNote {
    /// 메모가 속한 Series의 ID
    pub series_id: i32,
    /// 메모를 작성한 사용자의 ID
    pub user_id: i32,
    /// 프로젝트 ID (None이면 전역 note, Some(id)이면 프로젝트별 note)
    pub project_id: Option<i32>,
    /// 사용자가 작성한 텍스트 메모
    pub note: String,
}

impl NewSeriesUserNote {
    /// 새로운 NewSeriesUserNote 인스턴스를 생성합니다.
    ///
    /// # 매개변수
    /// - `series_id`: Series ID
    /// - `user_id`: 사용자 ID
    /// - `note`: 메모 텍스트
    ///
    /// # 반환값
    /// 생성된 `NewSeriesUserNote` 인스턴스 (project_id는 None)
    pub fn new(series_id: i32, user_id: i32, note: String) -> Self {
        Self {
            series_id,
            user_id,
            project_id: None,
            note,
        }
    }

    /// 프로젝트 ID를 설정합니다.
    ///
    /// # 매개변수
    /// - `project_id`: 프로젝트 ID
    ///
    /// # 반환값
    /// 프로젝트 ID가 설정된 `NewSeriesUserNote` 인스턴스
    pub fn with_project_id(mut self, project_id: i32) -> Self {
        self.project_id = Some(project_id);
        self
    }
}

/// Series User Note 업데이트용 구조체
///
/// 이 구조체는 Series note를 업데이트할 때 사용되며,
/// 변경할 필드만 포함합니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSeriesUserNote {
    /// 업데이트할 메모 텍스트
    pub note: String,
}

impl UpdateSeriesUserNote {
    /// 새로운 UpdateSeriesUserNote 인스턴스를 생성합니다.
    ///
    /// # 매개변수
    /// - `note`: 업데이트할 메모 텍스트
    ///
    /// # 반환값
    /// 생성된 `UpdateSeriesUserNote` 인스턴스
    pub fn new(note: String) -> Self {
        Self { note }
    }
}

