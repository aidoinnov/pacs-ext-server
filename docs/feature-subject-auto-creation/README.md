# Subject 자동 생성 기능

## 📋 목차
- [01-작업계획.md](./01-작업계획.md) - 작업 배경 및 계획
- [02-작업내용.md](./02-작업내용.md) - 구현 상세 내용
- [03-기술문서.md](./03-기술문서.md) - 기술 사양 및 API 문서
- [04-마이그레이션-가이드.md](./04-마이그레이션-가이드.md) - 기존 데이터 마이그레이션 가이드

## 🎯 개요

Study를 프로젝트에 할당할 때 Patient ID를 기반으로 Subject를 자동 생성하는 기능입니다.

### 주요 기능
- ✅ Study 할당 시 PACS Archive에서 메타데이터 자동 조회
- ✅ Patient ID 기반 Subject 자동 생성
- ✅ 중복 방지 (기존 Subject 재사용)
- ✅ Patient ID가 없는 경우 자동 코드 생성 (A-001, A-002, ...)
- ✅ 기존 데이터 마이그레이션 도구 제공

### 관련 커밋
```
07f69c2 - feat: Fetch Study metadata from PACS Archive when assigning to project
9fb3ee4 - feat: Align Rust subject code generation with Python migration script
6ee8d10 - docs: Update migration README with patient_id handling examples
e70505d - feat: Support Studies without patient_id in migration
bbb9cb0 - fix: Improve DB URL detection in migration script
ae9f5c6 - docs: Update migration script README with actual output examples
211a8cb - fix: Correct table name in migration script
6b95a10 - feat: Add Subject auto-creation migration tool
35fea18 - refactor: Simplify Study assignment API - remove patient metadata fields
0826928 - feat: Auto-create Subject when assigning Study to Project
```

## 📊 작업 결과

### 코드 변경
- `pacs-server/src/application/use_cases/project_data_access_use_case.rs`
  - QIDO 클라이언트 주입
  - PACS Archive에서 메타데이터 조회 로직 추가
  - Subject 자동 생성 로직 개선

- `pacs-server/src/main.rs`
  - ProjectDataAccessUseCase에 QIDO 클라이언트 주입

### 마이그레이션 도구
- `pacs-server/scripts/migrate_subjects.py`
  - 기존 Study에 Subject 자동 생성
  - Dry-run 모드 지원
  - 375개 프로젝트 테스트 완료

## 🚀 빠른 시작

### 1. 새로운 Study 할당
```bash
POST /api/projects/{project_id}/data/studies
{
  "study_uid": "1.2.840.113619.2.55.3.2831184079.123.1234567890.1"
}
```

→ PACS Archive에서 자동으로 메타데이터 조회 및 Subject 생성

### 2. 기존 데이터 마이그레이션
```bash
cd pacs-server/scripts
python migrate_subjects.py --dry-run  # 미리보기
python migrate_subjects.py            # 실제 실행
```

## 📚 상세 문서

각 문서를 참고하세요:
1. **작업계획** - 왜 이 기능이 필요했는지
2. **작업내용** - 어떻게 구현했는지
3. **기술문서** - API 사양 및 코드 구조
4. **마이그레이션 가이드** - 기존 데이터 처리 방법

