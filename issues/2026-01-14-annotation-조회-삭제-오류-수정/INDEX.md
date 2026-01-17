# 📚 문서 인덱스

## 🎯 빠른 시작

### 처음 읽는 분
1. **[SUMMARY.md](./SUMMARY.md)** - 요약 (5분)
2. **[README.md](./README.md)** - 전체 개요 (10분)

### 기술적 세부사항이 필요한 분
1. **[기술-분석.md](./기술-분석.md)** - sqlx 매핑 메커니즘 및 원인 분석
2. **[수정-상세-내역.md](./수정-상세-내역.md)** - 수정된 쿼리 목록 및 Before/After

### 향후 작업을 위한 분
1. **[체크리스트.md](./체크리스트.md)** - Entity 변경 시 체크리스트
2. **[다이어그램.md](./다이어그램.md)** - Mermaid 다이어그램

---

## 📄 문서 목록

### 1. [SUMMARY.md](./SUMMARY.md)
**요약 (Executive Summary)**

- 📌 한 줄 요약
- 🔴 문제 (증상, 원인, 결과)
- ✅ 해결 (수정 내용, 범위, 패턴)
- 📊 영향 분석
- ✅ 검증
- 🎯 핵심 교훈
- 🚀 다음 단계

**읽는 시간**: 5분  
**대상**: 모든 사람

---

### 2. [README.md](./README.md)
**이슈 개요 및 전체 설명**

- 📋 이슈 개요
- 🔍 원인 분석
- ✅ 해결 방법
- 🔧 수정 파일
- 📊 수정 전후 비교
- 🎯 핵심 교훈
- ✅ 검증
- 📝 관련 이슈
- 🚀 배포 체크리스트

**읽는 시간**: 10분  
**대상**: 프로젝트 관계자, 개발자

---

### 3. [수정-상세-내역.md](./수정-상세-내역.md)
**수정된 쿼리 목록 및 Before/After**

- 📝 수정된 쿼리 목록 (15개 메서드)
- Before/After 코드 비교
- 📊 통계
- 🎯 패턴

**읽는 시간**: 15분  
**대상**: 개발자, 코드 리뷰어

---

### 4. [기술-분석.md](./기술-분석.md)
**기술적 분석**

- 🔍 문제의 근본 원인
  - sqlx의 타입 매핑 메커니즘
  - `query_as!` vs `query_as::<_, T>`
- 🐛 에러 발생 메커니즘
  - Entity 구조체
  - SELECT 쿼리
  - sqlx 매핑 시도
  - 에러 전파
- 🔧 해결 방법의 원리
  - 필드 순서 일치
  - FromRow Derive Macro
- 📊 성능 영향 분석
- 🎯 예방 전략

**읽는 시간**: 20분  
**대상**: 개발자, 아키텍트

---

### 5. [체크리스트.md](./체크리스트.md)
**Entity 변경 시 체크리스트**

- ✅ 1단계: DB 스키마 변경
- ✅ 2단계: Entity 구조체 수정
- ✅ 3단계: Repository 쿼리 업데이트
- ✅ 4단계: Service 레이어 확인
- ✅ 5단계: Controller/API 확인
- ✅ 6단계: 테스트
- ✅ 7단계: 문서화
- ✅ 8단계: 배포 준비
- 🚨 자주 놓치는 항목
- 📊 체크리스트 요약
- 🎯 핵심 원칙

**읽는 시간**: 15분  
**대상**: 개발자 (필수 읽기!)

---

### 6. [다이어그램.md](./다이어그램.md)
**Mermaid 다이어그램**

- 🔄 문제 발생 흐름
- 🔍 sqlx 매핑 메커니즘
- 📊 Entity vs DB vs Query 비교
- 🔧 수정 범위
- 📈 수정 전후 비교
- 🎯 Entity 변경 시 체크 플로우
- 📊 영향 범위 분석

**읽는 시간**: 10분  
**대상**: 모든 사람 (시각적 이해)

---

## 🎯 사용 시나리오별 가이드

### 시나리오 1: "빠르게 이해하고 싶어요"
```
1. SUMMARY.md (5분)
2. 다이어그램.md (10분)
```
**총 소요 시간**: 15분

---

### 시나리오 2: "Entity를 변경해야 해요"
```
1. 체크리스트.md (15분) ⭐ 필수!
2. 기술-분석.md (20분)
3. 수정-상세-내역.md (15분)
```
**총 소요 시간**: 50분

---

### 시나리오 3: "유사한 문제를 디버깅하고 있어요"
```
1. README.md (10분)
2. 기술-분석.md (20분)
3. 수정-상세-내역.md (15분)
```
**총 소요 시간**: 45분

---

### 시나리오 4: "코드 리뷰를 해야 해요"
```
1. SUMMARY.md (5분)
2. 수정-상세-내역.md (15분)
3. 체크리스트.md (15분)
```
**총 소요 시간**: 35분

---

### 시나리오 5: "전체를 깊이 이해하고 싶어요"
```
1. SUMMARY.md (5분)
2. README.md (10분)
3. 기술-분석.md (20분)
4. 수정-상세-내역.md (15분)
5. 체크리스트.md (15분)
6. 다이어그램.md (10분)
```
**총 소요 시간**: 75분

---

## 📊 문서 통계

| 문서 | 크기 | 읽는 시간 | 난이도 |
|------|------|-----------|--------|
| SUMMARY.md | 5.9KB | 5분 | ⭐ 쉬움 |
| README.md | 6.8KB | 10분 | ⭐⭐ 보통 |
| 수정-상세-내역.md | 8.3KB | 15분 | ⭐⭐ 보통 |
| 기술-분석.md | 7.3KB | 20분 | ⭐⭐⭐ 어려움 |
| 체크리스트.md | 7.2KB | 15분 | ⭐⭐ 보통 |
| 다이어그램.md | 7.4KB | 10분 | ⭐ 쉬움 |

**총 크기**: 42.9KB  
**총 읽는 시간**: 75분

---

## 🔗 외부 링크

### 관련 코드
- [Annotation Entity](../../pacs-server/src/domain/entities/annotation.rs)
- [Annotation Repository](../../pacs-server/src/infrastructure/repositories/annotation_repository_impl.rs)
- [Migration 036](../../pacs-server/migrations/036_add_snapshot_image_to_annotations.sql)
- [Migration 037](../../pacs-server/migrations/037_fix_snapshot_uploaded_at_type.sql)

### 관련 문서
- [sqlx Documentation](https://docs.rs/sqlx/)
- [FromRow Derive Macro](https://docs.rs/sqlx/latest/sqlx/trait.FromRow.html)
- [PostgreSQL SELECT](https://www.postgresql.org/docs/current/sql-select.html)

---

## 📝 업데이트 이력

- **2026-01-14**: 초기 작성 (aido)

---

## 💡 팁

### 문서 검색
```bash
# 특정 키워드 검색
grep -r "snapshot_image_key" .

# 파일 목록 확인
ls -lh
```

### 문서 읽기 순서
1. **처음**: SUMMARY.md → 다이어그램.md
2. **상세**: README.md → 기술-분석.md
3. **실무**: 체크리스트.md → 수정-상세-내역.md

---

## 🎯 핵심 메시지

> **Entity 필드 추가 시 Repository의 모든 SELECT 쿼리를 빠짐없이 업데이트하세요!**

이 한 문장이 이 이슈의 핵심입니다. 자세한 내용은 [체크리스트.md](./체크리스트.md)를 참고하세요.

