# Phase 1-2: `Result`와 `Option` 마스터하기

Rust는 다른 언어의 `null`이나 예외(exceptions) 대신, `Option<T>`와 `Result<T, E>`라는 두 가지 열거형(Enum)을 사용해 값이 없거나 작업이 실패할 수 있는 상황을 명시적으로 처리합니다. 이 프로젝트의 거의 모든 함수가 이들을 반환하므로, 완벽한 이해는 필수입니다.

## 1. `Option<T>`: 값이 있을 수도, 없을 수도 있음

`Option<T>`는 `null`의 문제를 해결합니다. 값이 있는지 없는지를 컴파일 타임에 강제로 확인하게 만듭니다.

```rust
_// 정의_
_pub enum Option<T> {_ 
    _Some(T), // 값이 존재함_ 
    _None,    // 값이 존재하지 않음 (null에 해당)_ 
_}_ 
```

### `Option` 다루기

#### `match` 사용 (가장 정석적인 방법)

```rust
fn print_if_exists(value: Option<i32>) {
    match value {
        Some(number) => println!("Value is: {}", number),
        None => println!("Value is None."),
    }
}
```

#### `if let` 사용 (`Some` 케이스만 처리할 때 유용)

```rust
if let Some(number) = some_option {
    println!("Found a number: {}", number);
}
```

#### 유용한 메소드들

-   `.is_some()` / `.is_none()`: 존재 여부를 `bool`로 반환.
-   `.unwrap()`: `Some`이면 값을 꺼내고, `None`이면 패닉(panic)을 일으킴. (테스트 외에는 사용 자제)
-   `.unwrap_or(default)`: `None`일 경우 `default` 값을 반환.
-   `.map(|v| ...)`: `Some`일 경우에만 안의 값을 변환.
-   `.and_then(|v| ...)`: `Some`일 경우 다른 `Option`을 반환하는 함수를 호출.

## 2. `Result<T, E>`: 작업이 성공할 수도, 실패할 수도 있음

`Result<T, E>`는 예외 처리의 문제를 해결합니다. 함수의 시그니처만 봐도 실패 가능성을 알 수 있습니다.

```rust
_// 정의_
_pub enum Result<T, E> {_ 
    _Ok(T),   // 성공. T는 성공 시의 값 타입_ 
    _Err(E),  // 실패. E는 실패 시의 에러 타입_ 
_}_ 
```

### `Result` 다루기

#### `match` 사용

```rust
use std::fs::File;

fn main() {
    let f = File::create("hello.txt");

    let file = match f {
        Ok(file) => file,
        Err(error) => {
            panic!("Problem opening the file: {:?}", error)
        }
    };
}
```

#### `?` 연산자 (가장 중요하고 많이 사용됨)

`?` 연산자는 `Result`를 다루는 코드를 매우 간결하게 만들어 줍니다. `Result`가 `Ok(T)`이면 `T` 값을 추출하고, `Err(E)`이면 현재 함수를 즉시 종료하고 해당 `Err(E)`를 반환합니다.

**`?` 연산자는 반환 타입이 `Result` 또는 `Option`인 함수 내에서만 사용할 수 있습니다.**

```rust
use std::io;
use std::io::Read;
use std::fs::File;

// `?` 연산자가 없을 때
fn read_username_from_file_long() -> Result<String, io::Error> {
    let f = File::open("username.txt");

    let mut f = match f {
        Ok(file) => file,
        Err(e) => return Err(e), // 에러를 수동으로 반환
    };

    let mut s = String::new();

    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(e) => Err(e), // 에러를 수동으로 반환
    }
}

// `?` 연산자를 사용했을 때
fn read_username_from_file_short() -> Result<String, io::Error> {
    let mut f = File::open("username.txt")?; // 에러 시 자동 반환
    let mut s = String::new();
    f.read_to_string(&mut s)?; // 에러 시 자동 반환
    Ok(s)
}

// 더 짧게 체이닝도 가능
fn read_username_from_file_shorter() -> Result<String, io::Error> {
    let mut s = String::new();
    File::open("username.txt")?.read_to_string(&mut s)?;
    Ok(s)
}
```

이 프로젝트의 모든 유스케이스와 리포지토리 함수들은 `Result`를 반환하며, `?` 연산자를 통해 에러를 `presentation` 계층까지 효과적으로 전파합니다. 이 패턴을 이해하는 것이 코드 독해의 핵심입니다.
