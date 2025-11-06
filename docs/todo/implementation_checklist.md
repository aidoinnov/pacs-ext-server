# ✅ PACS 마스크 업로드 v2 구현 체크리스트

## 🎯 Phase 1: 데이터베이스 스키마 구현

### 1.1 마이그레이션 파일 생성
- [ ] `migrations/003_add_mask_tables.sql` 파일 생성
- [ ] `annotation_mask_group` 테이블 DDL 작성
- [ ] `annotation_mask` 테이블 DDL 작성
- [ ] 인덱스 생성 스크립트 작성
- [ ] 마이그레이션 실행 테스트

### 1.2 Rust 엔티티 생성
- [ ] `src/domain/entities/mask_group.rs` 생성
- [ ] `MaskGroup` 구조체 정의
- [ ] `NewMaskGroup` 구조체 정의
- [ ] `src/domain/entities/mask.rs` 생성
- [ ] `Mask` 구조체 정의
- [ ] `NewMask` 구조체 정의
- [ ] `mod.rs` 파일 업데이트

### 1.3 Repository Traits 정의
- [ ] `src/domain/repositories/mask_group_repository.rs` 생성
- [ ] `MaskGroupRepository` trait 정의
- [ ] `src/domain/repositories/mask_repository.rs` 생성
- [ ] `MaskRepository` trait 정의

## 🎯 Phase 2: Object Storage 연동

### 2.1 의존성 추가
- [ ] `Cargo.toml`에 `aws-sdk-s3` 추가
- [ ] `Cargo.toml`에 `aws-config` 추가
- [ ] `Cargo.toml`에 `tokio-util` 추가
- [ ] 의존성 설치 및 컴파일 확인

### 2.2 Object Storage Service 구현
- [ ] `src/application/services/object_storage_service.rs` 생성
- [ ] `ObjectStorageService` trait 정의
- [ ] `src/infrastructure/external/s3_service.rs` 생성
- [ ] `S3ObjectStorageService` 구현
- [ ] `src/infrastructure/external/minio_service.rs` 생성 (선택)
- [ ] `MinIOObjectStorageService` 구현 (선택)

### 2.3 설정 파일 업데이트
- [ ] `config/default.toml`에 Object Storage 설정 추가
- [ ] `src/infrastructure/config/settings.rs` 업데이트
- [ ] 환경 변수 설정 가이드 작성

## 🎯 Phase 3: Application Layer 구현

### 3.1 DTOs 생성
- [ ] `src/application/dto/mask_group_dto.rs` 생성
- [ ] `CreateMaskGroupRequest` DTO 정의
- [ ] `MaskGroupResponse` DTO 정의
- [ ] `SignedUrlRequest` DTO 정의
- [ ] `SignedUrlResponse` DTO 정의
- [ ] `CompleteUploadRequest` DTO 정의
- [ ] `src/application/dto/mask_dto.rs` 생성
- [ ] `MaskResponse` DTO 정의
- [ ] `mod.rs` 파일 업데이트

### 3.2 Use Cases 구현
- [ ] `src/application/use_cases/mask_group_use_case.rs` 생성
- [ ] `CreateMaskGroupUseCase` 구현
- [ ] `ListMaskGroupsUseCase` 구현
- [ ] `GenerateSignedUrlUseCase` 구현
- [ ] `CompleteUploadUseCase` 구현
- [ ] `DeleteMaskGroupUseCase` 구현
- [ ] `src/application/use_cases/mask_use_case.rs` 생성
- [ ] `ListMasksUseCase` 구현
- [ ] `GetMaskUseCase` 구현

### 3.3 Services 구현
- [ ] `src/application/services/mask_group_service.rs` 생성
- [ ] `MaskGroupService` 구현
- [ ] `src/application/services/mask_service.rs` 생성
- [ ] `MaskService` 구현

## 🎯 Phase 4: Infrastructure Layer 구현

### 4.1 Repository 구현
- [ ] `src/infrastructure/repositories/mask_group_repository_impl.rs` 생성
- [ ] `MaskGroupRepositoryImpl` 구현
- [ ] `src/infrastructure/repositories/mask_repository_impl.rs` 생성
- [ ] `MaskRepositoryImpl` 구현
- [ ] `mod.rs` 파일 업데이트

### 4.2 의존성 주입 설정
- [ ] `src/main.rs`에 새로운 서비스들 등록
- [ ] `src/application/mod.rs` 업데이트
- [ ] `src/infrastructure/mod.rs` 업데이트

## 🎯 Phase 5: Presentation Layer 구현

