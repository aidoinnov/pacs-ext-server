# Issues

이 폴더는 프로젝트에서 발견된 이슈들을 추적하고 문서화하는 곳입니다.

## 📋 현재 이슈 목록

### 🔴 High Priority

1. **[Sync API Timeout Issue](./sync-api-timeout-issue.md)**
   - **상태**: 미해결
   - **날짜**: 2025-12-18
   - **요약**: `POST /api/sync/run` API가 응답하지 않고 타임아웃 발생
   - **영향**: 수동 동기화 실행 불가 (자동 스케줄러는 작동)

### 🟡 Medium Priority

2. **[Duplicate Data Issue](./duplicate-data-issue.md)**
   - **상태**: 확인됨
   - **날짜**: 2025-12-18
   - **요약**: `project_data` 테이블에 동일한 Study가 중복 등록됨
   - **영향**: 데이터 중복으로 인한 스토리지 낭비 및 성능 저하 가능성

## 🔍 이슈 상태 정의

- 🔴 **미해결 (Open)**: 아직 해결되지 않은 이슈
- 🟡 **확인됨 (Confirmed)**: 이슈가 확인되었으나 해결 방안 미정
- 🟢 **진행중 (In Progress)**: 해결 작업 진행 중
- ✅ **해결됨 (Resolved)**: 이슈가 해결됨
- ⏸️ **보류 (On Hold)**: 일시적으로 작업 중단

## 📝 이슈 작성 가이드

새로운 이슈를 발견하면 다음 형식으로 문서를 작성해주세요:

```markdown
# [Issue Title]

**날짜**: YYYY-MM-DD
**상태**: 🔴/🟡/🟢/✅/⏸️
**우선순위**: High/Medium/Low

## 📋 문제 요약
간단한 문제 설명

## 🔍 증상
구체적인 증상 나열

## 🔎 원인 분석
예상되는 원인

## 🛠️ 해결 방안
제안된 해결 방법

## 🎯 다음 단계
해야 할 작업 목록

## 📌 참고 사항
추가 정보
```

## 🔗 관련 문서

- [Sync Engine Implementation](../server/implementation/sync_engine/implementation.md)
- [Database Schema](../database/)
- [API Documentation](../api/)

## 📞 문의

이슈에 대한 질문이나 제안사항이 있으면 팀에 문의해주세요.

