# 롤별 권한 관리 API 문서 인덱스

## 📚 문서 구조

이 폴더는 롤별 권한 관리 API에 대한 모든 문서를 포함합니다.

### 🚀 시작하기

| 문서 | 설명 | 대상 |
|------|------|------|
| **[README.md](README.md)** | 빠른 시작 가이드, 기본 사용법, 간단한 예시 | 모든 사용자 |
| **[quick-start-guide.md](quick-start-guide.md)** | 현재 구현된 API 빠른 이해 및 사용법, 서버 실행, 테스트 | 개발자, 시스템 관리자 |
| **[architecture-overview.md](architecture-overview.md)** | Clean Architecture 기반 전체 시스템 구조, 계층별 역할, 데이터 흐름 | 시스템 아키텍트, 백엔드 개발자 |
| **[api-reference.md](api-reference.md)** | 상세한 API 명세, 데이터 모델, 에러 코드 | 개발자 |

### 📖 상세 가이드

| 문서 | 설명 | 대상 |
|------|------|------|
| **[user-guide.md](user-guide.md)** | 상세한 사용법, UI 구현 방법, 문제 해결 | 프론트엔드 개발자 |
| **[api-examples.md](api-examples.md)** | 다양한 프레임워크별 구현 예시 | 개발자 |
| **[next-steps-implementation-guide.md](next-steps-implementation-guide.md)** | 다음 단계 기능 구현 가이드, 프로젝트별 롤-권한 관리, 사용자별 권한 관리 | 백엔드 개발자, 시스템 확장 담당자 |
| **[technical-documentation.md](technical-documentation.md)** | API 구현 세부사항, 아키텍처 | 백엔드 개발자 |

### 📋 프로젝트 문서

| 문서 | 설명 | 대상 |
|------|------|------|
| **[work-plan.md](work-plan.md)** | 프로젝트 계획서 | 프로젝트 관리자 |
| **[work-completion.md](work-completion.md)** | 작업 완료 보고서 | 프로젝트 관리자 |

## 🎯 사용자별 추천 문서

### 👨‍💻 프론트엔드 개발자
1. [README.md](README.md) - 빠른 시작
2. [quick-start-guide.md](quick-start-guide.md) - API 사용법
3. [user-guide.md](user-guide.md) - 상세 구현법
4. [api-examples.md](api-examples.md) - 프레임워크별 예시

### 🔧 백엔드 개발자
1. [README.md](README.md) - API 개요
2. [architecture-overview.md](architecture-overview.md) - 시스템 구조
3. [api-reference.md](api-reference.md) - 상세 명세
4. [next-steps-implementation-guide.md](next-steps-implementation-guide.md) - 기능 확장
5. [technical-documentation.md](technical-documentation.md) - 구현 세부사항

### 🏗️ 시스템 아키텍트
1. [architecture-overview.md](architecture-overview.md) - 전체 아키텍처
2. [technical-documentation.md](technical-documentation.md) - 기술적 세부사항
3. [next-steps-implementation-guide.md](next-steps-implementation-guide.md) - 확장 계획

### 📊 프로젝트 관리자
1. [work-plan.md](work-plan.md) - 프로젝트 계획
2. [work-completion.md](work-completion.md) - 완료 현황
3. [README.md](README.md) - 기능 개요

## 🔗 주요 API 엔드포인트

### 매트릭스 조회
- `GET /api/roles/global/permissions/matrix` - 롤-권한 매트릭스 조회

### 권한 관리
- `PUT /api/roles/{role_id}/permissions/{permission_id}` - 롤에 권한 할당/제거

## 📝 문서 업데이트 이력

- **2024-01-XX**: 초기 문서 작성
- **2024-01-XX**: 사용자 가이드 추가
- **2024-01-XX**: API 예시 추가
- **2024-01-XX**: 문서 구조 정리

## 🤝 기여하기

문서 개선이나 오류 수정이 필요하다면:
1. 해당 문서를 수정
2. 변경사항을 명확히 기록
3. 팀원들과 검토

## 📞 지원

API 사용 중 문제가 발생하면:
1. [README.md](README.md)의 문제 해결 섹션 확인
2. [api-reference.md](api-reference.md)의 에러 코드 확인
3. 필요시 개발팀에 문의

---

**마지막 업데이트**: 2024-01-XX  
**문서 버전**: 1.0.0
