# React Query 목록 갱신 안 되는 문제 해결 가이드

## 🚨 증상

- ✅ 서버 API는 정상 작동 (E2E 테스트 통과)
- ❌ React에서 생성/삭제/할당/해제 후 목록이 갱신 안 됨
- ❌ `invalidateQueries` 호출해도 변화 없음
- ❌ 새로고침(F5)하면 정상적으로 보임

---

## 🔍 원인 진단

### 1단계: React Query Devtools 확인

```bash
npm install @tanstack/react-query-devtools
```

```typescript
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';

<QueryClientProvider client={queryClient}>
  <App />
  <ReactQueryDevtools initialIsOpen={true} />  {/* ✅ 열린 상태로 시작 */}
</QueryClientProvider>
```

**확인 사항:**
- 🔍 `invalidateQueries` 호출 후 쿼리가 `fetching` 상태가 되는가?
- 🔍 쿼리가 `stale` 상태인가?
- 🔍 `dataUpdatedAt` 시간이 변경되는가?

---

### 2단계: Network 탭 확인

브라우저 개발자 도구 → Network 탭

**확인 사항:**
- 🔍 `invalidateQueries` 후 실제로 GET 요청이 발생하는가?
- 🔍 응답 상태 코드는? (200 OK vs 304 Not Modified)
- 🔍 응답 데이터가 최신인가?

---

## ✅ 해결 방법 (우선순위 순)

### 해결책 1: staleTime 설정 확인 ⭐⭐⭐

**가장 흔한 원인!**

```typescript
// ❌ 문제 코드
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: Infinity,  // 절대 갱신 안 됨!
    },
  },
});

// ✅ 해결 코드
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 0,  // 항상 stale 상태 → invalidate 시 즉시 재조회
    },
  },
});
```

---

### 해결책 2: queryKey 일치 확인 ⭐⭐⭐

**두 번째로 흔한 원인!**

```typescript
// ❌ 문제 코드
// 조회
useQuery({ queryKey: ['projects'] })

// 갱신
queryClient.invalidateQueries({ queryKey: ['project-list'] })  // 다른 키!

// ✅ 해결 코드
// 조회
useQuery({ queryKey: ['projects'] })

// 갱신
queryClient.invalidateQueries({ queryKey: ['projects'] })  // 같은 키!
```

**팁: queryKey를 상수로 관리**

```typescript
// src/constants/queryKeys.ts
export const QUERY_KEYS = {
  PROJECTS: ['projects'] as const,
  STUDIES: (params?: any) => ['studies', params] as const,
} as const;

// 사용
useQuery({ queryKey: QUERY_KEYS.PROJECTS })
queryClient.invalidateQueries({ queryKey: QUERY_KEYS.PROJECTS })
```

---

### 해결책 3: ETag 캐싱 방지 ⭐⭐

**서버가 ETag를 사용하는 경우!**

```typescript
// ✅ 방법 1: Cache-Control 헤더 추가 (간단)
export function useProjects() {
  return useQuery({
    queryKey: ['projects'],
    queryFn: async () => {
      const { data } = await api.get('/projects', {
        headers: { 'Cache-Control': 'no-cache' },  // ✅ ETag 무시
      });
      return data;
    },
  });
}

// ✅ 방법 2: Axios 인터셉터에서 전역 설정
api.interceptors.request.use((config) => {
  config.headers['Cache-Control'] = 'no-cache';
  return config;
});
```

---

### 해결책 4: refetchOnMount/refetchOnWindowFocus 활성화 ⭐

```typescript
// ❌ 문제 코드
useQuery({
  queryKey: ['projects'],
  queryFn: fetchProjects,
  refetchOnMount: false,  // 마운트 시 재조회 안 함
  refetchOnWindowFocus: false,  // 포커스 시 재조회 안 함
});

// ✅ 해결 코드
useQuery({
  queryKey: ['projects'],
  queryFn: fetchProjects,
  refetchOnMount: true,  // 마운트 시 재조회
  refetchOnWindowFocus: true,  // 포커스 시 재조회
});
```

---

### 해결책 5: onSuccess에서 올바르게 invalidate ⭐⭐

```typescript
// ❌ 문제 코드
const createProject = useMutation({
  mutationFn: (data) => api.post('/projects', data),
  // onSuccess 없음!
});

// ✅ 해결 코드
const createProject = useMutation({
  mutationFn: (data) => api.post('/projects', data),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['projects'] });
  },
});
```

