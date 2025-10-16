# AI Assistant Rules & Configuration Guide

## 📋 목차
1. [개요](#개요)
2. [.cursorrules 파일이란?](#cursorrules-파일이란)
3. [파일 구조 및 위치](#파일-구조-및-위치)
4. [규칙 작성 가이드](#규칙-작성-가이드)
5. [프로젝트별 규칙 관리](#프로젝트별-규칙-관리)
6. [AI 동작 방식](#ai-동작-방식)
7. [실제 적용 예시](#실제-적용-예시)
8. [모범 사례](#모범-사례)
9. [문제 해결](#문제-해결)
10. [참고 자료](#참고-자료)

---

## 개요

### AI Assistant Rules란?
AI Assistant Rules는 Cursor IDE에서 AI 어시스턴트가 프로젝트의 코딩 스타일, 아키텍처 패턴, 그리고 개발 규칙을 자동으로 학습하고 적용할 수 있도록 하는 설정 파일입니다.

### 주요 장점
- **자동 학습**: AI가 프로젝트의 패턴을 자동으로 파악
- **일관성 유지**: 팀 전체가 동일한 코딩 스타일 유지
- **생산성 향상**: 매번 규칙을 설명할 필요 없음
- **품질 보장**: 자동으로 모범 사례 적용

---

## .cursorrules 파일이란?

### 정의
`.cursorrules`는 Cursor IDE가 자동으로 인식하는 설정 파일로, AI 어시스턴트에게 프로젝트별 규칙과 가이드라인을 제공합니다.

### 동작 원리
1. **자동 인식**: Cursor IDE가 프로젝트 루트의 `.cursorrules` 파일을 자동으로 읽음
2. **컨텍스트 제공**: AI 어시스턴트에게 프로젝트 규칙을 컨텍스트로 전달
3. **일관적 적용**: 모든 AI 상호작용에서 동일한 규칙 적용

### 표준 위치
```
프로젝트_루트/
├── .cursorrules          # AI 규칙 파일
├── src/
├── docs/
└── ...
```

---

## 파일 구조 및 위치

### 1. 프로젝트 루트 위치 (권장)
```
/Users/aido/Code/pacs-ext-server/
├── .cursorrules          # ✅ 권장 위치
├── pacs-server/
├── docs/
└── README.md
```

### 2. 하위 프로젝트 위치
```
/Users/aido/Code/pacs-ext-server/
├── pacs-server/
│   ├── .cursorrules      # pacs-server 전용 규칙
│   ├── src/
│   └── Cargo.toml
├── simple-rust-server/
│   ├── .cursorrules      # simple-rust-server 전용 규칙
│   └── src/
└── simple-go-server/
    ├── .cursorrules      # simple-go-server 전용 규칙
    └── main.go
```

### 3. 파일명 변형
- `.cursorrules` (표준)
- `.cursor-rules`
- `cursor.rules`
- `AI_RULES.md` (마크다운 형식)

---

## 규칙 작성 가이드

### 1. 기본 구조

```markdown
# 프로젝트명 AI Assistant Rules

## 🏗️ Architecture & Design Patterns
[아키텍처 규칙]

## 🦀 Language-Specific Rules
[언어별 규칙]

## 📡 API Design
[API 설계 규칙]

## 🗄️ Database & Data Access
[데이터베이스 규칙]

## 🔐 Security & Authentication
[보안 규칙]

## 🧪 Testing
[테스트 규칙]

## 📁 File Organization
[파일 구조 규칙]

## 🚨 Anti-Patterns
[피해야 할 패턴들]
```

### 2. 규칙 작성 원칙

#### 명확성
```markdown
# ❌ 나쁜 예
- Use good practices

# ✅ 좋은 예
- Always use Result<T, E> for error handling, never panic
- Use async/await for I/O operations
- Implement proper validation for all inputs
```

#### 구체성
```markdown
# ❌ 나쁜 예
- Follow naming conventions

# ✅ 좋은 예
- Functions & Variables: snake_case
- Structs & Enums: PascalCase
- Constants: SCREAMING_SNAKE_CASE
- Files: snake_case.rs
```

#### 예시 포함
```markdown
# ✅ 좋은 예
## Error Handling
- Use Result<T, E> for error handling
- Example:
  ```rust
  fn process_data(input: &str) -> Result<String, MyError> {
      if input.is_empty() {
          return Err(MyError::InvalidInput);
      }
      Ok(input.to_uppercase())
  }
  ```
```

### 3. 카테고리별 규칙 작성

#### 아키텍처 규칙
```markdown
## 🏗️ Architecture & Design Patterns

### Clean Architecture
- Follow the 4-layer Clean Architecture pattern
- Domain → Application → Infrastructure → Presentation
- Dependencies should point inward (toward Domain)
- Use Repository pattern for data access abstraction

### Layer Responsibilities
- **Domain**: Entities, Repository interfaces, Business rules
- **Application**: Use Cases, DTOs, Service implementations
- **Infrastructure**: Database implementations, External services
- **Presentation**: Controllers, HTTP handlers, API documentation
```

#### 언어별 규칙
```markdown
## 🦀 Rust Style & Conventions

### Naming Conventions
- Functions & Variables: snake_case
- Structs & Enums: PascalCase
- Constants: SCREAMING_SNAKE_CASE

### Code Style
- Always use Result<T, E> for error handling
- Prefer async/await over manual futures
- Use Option<T> for nullable values
- Implement From trait for conversions
```

#### API 설계 규칙
```markdown
## 📡 API Design

### RESTful API Principles
- Use proper HTTP methods (GET, POST, PUT, DELETE)
- Use appropriate HTTP status codes
- Return consistent JSON response format
- Implement proper pagination for list endpoints

### OpenAPI Documentation
- All endpoints must be documented with OpenAPI
- Use #[derive(ToSchema)] for DTOs
- Provide examples and descriptions
```

---

## 프로젝트별 규칙 관리

### 1. 단일 프로젝트 규칙
```markdown
# PACS Server AI Rules

## Rust 프로젝트 전용 규칙
- Use actix-web for web framework
- Use sqlx for database operations
- Use serde for serialization
- Use utoipa for OpenAPI documentation
```

### 2. 멀티 프로젝트 규칙
```markdown
# Multi-Project AI Rules

## 공통 규칙
- Use consistent naming conventions
- Follow Clean Architecture
- Implement proper error handling

## 프로젝트별 규칙
### Rust Projects
- Use Cargo for dependency management
- Follow Rust naming conventions

### Go Projects
- Use go mod for dependency management
- Follow Go naming conventions
```

### 3. 팀 규칙 vs 개인 규칙
```markdown
# 팀 규칙 (.cursorrules)
- 공식적인 프로젝트 규칙
- 팀 전체가 따르는 표준
- 버전 관리에 포함

# 개인 규칙 (.cursorrules.local)
- 개인적인 선호도
- 팀 규칙과 충돌하지 않는 범위
- 버전 관리에서 제외
```

---

## AI 동작 방식

### 1. 규칙 로딩 과정
```
1. Cursor IDE 시작
2. 프로젝트 루트에서 .cursorrules 파일 검색
3. 파일 내용을 AI 컨텍스트에 로드
4. 모든 AI 상호작용에서 규칙 적용
```

### 2. 규칙 적용 우선순위
```
1. .cursorrules 파일의 규칙 (최우선)
2. 프로젝트 기존 코드 패턴
3. 일반적인 모범 사례
4. AI의 기본 지식
```

### 3. 동적 규칙 업데이트
```
1. .cursorrules 파일 수정
2. Cursor IDE 재시작 또는 새 세션 시작
3. 새로운 규칙이 AI 컨텍스트에 반영
```

---

## 실제 적용 예시

### 1. PACS Server 프로젝트 규칙

```markdown
# PACS Server AI Rules

## 🏗️ Architecture
- Follow Clean Architecture pattern
- Domain → Application → Infrastructure → Presentation
- Use Repository pattern for data access

## 🦀 Rust Style
- Use snake_case for functions and variables
- Use PascalCase for structs and enums
- Always use Result<T, E> for error handling
- Prefer async/await over manual futures

## 📡 API Design
- All endpoints must be documented with OpenAPI
- Use proper HTTP status codes
- Return consistent JSON responses
- Use DTOs for all API communication

## 🗄️ Database
- Use SQLx for database operations
- Always use transactions for multi-step operations
- Use proper error handling for database errors

## 🔐 Security
- Use Keycloak for authentication
- Implement proper JWT token validation
- Use middleware for authentication checks

## 🧪 Testing
- Write unit tests for business logic
- Write integration tests for API endpoints
- Use test database for integration tests
```

### 2. 규칙 적용 결과

#### Before (규칙 없음)
```rust
// AI가 제안한 코드
fn get_user(id: i32) -> User {
    let user = database.get_user(id).unwrap();
    user
}
```

#### After (규칙 적용)
```rust
// AI가 제안한 코드 (규칙 적용)
async fn get_user(
    id: i32,
    user_repo: &dyn UserRepository
) -> Result<UserResponse, AppError> {
    let user = user_repo.find_by_id(id).await?;
    Ok(UserResponse::from(user))
}
```

---

## 모범 사례

### 1. 규칙 작성 모범 사례

#### ✅ Do
- **구체적으로 작성**: "Use Result<T, E>" vs "Handle errors properly"
- **예시 포함**: 코드 예시로 규칙 설명
- **카테고리별 정리**: 관련 규칙들을 그룹화
- **정기적 업데이트**: 프로젝트 발전에 따라 규칙 갱신
- **팀 검토**: 팀원들과 규칙 검토 및 합의

#### ❌ Don't
- **너무 추상적**: "Write good code"
- **모순된 규칙**: 서로 충돌하는 규칙들
- **과도한 규칙**: 모든 것을 규칙화하려 하지 말기
- **무시되는 규칙**: 실제로 적용되지 않는 규칙들

### 2. 파일 관리 모범 사례

#### 버전 관리
```bash
# .cursorrules 파일을 Git에 포함
git add .cursorrules
git commit -m "Add AI assistant rules for project consistency"
```

#### 팀 공유
```bash
# 팀원들과 규칙 공유
git pull origin main  # 최신 규칙 가져오기
```

#### 백업
```bash
# 규칙 파일 백업
cp .cursorrules .cursorrules.backup
```

### 3. 규칙 테스트

#### 규칙 검증
```markdown
# 규칙 테스트 방법
1. 새로운 기능 개발 시 AI에게 코드 작성 요청
2. AI가 제안한 코드가 규칙을 따르는지 확인
3. 규칙을 따르지 않으면 규칙 수정 또는 명확화
```

#### 피드백 루프
```
규칙 작성 → AI 적용 → 결과 검토 → 규칙 개선 → 재적용
```

---

## 문제 해결

### 1. 일반적인 문제들

#### 규칙이 적용되지 않는 경우
```bash
# 해결 방법
1. .cursorrules 파일 위치 확인
2. Cursor IDE 재시작
3. 파일 형식 확인 (Markdown 형식)
4. 규칙 문법 확인
```

#### 규칙이 너무 엄격한 경우
```markdown
# 해결 방법
- 규칙을 더 유연하게 수정
- 예외 상황에 대한 가이드라인 추가
- "가능한 한" 같은 표현 사용
```

#### 규칙이 충돌하는 경우
```markdown
# 해결 방법
- 우선순위 명시
- 구체적인 상황별 규칙 작성
- 팀과 충돌 규칙 논의
```

### 2. 디버깅 방법

#### 규칙 확인
```bash
# .cursorrules 파일 내용 확인
cat .cursorrules

# 파일 위치 확인
find . -name ".cursorrules" -type f
```

#### AI 응답 검증
```
1. AI에게 명확한 규칙 준수 요청
2. 제안된 코드가 규칙을 따르는지 확인
3. 규칙을 따르지 않으면 구체적으로 지적
```

---

## 참고 자료

### 공식 문서
- [Cursor IDE Documentation](https://cursor.sh/docs)
- [AI Assistant Rules](https://cursor.sh/docs/ai-rules)

### 관련 도구
- **Cursor IDE**: AI-powered code editor
- **Git**: 버전 관리
- **Markdown**: 규칙 파일 형식

### 프로젝트 내 관련 파일
- `.cursorrules`: AI 규칙 파일
- `docs/technical/`: 기술 문서
- `docs/study/`: 학습 가이드

### 추가 학습 자료
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Rust Style Guide](https://doc.rust-lang.org/book/appendix-02-grammar.html)
- [RESTful API Design](https://restfulapi.net/)

---

## 마무리

`.cursorrules` 파일을 통해 AI 어시스턴트가 프로젝트의 규칙과 패턴을 자동으로 학습하고 적용할 수 있습니다. 이를 통해 일관성 있는 코드 품질을 유지하고, 팀의 생산성을 향상시킬 수 있습니다.

**핵심 포인트:**
1. **명확한 규칙**: 구체적이고 실행 가능한 규칙 작성
2. **지속적 개선**: 프로젝트 발전에 따라 규칙 업데이트
3. **팀 협업**: 팀원들과 규칙 공유 및 검토
4. **실용적 접근**: 과도한 규칙보다는 실용적인 가이드라인

이 가이드를 참고하여 프로젝트에 맞는 AI 규칙을 작성하고, 더 나은 개발 경험을 만들어보세요!
