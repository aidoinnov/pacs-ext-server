# TimePoint API 요청사항 객관적 검토

## 검토 기준

1. **API 설계 일관성** - 기존 API 패턴과의 일치 여부
2. **데이터 구조 적절성** - 필요한 정보의 충분성과 불필요한 데이터 최소화
3. **성능** - 쿼리/조인 비용, 확장성
4. **호환성** - 기존 코드와의 호환성
5. **구현 난이도** - 서버/클라이언트 양측 구현 비용
6. **대안 비교** - 다른 방법들과의 장단점 비교

---

## 제안 방법 요약

### 현재 제안
- **API**: 기존 `/api/me/dicom/studies` 활용
- **파라미터**: `include_timepoint` (boolean, optional, 기본값: `false`)
- **응답**: `_ext.timepoint` 객체 (id, name, visit_type, visit_no) 또는 `null`

---

## 1. API 설계 일관성 검토

### ✅ 긍정적 측면

1. **기존 패턴 일치**
   - `_ext.project`: 객체로 포함 ✅
   - `_ext.subject`: 객체로 포함 ✅
   - `_ext.timepoint`: 객체로 포함 (일관성 유지) ✅

2. **옵션 파라미터 패턴**
   - `report_status` 필터처럼 선택적 파라미터 사용 ✅
   - 기존 API 구조 변경 없음 ✅

### ✅ 확인 완료

1. **응답 구조**
   - 서버는 `_ext` 객체 사용 중 ✅
   - `_ext.project`, `_ext.subject`, `_ext.review` 등 이미 구현됨
   - `_ext.timepoint` 추가는 기존 패턴과 완벽히 일치 ✅

2. **레거시 호환성**
   - `_ext` 형식으로 통일됨 ✅
   - `mapper.ts`에서 `_ext` 우선 처리하므로 문제 없음 ✅

---

## 2. 데이터 구조 적절성 검토

### ✅ 충분한 정보

제안된 필드:
- `id`: TimePoint 고유 ID ✅
- `name`: 표시 레이블 ✅
- `visit_type`: Visit Type ✅
- `visit_no`: Visit Number ✅

### ❓ 검토 필요 사항

1. **추가 필드 필요 여부**
   - `order_index`: TimePoint 정렬에 사용 (UI에서 사용 가능)
   - **제안**: `order_index`는 UI에서 필요하지 않을 수 있음 (TimePoint 목록 조회로 충분)
   - **결론**: 현재 제안된 4개 필드로 충분 ✅

2. **null 처리**
   - 할당되지 않은 Study: `timepoint: null` ✅
   - 명확하고 일관적 ✅

---

## 3. 성능 검토

### ✅ 긍정적 측면

1. **선택적 조인**
   - `include_timepoint=true`일 때만 조인 쿼리 수행 ✅
   - 기본 사용 케이스에는 오버헤드 없음 ✅

2. **배치 쿼리 가능**
   - 여러 Study의 TimePoint를 한 번에 조회 가능 ✅
   - `IN` 쿼리 또는 LEFT JOIN으로 효율적 조회 ✅

3. **단일 API 호출**
   - 별도 TimePoint 조회 API 호출 불필요 ✅
   - 네트워크 요청 최소화 ✅

### ⚠️ 주의할 점

1. **조인 비용**
   - Study 수가 많을 때 (`page_size=1000`) 조인 비용 증가 가능
   - **완화**: `include_timepoint=true`일 때만 발생
   - **권장**: 인덱스 최적화 (study.timepoint_id에 인덱스)

2. **중복 데이터**
   - 같은 TimePoint 정보가 여러 Study에 중복 포함
   - **예**: 100개 Study가 같은 TimePoint에 할당 → TimePoint 객체 100번 반복
   - **영향**: JSON 응답 크기 증가 (보통 수백 KB 수준, 큰 문제 아님)
   - **대안**: TimePoint 정보를 별도 배열로 분리 (복잡도 증가, 권장하지 않음)

---

## 4. 호환성 검토

### ✅ 완벽한 호환성

1. **기본값 처리**
   - `include_timepoint` 기본값 `false` → 기존 동작 유지 ✅
   - 기존 코드 영향 없음 ✅

2. **타입 정의**
   - `QidoEntity._ext`에 `timepoint?` 필드 추가
   - TypeScript 타입 안정성 유지 ✅

3. **Mapper 호환성**
   - `mapper.ts`에서 `_ext` 보존 중 ✅
   - 추가 매핑 로직 불필요 (필요 시 추가 가능) ✅

---

## 5. 구현 난이도 검토

### 서버 측

**난이도: 낮음 ~ 중간**

1. **Query Parameter 추가** (쉬움)
   - `include_timepoint` 파라미터 파싱
   - 기본값 `false` 처리

2. **조인 쿼리 추가** (중간)
   - `include_timepoint=true`일 때 LEFT JOIN
   - TimePoint 테이블 조인
   - NULL 처리

3. **응답 구성** (쉬움)
   - `_ext.timepoint` 객체 생성
   - 필요한 필드만 선택

**예상 작업 시간**: 2-4시간

### 클라이언트 측

