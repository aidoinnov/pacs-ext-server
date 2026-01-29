# React Query로 Study 할당/해제 실시간 갱신 가이드

## 개요

Study를 프로젝트에 할당하거나 해제한 후 `is_assigned` 상태가 즉시 반영되도록 하는 가이드입니다.

---

## ✅ 올바른 구현 방법

### 1. Study 목록 조회 Hook

```typescript
// src/hooks/useStudies.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '@/lib/api';

interface StudyListParams {
  projectId?: number;
  checkAssignmentForProject?: number;
  page?: number;
  pageSize?: number;
  modality?: string;
  patientId?: string;
}

// Study 목록 조회
export function useStudies(params: StudyListParams) {
  return useQuery({
    queryKey: ['studies', params],  // ✅ params를 queryKey에 포함
    queryFn: async () => {
      const { data } = await api.get('/dicom/studies', {
        params,
        headers: { 'Cache-Control': 'no-cache' },  // ✅ ETag 캐싱 방지
      });
      return data;
    },
    staleTime: 0,  // ✅ 항상 최신 데이터 확인
    refetchOnWindowFocus: true,
    enabled: !!params.checkAssignmentForProject || !!params.projectId,  // ✅ 조건부 활성화
  });
}

// 할당 확인용 Study 목록 조회
export function useStudiesWithAssignment(projectId: number, params?: Omit<StudyListParams, 'checkAssignmentForProject'>) {
  return useStudies({
    ...params,
    checkAssignmentForProject: projectId,
    page: params?.page || 1,
    pageSize: params?.pageSize || 50,
  });
}
```

---

### 2. Study 할당/해제 Hook

```typescript
// src/hooks/useStudyAssignment.ts
import { useMutation, useQueryClient } from '@tanstack/react-query';
import api from '@/lib/api';

interface AssignStudyRequest {
  projectId: number;
  studyUid: string;
}

// Study 할당
export function useAssignStudy() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ projectId, studyUid }: AssignStudyRequest) => {
      const { data } = await api.post(`/projects/${projectId}/studies/assign`, {
        study_uid: studyUid,
      });
      return data;
    },
    onSuccess: (_, variables) => {
      // ✅ 해당 프로젝트의 Study 목록 갱신
      queryClient.invalidateQueries({
        queryKey: ['studies', { checkAssignmentForProject: variables.projectId }],
      });
      
      // ✅ 모든 Study 목록 갱신 (다른 화면에서도 사용 중일 수 있음)
      queryClient.invalidateQueries({
        queryKey: ['studies'],
      });
    },
  });
}

// Study 할당 해제
export function useUnassignStudy() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ projectId, studyUid }: AssignStudyRequest) => {
      await api.delete(`/projects/${projectId}/studies/${studyUid}/unassign`);
    },
    onSuccess: (_, variables) => {
      // ✅ 해당 프로젝트의 Study 목록 갱신
      queryClient.invalidateQueries({
        queryKey: ['studies', { checkAssignmentForProject: variables.projectId }],
      });
      
      // ✅ 모든 Study 목록 갱신
      queryClient.invalidateQueries({
        queryKey: ['studies'],
      });
    },
  });
}
```

---

### 3. 컴포넌트에서 사용

```typescript
// src/components/StudyAssignmentList.tsx
import { useStudiesWithAssignment, useAssignStudy, useUnassignStudy } from '@/hooks/useStudies';

interface Props {
  projectId: number;
}

export function StudyAssignmentList({ projectId }: Props) {
  const { data: studies, isLoading, refetch } = useStudiesWithAssignment(projectId, {
    page: 1,
    pageSize: 50,
  });
  
  const assignStudy = useAssignStudy();
  const unassignStudy = useUnassignStudy();

  const handleToggleAssignment = async (studyUid: string, isAssigned: boolean) => {
    try {
      if (isAssigned) {
        // 할당 해제
        await unassignStudy.mutateAsync({
          projectId,
          studyUid,
        });
        console.log('✅ Study 할당 해제 완료');
      } else {
        // 할당
        await assignStudy.mutateAsync({
          projectId,
          studyUid,
        });
        console.log('✅ Study 할당 완료');
      }
    } catch (error) {
      console.error('❌ Study 할당/해제 실패:', error);
    }
  };

  if (isLoading) return <div>로딩 중...</div>;

  return (
    <div>
      <h2>Study 목록 (프로젝트 ID: {projectId})</h2>
      
      {/* ✅ 수동 새로고침 버튼 */}
      <button onClick={() => refetch()}>새로고침</button>
      
      <table>
        <thead>
          <tr>
            <th>Study UID</th>
            <th>Patient ID</th>
            <th>할당 상태</th>
            <th>액션</th>
          </tr>
        </thead>
        <tbody>
          {studies?.map((study) => {
            const studyUid = study['0020000D']?.Value?.[0];
            const patientId = study['00100020']?.Value?.[0];
            const isAssigned = study.is_assigned;

            return (
              <tr key={studyUid}>
                <td>{studyUid}</td>
                <td>{patientId}</td>
                <td>
                  {isAssigned ? (
                    <span style={{ color: 'green' }}>✅ 할당됨</span>
                  ) : (
                    <span style={{ color: 'gray' }}>⭕ 미할당</span>
                  )}
                </td>
                <td>
                  <button
                    onClick={() => handleToggleAssignment(studyUid, isAssigned)}
                    disabled={assignStudy.isPending || unassignStudy.isPending}
                  >
                    {isAssigned ? '할당 해제' : '할당'}
                  </button>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
```

---

### 4. Optimistic Update (선택사항 - 더 빠른 UI)

