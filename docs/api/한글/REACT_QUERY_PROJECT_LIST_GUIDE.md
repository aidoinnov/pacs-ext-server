# React Query로 프로젝트 목록 실시간 갱신 가이드

## 문제 상황

프로젝트를 생성하거나 삭제한 후 `invalidateQueries`를 호출해도 목록이 갱신되지 않는 문제가 발생할 수 있습니다.

**주요 원인:**
1. ❌ ETag 캐싱으로 인한 304 Not Modified 응답
2. ❌ React Query의 staleTime 설정
3. ❌ 잘못된 queryKey 사용
4. ❌ Axios/Fetch의 브라우저 캐시

---

## ✅ 해결 방법

### 1. ETag 처리가 포함된 Axios 설정

```typescript
// src/lib/api.ts
import axios from 'axios';

const api = axios.create({
  baseURL: 'http://localhost:8080/api',
  headers: {
    'Content-Type': 'application/json',
  },
});

// 요청 인터셉터: Authorization 헤더 추가
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// 응답 인터셉터: ETag 처리
api.interceptors.response.use(
  (response) => {
    // ETag 저장 (다음 요청에 사용)
    const etag = response.headers['etag'];
    if (etag && response.config.url) {
      sessionStorage.setItem(`etag:${response.config.url}`, etag);
    }
    return response;
  },
  (error) => {
    // 304 Not Modified는 에러가 아님
    if (error.response?.status === 304) {
      return Promise.resolve(error.response);
    }
    return Promise.reject(error);
  }
);

export default api;
```

---

### 2. 프로젝트 목록 조회 Hook (올바른 방법)

```typescript
// src/hooks/useProjects.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '@/lib/api';

// 프로젝트 목록 조회
export function useProjects() {
  return useQuery({
    queryKey: ['projects'],  // ✅ 단순하고 명확한 queryKey
    queryFn: async () => {
      const response = await api.get('/projects');
      return response.data;
    },
    staleTime: 0,  // ✅ 항상 최신 데이터 확인
    gcTime: 5 * 60 * 1000,  // 5분 (구 cacheTime)
    refetchOnWindowFocus: true,  // ✅ 창 포커스 시 재조회
    refetchOnMount: true,  // ✅ 마운트 시 재조회
  });
}

// 프로젝트 생성
export function useCreateProject() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (data: CreateProjectRequest) => {
      const response = await api.post('/projects', data);
      return response.data;
    },
    onSuccess: () => {
      // ✅ 방법 1: invalidateQueries (권장)
      queryClient.invalidateQueries({ queryKey: ['projects'] });
      
      // ✅ 방법 2: refetchQueries (즉시 재조회)
      // queryClient.refetchQueries({ queryKey: ['projects'] });
    },
  });
}

// 프로젝트 삭제
export function useDeleteProject() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (projectId: number) => {
      await api.delete(`/projects/${projectId}`);
    },
    onSuccess: () => {
      // ✅ invalidateQueries로 목록 갱신
      queryClient.invalidateQueries({ queryKey: ['projects'] });
    },
  });
}
```

---

### 3. 컴포넌트에서 사용 (올바른 방법)

