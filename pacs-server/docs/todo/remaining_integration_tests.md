# 🎯 남은 통합테스트 작업 TODO 리스트

## 📊 현재 상태
- **총 테스트 파일**: 20개
- **구현된 테스트**: 18개 (90%)
- **빠진 테스트**: 2개 (10%)
- **성공률**: 약 85% (일부 테스트 실패 있음)

## 📋 우선순위 1: 핵심 통합테스트 (즉시 필요)

### 1. `mask_controller_test.rs` 생성
- **목적**: 마스크 컨트롤러 통합테스트
- **테스트할 엔드포인트**:
  - `POST /api/annotations/{id}/mask-groups/{group_id}/masks` - 마스크 생성
  - `GET /api/annotations/{id}/mask-groups/{group_id}/masks` - 마스크 목록 조회
  - `GET /api/annotations/{id}/mask-groups/{group_id}/masks/{mask_id}` - 마스크 상세 조회
  - `PUT /api/annotations/{id}/mask-groups/{group_id}/masks/{mask_id}` - 마스크 수정
  - `DELETE /api/annotations/{id}/mask-groups/{group_id}/masks/{mask_id}` - 마스크 삭제
  - `POST /api/annotations/{id}/mask-groups/{group_id}/masks/{mask_id}/download-url` - 다운로드 URL 생성
  - `GET /api/annotations/{id}/mask-groups/{group_id}/masks/stats` - 마스크 통계 조회

### 2. `mask_group_controller_test.rs` 생성
- **목적**: 마스크 그룹 컨트롤러 통합테스트
- **테스트할 엔드포인트**:
  - `POST /api/annotations/{id}/mask-groups` - 마스크 그룹 생성
  - `GET /api/annotations/{id}/mask-groups` - 마스크 그룹 목록 조회
  - `GET /api/annotations/{id}/mask-groups/{group_id}` - 마스크 그룹 상세 조회
  - `PUT /api/annotations/{id}/mask-groups/{group_id}` - 마스크 그룹 수정
  - `DELETE /api/annotations/{id}/mask-groups/{group_id}` - 마스크 그룹 삭제
  - `POST /api/annotations/{id}/mask-groups/{group_id}/upload-url` - 업로드 URL 생성
  - `POST /api/annotations/{id}/mask-groups/{group_id}/complete-upload` - 업로드 완료 처리

### 3. `mask_upload_workflow_test.rs` 생성
- **목적**: 마스크 업로드 전체 워크플로우 통합 테스트
- **테스트할 플로우**:
  1. 마스크 그룹 생성
  2. 업로드 URL 요청
  3. S3/MinIO에 파일 업로드 (Mock)
  4. 업로드 완료 알림
  5. 마스크 메타데이터 생성
  6. 다운로드 URL 생성
  7. 마스크 조회 및 검증

### 4. `annotation_controller_test.rs` 수정
- **목적**: 실패 테스트 해결
- **현재 상태**: 5개 통과, 4개 실패
- **해결할 문제**:
  - 테스트 데이터 설정 문제
  - 데이터베이스 정리 문제
  - HTTP 상태 코드 불일치

## 📋 우선순위 2: 고급 통합테스트 (중요)

### 5. `comprehensive_integration_test.rs` 생성
- **목적**: 전체 API 엔드포인트 통합 테스트
- **테스트 범위**:
  - 모든 컨트롤러 엔드포인트
  - 엔드포인트 간 연동
  - 전체 워크플로우

### 6. `performance_test.rs` 생성
- **목적**: 성능 테스트
- **테스트 시나리오**:
  - 대용량 파일 업로드 (100MB+)
  - 병렬 업로드 (10개 동시)
  - 메모리 사용량 모니터링
  - 응답 시간 측정

### 7. `error_handling_test.rs` 생성
- **목적**: 에러 시나리오 통합 테스트
- **테스트 시나리오**:
  - 네트워크 오류
  - 권한 오류
  - 데이터 유효성 오류
  - 데이터베이스 연결 오류
  - 객체 스토리지 오류

### 8. `authentication_integration_test.rs` 생성
- **목적**: JWT 인증 통합 테스트
- **테스트 시나리오**:
  - 토큰 생성 및 검증
  - 토큰 만료 처리
  - 권한 기반 접근 제어
  - 인증 실패 시나리오

## 📋 우선순위 3: 인프라 및 보안 테스트 (권장)

