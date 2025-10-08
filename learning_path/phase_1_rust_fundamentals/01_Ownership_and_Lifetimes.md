# Phase 1-1: 소유권, 대여, 생명주기

Rust의 가장 중요하고 독특한 기능입니다. 이 세 가지 개념은 가비지 컬렉터 없이 메모리 안전성을 보장하는 핵심 원리입니다.

## 1. 소유권 (Ownership)

-   **규칙 1**: Rust의 모든 값은 `소유자(owner)`라는 변수를 갖습니다.
-   **규칙 2**: 한 번에 딱 하나의 소유자만 있을 수 있습니다.
-   **규칙 3**: 소유자가 스코프(scope)를 벗어나면, 값은 메모리에서 해제(`drop`)됩니다.

### 값의 이동 (Move)

`String`, `Vec<T>`, `Box<T>` 등 스택(stack)에 포인터를, 힙(heap)에 실제 데이터를 저장하는 타입의 경우, 할당은 "이동(Move)"으로 처리됩니다.

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // s1의 소유권이 s2로 "이동"했습니다.

    // println!("{}", s1); // 컴파일 에러! s1은 더 이상 유효하지 않습니다.
}
```

### 값의 복사 (Copy)

`i32`, `f64`, `bool`, `char` 등 스택에 모든 데이터가 저장되는 타입은 `Copy` Trait을 구현합니다. 이 타입들은 할당 시 값이 "복사(Copy)"됩니다.

```rust
fn main() {
    let x = 5; // i32는 Copy Trait을 구현
    let y = x; // x의 값이 y로 복사됩니다.

    println!("x = {}, y = {}", x, y); // 문제 없음! x와 y 모두 유효합니다.
}
```

## 2. 대여 (Borrowing) & 참조 (References)

소유권을 넘기지 않고 값에 접근하고 싶을 때 "대여"를 사용합니다. `&` 기호를 사용해 참조(reference)를 만듭니다.

-   **불변 대여 (Immutable Borrow)**: `&T`
    -   하나의 값에 대해 여러 개의 불변 대여를 동시에 만들 수 있습니다.
    -   불변 대여가 활성화된 동안에는 값을 변경할 수 없습니다.
-   **가변 대여 (Mutable Borrow)**: `&mut T`
    -   하나의 값에 대해 단 하나의 가변 대여만 만들 수 있습니다.
    -   가변 대여가 활성화된 동안에는 다른 어떤 대여(불변, 가변 모두)도 만들 수 없습니다.

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s; // ok
    let r2 = &s; // ok
    println!("{} and {}", r1, r2); // r1, r2는 여기서 마지막으로 사용됨

    let r3 = &mut s; // ok! r1, r2의 스코프가 끝났기 때문
    r3.push_str(", world");
    println!("{}", r3);
}
```

## 3. 생명주기 (Lifetimes)

생명주기는 참조가 유효한 스코프를 나타내는 컴파일 타임 개념입니다. "댕글링 포인터(dangling pointer)", 즉 유효하지 않은 메모리를 가리키는 참조가 생기는 것을 막아줍니다.

대부분의 경우 컴파일러가 생명주기를 추론하지만, 함수나 구조체에서 참조를 다룰 때 명시적으로 지정해야 할 수 있습니다. `'a` (tick a)와 같은 형태로 표현합니다.

아래 함수는 `x`와 `y` 중 더 긴 문자열의 참조를 반환합니다. 반환된 참조가 `x`와 `y` 중 더 짧은 생명주기보다 오래 살아남지 못하도록 컴파일러에게 알려줘야 합니다.

```rust
// 'a는 x, y, 그리고 반환값의 생명주기가 모두 같거나
// 가장 짧은 것과 연결되어야 함을 명시합니다.
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

fn main() {
    let string1 = String::from("abcd");
    let result;
    {
        let string2 = String::from("xyz");
        // result의 생명주기는 string2의 스코프에 묶입니다.
        result = longest(string1.as_str(), string2.as_str());
        println!("The longest string is {}", result);
    }
    // println!("The longest string is {}", result); // 컴파일 에러! result가 참조하던 string2가 해제됨
}
```

**이 프로젝트에서 이 개념들은 함수의 인자를 전달하고, 구조체 내에서 데이터를 참조하며, 여러 스레드 간에 데이터를 안전하게 공유하는 모든 곳에서 사용됩니다.**
