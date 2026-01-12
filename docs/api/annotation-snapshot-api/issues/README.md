# Annotation Snapshot API - Issues

이 디렉토리는 Annotation Snapshot API 개발 과정에서 발생한 설계 결정, 기술적 이슈, 해결 방법을 문서화합니다.

---

## 📋 이슈 목록

### ✅ Resolved

| 번호 | 제목 | 카테고리 | 작성일 | 상태 |
|------|------|----------|--------|------|
| [ISSUE-001](./ISSUE-001-timestamp-responsibility.md) | 타임스탬프 필드의 책임 소재 | Design Decision, Security | 2026-01-11 | ✅ Resolved |
| [ISSUE-002](./ISSUE-002-no-update-annotation-entity.md) | UpdateAnnotation Entity 부재 | Architecture, Implementation | 2026-01-11 | ✅ Resolved |
| [ISSUE-003](./ISSUE-003-e2e-test-timeout.md) | E2E 테스트 웹 실행 시 타임아웃 문제 | Testing, Performance | 2026-01-12 | ✅ Resolved |

### 🔄 In Progress

_현재 진행 중인 이슈 없음_

### 📝 Open

_현재 열린 이슈 없음_

---

## 📚 카테고리별 분류

### Design Decision
- [ISSUE-001](./ISSUE-001-timestamp-responsibility.md) - 타임스탬프 필드의 책임 소재

### Security
- [ISSUE-001](./ISSUE-001-timestamp-responsibility.md) - 타임스탬프 필드의 책임 소재

### Data Integrity
- [ISSUE-001](./ISSUE-001-timestamp-responsibility.md) - 타임스탬프 필드의 책임 소재

### Architecture
- [ISSUE-002](./ISSUE-002-no-update-annotation-entity.md) - UpdateAnnotation Entity 부재

### Implementation
- [ISSUE-002](./ISSUE-002-no-update-annotation-entity.md) - UpdateAnnotation Entity 부재

### Testing
- [ISSUE-003](./ISSUE-003-e2e-test-timeout.md) - E2E 테스트 웹 실행 시 타임아웃 문제

### Performance
- [ISSUE-003](./ISSUE-003-e2e-test-timeout.md) - E2E 테스트 웹 실행 시 타임아웃 문제

---

## 🎯 이슈 작성 가이드

새로운 이슈를 작성할 때는 다음 템플릿을 사용하세요:

```markdown
# ISSUE-XXX: [이슈 제목]

> **이슈 번호**: ISSUE-XXX  
> **작성일**: YYYY-MM-DD  
> **상태**: 🔄 In Progress / ✅ Resolved / ❌ Closed  
> **카테고리**: [카테고리1, 카테고리2]

---

## 📋 이슈 요약

[간단한 요약]

---

## 🤔 문제 상황

[문제 상황 설명]

---

## 🔍 분석

[분석 내용]

---

## ✅ 결정 사항

[최종 결정]

---

## 🎯 구현 가이드

[구현 방법]

---

## 🔗 관련 문서

- [관련 문서 링크]

---

## 📝 교훈

[배운 점]
```

---

## 🔗 관련 문서

- [WORKLOG.md](../WORKLOG.md) - 작업 로그
- [ARCHITECTURE.md](../ARCHITECTURE.md) - 아키텍처 문서
- [API_SPEC.md](../API_SPEC.md) - API 명세서

