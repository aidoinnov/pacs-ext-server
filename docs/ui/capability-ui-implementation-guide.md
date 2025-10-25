# Capability UI 구현 가이드

## 개요

이 문서는 Capability API를 사용하여 권한 관리 UI를 구현하는 방법을 설명합니다. React/TypeScript 기반으로 작성되었지만, 다른 프레임워크에도 적용 가능합니다.

## 1. 기본 설정

### API 클라이언트 설정

```typescript
// api/capabilityApi.ts
const API_BASE_URL = 'http://localhost:8080/api';

export interface RoleInfo {
  id: number;
  name: string;
  description: string;
  scope: string;
}

export interface CapabilityInfo {
  id: number;
  name: string;
  display_name: string;
  description: string | null;
  category: string;
  permission_count: number;
}

export interface RoleCapabilityMatrixResponse {
  roles: RoleInfo[];
  capabilities_by_category: Record<string, CapabilityInfo[]>;
  assignments: Array<{
    role_id: number;
    capability_id: number;
    assigned: boolean;
  }>;
}

export class CapabilityApi {
  static async getGlobalMatrix(): Promise<RoleCapabilityMatrixResponse> {
    const response = await fetch(`${API_BASE_URL}/roles/global/capabilities/matrix`);
    if (!response.ok) {
      throw new Error('Failed to fetch role capability matrix');
    }
    return response.json();
  }

  static async updateAssignment(
    roleId: number, 
    capabilityId: number, 
    assign: boolean
  ): Promise<void> {
    const response = await fetch(
      `${API_BASE_URL}/roles/${roleId}/capabilities/${capabilityId}`,
      {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ assign })
      }
    );
    if (!response.ok) {
      throw new Error('Failed to update assignment');
    }
  }

  static async getCapabilityDetail(capabilityId: number) {
    const response = await fetch(`${API_BASE_URL}/capabilities/${capabilityId}`);
    if (!response.ok) {
      throw new Error('Failed to fetch capability detail');
    }
    return response.json();
  }
}
```

## 2. 메인 매트릭스 컴포넌트

### RoleCapabilityMatrix.tsx

```typescript
import React, { useState, useEffect } from 'react';
import { CapabilityApi, RoleCapabilityMatrixResponse } from '../api/capabilityApi';
import { CapabilityCategoryGroup } from './CapabilityCategoryGroup';
import { CapabilityDetailModal } from './CapabilityDetailModal';

export const RoleCapabilityMatrix: React.FC = () => {
  const [matrix, setMatrix] = useState<RoleCapabilityMatrixResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedCapability, setSelectedCapability] = useState<number | null>(null);

  useEffect(() => {
    fetchMatrix();
  }, []);

  const fetchMatrix = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await CapabilityApi.getGlobalMatrix();
      setMatrix(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const handleToggleAssignment = async (
    roleId: number, 
    capabilityId: number, 
    assign: boolean
  ) => {
    try {
      await CapabilityApi.updateAssignment(roleId, capabilityId, assign);
      // 로컬 상태 업데이트
      setMatrix(prev => {
        if (!prev) return prev;
        const assignmentIndex = prev.assignments.findIndex(
          a => a.role_id === roleId && a.capability_id === capabilityId
        );
        if (assignmentIndex >= 0) {
          prev.assignments[assignmentIndex].assigned = assign;
        } else {
          prev.assignments.push({ role_id: roleId, capability_id: capabilityId, assigned: assign });
        }
        return { ...prev };
      });
    } catch (err) {
      console.error('Failed to update assignment:', err);
      // 에러 발생 시 원래 상태로 복원
      fetchMatrix();
    }
  };

  const handleCapabilityClick = (capabilityId: number) => {
    setSelectedCapability(capabilityId);
  };

  if (loading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
        <p>Error: {error}</p>
        <button 
          onClick={fetchMatrix}
          className="mt-2 bg-red-500 text-white px-4 py-2 rounded hover:bg-red-600"
        >
          Retry
        </button>
      </div>
    );
  }

  if (!matrix) return null;

  return (
    <div className="role-capability-matrix">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900">역할-권한 매트릭스</h1>
        <p className="text-gray-600">각 역할에 대한 권한을 관리합니다.</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
        {Object.entries(matrix.capabilities_by_category).map(([category, capabilities]) => (
          <CapabilityCategoryGroup
            key={category}
            category={category}
            capabilities={capabilities}
            roles={matrix.roles}
            assignments={matrix.assignments}
            onToggleAssignment={handleToggleAssignment}
            onCapabilityClick={handleCapabilityClick}
          />
        ))}
      </div>

      {selectedCapability && (
        <CapabilityDetailModal
          capabilityId={selectedCapability}
          onClose={() => setSelectedCapability(null)}
        />
      )}
    </div>
  );
};
```

