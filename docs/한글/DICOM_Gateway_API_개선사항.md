# DICOM Gateway API 개선사항

## 📋 개요

DICOM Gateway API의 기본 뷰(`view=default`)에 TimePoint 정보를 포함하도록 개선했습니다.

## 🎯 목적

프론트엔드에서 Study 목록을 조회할 때, 각 Study가 어떤 TimePoint에 할당되어 있는지 바로 확인할 수 있도록 하기 위함입니다.

## 🔧 변경 사항

### 1. Controller 수정

**파일**: `pacs-server/src/presentation/controllers/dicom_gateway_controller.rs`

기본 뷰에서 TimePoint 정보를 포함하도록 수정:

```rust
// view 파라미터 처리
let view = query_params
    .get("view")
    .and_then(|v| v.first())
    .map(|s| s.as_str())
    .unwrap_or("default"); // 기본값: "default"

let include_timepoint = match view {
    "minimal" => false,
    "default" => true,  // ✅ 기본 뷰에서 timepoint 포함
    "full" => true,
    _ => false,
};
```

### 2. TimePoint 정보 조회 로직

각 Study에 대해 TimePoint 정보를 조회하여 `_ext` 필드에 추가:

```rust
if include_timepoint {
    // Study가 할당된 TimePoint 조회
    if let Ok(timepoint_studies) = timepoint_study_repo
        .find_by_study_id(study.id)
        .await
    {
        if let Some(ts) = timepoint_studies.first() {
            if let Ok(Some(timepoint)) = timepoint_repo
                .find_by_id(ts.timepoint_id)
                .await
            {
                ext_obj.insert(
                    "timepoint".to_string(),
                    serde_json::to_value(&timepoint).unwrap_or(Value::Null),
                );
            }
        }
    }
}
```

## 📝 사용 방법

### 기본 뷰 (TimePoint 포함)

```http
GET /api/me/dicom/studies?view=default&project_id=2&patient_id=Lung_Dx-A0011&page=1&page_size=20
Authorization: Bearer {token}
```

**응답 예시**:

```json
[
  {
    "0020000D": {
      "vr": "UI",
      "Value": ["1.3.6.1.4.1.14519.5.2.1.6655.2359.305690637242184753624524107418"]
    },
    "00100020": {
      "vr": "LO",
      "Value": ["Lung_Dx-A0011"]
    },
    "_ext": {
      "project": {
        "id": 2,
        "name": "LIDC-IDRI"
      },
      "report_status": null,
      "review": null,
      "subject": {
        "id": 140,
        "patient_id": "Lung_Dx-A0011"
      },
      "timepoint": {
        "id": 121,
        "name": "Baseline",
        "visit_type": "Baseline",
        "visit_no": 1,
        "order_index": 1
      }
    }
  }
]
```

### Minimal 뷰 (TimePoint 제외)

```http
GET /api/me/dicom/studies?view=minimal&project_id=2&patient_id=Lung_Dx-A0011
```

**응답**: TimePoint 정보 없이 기본 DICOM 태그만 반환

### Full 뷰 (모든 정보 포함)

```http
GET /api/me/dicom/studies?view=full&project_id=2&patient_id=Lung_Dx-A0011
```

**응답**: TimePoint 포함 모든 확장 정보 반환

## 🔍 View 옵션 정리

| View | TimePoint 포함 | 설명 |
|------|---------------|------|
| `minimal` | ❌ | 기본 DICOM 태그만 |
| `default` | ✅ | 기본 정보 + TimePoint |
| `full` | ✅ | 모든 확장 정보 |
| 미지정 | ✅ | `default`와 동일 |

## ✅ 테스트

**파일**: `tests/e2e/test_05_subject_timepoint.py`

```python
def test_01_5_gateway_default_view_includes_timepoint(self):
    """기본 뷰에서 timepoint 정보가 포함되는지 확인"""
    response = self.client.get(
        f"/api/me/dicom/studies",
        params={
            "view": "default",
            "project_id": self.project_id,
            "patient_id": self.patient_id,
            "page": 1,
            "page_size": 20,
        },
    )
    
    assert response.status_code == 200
    studies = response.json()
    assert len(studies) > 0
    
    # _ext.timepoint 필드 확인
    assert "_ext" in studies[0]
    assert "timepoint" in studies[0]["_ext"]
    
    timepoint = studies[0]["_ext"]["timepoint"]
    assert timepoint is not None
    assert "id" in timepoint
    assert "name" in timepoint
```

## 📊 영향 범위

- ✅ DICOM Gateway API `/api/me/dicom/studies`
- ✅ 기본 뷰 동작 변경
- ✅ 기존 minimal/full 뷰 호환성 유지
- ✅ 성능 영향 최소화 (필요한 경우에만 조회)

## 🚀 프론트엔드 활용

프론트엔드에서 Study 목록을 표시할 때 각 Study의 TimePoint 정보를 바로 사용할 수 있습니다:

```javascript
// Study 목록 조회
const studies = await api.get('/api/me/dicom/studies', {
  params: {
    view: 'default',
    project_id: 2,
    patient_id: 'Lung_Dx-A0011'
  }
});

// TimePoint 정보 표시
studies.forEach(study => {
  const timepoint = study._ext?.timepoint;
  if (timepoint) {
    console.log(`Study는 ${timepoint.name} (Visit ${timepoint.visit_no})에 할당됨`);
  } else {
    console.log('Study는 아직 TimePoint에 할당되지 않음');
  }
});
```

