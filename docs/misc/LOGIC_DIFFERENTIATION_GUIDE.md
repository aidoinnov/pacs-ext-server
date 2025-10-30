### "이게 도메인이야, 비즈니스야?" 구분을 위한 리트머스 테스트

어떤 코드 조각을 봤을 때, 다음 두 가지 질문을 던져보면 90% 이상 구분할 수 있습니다.

**질문 1: "이 규칙은 우리 웹사이트가 망하고, 회사가 다른 사업(예: 모바일 게임)을 해도 '사용자'나 '프로젝트'라는 개념 자체에 변함없이 적용되는 규칙인가?"
**
*   **YES** 라면, 그것은 **도메인 로직**입니다.
*   **NO** 라면, 아래 질문 2로 넘어갑니다.

**질문 2: "이 규칙은 우리 웹사이트가 '사용자'를 가지고 무언가를 '처리'하는 특정 과정(프로세스)에 대한 규칙인가?"
**
*   **YES** 라면, 그것은 **비즈니스 로직 (또는 애플리케이션 로직)** 입니다.

---

### 실제 코드로 보는 극단적인 비교

`pacs-server`의 "사용자"를 예로 들어보겠습니다.

#### 1. 도메인 로직: 개념의 '본질' (질문 1에 YES)

이 코드는 `src/domain/entities/user.rs`에 있어야 합니다.

```rust
// src/domain/entities/user.rs

pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    // 비밀번호 해시, 생성일 등...
}

impl User {
    // 'User'라는 개념을 생성하는 함수
    pub fn new(id: Uuid, email: String, name: String) -> Result<Self, &'static str> {
        // --- 이것이 바로 '도메인 규칙' ---
        if name.is_empty() {
            return Err("사용자 이름은 비어있을 수 없습니다."); // 규칙 1
        }
        if !email.contains('@') {
            return Err("유효하지 않은 이메일 형식입니다."); // 규칙 2
        }
        // ---

        Ok(Self { id, email, name })
    }

    pub fn change_name(&mut self, new_name: String) -> Result<(), &'static str> {
        // --- 이것도 '도메인 규칙' ---
        if new_name.is_empty() {
            return Err("사용자 이름은 비어있을 수 없습니다.");
        }
        // ---
        self.name = new_name;
        Ok(())
    }
}
```

**왜 이것이 도메인 로직인가?**
*   "사용자 이름은 비어있을 수 없다"는 규칙은 우리 회사가 웹사이트를 만들든, ERP를 만들든, "사용자"라는 데이터를 다룬다면 **변하지 않을 본질적인 정책**입니다.
*   이 `User` 구조체와 `impl` 블록은 **데이터베이스, 웹 프레임워크, 이메일 전송 라이브러리의 존재를 전혀 모릅니다.** 오직 `User` 그 자체의 완전성(integrity)에만 관심 있습니다.

#### 2. 비즈니스 로직: 개념의 '활용' (질문 2에 YES)

이 코드는 `src/application/use_cases/user_use_case.rs`에 있어야 합니다.

```rust
// src/application/use_cases/user_use_case.rs

pub struct UserUseCase {
    // 의존성: DB와 통신할 레포지토리, 이메일 보낼 서비스 등
    user_repo: Arc<dyn UserRepository>,
    email_service: Arc<dyn EmailService>,
}

impl UserUseCase {
    // '회원가입'이라는 비즈니스 프로세스
    pub async fn register_user(&self, dto: CreateUserDto) -> Result<UserDto, AppError> {
        // --- 이것이 바로 '비즈니스 규칙/프로세스' ---

        // 프로세스 1: 이메일이 이미 존재하는가? (DB 확인 필요)
        if self.user_repo.find_by_email(&dto.email).await?.is_some() {
            return Err(AppError::EmailAlreadyExists);
        }

        // 프로세스 2: 비밀번호를 암호화한다. (보안 정책)
        let hashed_password = hash_password(&dto.password);

        // 프로세스 3: '도메인 객체'를 생성한다. (위에서 정의한 도메인 규칙이 여기서 검증됨)
        let new_user = User::new(Uuid::new_v4(), dto.email, dto.name, hashed_password)?;

        // 프로세스 4: 생성된 도메인 객체를 DB에 저장한다.
        self.user_repo.save(&new_user).await?;

        // 프로세스 5: 환영 이메일을 보낸다. (외부 서비스 연동)
        self.email_service.send_welcome_email(&new_user.email).await?;

        // 프로세스 6: 응답에 필요한 데이터만 가공해서(DTO) 반환한다.
        Ok(UserDto::from(new_user))

        // ---
    }
}
```

**왜 이것이 비즈니스 로직인가?**
*   이 `register_user` 함수는 "사용자"의 본질이 아니라, **"우리 웹사이트에서 사용자를 가입시키는 절차"** 를 정의합니다.
*   만약 비즈니스 요구사항이 "가입 시, 이메일 대신 SMS 인증을 하고, 환영 쿠폰을 지급한다"로 바뀌면, `User`의 본질은 그대로 둔 채 이 `register_user` 함수의 내용만 바뀌게 됩니다.
*   이 코드는 **데이터베이스(`user_repo`)와 이메일 서비스(`email_service`)의 존재를 명확히 알고 있으며, 이들을 조율(Orchestration)하여 하나의 기능을 완성**합니다.

---

### 그래서, 왜 이렇게까지 나누나요?

**"안정적인 것과 불안정한 것을 분리하기 위해서"** 입니다.

*   **안정적인 것 (Stable):** `src/domain`의 `User`와 그 본질적인 규칙. 이것은 비즈니스의 근간이므로 거의 변하지 않습니다.
*   **불안정한 것 (Volatile):** `src/application`의 `회원가입 절차`. 이것은 마케팅, 정책, 기술의 변화에 따라 수시로 바뀔 수 있습니다.

**안정적인 `도메인` 코드를, 변화무쌍한 `비즈니스` 코드와 외부 기술(`인프라`)로부터 철저히 보호하는 것.** 이것이 바로 이 아키텍처의 핵심 가치입니다.