```typescript
// src/components/ProjectList.tsx
import { useProjects, useCreateProject, useDeleteProject } from '@/hooks/useProjects';

export function ProjectList() {
  const { data, isLoading, error, refetch } = useProjects();
  const createProject = useCreateProject();
  const deleteProject = useDeleteProject();

  const handleCreate = async () => {
    try {
      await createProject.mutateAsync({
        name: '새 프로젝트',
        description: '설명',
        sponsor: '스폰서',
        start_date: '2025-01-01',
      });
      
      // ✅ mutateAsync 사용 시 자동으로 onSuccess 호출됨
      // 추가 작업이 필요하면 여기서 수행
      console.log('프로젝트 생성 완료!');
    } catch (error) {
      console.error('프로젝트 생성 실패:', error);
    }
  };

  const handleDelete = async (projectId: number) => {
    try {
      await deleteProject.mutateAsync(projectId);
      console.log('프로젝트 삭제 완료!');
    } catch (error) {
      console.error('프로젝트 삭제 실패:', error);
    }
  };

  if (isLoading) return <div>로딩 중...</div>;
  if (error) return <div>에러: {error.message}</div>;

  return (
    <div>
      <button onClick={handleCreate}>프로젝트 생성</button>
      
      {/* ✅ 수동 새로고침 버튼 (디버깅용) */}
      <button onClick={() => refetch()}>새로고침</button>
      
      <ul>
        {data?.projects?.map((project) => (
          <li key={project.id}>
            {project.name}
            <button onClick={() => handleDelete(project.id)}>삭제</button>
          </li>
        ))}
      </ul>
    </div>
  );
}
```

---

### 4. QueryClient 설정 (App.tsx)

```typescript
// src/App.tsx
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 0,  // ✅ 기본값: 항상 stale 상태
      gcTime: 5 * 60 * 1000,  // 5분
      refetchOnWindowFocus: true,  // ✅ 창 포커스 시 재조회
      refetchOnMount: true,  // ✅ 마운트 시 재조회
      retry: 1,  // 실패 시 1번 재시도
    },
  },
});

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <YourApp />
      {/* ✅ 개발 중 React Query 상태 확인 */}
      <ReactQueryDevtools initialIsOpen={false} />
    </QueryClientProvider>
  );
}
```

---

## 🔍 문제 해결 체크리스트

### ✅ 1. React Query 설정 확인

```typescript
// ❌ 잘못된 설정
staleTime: Infinity  // 절대 갱신 안 됨!
refetchOnWindowFocus: false  // 포커스 시 갱신 안 됨
refetchOnMount: false  // 마운트 시 갱신 안 됨

// ✅ 올바른 설정
staleTime: 0  // 항상 최신 데이터 확인
refetchOnWindowFocus: true
refetchOnMount: true
```

### ✅ 2. invalidateQueries 호출 확인

```typescript
// ❌ 잘못된 방법
queryClient.invalidateQueries(['projects', 'list']);  // queryKey 불일치!

// ✅ 올바른 방법
queryClient.invalidateQueries({ queryKey: ['projects'] });
```

### ✅ 3. ETag 캐싱 문제 해결

서버는 ETag를 사용하여 캐싱을 지원합니다. 하지만 React Query와 함께 사용할 때는 주의가 필요합니다.

**방법 1: ETag 무시 (간단)**
```typescript
export function useProjects() {
  return useQuery({
    queryKey: ['projects'],
    queryFn: async () => {
      // ✅ 캐시 무시 헤더 추가
      const response = await api.get('/projects', {
        headers: {
          'Cache-Control': 'no-cache',
        },
      });
      return response.data;
    },
    staleTime: 0,
  });
}
```

**방법 2: ETag 활용 (최적화)**
```typescript
export function useProjects() {
  return useQuery({
    queryKey: ['projects'],
    queryFn: async () => {
      const url = '/projects';
      const etag = sessionStorage.getItem(`etag:${url}`);
      
      const response = await api.get(url, {
        headers: etag ? { 'If-None-Match': etag } : {},
      });
      
      // 304 Not Modified인 경우 캐시된 데이터 사용
      if (response.status === 304) {
        const cached = sessionStorage.getItem(`data:${url}`);
        return cached ? JSON.parse(cached) : null;
      }
      
      // 새 데이터 저장
      sessionStorage.setItem(`data:${url}`, JSON.stringify(response.data));
      return response.data;
    },
    staleTime: 0,
  });
}
```

---

## 🚨 흔한 실수와 해결 방법

### 실수 1: queryKey 불일치

