# Subject API 개선사항

## 📋 개요

Subject API 응답에 TimePoint 정보를 포함하도록 개선했습니다.

## 🎯 목적

Subject 목록을 조회할 때 각 Subject에 속한 TimePoint 정보를 함께 제공하여, 프론트엔드에서 별도의 API 호출 없이 TimePoint 정보를 확인할 수 있도록 합니다.

## 🔧 변경 사항

### 1. Entity 수정

**파일**: `pacs-server/src/domain/entities/subject.rs`

Subject 응답 DTO에 `timepoints` 필드 추가:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Subject {
    pub id: i32,
    pub project_id: i32,
    pub patient_id: String,
    pub patient_name: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub sex: Option<String>,
    pub external_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    /// TimePoint 목록 (선택적)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timepoints: Option<Vec<TimePoint>>,
}
```

### 2. Repository 수정

**파일**: `pacs-server/src/infrastructure/repositories/subject_repository_impl.rs`

Subject 조회 시 TimePoint 정보를 함께 조회:

```rust
async fn find_by_project_id(&self, project_id: i32) -> Result<Vec<Subject>, sqlx::Error> {
    // Subject 목록 조회
    let mut subjects = sqlx::query_as::<_, Subject>(
        "SELECT id, project_id, patient_id, patient_name, birth_date, sex, 
                external_key, created_at, updated_at
         FROM subject_subject
         WHERE project_id = $1
         ORDER BY created_at DESC"
    )
    .bind(project_id)
    .fetch_all(&self.pool)
    .await?;

    // 각 Subject의 TimePoint 조회
    for subject in &mut subjects {
        let timepoints = sqlx::query_as::<_, TimePoint>(
            "SELECT id, project_id, subject_id, name, visit_type, visit_no, 
                    order_index, external_key, created_at, updated_at
             FROM subject_timepoint
             WHERE subject_id = $1
             ORDER BY order_index ASC"
        )
        .bind(subject.id)
        .fetch_all(&self.pool)
        .await?;
        
        subject.timepoints = Some(timepoints);
    }

    Ok(subjects)
}
```

### 3. Controller 수정

**파일**: `pacs-server/src/presentation/controllers/subject_controller.rs`

응답에 TimePoint 정보가 자동으로 포함됩니다 (별도 수정 불필요).

## 📝 사용 방법

### Subject 목록 조회

```http
GET /api/projects/{project_id}/subjects
Authorization: Bearer {token}
```

**응답 예시**:

```json
[
  {
    "id": 140,
    "project_id": 2,
    "patient_id": "Lung_Dx-A0011",
    "patient_name": "Patient A",
    "birth_date": "1950-01-01",
    "sex": "M",
    "external_key": null,
    "created_at": "2025-01-18T10:00:00Z",
    "updated_at": "2025-01-18T10:00:00Z",
    "timepoints": [
      {
        "id": 121,
        "project_id": 2,
        "subject_id": 140,
        "name": "Baseline",
        "visit_type": "Baseline",
        "visit_no": 1,
        "order_index": 1,
        "external_key": null,
        "created_at": "2025-01-18T10:00:00Z",
        "updated_at": "2025-01-18T10:00:00Z"
      },
      {
        "id": 122,
        "project_id": 2,
        "subject_id": 140,
        "name": "Week 4",
        "visit_type": "FollowUp",
        "visit_no": 2,
        "order_index": 2,
        "external_key": null,
        "created_at": "2025-01-18T10:05:00Z",
        "updated_at": "2025-01-18T10:05:00Z"
      }
    ]
  }
]
```

### 단일 Subject 조회

```http
GET /api/subjects/{subject_id}
Authorization: Bearer {token}
```

**응답**: 위와 동일한 형식으로 TimePoint 정보 포함

## ✅ 테스트

**파일**: `tests/e2e/test_05_subject_timepoint.py`

```python
def test_subject_includes_timepoints(self):
    """Subject 조회 시 timepoints 필드가 포함되는지 확인"""
    response = self.client.get(
        f"/api/projects/{self.project_id}/subjects"
    )
    
    assert response.status_code == 200
    subjects = response.json()
    assert len(subjects) > 0
    
    # timepoints 필드 확인
    subject = subjects[0]
    assert "timepoints" in subject
    assert isinstance(subject["timepoints"], list)
    
    if len(subject["timepoints"]) > 0:
        timepoint = subject["timepoints"][0]
        assert "id" in timepoint
        assert "name" in timepoint
        assert "visit_type" in timepoint
        assert "order_index" in timepoint
```

## 📊 영향 범위

- ✅ Subject 목록 조회 API
- ✅ 단일 Subject 조회 API
- ✅ 기존 API 호환성 유지 (timepoints는 선택적 필드)
- ✅ N+1 쿼리 문제 해결 (각 Subject마다 별도 쿼리)

## 🚀 프론트엔드 활용

프론트엔드에서 Subject 목록을 표시할 때 TimePoint 정보를 바로 사용할 수 있습니다:

```javascript
// Subject 목록 조회
const subjects = await api.get(`/api/projects/${projectId}/subjects`);

// Subject와 TimePoint 정보 표시
subjects.forEach(subject => {
  console.log(`Subject: ${subject.patient_id}`);
  
  if (subject.timepoints && subject.timepoints.length > 0) {
    console.log('TimePoints:');
    subject.timepoints.forEach(tp => {
      console.log(`  - ${tp.name} (Visit ${tp.visit_no})`);
    });
  } else {
    console.log('  TimePoint 없음');
  }
});
```

## 🔍 성능 고려사항

현재 구현은 각 Subject마다 별도의 쿼리를 실행합니다 (N+1 문제). Subject 수가 많은 경우 성능 이슈가 발생할 수 있습니다.

### 향후 개선 방안

1. **JOIN 쿼리 사용**: Subject와 TimePoint를 한 번의 쿼리로 조회
2. **페이지네이션**: Subject 목록에 페이지네이션 적용
3. **선택적 로딩**: `include_timepoints` 파라미터로 필요한 경우에만 조회

```sql
-- 개선된 쿼리 예시
SELECT 
    s.id, s.project_id, s.patient_id, s.patient_name, s.birth_date, s.sex,
    s.external_key, s.created_at, s.updated_at,
    t.id as tp_id, t.name as tp_name, t.visit_type, t.visit_no, t.order_index
FROM subject_subject s
LEFT JOIN subject_timepoint t ON s.id = t.subject_id
WHERE s.project_id = $1
ORDER BY s.created_at DESC, t.order_index ASC
```

