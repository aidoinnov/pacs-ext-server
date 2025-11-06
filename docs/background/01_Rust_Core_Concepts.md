# 01. 핵심 Rust 개념

이 프로젝트는 Rust 언어의 특정 기능들을 적극적으로 활용합니다. 다음 개념들은 코드를 이해하는 데 필수적입니다.

## 1. 소유권(Ownership), 대여(Borrowing), 생명주기(Lifetimes)

-   **소유권**: Rust에서 모든 값은 `소유자(owner)`라는 변수를 갖습니다. 한 번에 딱 하나의 소유자만 있을 수 있으며, 소유자가 스코프를 벗어나면 값은 자동으로 해제됩니다. 이로 인해 가비지 컬렉터 없이도 메모리 안전성을 보장합니다.
-   **대여**: 값의 소유권을 넘기지 않고 참조(`&` 또는 `&mut`)를 빌려주는 것을 의미합니다. 함수에 값을 전달할 때 불필요한 데이터 복사를 피하고 효율적으로 동작하게 합니다. 이 프로젝트의 거의 모든 함수 인자는 참조를 통해 값을 '대여'합니다.
-   **생명주기**: 참조가 유효한 스코프를 의미합니다. 대부분의 경우 컴파일러가 생명주기를 추론하지만, 복잡한 구조체나 함수에서는 명시적으로 생명주기(`'a`)를 지정해야 할 수 있습니다.

## 2. 에러 처리: `Result`와 `?` 연산자

Rust는 예외(Exception) 대신 `Result<T, E>` 열거형(Enum)을 사용해 에러를 처리합니다.

-   `Ok(T)`: 성공했을 때의 값을 담습니다.
-   `Err(E)`: 실패했을 때의 에러를 담습니다.

이 프로젝트의 거의 모든 함수는 `Result`를 반환하며, 이는 해당 함수가 실패할 수 있음을 명시적으로 보여줍니다.

`?` 연산자는 `Result`를 반환하는 함수를 간결하게 체이닝(chaining)하기 위해 사용됩니다.

```rust
// `?` 연산자가 없다면 코드는 이렇게 길어집니다.
fn old_way() -> Result<String, MyError> {
    let result = some_function_that_can_fail();
    let value = match result {
        Ok(v) => v,
        Err(e) => return Err(e), // 에러를 수동으로 반환
    };

    // ...
    Ok(value)
}

// `?` 연산자를 사용하면 매우 간결해집니다.
fn new_way() -> Result<String, MyError> {
    // some_function_that_can_fail()이 Err를 반환하면, `?`가 즉시 함수를 종료하고
    // 해당 Err를 반환합니다. 성공하면 Ok 안의 값을 남깁니다.
    let value = some_function_that_can_fail()?;

    // ...
    Ok(value)
}
```

**모든 `?` 연산자는 현재 함수의 반환 타입이 `Result`일 때만 사용할 수 있습니다.**

## 3. 비동기 프로그래밍: `async` / `await`

웹 서버는 동시에 수많은 요청을 효율적으로 처리해야 합니다. `async/await`는 I/O 작업(데이터베이스 조회, 외부 API 호출 등)이 완료되기를 기다리는 동안 다른 작업을 수행할 수 있게 하여, 적은 수의 스레드로 높은 동시성을 달성합니다.

-   `async fn`: 이 함수가 비동기 함수임을 나타냅니다. 호출 시 즉시 실행되지 않고, `Future`라는 "미래에 완료될 값"을 반환합니다.
-   `.await`: `Future`가 완료될 때까지 기다렸다가 결과 값을 추출합니다. `await`가 호출되면 현재 스레드는 다른 작업을 처리할 수 있습니다.

```rust
// 컨트롤러 함수는 비동기여야 합니다.
// DB 조회나 외부 서비스 호출이 있기 때문입니다.
pub async fn get_user(user_id: Uuid) -> Result<User, Error> {
    // `find_by_id`는 DB를 조회하는 비동기 함수입니다.
    // `.await`를 사용해 결과가 올 때까지 기다립니다.
    let user = user_repository.find_by_id(user_id).await?;
    Ok(user)
}
```

이 프로젝트의 모든 I/O 관련 작업(컨트롤러, 유스케이스, 리포지토리)은 `async/await`를 기반으로 작성되어 있습니다.

## 4. Trait (트레이트)

Trait은 특정 타입이 가질 수 있는 공유된 동작(메소드 집합)을 정의합니다. 다른 언어의 인터페이스(Interface)와 유사하며, 클린 아키텍처의 의존성 역전을 구현하는 핵심 도구입니다.

-   `domain` 계층에서는 Repository의 `Trait`을 정의합니다. (예: `pub trait IUserRepository { ... }`)
-   `infrastructure` 계층에서는 이 `Trait`을 실제 타입(구조체)에 대해 구현(`impl`)합니다. (예: `impl IUserRepository for UserRepositoryImpl { ... }`)

이를 통해 `application` 계층은 데이터베이스의 실제 구현 기술(`PostgreSQL` 등)에 대해 알 필요 없이, 추상적인 `IUserRepository` Trait에만 의존할 수 있습니다.