```typescript
// ❌ 잘못된 예
// 조회
useQuery({ queryKey: ['projects'] })

// 갱신
queryClient.invalidateQueries({ queryKey: ['project-list'] })  // 다른 키!
```

```typescript
// ✅ 올바른 예
// 조회
useQuery({ queryKey: ['projects'] })

// 갱신
queryClient.invalidateQueries({ queryKey: ['projects'] })  // 같은 키!
```

### 실수 2: onSuccess에서 await 사용

```typescript
// ❌ 잘못된 예
onSuccess: async () => {
  await queryClient.invalidateQueries({ queryKey: ['projects'] });
}

// ✅ 올바른 예
onSuccess: () => {
  queryClient.invalidateQueries({ queryKey: ['projects'] });
}
```

### 실수 3: mutate vs mutateAsync 혼동

```typescript
// ❌ 잘못된 예 (onSuccess가 두 번 호출될 수 있음)
const handleCreate = async () => {
  await createProject.mutateAsync(data);
  queryClient.invalidateQueries({ queryKey: ['projects'] });  // 중복!
};

// ✅ 올바른 예 (onSuccess에서만 처리)
const handleCreate = async () => {
  await createProject.mutateAsync(data);
  // onSuccess에서 자동으로 invalidate됨
};
```

---

## 🎯 최종 권장 패턴

```typescript
// src/hooks/useProjects.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '@/lib/api';

const QUERY_KEY = ['projects'] as const;

export function useProjects() {
  return useQuery({
    queryKey: QUERY_KEY,
    queryFn: async () => {
      const { data } = await api.get('/projects', {
        headers: { 'Cache-Control': 'no-cache' },
      });
      return data;
    },
    staleTime: 0,
    refetchOnWindowFocus: true,
  });
}

export function useCreateProject() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (data: CreateProjectRequest) => 
      api.post('/projects', data).then(res => res.data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEY });
    },
  });
}

export function useDeleteProject() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (id: number) => 
      api.delete(`/projects/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEY });
    },
  });
}
```

---

## 📊 디버깅 방법

### 1. React Query Devtools 사용

```bash
npm install @tanstack/react-query-devtools
```

```typescript
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';

<QueryClientProvider client={queryClient}>
  <App />
  <ReactQueryDevtools initialIsOpen={false} />
</QueryClientProvider>
```

### 2. 콘솔 로그로 확인

```typescript
export function useProjects() {
  return useQuery({
    queryKey: ['projects'],
    queryFn: async () => {
      console.log('🔍 프로젝트 목록 조회 시작');
      const { data } = await api.get('/projects');
      console.log('✅ 프로젝트 목록 조회 완료:', data);
      return data;
    },
  });
}

export function useCreateProject() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async (data) => {
      console.log('➕ 프로젝트 생성 시작:', data);
      const response = await api.post('/projects', data);
      console.log('✅ 프로젝트 생성 완료:', response.data);
      return response.data;
    },
    onSuccess: () => {
      console.log('🔄 프로젝트 목록 갱신 시작');
      queryClient.invalidateQueries({ queryKey: ['projects'] });
    },
  });
}
```

### 3. Network 탭 확인

브라우저 개발자 도구 → Network 탭에서:
- ✅ `GET /api/projects` 요청이 실제로 발생하는지 확인
- ✅ 응답 상태 코드 확인 (200 OK vs 304 Not Modified)
- ✅ ETag 헤더 확인

---

## 🎉 요약

1. **staleTime: 0** 설정으로 항상 최신 데이터 확인
2. **queryKey 일치** 확인 (조회와 갱신에서 동일한 키 사용)
3. **invalidateQueries** 사용 (onSuccess에서 호출)
4. **ETag 캐싱** 처리 (`Cache-Control: no-cache` 또는 적절한 ETag 처리)
5. **React Query Devtools** 사용하여 디버깅

이 가이드를 따르면 프로젝트 생성/삭제 후 목록이 즉시 갱신됩니다! ✅

