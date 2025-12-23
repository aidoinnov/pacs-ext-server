# 어노테이션 테스트 섹션 사전 정리 단계 추가 계획

## 문제 분석

**증상**:

- 테스트 실행 시 "Project name already exists" 에러 발생
- 특히 "Annotation 권한 관리 테스트 프로젝트"는 고정 이름이라 재실행 시 항상 에러 발생

**영향받는 섹션**:

1. **"🔐 Annotation 권한 관리"** (sectionIndex === 7)

- 프로젝트 이름: "Annotation 권한 관리 테스트 프로젝트" (고정)
- 문제: 고정 이름이라 재실행 시 항상 중복 에러 발생

2. **"👁️ 권한 기반 Annotation 조회 (READ_ALL)"** (sectionIndex === 8)

- 프로젝트 이름: "READ_ALL Permission Test {timestamp}" (동적)
- 문제: 타임스탬프 기반이지만, 같은 밀리초에 실행하면 중복 가능

3. **"🔐 Annotation 권한 조회 API 개선"** (sectionIndex === 9)

- 프로젝트 이름: "Annotation Permissions API Test {timestamp}" (동적)
- 문제: 타임스탬프 기반이지만, 같은 밀리초에 실행하면 중복 가능

**참고 패턴**:

- "순차 시나리오" 섹션에는 이미 "0️⃣ 사전 정리 (기존 테스트 데이터 삭제)" 단계가 있음
- 프로젝트 목록 조회 → 이름으로 검색 → 삭제 패턴 사용

## 해결 방안

### 각 섹션별 사전 정리 단계 추가

1. **"🔐 Annotation 권한 관리"** 섹션

- "0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)" 추가
- 프로젝트 이름: "Annotation 권한 관리 테스트 프로젝트" (정확히 일치)
- 기존 테스트 인덱스 1씩 증가

2. **"👁️ 권한 기반 Annotation 조회 (READ_ALL)"** 섹션

- "0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)" 추가
- 프로젝트 이름 패턴: "READ_ALL Permission Test"로 시작하는 프로젝트 검색
- 기존 테스트 인덱스 1씩 증가

3. **"🔐 Annotation 권한 조회 API 개선"** 섹션

- "0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)" 추가
- 프로젝트 이름 패턴: "Annotation Permissions API Test"로 시작하는 프로젝트 검색
- 기존 테스트 인덱스 1씩 증가

## 구현 계획

### 1. 테스트 섹션 정의 수정

#### 1.1 "🔐 Annotation 권한 관리" 섹션

- "0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)" 테스트 항목 추가
- 기존 "1️⃣ 테스트용 프로젝트 생성" → "2️⃣"로 변경
- 나머지 테스트 인덱스도 1씩 증가 (2️⃣~1️⃣2️⃣)

#### 1.2 "👁️ 권한 기반 Annotation 조회 (READ_ALL)" 섹션

- "0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)" 테스트 항목 추가
- 기존 "1️⃣ 테스트용 프로젝트 생성" → "2️⃣"로 변경
- 나머지 테스트 인덱스도 1씩 증가 (2️⃣~1️⃣1️⃣)

#### 1.3 "🔐 Annotation 권한 조회 API 개선" 섹션

- "0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)" 테스트 항목 추가
- 기존 "1️⃣ 테스트용 프로젝트 생성" → "2️⃣"로 변경
- 나머지 테스트 인덱스도 1씩 증가 (2️⃣~1️⃣6️⃣)

### 2. 테스트 함수 수정

#### 2.1 `runAnnotationPermissionTest` 함수

- `testIndex === 0`: 사전 정리 (기존 프로젝트 삭제)
- 프로젝트 이름: "Annotation 권한 관리 테스트 프로젝트" (정확히 일치)
- `testIndex === 1`: 테스트용 프로젝트 생성 (기존 testIndex === 0 로직)
- 나머지 테스트 인덱스 1씩 증가

#### 2.2 `runReadAllPermissionTest` 함수