### 9. `object_storage_integration_test.rs` 생성
- **목적**: S3/MinIO 실제 연동 테스트
- **테스트 시나리오**:
  - 실제 S3/MinIO 연결
  - 파일 업로드/다운로드
  - Signed URL 생성 및 검증
  - 권한 및 ACL 테스트

### 10. `database_cleanup_test.rs` 생성
- **목적**: 데이터베이스 정리 및 롤백 테스트
- **테스트 시나리오**:
  - 트랜잭션 롤백
  - 데이터 정리 프로세스
  - 외래키 제약 조건
  - 데이터 일관성

### 11. `cors_security_test.rs` 생성
- **목적**: CORS 및 보안 헤더 테스트
- **테스트 시나리오**:
  - CORS 정책 검증
  - 보안 헤더 확인
  - CSRF 보호
  - XSS 방지

### 12. `api_documentation_test.rs` 생성
- **목적**: API 문서화 테스트
- **테스트 시나리오**:
  - Swagger/OpenAPI 문서 정확성
  - 엔드포인트 문서화
  - 요청/응답 스키마 검증
  - 예제 데이터 검증

## 📋 우선순위 4: 고급 테스트 (선택사항)

### 13. `load_test.rs` 생성
- **목적**: 부하 테스트
- **테스트 시나리오**:
  - 동시 사용자 (100+)
  - 대용량 요청 처리
  - 시스템 한계 테스트
  - 성능 저하 지점 파악

### 14. `concurrent_access_test.rs` 생성
- **목적**: 동시 접근 테스트
- **테스트 시나리오**:
  - 동시 데이터 수정
  - 락 처리
  - 경쟁 조건
  - 데이터 일관성

### 15. `data_validation_test.rs` 생성
- **목적**: 데이터 유효성 검증 통합 테스트
- **테스트 시나리오**:
  - 입력 데이터 검증
  - 비즈니스 규칙 검증
  - 데이터 타입 검증
  - 범위 및 제약 조건

### 16. `audit_log_test.rs` 생성
- **목적**: 감사 로그 및 접근 기록 테스트
- **테스트 시나리오**:
  - 접근 로그 기록
  - 감사 추적
  - 로그 검색 및 필터링
  - 보안 이벤트 모니터링

## 📋 우선순위 5: 운영 테스트 (장기)

### 17. `database_migration_test.rs` 생성
- **목적**: 데이터베이스 마이그레이션 테스트
- **테스트 시나리오**:
  - 스키마 변경
  - 데이터 마이그레이션
  - 롤백 프로세스
  - 버전 호환성

### 18. `configuration_test.rs` 생성
- **목적**: 환경별 설정 테스트
- **테스트 시나리오**:
  - 개발/스테이징/프로덕션 환경
  - 설정 파일 검증
  - 환경 변수 처리
  - 설정 오류 처리

### 19. `monitoring_test.rs` 생성
- **목적**: 헬스체크 및 모니터링 테스트
- **테스트 시나리오**:
  - 헬스체크 엔드포인트
  - 메트릭 수집
  - 알림 시스템
  - 로그 수집

### 20. `backup_restore_test.rs` 생성
- **목적**: 데이터 백업 및 복원 테스트
- **테스트 시나리오**:
  - 데이터 백업
  - 복원 프로세스
  - 백업 무결성
  - 복원 검증

## 🚀 다음 단계

### 즉시 시작할 작업
1. **`mask_controller_test.rs` 생성** - 가장 중요한 누락된 테스트
2. **`mask_group_controller_test.rs` 생성** - 마스크 업로드 핵심 기능
3. **`annotation_controller_test.rs` 수정** - 기존 실패 테스트 해결
4. **`mask_upload_workflow_test.rs` 생성** - 전체 워크플로우 검증

### 완료 시점
- **우선순위 1 완료**: 100% 핵심 통합테스트 커버리지 달성
- **우선순위 2 완료**: 고품질 통합테스트 환경 구축
- **우선순위 3 완료**: 프로덕션 준비 완료
- **우선순위 4-5 완료**: 엔터프라이즈급 테스트 환경 구축

## 📝 참고사항

- 각 테스트 파일은 기존 테스트 패턴을 따라 작성
- 데이터베이스 정리 로직 포함
- Mock 객체 활용으로 외부 의존성 최소화
- 실제 API 엔드포인트 호출로 통합 테스트 구현
- 에러 시나리오 및 경계값 테스트 포함