---

## 🎯 완벽한 설정 (복사해서 사용)

### 1. QueryClient 설정

```typescript
// src/lib/queryClient.ts
import { QueryClient } from '@tanstack/react-query';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 0,  // ✅ 항상 stale
      gcTime: 5 * 60 * 1000,  // 5분 (구 cacheTime)
      refetchOnWindowFocus: true,  // ✅ 포커스 시 재조회
      refetchOnMount: true,  // ✅ 마운트 시 재조회
      retry: 1,
    },
    mutations: {
      retry: 0,
    },
  },
});
```

### 2. Axios 설정

```typescript
// src/lib/api.ts
import axios from 'axios';

const api = axios.create({
  baseURL: 'http://localhost:8080/api',
});

// 요청 인터셉터
api.interceptors.request.use((config) => {
  // 인증 토큰
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  
  // ✅ ETag 캐싱 방지
  config.headers['Cache-Control'] = 'no-cache';
  
  return config;
});

export default api;
```

### 3. Hook 패턴

```typescript
// src/hooks/useProjects.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '@/lib/api';

const QUERY_KEY = ['projects'] as const;

export function useProjects() {
  return useQuery({
    queryKey: QUERY_KEY,
    queryFn: async () => {
      const { data } = await api.get('/projects');
      return data;
    },
  });
}

export function useCreateProject() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (data) => api.post('/projects', data).then(res => res.data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEY });
    },
  });
}

export function useDeleteProject() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (id: number) => api.delete(`/projects/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEY });
    },
  });
}
```

---

## 🔧 디버깅 체크리스트

### ✅ 1. React Query 설정
- [ ] `staleTime: 0` 설정됨
- [ ] `refetchOnMount: true` 설정됨
- [ ] `refetchOnWindowFocus: true` 설정됨

### ✅ 2. queryKey
- [ ] 조회와 갱신에서 동일한 queryKey 사용
- [ ] queryKey에 필요한 params 포함 (예: `['studies', { projectId: 2 }]`)

### ✅ 3. Mutation
- [ ] `onSuccess`에서 `invalidateQueries` 호출
- [ ] 올바른 queryKey로 invalidate

### ✅ 4. API 설정
- [ ] `Cache-Control: no-cache` 헤더 추가
- [ ] Authorization 헤더 정상 작동

### ✅ 5. 디버깅 도구
- [ ] React Query Devtools 설치 및 확인
- [ ] 브라우저 Network 탭 확인
- [ ] 콘솔 로그로 확인

---

## 📚 관련 문서

- [프로젝트 목록 갱신 가이드](./REACT_QUERY_PROJECT_LIST_GUIDE.md)
- [Study 할당/해제 갱신 가이드](./REACT_QUERY_STUDY_ASSIGNMENT_GUIDE.md)

---

## 🆘 그래도 안 되면?

### 1. 강제 새로고침 버튼 추가

```typescript
const { refetch } = useProjects();

<button onClick={() => refetch()}>강제 새로고침</button>
```

### 2. 콘솔 로그 추가

```typescript
export function useProjects() {
  return useQuery({
    queryKey: ['projects'],
    queryFn: async () => {
      console.log('🔍 프로젝트 목록 조회 시작');
      const { data } = await api.get('/projects');
      console.log('✅ 프로젝트 목록:', data);
      return data;
    },
  });
}

export function useCreateProject() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async (data) => {
      console.log('➕ 프로젝트 생성:', data);
      const response = await api.post('/projects', data);
      console.log('✅ 생성 완료:', response.data);
      return response.data;
    },
    onSuccess: () => {
      console.log('🔄 invalidateQueries 호출');
      queryClient.invalidateQueries({ queryKey: ['projects'] });
    },
  });
}
```

### 3. React Query Devtools로 상태 확인

- Query가 `fetching` 상태가 되는지 확인
- `dataUpdatedAt` 시간이 변경되는지 확인
- Query가 `stale` 상태인지 확인

---

## 🎉 최종 체크

이 가이드를 따랐다면:
- ✅ 프로젝트 생성/삭제 후 목록 즉시 갱신
- ✅ Study 할당/해제 후 `is_assigned` 즉시 반영
- ✅ 브라우저 새로고침 없이 실시간 업데이트

**여전히 문제가 있다면 위의 디버깅 체크리스트를 하나씩 확인하세요!**