### 5.1 Controllers 생성
- [ ] `src/presentation/controllers/mask_group_controller.rs` 생성
- [ ] `create_mask_group` 엔드포인트 구현
- [ ] `list_mask_groups` 엔드포인트 구현
- [ ] `get_mask_group` 엔드포인트 구현
- [ ] `delete_mask_group` 엔드포인트 구현
- [ ] `generate_signed_url` 엔드포인트 구현
- [ ] `complete_upload` 엔드포인트 구현
- [ ] `src/presentation/controllers/mask_controller.rs` 생성
- [ ] `list_masks` 엔드포인트 구현
- [ ] `get_mask` 엔드포인트 구현
- [ ] `mod.rs` 파일 업데이트

### 5.2 Swagger 문서화
- [ ] 모든 엔드포인트에 `utoipa::path` 어노테이션 추가
- [ ] `src/presentation/openapi.rs` 업데이트
- [ ] Swagger UI에서 새로운 엔드포인트 확인

### 5.3 라우팅 설정
- [ ] `src/presentation/routes/api_routes.rs` 업데이트
- [ ] 마스크 관련 라우트 추가
- [ ] `main.rs`에서 라우트 등록

## 🎯 Phase 6: 테스트 구현

### 6.1 단위 테스트
- [ ] Repository 테스트 작성
- [ ] Service 테스트 작성
- [ ] Use Case 테스트 작성
- [ ] Object Storage Service 테스트 작성

### 6.2 통합 테스트
- [ ] `tests/mask_group_controller_test.rs` 생성
- [ ] 마스크 그룹 생성 테스트
- [ ] 마스크 그룹 목록 조회 테스트
- [ ] Signed URL 생성 테스트
- [ ] 업로드 완료 테스트
- [ ] 마스크 그룹 삭제 테스트
- [ ] `tests/mask_controller_test.rs` 생성
- [ ] 마스크 목록 조회 테스트
- [ ] 마스크 상세 조회 테스트

### 6.3 성능 테스트
- [ ] 병렬 업로드 테스트
- [ ] 대용량 파일 업로드 테스트
- [ ] Signed URL 만료 처리 테스트

## 🎯 Phase 7: 설정 및 배포

### 7.1 환경 설정
- [ ] AWS 자격 증명 설정
- [ ] S3 버킷 생성 및 설정
- [ ] IAM 정책 설정
- [ ] CORS 설정
- [ ] 환경 변수 설정

### 7.2 배포 준비
- [ ] Docker 이미지 빌드 테스트
- [ ] 프로덕션 설정 파일 준비
- [ ] 배포 스크립트 작성
- [ ] 모니터링 설정

## 🎯 Phase 8: 문서화 및 검증

### 8.1 API 문서화
- [ ] Swagger 문서 완성
- [ ] API 사용 예제 작성
- [ ] 에러 코드 문서화
- [ ] 인증/인가 가이드 작성

### 8.2 운영 문서화
- [ ] 배포 가이드 작성
- [ ] 모니터링 가이드 작성
- [ ] 장애 대응 절차 작성
- [ ] 백업/복구 절차 작성

### 8.3 최종 검증
- [ ] 전체 플로우 테스트
- [ ] 성능 벤치마크
- [ ] 보안 검토
- [ ] 코드 리뷰

## 🚨 중요 체크포인트

### 보안 검증
- [ ] Signed URL TTL 설정 확인
- [ ] IAM 정책 제한 확인
- [ ] CORS 설정 확인
- [ ] 파일명 검증 확인
- [ ] HTTPS 통신 확인

### 성능 검증
- [ ] 100개 slice 병렬 업로드 1분 이내
- [ ] API 응답 시간 200ms 이하
- [ ] 메모리 사용량 최적화
- [ ] DB 쿼리 최적화

### 데이터 무결성 검증
- [ ] 파일 체크섬 검증
- [ ] DB 트랜잭션 처리
- [ ] 롤백 메커니즘 확인
- [ ] 오류 처리 확인

## 📊 진행률 추적

- **Phase 1**: 0/8 (0%)
- **Phase 2**: 0/6 (0%)
- **Phase 3**: 0/9 (0%)
- **Phase 4**: 0/4 (0%)
- **Phase 5**: 0/6 (0%)
- **Phase 6**: 0/8 (0%)
- **Phase 7**: 0/4 (0%)
- **Phase 8**: 0/6 (0%)

**전체 진행률**: 0/47 (0%)

## 🎯 다음 작업

1. **즉시 시작**: Phase 1 - 데이터베이스 스키마 구현
2. **우선순위**: 마이그레이션 파일 생성 및 실행
3. **예상 소요 시간**: 1-2일
4. **담당자**: Backend Developer

---

**작성일**: 2025-10-07  
**작성자**: AI Assistant  
**버전**: 1.0  
**마지막 업데이트**: 2025-10-07
