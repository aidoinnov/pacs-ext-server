# backend-api-service-constitution

본 스킬은 백엔드에서 **API + Service + Domain + Test**를 생성할 때,
단순 구현이 아닌 **합리적인 설계 선택, 테스트 기반 구현, 검증 가능한 완료**를 강제한다.

---

## 0. 목적

- “이걸 조회/처리할 테이블과 API가 필요하다”는 요구를 입력으로 받아
  - 최적의 설계안 도출
  - DDD 구조에 맞는 구현
  - 단계별 테스트 수행
  - TODO 기반 완료 검증
  을 수행한다.
- 기능 변경 없는 신규 기능 또는 독립 기능 추가를 기본 대상으로 한다.

---

## 1. 입력 규약

입력은 불완전해도 된다.  
필요 시 **Planning 단계에서 질문 1회만 허용**한다.

입력 예:
- 도메인 설명 (자연어 가능)
- 필요한 조회 / 명령
- 참고할 기존 API 또는 테스트
- 기존 Python E2E 테스트 위치

---

## 2. Planning Phase (필수)

### 2-1. 설계안 생성 및 점수화

최소 2개의 설계안을 생성하고, 아래 기준으로 점수화한다.

| 기준 | 설명 | 점수 |
|----|----|----|
| DDD 적합성 | Aggregate / Entity / Service 분리 | /10 |
| API 충돌 가능성 | 기존 API scope/path 충돌 여부 | /10 |
| 테스트 용이성 | 단위/통합/E2E 분리 가능성 | /10 |
| 확장성 | 정책·필드 추가 영향도 | /10 |
| 구현 복잡도 | 과도한 추상화 여부 | /10 |

총점이 가장 높은 안을 **최종 설계안**으로 선택한다.

---

### 2-2. Planning File 생성 (작업 시작 전 필수)

모든 작업은 Planning File 생성으로 시작한다.

```

docs/plans/plan_<feature_name>.md

````

---

### 2-3. Planning File 필수 구성

```md
## 1. 작업 개요
- 목적
- 대상 도메인
- 영향 범위

## 2. 최종 설계안 요약
- 선택된 설계안
- 점수 산정 결과

## 3. TODO
- [ ] Domain Entity 정의
- [ ] Repository Trait 정의
- [ ] Repository 단위 테스트 통과
- [ ] Application Service 구현
- [ ] Service 통합 테스트 통과
- [ ] REST API Path 확정 및 충돌 검증
- [ ] Controller 구현
- [ ] API 단위 테스트 통과
- [ ] Python E2E 테스트 작성
- [ ] 전체 테스트 통과
````

---

## 3. API 설계 원칙 (Rust 충돌 회피)

* REST는 **명사 기반**
* 행동을 path에 포함하지 않는다

❌ `/getStats`, `/listByStudy`
✅ `/studies/{id}/annotation-statistics`

### API 통합 규칙

아래 조건 중 하나라도 만족하면 API를 합친다.

* 동일 Aggregate Root
* 동일 Read Model
* 동일 권한/정책

---

## 4. 구현 순서 (아래 → 위, 역전 금지)

### Step 1. Domain Layer

* Entity / ValueObject / Domain Service
* 단위 테스트 필수

### Step 2. Repository Layer

* Trait 정의
* 테스트용 구현
* 단위 테스트 필수

### Step 3. Application Layer

* UseCase / Service
* 트랜잭션 경계 명확히
* 통합 테스트 필수

### Step 4. API Layer

* Controller / Handler
* Request / Response DTO
* API 단위 테스트 필수

---

## 5. 시퀀스 검증 (필수)

텍스트 기반 시퀀스를 먼저 작성한다.

```text
Client
 → API
   → Application Service
     → Domain
     → Repository
   ← Response
```

이 시퀀스가 **테스트 코드로 재현 가능해야 한다**.

---

## 6. 테스트 전략

### 단위 테스트

* Domain / Repo / Service 분리
* Mock 남용 금지

### 통합 테스트

* Service + Repository 조합
* 실제 DB 또는 테스트 컨테이너 사용

### E2E 테스트 (Python)

* 기존 E2E 폴더 구조 유지
* 같은 패턴, 같은 헬퍼 사용
* 신규 파일만 추가

---

## 7. TODO 체크 규칙

* 구현 + 테스트 통과가 확인된 항목만 체크
* 테스트 미통과 상태에서 체크 금지

---

## 8. Validator 단계 (필수)

구현 완료 후 Validator는 Planning File의 TODO를 **항목별로 재검증**한다.

```md
## Validation Result

- [x] Domain Entity 정의 (Validator)
- [x] Repository Trait 정의 (Validator)
- [x] Repository 단위 테스트 통과 (Validator)
- [x] Application Service 구현 (Validator)
- [x] Service 통합 테스트 통과 (Validator)
- [x] REST API Path 충돌 없음 (Validator)
- [x] Controller 구현 (Validator)
- [x] API 단위 테스트 통과 (Validator)
- [x] Python E2E 테스트 통과 (Validator)
- [x] 전체 테스트 통과 (Validator)
```

---

## 9. 최종 완료 조건 (Exit Criteria)

아래 조건 **모두 충족 시에만 완료로 간주한다.**

* [ ] 모든 단위 테스트 통과
* [ ] 모든 통합 테스트 통과
* [ ] 기존 + 신규 E2E 테스트 통과
* [ ] API Path 충돌 없음
* [ ] DDD 레이어 역전 없음
* [ ] Planning File의 모든 TODO 체크 완료
* [ ] Validator가 모든 TODO 항목 재검증 완료
