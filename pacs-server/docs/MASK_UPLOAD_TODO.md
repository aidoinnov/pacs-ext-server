# 🚀 PACS 마스크 업로드 시스템 TODO

## 📋 개요
PACS 마스크 업로드 시스템의 현재 진행 상황과 남은 작업들을 정리한 문서입니다.

## ✅ 완료된 작업들

### Phase 1: 데이터베이스 스키마 구현 ✅
- [x] 마스크 관련 테이블 생성 (`annotation_mask_group`, `annotation_mask`)
- [x] 마이그레이션 스크립트 작성 및 실행
- [x] Rust 엔티티 및 DTO 생성 (`MaskGroup`, `Mask`, 관련 DTO들)
- [x] 데이터베이스 스키마와 Rust 엔티티 타입 정렬

### Phase 2: Object Storage 연동 ✅
- [x] 의존성 추가 (aws-sdk-s3, tokio-util, thiserror 등)
- [x] ObjectStorageService trait 정의
- [x] S3ObjectStorageService 구현체
- [x] MinIOObjectStorageService 구현체 (로컬 개발용)
- [x] Signed URL 발급 로직 (PUT/GET URL, TTL 설정)
- [x] ObjectStorageServiceFactory 및 Builder 패턴 구현

### Phase 4: Repository 구현체 ✅
- [x] MaskGroupRepository trait 정의
- [x] MaskRepository trait 정의
- [x] MaskGroupRepositoryImpl 구현
- [x] MaskRepositoryImpl 구현
- [x] 데이터베이스 CRUD 작업 완성
- [x] 동적 쿼리 바인딩으로 필터링 지원
- [x] BigDecimal → i64 변환 처리
- [x] main.rs 의존성 주입
- [x] 통합 테스트 통과 (9/9)

## 🚧 남은 작업들

### Phase 3: API 엔드포인트 구현 (2-3일 예상)
- [ ] `POST /api/annotations/{annotation_id}/mask-groups` - 그룹 생성
- [ ] `GET /api/annotations/{annotation_id}/mask-groups` - 그룹 목록 조회
- [ ] `GET /api/annotations/{annotation_id}/mask-groups/{group_id}` - 그룹 상세 조회
- [ ] `DELETE /api/annotations/{annotation_id}/mask-groups/{group_id}` - 그룹 삭제
- [ ] `POST /api/annotations/{annotation_id}/mask-groups/{group_id}/signed-url` - Signed URL 발급
- [ ] `POST /api/annotations/{annotation_id}/mask-groups/{group_id}/complete` - 업로드 완료 처리
- [ ] `GET /api/annotations/{annotation_id}/mask-groups/{group_id}/masks` - 마스크 목록 조회
- [ ] `GET /api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}` - 마스크 상세 조회

### Phase 4: 서비스 레이어 구현 (1-2일 예상)
- [ ] Use Case 구현
  - [ ] CreateMaskGroupUseCase
  - [ ] GenerateSignedUrlUseCase
  - [ ] CompleteUploadUseCase
  - [ ] ListMaskGroupsUseCase
  - [ ] DeleteMaskGroupUseCase
- [ ] 서비스 구현
  - [ ] MaskGroupService
  - [ ] MaskService

### Phase 5: 컨트롤러 구현 (1일 예상)
- [ ] Mask Group Controller
- [ ] Mask Controller
- [ ] Swagger 문서화
- [ ] 에러 핸들링

### Phase 6: 테스트 구현 (2-3일 예상)
- [ ] 단위 테스트
  - [ ] Repository 테스트
  - [ ] Service 테스트
  - [ ] Use Case 테스트
- [ ] 통합 테스트
  - [ ] API 엔드포인트 테스트
  - [ ] Object Storage 연동 테스트
  - [ ] 전체 플로우 테스트
- [ ] 성능 테스트
  - [ ] 병렬 업로드 테스트
  - [ ] 대용량 파일 업로드 테스트

### Phase 7: 설정 및 배포 (1일 예상)
- [ ] 환경 설정 파일 업데이트
- [ ] 배포 준비
- [ ] 모니터링 설정

## 🎯 다음 우선순위 작업

1. **Use Case 구현** - 비즈니스 로직 레이어
2. **Service 구현** - 도메인 서비스 레이어
3. **API 엔드포인트 구현** - 컨트롤러 레이어

## 📊 진행률

- **Phase 1**: 100% 완료 ✅
- **Phase 2**: 100% 완료 ✅
- **Phase 3**: 0% 진행 중 🚧
- **Phase 4**: 50% 완료 (Repository만 완료) 🚧
- **Phase 5**: 0% 진행 중 🚧
- **Phase 6**: 0% 진행 중 🚧
- **Phase 7**: 0% 진행 중 🚧

**전체 진행률**: 약 35% 완료

## 🔧 기술적 성과

### 해결된 주요 이슈들
- BigDecimal → i64 변환 처리
- 동적 쿼리 바인딩으로 유연한 필터링
- Option<T> 타입과 데이터베이스 NULL 값 처리
- ServiceError를 통한 일관된 에러 처리
- AWS SDK lifetime 이슈 해결

### 테스트 결과
```
running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---
**최종 업데이트**: 2025-10-07  
**작성자**: AI Assistant  
**버전**: 1.0