## 3. 카테고리별 그룹 컴포넌트

### CapabilityCategoryGroup.tsx

```typescript
import React from 'react';
import { CapabilityInfo, RoleInfo } from '../api/capabilityApi';

interface CapabilityCategoryGroupProps {
  category: string;
  capabilities: CapabilityInfo[];
  roles: RoleInfo[];
  assignments: Array<{
    role_id: number;
    capability_id: number;
    assigned: boolean;
  }>;
  onToggleAssignment: (roleId: number, capabilityId: number, assign: boolean) => void;
  onCapabilityClick: (capabilityId: number) => void;
}

export const CapabilityCategoryGroup: React.FC<CapabilityCategoryGroupProps> = ({
  category,
  capabilities,
  roles,
  assignments,
  onToggleAssignment,
  onCapabilityClick
}) => {
  const getAssignment = (roleId: number, capabilityId: number) => {
    return assignments.find(
      a => a.role_id === roleId && a.capability_id === capabilityId
    )?.assigned || false;
  };

  return (
    <div className="bg-white rounded-lg shadow-md p-6">
      <h2 className="text-lg font-semibold text-gray-900 mb-4 border-b pb-2">
        {category}
      </h2>
      
      <div className="space-y-4">
        {capabilities.map(capability => (
          <div key={capability.id} className="border rounded-lg p-4 hover:bg-gray-50">
            <div className="flex items-center justify-between mb-2">
              <div className="flex-1">
                <h3 
                  className="font-medium text-gray-900 cursor-pointer hover:text-blue-600"
                  onClick={() => onCapabilityClick(capability.id)}
                >
                  {capability.display_name}
                </h3>
                <p className="text-sm text-gray-600">{capability.description}</p>
                <span className="inline-block bg-blue-100 text-blue-800 text-xs px-2 py-1 rounded-full mt-1">
                  {capability.permission_count}개 권한
                </span>
              </div>
            </div>
            
            <div className="mt-3">
              <div className="grid grid-cols-2 gap-2">
                {roles.map(role => {
                  const isAssigned = getAssignment(role.id, capability.id);
                  return (
                    <label key={role.id} className="flex items-center space-x-2">
                      <input
                        type="checkbox"
                        checked={isAssigned}
                        onChange={(e) => onToggleAssignment(role.id, capability.id, e.target.checked)}
                        className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                      />
                      <span className="text-sm text-gray-700">{role.name}</span>
                    </label>
                  );
                })}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
```

## 4. Capability 상세 모달

### CapabilityDetailModal.tsx

```typescript
import React, { useState, useEffect } from 'react';
import { CapabilityApi } from '../api/capabilityApi';

interface CapabilityDetailModalProps {
  capabilityId: number;
  onClose: () => void;
}

interface CapabilityDetail {
  capability: {
    id: number;
    name: string;
    display_name: string;
    description: string | null;
    category: string;
    permission_count: number;
  };
  permissions: Array<{
    id: number;
    category: string;
    resource_type: string;
    action: string;
  }>;
}

export const CapabilityDetailModal: React.FC<CapabilityDetailModalProps> = ({
  capabilityId,
  onClose
}) => {
  const [detail, setDetail] = useState<CapabilityDetail | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (capabilityId) {
      fetchCapabilityDetail(capabilityId);
    }
  }, [capabilityId]);

  const fetchCapabilityDetail = async (id: number) => {
    try {
      setLoading(true);
      const data = await CapabilityApi.getCapabilityDetail(id);
      setDetail(data);
    } catch (error) {
      console.error('Failed to fetch capability detail:', error);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <div className="bg-white rounded-lg p-6 max-w-md w-full mx-4">
          <div className="flex justify-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
          </div>
        </div>
      </div>
    );
  }

  if (!detail) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto">
        <div className="p-6">
          <div className="flex justify-between items-start mb-4">
            <div>
              <h2 className="text-xl font-bold text-gray-900">
                {detail.capability.display_name}
              </h2>
              <p className="text-gray-600">{detail.capability.description}</p>
              <span className="inline-block bg-blue-100 text-blue-800 text-sm px-3 py-1 rounded-full mt-2">
                {detail.capability.category} • {detail.capability.permission_count}개 권한
              </span>
            </div>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600 text-2xl"
            >
              ×
            </button>
          </div>

          <div className="border-t pt-4">
            <h3 className="text-lg font-semibold text-gray-900 mb-3">
              포함된 권한 ({detail.permissions.length}개)
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
              {detail.permissions.map(permission => (
                <div key={permission.id} className="bg-gray-50 rounded-lg p-3">
                  <div className="flex items-center justify-between">
                    <span className="font-medium text-gray-900">
                      {permission.resource_type}
                    </span>
                    <span className="bg-green-100 text-green-800 text-xs px-2 py-1 rounded-full">
                      {permission.action}
                    </span>
                  </div>
                  <p className="text-sm text-gray-600 mt-1">
                    {permission.category}
                  </p>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
```

