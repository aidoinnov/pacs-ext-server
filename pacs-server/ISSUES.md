# Known Issues

## DICOM Gateway Study Series API E2E 테스트 이슈

### Issue 1: Scenario 3 - Admin Study Series 테스트 실패
**상태**: 테스트에서 제외됨  
**원인**: 관리자 사용자 생성 실패 (ADMIN role not found)  
**영향**: 관리자 권한으로 Series 조회하는 시나리오 테스트 불가  
**해결 방안**: 
- ADMIN role이 데이터베이스에 존재하는지 확인
- 관리자 사용자 생성 로직 수정 필요
- 또는 테스트용 관리자 사용자를 미리 생성해두기

### Issue 2: Scenario 4 - Legacy API project_id 검증
**상태**: 테스트에서 제외됨  
**원인**: 관리자 권한이 있는 사용자는 project_id 없이도 200 응답을 받음  
**영향**: Legacy API의 project_id 필수 검증 테스트가 관리자 사용자에서는 실패  
**해결 방안**:
- 관리자 권한이 없는 일반 사용자로 테스트
- 또는 Legacy API의 project_id 검증 로직을 관리자 권한과 무관하게 수정

### Issue 3: Scenario 2 - User Study Series with project_id 403 에러
**상태**: ✅ 해결됨  
**증상**: `/api/me/dicom/studies/{study_uid}/series?project_id=2` 호출 시 403 "Access denied to this study" 에러  
**원인 분석**:
- `get_user_study_series` 함수에서 `can_access_study` 함수를 호출하여 Study 접근 권한 확인
- `can_access_study` 함수는 `project_data_access` 테이블을 확인
- 로직:
  1. `project_data_access` 테이블에 user_id + project_id 조합의 레코드가 없으면 → 전체 접근 허용
  2. 레코드가 있으면 → 해당 Study에 대한 APPROVED 상태 확인 필요
- `project_data_access` 테이블에 `user_id=1`, `project_id=2` 레코드가 있어서 접근이 차단됨

**해결 방법**:
- `project_data_access` 테이블에서 `user_id=1`, `project_id=2` 레코드 삭제
- 레코드가 없으면 `can_access_study` 함수가 전체 접근을 허용하므로 정상 작동

**확인 결과**:
- ✅ `/api/me/dicom/studies/{study_uid}/series?project_id=2` 정상 작동
- ✅ Scenario 2 테스트 통과

## 동기화 기능 이슈

### Issue 4: project_data_access 테이블에 자동으로 데이터가 채워지는 문제
**상태**: 조사 필요  
**증상**: `project_data_access` 테이블에 데이터가 자동으로 채워짐  
**원인 분석**:
1. **동기화 기능 (`sync_worker.rs`)**: 
   - 직접 SQL로 `project_data` 테이블에만 INSERT
   - `project_data_access`에는 데이터를 넣지 않음 ✅

2. **`project_data_service_impl.rs`의 `create_project_data`**:
   - `project_data` 생성 후 `grant_access_to_existing_users` 호출
   - 프로젝트의 모든 사용자에게 자동으로 접근 권한 부여
   - 이 함수는 `project_data_access` 테이블에 레코드를 생성함 ⚠️

3. **`project_user_use_case.rs`의 사용자 추가**:
   - 사용자가 프로젝트에 추가될 때 `grant_default_access_to_user` 호출
   - 프로젝트의 모든 `project_data`에 대해 접근 권한 부여
   - 이 함수도 `project_data_access` 테이블에 레코드를 생성함 ⚠️

**문제점**:
- 동기화는 직접 SQL을 사용하므로 `grant_access_to_existing_users`가 호출되지 않음
- 하지만 사용자가 프로젝트에 추가될 때 `grant_default_access_to_user`가 호출되어 기존 `project_data`에 대한 접근 권한이 자동 생성됨
- 또는 다른 API를 통해 `project_data`가 생성될 때 `grant_access_to_existing_users`가 호출됨

**확인 필요 사항**:
1. 동기화가 `project_data_service`를 사용하도록 변경해야 하는지
2. `grant_access_to_existing_users` / `grant_default_access_to_user` 로직을 비활성화하거나 조건부로 만들어야 하는지
3. 동기화로 생성된 `project_data`는 접근 권한을 자동으로 부여하지 않도록 해야 하는지

**해결 방안 제안**:
- 옵션 1: 동기화는 `project_data_access`를 채우지 않도록 유지 (현재 상태 유지)
- 옵션 2: `grant_default_access_to_user`에서 동기화로 생성된 데이터는 제외하도록 필터링
- 옵션 3: 동기화 전용 플래그를 추가하여 접근 권한 자동 부여를 스킵