**난이도: 낮음**

1. **API 파라미터 추가**
   ```typescript
   include_timepoint?: boolean;  // MyStudiesParams에 추가
   ```

2. **타입 정의 업데이트**
   ```typescript
   _ext?: {
     timepoint?: {
       id: number;
       name: string;
       visit_type: string;
       visit_no: number | null;
     };
   };
   ```

3. **Mapper 업데이트** (선택적)
   - `timepoint_id` 추출 로직 추가
   - 또는 `_ext.timepoint` 직접 사용

**예상 작업 시간**: 1-2시간

---

## 6. 대안 방법 비교

### 대안 1: 별도 API 엔드포인트

**예시**: `/api/subjects/{subject_id}/studies`

**장점**:
- 명확한 의도 (Subject의 Study 조회)
- RESTful 설계 원칙에 부합
- TimePoint 정보 항상 포함 가능

**단점**:
- ❌ 새 API 개발 필요 (작업량 증가)
- ❌ 기존 API와 중복 로직
- ❌ 두 가지 API를 선택해야 함 (복잡도 증가)
- ❌ `patient_id` 필터와의 차이 불명확

**결론**: 현재 제안이 더 효율적 ✅

---

### 대안 2: 항상 포함 (옵션 없음)

**예시**: `_ext.timepoint`를 항상 응답에 포함

**장점**:
- 파라미터 불필요
- 클라이언트 코드 단순화

**단점**:
- ❌ 항상 조인 쿼리 수행 (성능 오버헤드)
- ❌ TimePoint 정보가 불필요한 경우도 조회
- ❌ 대부분의 Study 조회에서 불필요한 데이터 포함

**결론**: 옵션 파라미터가 더 효율적 ✅

---

### 대안 3: 별도 조회 API

**예시**: `/api/me/dicom/studies` + `/api/timepoints/studies?study_uids=...`

**장점**:
- 각 API의 책임 분리
- 필요한 경우에만 TimePoint 조회

**단점**:
- ❌ 두 번의 API 호출 (네트워크 오버헤드)
- ❌ 클라이언트 코드 복잡도 증가
- ❌ Race condition 가능성 (첫 호출 후 두 번째 호출 사이 데이터 변경)

**결론**: 단일 API 호출이 더 효율적 ✅

---

## 7. 잠재적 문제점 및 개선 사항

### ✅ 문제점 1: 필드명 일관성 (확인 완료)

**현재 제안**:
- `visit_type` (snake_case)
- `visit_no` (snake_case)

**기존 패턴 확인 완료**:
- `ProjectInfo`: snake_case 사용 (`role_name`)
- `SubjectInfo`: `#[serde(rename_all = "camelCase")]`로 camelCase 변환
- `ReviewInfo`: `#[serde(rename_all = "camelCase")]`로 camelCase 변환

**권장**:
- `TimePointInfo`에 `#[serde(rename_all = "camelCase")]` 적용
- 응답: `visitType`, `visitNo` (camelCase)
- 또는 `ProjectInfo` 패턴 따라 snake_case 유지

---

### ⚠️ 문제점 2: `patient_id` vs `subject_id` 필터

**현재 제안**: `patient_id`로 필터링

**실제 API 구조**: 
- `_ext.subject.id` (Subject ID)
- `00100020` (PatientID - DICOM)

**검토 필요**:
- `patient_id` 파라미터가 DICOM PatientID인지, Subject ID인지 확인
- Subject 기반 필터링이 필요한 경우 `subject_id` 파라미터 추가 필요할 수 있음

**권장**: 
- 기존 `patient_id` 파라미터 확인 (문서상 DICOM PatientID로 명시됨)
- Subject ID로 필터링이 필요하면 별도 파라미터 추가 검토

---

### ⚠️ 문제점 3: TimePoint 정보 업데이트 동기화

**시나리오**:
1. TimePoint 설정 화면에서 `include_timepoint=true`로 조회
2. TimePoint 이름 수정
3. 기존 조회 결과의 `_ext.timepoint.name`은 이전 값

**영향**:
- 캐시된 데이터와 실제 데이터 불일치
- TimePoint 목록은 최신 데이터, Study 응답은 이전 데이터

**완화책**:
- React Query 캐시 무효화로 해결 가능 ✅
- TimePoint 수정 후 Study 조회 쿼리 무효화

**결론**: 큰 문제 아님, 기존 패턴과 동일 ✅

---

## 8. 최근 구현 사례

### ✅ StudyDescription 추가 (2026-01-18)

**구현 내용**:
- DICOM 태그 `00081030` (StudyDescription)을 응답에 추가
- `project_data_study` 테이블에서 배치 조회
- 성능 최적화: 한 번의 쿼리로 여러 스터디의 description 조회

**구현 패턴**:
```rust
// 1. 배치 조회 메서드 추가
pub async fn fetch_study_descriptions(&self, study_uids: &[String]) -> HashMap<String, String> {
    let results = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT study_uid, study_description FROM project_data_study WHERE study_uid = ANY($1)"
    )
    .bind(study_uids)
    .fetch_all(self.pool)
    .await;

    // HashMap으로 변환
}

// 2. 핸들러에서 사용
let study_desc_cache = ext_builder.fetch_study_descriptions(&all_study_uids).await;

// 3. 각 스터디에 추가
if let Some(study_desc) = study_desc_cache.get(&study_uid) {
    obj.insert("00081030".to_string(), serde_json::json!({
        "vr": "LO",
        "Value": [study_desc]
    }));
}
```