사용자 경험을 개선하기 위해 서버 응답을 기다리지 않고 UI를 먼저 업데이트할 수 있습니다.

```typescript
// src/hooks/useStudyAssignment.ts (Optimistic Update 버전)
export function useAssignStudy() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ projectId, studyUid }: AssignStudyRequest) => {
      const { data } = await api.post(`/projects/${projectId}/studies/assign`, {
        study_uid: studyUid,
      });
      return data;
    },
    onMutate: async ({ projectId, studyUid }) => {
      // ✅ 진행 중인 쿼리 취소
      await queryClient.cancelQueries({
        queryKey: ['studies', { checkAssignmentForProject: projectId }],
      });

      // ✅ 이전 데이터 백업
      const previousStudies = queryClient.getQueryData([
        'studies',
        { checkAssignmentForProject: projectId },
      ]);

      // ✅ Optimistic Update: UI 먼저 업데이트
      queryClient.setQueryData(
        ['studies', { checkAssignmentForProject: projectId }],
        (old: any) => {
          if (!old) return old;
          return old.map((study: any) => {
            const uid = study['0020000D']?.Value?.[0];
            if (uid === studyUid) {
              return { ...study, is_assigned: true };
            }
            return study;
          });
        }
      );

      // ✅ 롤백을 위해 이전 데이터 반환
      return { previousStudies };
    },
    onError: (err, variables, context) => {
      // ✅ 에러 발생 시 롤백
      if (context?.previousStudies) {
        queryClient.setQueryData(
          ['studies', { checkAssignmentForProject: variables.projectId }],
          context.previousStudies
        );
      }
    },
    onSettled: (_, __, variables) => {
      // ✅ 성공/실패 관계없이 최종적으로 서버 데이터로 갱신
      queryClient.invalidateQueries({
        queryKey: ['studies', { checkAssignmentForProject: variables.projectId }],
      });
    },
  });
}

export function useUnassignStudy() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ projectId, studyUid }: AssignStudyRequest) => {
      await api.delete(`/projects/${projectId}/studies/${studyUid}/unassign`);
    },
    onMutate: async ({ projectId, studyUid }) => {
      await queryClient.cancelQueries({
        queryKey: ['studies', { checkAssignmentForProject: projectId }],
      });

      const previousStudies = queryClient.getQueryData([
        'studies',
        { checkAssignmentForProject: projectId },
      ]);

      // ✅ Optimistic Update: is_assigned를 false로 변경
      queryClient.setQueryData(
        ['studies', { checkAssignmentForProject: projectId }],
        (old: any) => {
          if (!old) return old;
          return old.map((study: any) => {
            const uid = study['0020000D']?.Value?.[0];
            if (uid === studyUid) {
              return { ...study, is_assigned: false };
            }
            return study;
          });
        }
      );

      return { previousStudies };
    },
    onError: (err, variables, context) => {
      if (context?.previousStudies) {
        queryClient.setQueryData(
          ['studies', { checkAssignmentForProject: variables.projectId }],
          context.previousStudies
        );
      }
    },
    onSettled: (_, __, variables) => {
      queryClient.invalidateQueries({
        queryKey: ['studies', { checkAssignmentForProject: variables.projectId }],
      });
    },
  });
}
```

---

## 🚨 흔한 실수

### 실수 1: queryKey에 params 미포함

```typescript
// ❌ 잘못된 예
useQuery({
  queryKey: ['studies'],  // params가 없음!
  queryFn: () => api.get('/dicom/studies', { params: { checkAssignmentForProject: 2 } }),
});

// ✅ 올바른 예
useQuery({
  queryKey: ['studies', { checkAssignmentForProject: 2 }],  // params 포함!
  queryFn: () => api.get('/dicom/studies', { params: { checkAssignmentForProject: 2 } }),
});
```

### 실수 2: invalidateQueries 범위 너무 좁음

```typescript
// ❌ 잘못된 예 (특정 프로젝트만 갱신)
onSuccess: () => {
  queryClient.invalidateQueries({
    queryKey: ['studies', { checkAssignmentForProject: 2 }],  // 프로젝트 2만!
  });
}

// ✅ 올바른 예 (모든 Study 쿼리 갱신)
onSuccess: () => {
  queryClient.invalidateQueries({
    queryKey: ['studies'],  // 모든 studies 쿼리!
  });
}
```

---

## 📊 디버깅

```typescript
export function useStudiesWithAssignment(projectId: number) {
  const query = useQuery({
    queryKey: ['studies', { checkAssignmentForProject: projectId }],
    queryFn: async () => {
      console.log(`🔍 Study 목록 조회 (프로젝트 ${projectId})`);
      const { data } = await api.get('/dicom/studies', {
        params: { checkAssignmentForProject: projectId, page: 1, pageSize: 50 },
      });
      console.log(`✅ Study 목록 조회 완료:`, data.length, '개');
      return data;
    },
    staleTime: 0,
  });

  console.log('Query 상태:', {
    isLoading: query.isLoading,
    isFetching: query.isFetching,
    isStale: query.isStale,
    dataUpdatedAt: new Date(query.dataUpdatedAt).toLocaleTimeString(),
  });

  return query;
}
```

---

## 🎉 요약

1. **queryKey에 params 포함** - 각 파라미터 조합마다 별도 캐시
2. **invalidateQueries로 광범위하게 갱신** - `['studies']`로 모든 Study 쿼리 갱신
3. **staleTime: 0** - 항상 최신 데이터 확인
4. **Optimistic Update** (선택) - 더 빠른 UI 반응
5. **Cache-Control: no-cache** - ETag 캐싱 방지

이 가이드를 따르면 Study 할당/해제 후 `is_assigned` 상태가 즉시 반영됩니다! ✅