## 5. 고급 기능

### 배치 업데이트 훅

```typescript
// hooks/useBatchUpdate.ts
import { useState, useCallback } from 'react';

interface PendingUpdate {
  roleId: number;
  capabilityId: number;
  assign: boolean;
}

export const useBatchUpdate = () => {
  const [pendingUpdates, setPendingUpdates] = useState<PendingUpdate[]>([]);
  const [isUpdating, setIsUpdating] = useState(false);

  const addUpdate = useCallback((roleId: number, capabilityId: number, assign: boolean) => {
    setPendingUpdates(prev => {
      const existing = prev.find(
        p => p.roleId === roleId && p.capabilityId === capabilityId
      );
      if (existing) {
        return prev.map(p => 
          p.roleId === roleId && p.capabilityId === capabilityId 
            ? { ...p, assign }
            : p
        );
      }
      return [...prev, { roleId, capabilityId, assign }];
    });
  }, []);

  const executeBatch = useCallback(async () => {
    if (pendingUpdates.length === 0) return;

    setIsUpdating(true);
    try {
      await Promise.all(
        pendingUpdates.map(update =>
          CapabilityApi.updateAssignment(
            update.roleId, 
            update.capabilityId, 
            update.assign
          )
        )
      );
      setPendingUpdates([]);
    } catch (error) {
      console.error('Batch update failed:', error);
    } finally {
      setIsUpdating(false);
    }
  }, [pendingUpdates]);

  const clearPending = useCallback(() => {
    setPendingUpdates([]);
  }, []);

  return {
    pendingUpdates,
    isUpdating,
    addUpdate,
    executeBatch,
    clearPending,
    hasPendingUpdates: pendingUpdates.length > 0
  };
};
```

### 검색 및 필터링

```typescript
// components/CapabilitySearch.tsx
import React, { useState, useMemo } from 'react';
import { CapabilityInfo } from '../api/capabilityApi';

interface CapabilitySearchProps {
  capabilities: CapabilityInfo[];
  onFilteredCapabilities: (capabilities: CapabilityInfo[]) => void;
}

export const CapabilitySearch: React.FC<CapabilitySearchProps> = ({
  capabilities,
  onFilteredCapabilities
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('');

  const categories = useMemo(() => {
    return Array.from(new Set(capabilities.map(c => c.category)));
  }, [capabilities]);

  const filteredCapabilities = useMemo(() => {
    return capabilities.filter(capability => {
      const matchesSearch = capability.display_name
        .toLowerCase()
        .includes(searchTerm.toLowerCase()) ||
        capability.description?.toLowerCase().includes(searchTerm.toLowerCase());
      
      const matchesCategory = !selectedCategory || capability.category === selectedCategory;
      
      return matchesSearch && matchesCategory;
    });
  }, [capabilities, searchTerm, selectedCategory]);

  React.useEffect(() => {
    onFilteredCapabilities(filteredCapabilities);
  }, [filteredCapabilities, onFilteredCapabilities]);

  return (
    <div className="mb-6 space-y-4">
      <div className="flex space-x-4">
        <input
          type="text"
          placeholder="권한 검색..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <select
          value={selectedCategory}
          onChange={(e) => setSelectedCategory(e.target.value)}
          className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value="">모든 카테고리</option>
          {categories.map(category => (
            <option key={category} value={category}>
              {category}
            </option>
          ))}
        </select>
      </div>
    </div>
  );
};
```

## 6. 스타일링 (Tailwind CSS)