- `testIndex === 0`: 사전 정리 (기존 프로젝트 삭제)
- 프로젝트 이름 패턴: "READ_ALL Permission Test"로 시작
- `testIndex === 1`: 테스트용 프로젝트 생성 (기존 testIndex === 0 로직)
- 나머지 테스트 인덱스 1씩 증가

#### 2.3 `runAnnotationPermissionsApiTest` 함수

- `testIndex === 0`: 사전 정리 (기존 프로젝트 삭제)
- 프로젝트 이름 패턴: "Annotation Permissions API Test"로 시작
- `testIndex === 1`: 테스트용 프로젝트 생성 (기존 testIndex === 0 로직)
- 나머지 테스트 인덱스 1씩 증가

### 3. 사전 정리 로직 구현

**공통 패턴**:

1. 프로젝트 목록 조회: `GET /api/projects`
2. 이름 패턴으로 검색:

- 고정 이름: 정확히 일치하는 프로젝트 찾기
- 동적 이름: 이름이 특정 패턴으로 시작하는 프로젝트 찾기

3. 찾은 프로젝트 삭제: `DELETE /api/projects/{id}`
4. 에러 처리: 프로젝트가 없어도 정상 처리 (404 무시)

**구현 예시**:

```typescript
if (testIndex === 0) {
  // 0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)
  const config = await getAxiosConfig('SUPER_ADMIN');
  
  // 프로젝트 목록 조회
  const projectsResponse = await axios.get(`${apiUrl}/api/projects`, config);
  const projects = projectsResponse.data.projects || [];
  
  // 이름 패턴으로 검색
  const projectNamePattern = 'Annotation 권한 관리 테스트 프로젝트'; // 또는 패턴
  const existingProjects = projects.filter((p: any) => 
    p.name === projectNamePattern // 고정 이름
    // 또는 p.name.startsWith(projectNamePattern) // 동적 이름
  );
  
  // 찾은 프로젝트 삭제
  const deletedProjects = [];
  for (const project of existingProjects) {
    try {
      await axios.delete(`${apiUrl}/api/projects/${project.id}`, config);
      deletedProjects.push({ id: project.id, name: project.name });
      console.log(`  ✅ 기존 프로젝트 삭제: ${project.name} (ID: ${project.id})`);
    } catch (error: any) {
      // 404 에러는 무시 (이미 삭제된 경우)
      if (error.response?.status !== 404) {
        console.warn(`  ⚠️ 프로젝트 삭제 실패: ${project.name}`, error.message);
      }
    }
  }
  
  return {
    request: { method: 'DELETE', url: '/api/projects (기존 테스트 프로젝트)' },
    response: {
      message: '사전 정리 완료',
      deleted_count: deletedProjects.length,
      deleted_projects: deletedProjects,
    },
  };
}
```



## 변경 파일 목록

1. `auth-dashboard/src/components/ApiHealthCheck.tsx`

- 세 개의 어노테이션 테스트 섹션에 "0️⃣ 사전 정리" 항목 추가
- 모든 테스트 인덱스 조정
- 세 개의 테스트 함수에 사전 정리 로직 추가

## 테스트 시나리오

1. **고정 이름 프로젝트**:

- "Annotation 권한 관리 테스트 프로젝트"가 이미 존재하는 경우
- 사전 정리 단계에서 삭제 확인
- 프로젝트 생성 단계에서 정상 생성 확인

2. **동적 이름 프로젝트**:

- "READ_ALL Permission Test" 또는 "Annotation Permissions API Test"로 시작하는 프로젝트가 여러 개 존재하는 경우
- 사전 정리 단계에서 모두 삭제 확인
- 프로젝트 생성 단계에서 정상 생성 확인

3. **프로젝트가 없는 경우**:

- 사전 정리 단계에서 삭제할 프로젝트가 없는 경우
- 정상 처리 (에러 없이 진행)

4. **재실행 시나리오**:

- 테스트 실행 → 중단 → 재실행
- 사전 정리 단계에서 이전 테스트 프로젝트 삭제 확인

## 예상 효과

- 중복 프로젝트 이름 에러 방지
- 테스트 재실행 안정성 향상
- 테스트 격리 개선