**결론**: TimePoint도 동일한 패턴으로 구현 가능 ✅

---

## 9. 구현 완료

### ✅ 구현 내용

1. **TimePointInfo 구조체 추가**
   ```rust
   #[derive(Debug, Clone, serde::Serialize)]
   #[serde(rename_all = "camelCase")]
   pub struct TimePointInfo {
       pub id: i32,
       pub name: String,
       pub visit_type: String,
       pub visit_no: Option<i32>,
   }
   ```

2. **fetch_timepoints 메서드 추가**
   - `project_data_study` 테이블과 `project_timepoint` 테이블 JOIN
   - study_uid 기반 배치 조회
   - HashMap<study_uid, TimePointInfo> 반환

3. **GatewayQuery에 include_timepoint 파라미터 추가**
   ```rust
   pub struct GatewayQuery {
       // ...
       #[serde(default)]
       pub include_timepoint: Option<bool>, // 기본값: false
       // ...
   }
   ```

4. **get_all_user_studies 핸들러 수정**
   - `include_timepoint=true`일 때만 TimePoint 조회
   - `_ext.timepoint` 필드에 추가
   - TimePoint가 없는 경우 `null` 반환

### 📝 테스트 방법

```bash
# TimePoint 정보 포함 요청
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/dicom/gateway/me/studies?include_timepoint=true&page_size=10"

# TimePoint 정보 제외 요청 (기본값)
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/dicom/gateway/me/studies?page_size=10"
```

### 예상 응답

```json
[
  {
    "0020000D": {
      "vr": "UI",
      "Value": ["1.2.3.4.5"]
    },
    "_ext": {
      "project": {
        "id": 1,
        "name": "Project A",
        "role_name": "Investigator"
      },
      "subject": {
        "id": 10,
        "subjectCode": "SUB001",
        "patientId": "P001"
      },
      "timepoint": {
        "id": 5,
        "name": "Baseline",
        "visitType": "screening",
        "visitNo": 1
      },
      "report_status": "unread",
      "review": {
        "reviewStage": "pending",
        "availableStages": ["pending", "approved"],
        "annotationSummary": {}
      }
    }
  }
]
```

---

## 10. 최종 평가

### ✅ 장점

1. **기존 API 활용**: 새 API 개발 불필요
2. **선택적 조인**: 성능 오버헤드 최소화
3. **단일 호출**: 네트워크 요청 최소화
4. **호환성**: 기존 코드 영향 없음
5. **일관성**: 기존 `_ext` 패턴과 일치
6. **구현 용이**: 서버/클라이언트 양측 모두 낮은 난이도

### ⚠️ 주의사항

1. **성능**: Study 수가 많을 때 조인 비용 (완화 가능)
2. **중복 데이터**: 같은 TimePoint 정보 반복 (허용 가능 범위)
3. **네이밍**: 필드명 일관성 확인 필요

### 🎯 결론

**제안 방법이 적절합니다.**

- ✅ 기존 API 패턴과 일치
- ✅ 필요한 정보 모두 포함
- ✅ 성능 오버헤드 최소화
- ✅ 구현 난이도 낮음
- ✅ 대안 대비 우수

**권장 사항**:
1. 서버 구현 시 인덱스 최적화 (`study.timepoint_id`)
2. 필드명은 서버 응답 그대로 사용 (클라이언트에서 변환)
3. `patient_id` 파라미터 확인 (DICOM PatientID로 필터링 가능한지)

---

## 수정 제안

### 필드명 일관성 확인 필요

문서에서 "label, type, visit no"라고 표현했지만, 실제 API 필드는:
- `name` (label 대신)
- `visit_type` (type 대신)
- `visit_no` ✅

**현재 제안이 올바릅니다.** (`name`이 label 역할)

### 추가 검토 사항

1. **`order_index` 포함 여부**
   - 현재 제안: 포함 안 함
   - **권장**: 포함 안 함 (UI에서 TimePoint 목록 조회로 충분)

2. **필드 타입 확인**
   - `visit_type`: string ✅
   - `visit_no`: number | null ✅
   - `id`: number ✅ (문서에 명시)

---

## 최종 요청사항 (서버)

### 1. Query Parameter 추가
```
include_timepoint (boolean, optional, 기본값: false)
```

### 2. Response 필드 추가 (`include_timepoint=true`일 때)
```json
{
  "_ext": {
    "timepoint": {
      "id": number,
      "name": string,
      "visit_type": string,
      "visit_no": number | null
    } | null
  }
}
```

### 3. 필터링
- `project_id`: 프로젝트 필터 (기존)
- `patient_id`: DICOM PatientID 필터 (기존)

### 4. 성능 최적화 권장
- `study.timepoint_id` 인덱스 확인
- LEFT JOIN 최적화
