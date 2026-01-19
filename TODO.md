# TODO List

## 🔒 Security & Authorization

### TimePoint Annotation API 권한 체크 추가
**Priority:** High  
**Status:** Pending

**문제:**
- `GET /api/timepoints/{timepoint_id}/annotations` API에 권한 체크가 없음
- 누구나 TimePoint의 모든 annotation을 조회할 수 있는 보안 취약점

**현재 구현:**
```rust
// pacs-server/src/presentation/controllers/timepoint_controller.rs
pub async fn get_annotations_by_timepoint<A: AnnotationRepository + 'static, S: ...>(
    annotation_repository: web::Data<A>,
    signed_url_service: web::Data<S>,
    timepoint_id: web::Path<i32>,
) -> impl Responder {
    // ❌ 권한 체크 없음!
    match annotation_repository.find_by_timepoint(*timepoint_id).await {
```

**해결 방안:**
1. **UseCase 레이어 추가**
   - `AnnotationUseCase`에 `get_annotations_by_timepoint_with_permission` 메서드 추가
   
2. **권한 체크 로직**
   - TimePoint → Subject → Project 확인
   - 사용자가 해당 Project의 멤버인지 확인
   - `READ_ALL` 권한 확인
     - 권한 있음: 모든 annotation 반환
     - 권한 없음: 본인 annotation만 반환

3. **일관성 유지**
   - 일반 annotation API (`GET /api/annotations?project_id=...`)와 동일한 권한 정책 적용

**참고:**
- 일반 Annotation API는 `get_annotations_by_project_with_permission` 사용
- 파일: `pacs-server/src/application/use_cases/annotation_use_case.rs:996-1022`

---

## 📝 Notes
- 2026-01-19: TimePoint Annotation API 권한 체크 이슈 발견

