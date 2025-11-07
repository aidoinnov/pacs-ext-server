# 🚀 프론트엔드 팀 - 여기서 시작하세요!

## 📚 필수 문서 (반드시 읽기)

### 1️⃣ **FRONTEND-INTEGRATION-GUIDE.md**
- 전체 통합 전략 및 아키텍처
- 데이터 로딩 흐름
- 캐시 전략
- **대상**: 프론트엔드 리더, 모든 개발자

### 2️⃣ **FRONTEND-API-SPEC.md**
- 완전한 API 명세
- 요청/응답 형식
- 에러 처리
- **대상**: 모든 개발자

### 3️⃣ **ANNOTATION-DATA-FIELD-STRATEGY.md**
- 데이터 필드 전략
- 시퀀스 다이어그램
- Version 검사 로직
- **대상**: 모든 개발자

---

## 📖 참고 문서 (필요시 읽기)

### 4️⃣ **ANNOTATION-LIST-OPTIMIZATION.md**
- 목록 최적화 전략
- 페이지네이션 설계
- **대상**: 성능 최적화 담당자

### 5️⃣ **VERSION-FIELD-EXPLANATION.md**
- Version 필드의 3가지 용도
- Optimistic Locking 설명
- **대상**: 동시성 제어 담당자

### 6️⃣ **BACKEND-SUMMARY-API-IMPLEMENTATION.md**
- 백엔드 구현 상세 정보
- SQL 쿼리 예제
- **대상**: 백엔드 개발자 (참고용)

---

## 🎯 핵심 요약

### API 엔드포인트

```
1. 사이드바 목록 (페이지네이션)
   GET /api/annotations/summary?series_instance_uid={uid}&page=1&limit=20

2. 상세 정보 (캔버스 그리기)
   GET /api/annotations/{id}

3. 수정 (Optimistic Locking)
   PUT /api/annotations/{id}
   { "base_version": 2, "annotation_data": {...} }

4. 생성
   POST /api/annotations

5. 삭제
   DELETE /api/annotations/{id}
```

### 2단계 로딩 전략

```
Step 1: 사이드바 목록 (요약 정보)
- GET /api/annotations/summary
- 응답: 50KB, 200-300ms
- 필드: type, label, color, tool_name, measurements, created_by_name, UIDs, version
- ✅ annotation_data 불필요

Step 2: 캔버스 그리기 (전체 정보)
- GET /api/annotations/{id}
- 응답: 500KB
- 필드: annotation_data (coordinates 포함!)
- ⚠️ annotation_data 필수!
```

### 중요 사항

```
1. Version 검사
   - 사이드바 version과 상세 정보 version 비교
   - 버전 불일치 시 최신 버전 사용

2. Optimistic Locking
   - 수정 시 base_version 필수
   - 409 Conflict 시 최신 버전 조회 후 재시도

3. 캐시 검증
   - HEAD 요청으로 캐시 검증
   - 304 Not Modified 시 캐시 사용
```

---

## 🔄 구현 순서

1. 요약 목록 API 호출 (사이드바 표시)
2. 상세 정보 API 호출 (캔버스 그리기)
3. Version 검사 로직 구현
4. Optimistic Locking 처리
5. 캐시 검증 (HEAD 요청)

---

## 📞 문의사항

API 명세에 대한 질문이나 추가 필요사항이 있으면 백엔드 팀에 문의하세요!