```css
/* styles/capability-matrix.css */
.role-capability-matrix {
  @apply p-6 bg-gray-50 min-h-screen;
}

.capability-item {
  @apply transition-all duration-200 hover:shadow-md;
}

.capability-checkbox {
  @apply rounded border-gray-300 text-blue-600 focus:ring-blue-500 focus:ring-2;
}

.permission-badge {
  @apply inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium;
}

.permission-badge-create {
  @apply bg-green-100 text-green-800;
}

.permission-badge-read {
  @apply bg-blue-100 text-blue-800;
}

.permission-badge-update {
  @apply bg-yellow-100 text-yellow-800;
}

.permission-badge-delete {
  @apply bg-red-100 text-red-800;
}
```

## 7. 에러 처리 및 로딩 상태

```typescript
// components/ErrorBoundary.tsx
import React, { Component, ErrorInfo, ReactNode } from 'react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Capability Matrix Error:', error, errorInfo);
  }

  public render() {
    if (this.state.hasError) {
      return this.props.fallback || (
        <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
          <h2 className="font-bold">Something went wrong</h2>
          <p>{this.state.error?.message}</p>
          <button 
            onClick={() => this.setState({ hasError: false, error: undefined })}
            className="mt-2 bg-red-500 text-white px-4 py-2 rounded hover:bg-red-600"
          >
            Try again
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
```

## 8. 테스트 예시

```typescript
// __tests__/RoleCapabilityMatrix.test.tsx
import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { RoleCapabilityMatrix } from '../components/RoleCapabilityMatrix';
import { CapabilityApi } from '../api/capabilityApi';

// Mock API
jest.mock('../api/capabilityApi');
const mockCapabilityApi = CapabilityApi as jest.Mocked<typeof CapabilityApi>;

describe('RoleCapabilityMatrix', () => {
  const mockMatrix = {
    roles: [
      { id: 1, name: 'ADMIN', description: '관리자', scope: 'GLOBAL' },
      { id: 2, name: 'USER', description: '일반 사용자', scope: 'GLOBAL' }
    ],
    capabilities_by_category: {
      '관리': [
        {
          id: 1,
          name: 'USER_MANAGEMENT',
          display_name: '사용자 관리',
          description: '사용자 계정 관리',
          category: '관리',
          permission_count: 4
        }
      ]
    },
    assignments: [
      { role_id: 1, capability_id: 1, assigned: true },
      { role_id: 2, capability_id: 1, assigned: false }
    ]
  };

  beforeEach(() => {
    mockCapabilityApi.getGlobalMatrix.mockResolvedValue(mockMatrix);
    mockCapabilityApi.updateAssignment.mockResolvedValue();
  });

  it('renders role capability matrix', async () => {
    render(<RoleCapabilityMatrix />);
    
    await waitFor(() => {
      expect(screen.getByText('역할-권한 매트릭스')).toBeInTheDocument();
      expect(screen.getByText('사용자 관리')).toBeInTheDocument();
    });
  });

  it('toggles capability assignment', async () => {
    render(<RoleCapabilityMatrix />);
    
    await waitFor(() => {
      const checkbox = screen.getByRole('checkbox', { name: /USER/ });
      fireEvent.click(checkbox);
    });

    expect(mockCapabilityApi.updateAssignment).toHaveBeenCalledWith(2, 1, true);
  });
});
```

## 9. 성능 최적화

### 메모이제이션

```typescript
import React, { memo, useMemo } from 'react';

export const CapabilityItem = memo(({ 
  capability, 
  roles, 
  assignments, 
  onToggleAssignment 
}: CapabilityItemProps) => {
  const roleAssignments = useMemo(() => {
    return roles.map(role => ({
      role,
      assigned: assignments.find(
        a => a.role_id === role.id && a.capability_id === capability.id
      )?.assigned || false
    }));
  }, [roles, assignments, capability.id]);

  return (
    // JSX...
  );
});
```

### 가상화 (대량 데이터용)

```typescript
import { FixedSizeList as List } from 'react-window';

const VirtualizedCapabilityList = ({ capabilities, height = 400 }) => {
  const Row = ({ index, style }) => (
    <div style={style}>
      <CapabilityItem capability={capabilities[index]} />
    </div>
  );

  return (
    <List
      height={height}
      itemCount={capabilities.length}
      itemSize={120}
    >
      {Row}
    </List>
  );
};
```

이 가이드를 따라 구현하면 사용자 친화적이고 효율적인 권한 관리 UI를 만들 수 있습니다.
