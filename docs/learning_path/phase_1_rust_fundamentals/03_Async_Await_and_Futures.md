# Phase 1-3: `async/await`와 `Future`

웹 서버와 같이 I/O(입출력) 작업이 많은 애플리케이션에서 `async/await`는 높은 성능과 동시성을 달성하기 위한 필수적인 도구입니다. Rust의 비동기 모델은 "zero-cost abstraction"을 지향하여, 성능 저하 없이 높은 수준의 추상화를 제공합니다.

## 1. 동기(Synchronous) vs 비동기(Asynchronous)

-   **동기**: 하나의 작업이 끝날 때까지 다음 작업은 "대기(blocking)"합니다. 데이터베이스에서 1초가 걸리는 쿼리를 100명이 동시에 요청하면, 이론적으로 마지막 사람은 100초를 기다려야 합니다. (실제로는 스레드 풀로 처리하지만, 스레드는 비싼 자원입니다.)

-   **비동기**: 작업이 완료되기를 기다리는 동안(예: DB 응답 대기) 스레드를 다른 작업에 양보합니다. 작업이 완료되면(예: DB 응답 도착) 다시 이어서 실행합니다. 이를 통해 적은 수의 스레드로 수많은 동시 요청을 효율적으로 처리할 수 있습니다.

## 2. `async/await` 구문

### `async fn`

함수 시그니처 앞에 `async` 키워드를 붙이면 해당 함수는 비동기 함수가 됩니다. 이 함수는 호출 시 즉시 코드를 실행하는 대신, `Future`라는 타입을 반환합니다.

```rust
async fn do_something() -> String {
    // ... 어떤 작업 ...
    String::from("done")
}

fn main() {
    let future = do_something(); // 코드가 실행되지 않음. `Future`만 생성됨.
    // `future`를 실행기에 넘겨야 실제 코드가 실행됨.
}
```

### `Future` Trait

`Future`는 "미래의 어느 시점에 완료될 값"을 나타내는 Trait입니다. `poll`이라는 메소드를 가지며, 비동기 런타임은 이 `poll` 메소드를 반복적으로 호출하여 `Future`가 완료되었는지 확인합니다.

### `.await`

`Future`를 실행하고 완료될 때까지 기다리는 역할을 합니다. `.await`가 호출되면, `Future`가 아직 완료되지 않았다면 현재 스레드의 제어권을 비동기 런타임에게 넘겨 다른 작업을 처리하도록 합니다. `Future`가 완료되면 런타임은 중단된 지점부터 다시 실행을 시작합니다.

`.await`는 `async` 함수 또는 `async` 블록 내에서만 사용할 수 있습니다.

```rust
async fn read_from_db() -> String { /* ... DB 조회 ... */ "db_data".to_string() }
async fn read_from_api() -> String { /* ... API 호출 ... */ "api_data".to_string() }

async fn my_use_case() {
    println!("데이터 조회를 시작합니다.");

    // .await를 만나면, read_from_db()가 완료될 때까지 기다리면서
    // 다른 async 함수(예: 다른 사용자의 요청)를 처리할 수 있습니다.
    let db_data = read_from_db().await;
    println!("DB 데이터: {}", db_data);

    let api_data = read_from_api().await;
    println!("API 데이터: {}", api_data);
}
```

## 3. 비동기 런타임 (Async Runtime)

`async/await` 자체는 언어 기능일 뿐, 실제로 `Future`를 실행하고 관리하는 주체는 **비동기 런타임**입니다. `tokio`와 `async-std`가 가장 대표적인 런타임입니다.

이 프로젝트는 `actix-web`을 사용하므로, 내부적으로 `tokio` 런타임을 사용합니다. `main` 함수 위에 `#[actix_web::main]` 또는 `#[tokio::main]` 어노테이션을 붙여 런타임을 시작합니다.

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // my_use_case()는 여기서 .await를 통해 실행될 수 있습니다.
    my_use_case().await;

    HttpServer::new(...) // Actix-web 서버도 비동기적으로 동작합니다.
        .run()
        .await
}
```

## 4. 동시 실행 (`join!`, `select!`)

`futures` 라이브러리의 `join!` 매크로를 사용하면 여러 `Future`를 동시에 실행하고 모든 결과가 준비될 때까지 기다릴 수 있습니다.

```rust
use futures::join;

async fn concurrent_use_case() {
    println!("DB와 API 조회를 동시에 시작합니다.");

    // 두 함수는 동시에 실행됩니다. 전체 소요 시간은 둘 중 더 오래 걸리는 시간에 맞춰집니다.
    let (db_result, api_result) = join!(read_from_db(), read_from_api());

    println!("DB 데이터: {}", db_result);
    println!("API 데이터: {}", api_result);
}
```

이 프로젝트의 모든 I/O 바운드 작업(컨트롤러, 유스케이스, 리포지토리)은 `async/await`를 기반으로 작성되어 있어, 높은 동시성과 처리량을 보장합니다.
