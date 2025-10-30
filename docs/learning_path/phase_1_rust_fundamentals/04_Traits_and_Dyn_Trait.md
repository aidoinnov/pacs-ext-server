# Phase 1-4: Trait과 `dyn Trait`

Trait은 Rust에서 코드의 재사용성과 추상화를 달성하는 가장 중요한 도구입니다. 다른 언어의 인터페이스(Interface)와 유사한 역할을 하며, 특히 클린 아키텍처의 의존성 역전 원칙을 구현하는 데 핵심적으로 사용됩니다.

## 1. Trait: 공유된 동작 정의

Trait은 어떤 타입이 반드시 구현해야 하는 메소드의 집합을 정의합니다.

```rust
// `Summary` Trait은 `summarize` 메소드를 가져야 함을 정의
pub trait Summary {
    fn summarize(&self) -> String;
}

pub struct NewsArticle {
    pub headline: String,
    pub author: String,
}

pub struct Tweet {
    pub username: String,
    pub content: String,
}

// NewsArticle에 대해 Summary Trait을 구현
impl Summary for NewsArticle {
    fn summarize(&self) -> String {
        format!("{}, by {}", self.headline, self.author)
    }
}

// Tweet에 대해 Summary Trait을 구현
impl Summary for Tweet {
    fn summarize(&self) -> String {
        format!("@{}: {}", self.username, self.content)
    }
}
```

이제 `Summary` Trait을 구현한 어떤 타입이든 인자로 받는 함수를 작성할 수 있습니다. 이를 **제네릭(Generics)**을 이용한 **정적 디스패치(Static Dispatch)**라고 합니다.

```rust
// item은 Summary Trait을 구현한 어떤 타입이든 될 수 있습니다.
pub fn notify<T: Summary>(item: &T) {
    println!("Breaking news! {}", item.summarize());
}

fn main() {
    let tweet = Tweet { ... };
    let article = NewsArticle { ... };

    notify(&tweet);   // 동작함
    notify(&article); // 동작함
}
```

## 2. Trait 객체 (`dyn Trait`): 동적 디스패치

때로는 런타임에 여러 다른 타입의 객체들을 동일한 컬렉션에 담고 싶을 때가 있습니다. 예를 들어, `Vec`에 `NewsArticle`과 `Tweet`을 함께 담는 경우입니다. 제네릭은 컴파일 타임에 타입이 결정되어야 하므로 이런 경우에 사용할 수 없습니다.

이때 **Trait 객체**를 사용합니다. Trait 객체는 `&dyn Trait` 또는 `Box<dyn Trait>` 형태로 표현되며, "이것은 `Trait`을 구현하는 어떤 타입의 인스턴스다"라는 것을 의미합니다. 이를 **동적 디스패치(Dynamic Dispatch)**라고 합니다.

```rust
fn main() {
    let tweet = Tweet { ... };
    let article = NewsArticle { ... };

    // `Box<dyn Summary>`를 사용하여 다른 타입들을 동일한 벡터에 담음
    let items: Vec<Box<dyn Summary>> = vec![
        Box::new(tweet),
        Box::new(article),
    ];

    for item in items {
        // 런타임에 실제 타입을 확인하여 적절한 summarize 메소드를 호출
        println!("New item: {}", item.summarize());
    }
}
```

## 3. 프로젝트에서의 활용: 의존성 주입

이 프로젝트의 리포지토리 패턴은 Trait 객체를 핵심적으로 사용합니다.

1.  **`domain` 계층**: 리포지토리의 행위를 `IUserRepository` Trait으로 정의합니다.

    ```rust
    // in domain/repositories/user_repository.rs
    #[async_trait::async_trait]
    pub trait IUserRepository: Send + Sync { // Send + Sync는 멀티스레드 환경에서 안전하게 공유하기 위해 필요
        async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Error>;
    }
    ```

2.  **`application` 계층**: 유스케이스는 구체적인 구현체가 아닌, `Arc<dyn IUserRepository>` Trait 객체에 의존합니다.
    -   `Box<T>`: 단일 소유권을 갖는 스마트 포인터
    -   `Rc<T>`: 단일 스레드 내에서 여러 소유권을 갖는 참조 카운팅 스마트 포인터
    -   `Arc<T>`: 멀티스레드 환경에서 안전하게 여러 소유권을 갖는 참조 카운팅 스마트 포인터 (`Arc`는 Atomic Reference Counted를 의미)

    웹 서버는 멀티스레드 환경이므로, 여러 스레드에서 리포지토리 인스턴스를 공유하기 위해 `Arc`를 사용합니다.

    ```rust
    // in application/use_cases/user_use_case.rs
    use std::sync::Arc;

    pub struct UserUseCase {
        // UserRepositoryImpl 타입이 아닌, IUserRepository Trait 객체에 의존
        pub user_repo: Arc<dyn IUserRepository>,
    }
    ```

3.  **`infrastructure` 계층**: `UserRepositoryImpl` 구조체에 대해 `IUserRepository` Trait을 구현합니다.

4.  **`main.rs`**: `UserRepositoryImpl`의 인스턴스를 생성하고, `Arc::new()`로 감싼 뒤 `UserUseCase`에 주입합니다. 이때 `Arc<UserRepositoryImpl>`이 `Arc<dyn IUserRepository>` 타입으로 변환(type coercion)됩니다.

이 구조 덕분에 `UserUseCase`는 데이터베이스가 PostgreSQL인지, MySQL인지, 혹은 테스트를 위한 가짜 객체인지 전혀 알 필요 없이 자신의 비즈니스 로직에만 집중할 수 있습니다.
