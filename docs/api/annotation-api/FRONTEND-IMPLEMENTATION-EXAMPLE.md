# 💻 Annotation API - 프론트엔드 구현 예제

## 📋 목차

1. [캐시 매니저 구현](#캐시-매니저-구현)
2. [Study/Series 로드](#studyseries-로드)
3. [Instance 로드](#instance-로드)
4. [캐시 검증](#캐시-검증)
5. [Annotation 수정](#annotation-수정)

---

## 캐시 매니저 구현

### AnnotationCacheManager 클래스

```typescript
class AnnotationCacheManager {
  private cache: Map<string, CacheEntry> = new Map();
  private readonly CACHE_TTL = 5 * 60 * 1000; // 5분

  interface CacheEntry {
    version: number;
    etag: string;
    lastModified: string;
    data: Annotation[];
    timestamp: number;
  }

  // 캐시 저장
  set(key: string, data: Annotation[], version: number, etag: string, lastModified: string) {
    this.cache.set(key, {
      version,
      etag,
      lastModified,
      data,
      timestamp: Date.now()
    });
  }

  // 캐시 조회
  get(key: string): CacheEntry | null {
    const entry = this.cache.get(key);
    
    if (!entry) return null;
    
    // 캐시 만료 확인
    if (Date.now() - entry.timestamp > this.CACHE_TTL) {
      this.cache.delete(key);
      return null;
    }
    
    return entry;
  }

  // 캐시 무효화
  invalidate(key: string) {
    this.cache.delete(key);
  }

  // 전체 캐시 초기화
  clear() {
    this.cache.clear();
  }

  // 캐시 통계
  getStats() {
    return {
      size: this.cache.size,
      entries: Array.from(this.cache.entries()).map(([key, entry]) => ({
        key,
        version: entry.version,
        age: Date.now() - entry.timestamp
      }))
    };
  }
}
```

---

## Study/Series 로드

### Phase 1: 초기 데이터 로드

```typescript
class AnnotationService {
  private cache = new AnnotationCacheManager();
  private apiBase = '/api/annotations';

  // Study/Series 레벨 Annotation 조회
  async loadStudyAndSeriesAnnotations(
    studyInstanceUid: string,
    projectId?: number
  ): Promise<Annotation[]> {
    const cacheKey = `study:${studyInstanceUid}`;
    
    // 1. 캐시 확인
    const cached = this.cache.get(cacheKey);
    if (cached) {
      console.log('✅ 캐시에서 로드:', cacheKey);
      return cached.data;
    }

    // 2. API 요청
    console.log('📡 API에서 로드:', cacheKey);
    const params = new URLSearchParams({
      study_instance_uid: studyInstanceUid,
      level: 'study,series'
    });

    if (projectId) {
      params.append('project_id', projectId.toString());
    }

    const response = await fetch(`${this.apiBase}?${params}`);
    
    if (!response.ok) {
      throw new Error(`API 요청 실패: ${response.status}`);
    }

    const result = await response.json();
    const annotations = result.annotations;

    // 3. 응답 헤더에서 메타데이터 추출
    const etag = response.headers.get('ETag') || '';
    const lastModified = response.headers.get('Last-Modified') || '';
    const version = this.extractVersionFromETag(etag);

    // 4. 캐시 저장
    this.cache.set(cacheKey, annotations, version, etag, lastModified);

    return annotations;
  }

  // ETag에서 버전 추출
  private extractVersionFromETag(etag: string): number {
    // ETag 형식: "1" → 1
    const match = etag.match(/^"(\d+)"$/);
    return match ? parseInt(match[1], 10) : 0;
  }
}
```

---

## Instance 로드

### Phase 2: Instance 선택 시 데이터 로드

```typescript
async loadInstanceAnnotations(
  seriesInstanceUid: string,
  projectId?: number
): Promise<Annotation[]> {
  const cacheKey = `instance:${seriesInstanceUid}`;
  
  // 1. 캐시 확인
  const cached = this.cache.get(cacheKey);
  if (cached) {
    console.log('✅ 캐시에서 로드:', cacheKey);
    return cached.data;
  }

  // 2. API 요청
  console.log('📡 API에서 로드:', cacheKey);
  const params = new URLSearchParams({
    series_instance_uid: seriesInstanceUid,
    level: 'instance'
  });

  if (projectId) {
    params.append('project_id', projectId.toString());
  }

  const response = await fetch(`${this.apiBase}?${params}`);
  
  if (!response.ok) {
    throw new Error(`API 요청 실패: ${response.status}`);
  }

  const result = await response.json();
  const annotations = result.annotations;

  // 3. 응답 헤더에서 메타데이터 추출
  const etag = response.headers.get('ETag') || '';
  const lastModified = response.headers.get('Last-Modified') || '';
  const version = this.extractVersionFromETag(etag);

  // 4. 캐시 저장
  this.cache.set(cacheKey, annotations, version, etag, lastModified);

  return annotations;
}
```

---

## 캐시 검증

### HEAD 요청으로 최신 버전 확인

```typescript
async validateCache(
  annotationId: number,
  cachedVersion: number
): Promise<{ valid: boolean; newVersion?: number }> {
  try {
    // HEAD 요청으로 메타데이터만 조회
    const response = await fetch(
      `${this.apiBase}/${annotationId}`,
      {
        method: 'HEAD',
        headers: {
          'If-None-Match': `"${cachedVersion}"`
        }
      }
    );

    if (response.status === 304) {
      // 캐시 유효
      console.log('✅ 캐시 유효 (304 Not Modified)');
      return { valid: true };
    } else if (response.status === 200) {
      // 새로운 버전 있음
      const etag = response.headers.get('ETag') || '';
      const newVersion = this.extractVersionFromETag(etag);
      console.log('⚠️ 새로운 버전 있음:', newVersion);
      return { valid: false, newVersion };
    }

    return { valid: false };
  } catch (error) {
    console.error('캐시 검증 실패:', error);
    return { valid: false };
  }
}

// 사용 예
async function onInstanceSelected(instanceId: number) {
  const cached = annotationCache.get(`instance:${instanceId}`);
  
  if (cached) {
    // 캐시 검증
    const validation = await annotationService.validateCache(
      instanceId,
      cached.version
    );

    if (validation.valid) {
      // 캐시 사용
      displayAnnotations(cached.data);
    } else {
      // 새로운 데이터 조회
      const annotations = await annotationService.loadInstanceAnnotations(instanceId);
      displayAnnotations(annotations);
    }
  } else {
    // 캐시 없음 - 조회
    const annotations = await annotationService.loadInstanceAnnotations(instanceId);
    displayAnnotations(annotations);
  }
}
```

---

## Annotation 수정

### 버전 기반 동시성 제어

```typescript
async updateAnnotation(
  annotationId: number,
  newData: any,
  currentVersion: number
): Promise<{ success: boolean; data?: Annotation; conflict?: any }> {
  try {
    const response = await fetch(
      `${this.apiBase}/${annotationId}`,
      {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          base_version: currentVersion,  // ← 중요!
          annotation_data: newData
        })
      }
    );

    if (response.status === 200) {
      // 업데이트 성공
      const updated = await response.json();
      console.log('✅ 업데이트 성공:', updated);
      
      // 캐시 업데이트
      this.updateCacheEntry(annotationId, updated);
      
      return { success: true, data: updated };
    } else if (response.status === 409) {
      // 버전 충돌
      const conflict = await response.json();
      console.warn('⚠️ 버전 충돌:', conflict);
      
      return {
        success: false,
        conflict: {
          currentVersion: conflict.current_version,
          clientVersion: conflict.client_version,
          message: conflict.message
        }
      };
    } else {
      throw new Error(`예상치 못한 상태 코드: ${response.status}`);
    }
  } catch (error) {
    console.error('업데이트 실패:', error);
    return { success: false };
  }
}

// 캐시 엔트리 업데이트
private updateCacheEntry(annotationId: number, updated: Annotation) {
  // 모든 캐시 엔트리를 순회하며 해당 annotation 업데이트
  for (const [key, entry] of this.cache.entries()) {
    const index = entry.data.findIndex(a => a.id === annotationId);
    if (index !== -1) {
      entry.data[index] = updated;
      entry.version = updated.version;
    }
  }
}

// 사용 예
async function onAnnotationEdit(annotationId: number, newData: any) {
  const annotation = findAnnotationById(annotationId);
  
  const result = await annotationService.updateAnnotation(
    annotationId,
    newData,
    annotation.version
  );

  if (result.success) {
    showNotification('✅ 저장되었습니다');
    refreshUI();
  } else if (result.conflict) {
    showConflictDialog({
      title: '편집 충돌',
      message: `다른 사용자가 이 항목을 수정했습니다.\n현재 버전: ${result.conflict.currentVersion}`,
      onReload: async () => {
        // 최신 버전 조회
        const latest = await annotationService.getAnnotationById(annotationId);
        displayAnnotation(latest);
      }
    });
  } else {
    showNotification('❌ 저장 실패');
  }
}
```

---

## 🎯 완전한 사용 예제

```typescript
// 1. 서비스 초기화
const annotationService = new AnnotationService();

// 2. Study 선택 시
async function onStudySelected(studyInstanceUid: string) {
  try {
    const annotations = await annotationService.loadStudyAndSeriesAnnotations(
      studyInstanceUid
    );
    displayStudyAndSeriesAnnotations(annotations);
  } catch (error) {
    showError('Annotation 로드 실패');
  }
}

// 3. Instance 선택 시
async function onInstanceSelected(seriesInstanceUid: string) {
  try {
    const annotations = await annotationService.loadInstanceAnnotations(
      seriesInstanceUid
    );
    displayInstanceAnnotations(annotations);
  } catch (error) {
    showError('Instance Annotation 로드 실패');
  }
}

// 4. Annotation 수정
async function onAnnotationUpdate(annotationId: number, newData: any) {
  const annotation = findAnnotationById(annotationId);
  
  const result = await annotationService.updateAnnotation(
    annotationId,
    newData,
    annotation.version
  );

  if (result.success) {
    showNotification('✅ 저장되었습니다');
  } else if (result.conflict) {
    showConflictDialog(result.conflict);
  }
}

// 5. 캐시 통계 확인
function showCacheStats() {
  const stats = annotationService.getCacheStats();
  console.log('캐시 통계:', stats);
}
```

---

## 📊 성능 모니터링

```typescript
class PerformanceMonitor {
  private metrics = {
    cacheHits: 0,
    cacheMisses: 0,
    apiCalls: 0,
    totalTime: 0
  };

  recordCacheHit() {
    this.metrics.cacheHits++;
  }

  recordCacheMiss() {
    this.metrics.cacheMisses++;
  }

  recordApiCall(duration: number) {
    this.metrics.apiCalls++;
    this.metrics.totalTime += duration;
  }

  getReport() {
    const total = this.metrics.cacheHits + this.metrics.cacheMisses;
    const hitRate = total > 0 ? (this.metrics.cacheHits / total * 100).toFixed(2) : 0;
    const avgTime = this.metrics.apiCalls > 0 
      ? (this.metrics.totalTime / this.metrics.apiCalls).toFixed(2) 
      : 0;

    return {
      cacheHitRate: `${hitRate}%`,
      totalRequests: total,
      apiCalls: this.metrics.apiCalls,
      averageApiTime: `${avgTime}ms`
    };
  }
}
```

---

## ✅ 체크리스트

- [ ] AnnotationCacheManager 구현
- [ ] AnnotationService 구현
- [ ] Study/Series 로드 구현
- [ ] Instance 로드 구현
- [ ] 캐시 검증 구현
- [ ] Annotation 수정 구현
- [ ] 에러 처리 구현
- [ ] 성능 모니터링 구현
- [ ] UI 통합
- [ ] 테스트 작성

