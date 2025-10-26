# User-Centered Matrix API 성능 최적화 작업 계획

## 📋 작업 개요

User-Centered Matrix API의 응답 시간을 추가로 개선하기 위한 최적화 작업입니다.

## 🎯 목표

현재 0.294초 → 목표 0.25초 (약 15% 추가 개선)

## 📅 작업 일정

- 시작일: 2024-12-01
- 완료일: 2024-12-01
- 소요시간: 1일

## 🔧 작업 범위

### 1. 불필요한 데이터 조회 제거

**문제점**: `joined_at` 필드가 조회되지만 실제로는 사용되지 않음

**작업 내용**:
- `MembershipInfo` DTO에서 `joined_at` 필드 제거
- SQL 쿼리에서 `up.created_at` 제거
- `query_as` 타입 5개 → 4개 컬럼으로 변경

**수정 파일**:
- `pacs-server/src/application/dto/user_project_matrix_dto.rs`
- `pacs-server/src/domain/services/user_service.rs`

**예상 효과**: 5-10ms 감소

### 2. HashMap 메모리 최적화

**문제점**: HashMap이 동적으로 크기를 조정하면서 재할당 발생

**작업 내용**:
- `HashMap::with_capacity()`로 사전 용량 할당
- 용량: `user_ids.len().saturating_mul(project_ids.len())`

**수정 파일**:
- `pacs-server/src/domain/services/user_service.rs`

**예상 효과**: 2-3ms 감소

### 3. 데이터베이스 인덱스 추가

**문제점**: 개별 인덱스만 존재하여 복합 조건 쿼리 성능 저하

**작업 내용**:
- `(user_id, project_id)` 복합 인덱스 추가
- 마이그레이션 파일 생성 및 실행

**수정 파일**:
- `pacs-server/migrations/015_add_user_project_composite_index.sql`

**예상 효과**: 10-20ms 감소

## 📊 성능 측정 방법

### 측정 명령어

```bash
time curl -s "http://localhost:8080/api/user-project-matrix?user_page=1&user_page_size=10&project_page=1&project_page_size=10&user_sort_by=username&user_sort_order=asc" > /dev/null
```

### 측정 항목

- 응답 시간 (real time)
- 사용자 시간 (user time)
- 시스템 시간 (sys time)
- 성능 개선율 계산

## 📝 체크리스트

### 구현 단계
- [ ] DTO에서 `joined_at` 필드 제거
- [ ] SQL 쿼리에서 `joined_at` 제거
- [ ] HashMap 사전 용량 할당 추가
- [ ] 마이그레이션 파일 생성
- [ ] 데이터베이스 인덱스 추가

### 테스트 단계
- [ ] 서버 재시작
- [ ] API 기능 검증
- [ ] 성능 측정 및 비교
- [ ] 결과 기록

### 문서 작성
- [ ] 작업 문서 작성
- [ ] 기술 문서 작성
- [ ] CHANGELOG 업데이트
- [ ] Git 커밋 및 푸시

## ⚠️ 주의사항

1. 기존 기능에 영향을 주지 않도록 주의
2. 성능 테스트를 정확하게 수행
3. 마이그레이션 실행 전 백업 권장
4. 변경 사항에 대한 충분한 문서화

## 🔗 관련 작업

- 1차 성능 최적화 (4초 → 0.294초)
- User-Centered Matrix API 구현
- N+1 쿼리 문제 해결

