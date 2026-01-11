오케이, **서버에 바로 던질 수 있게 “초간략 설명본”**으로 정리해줄게.
(회의 슬라이드 1장 / 이슈 코멘트용 수준)

---

## 📌 Annotation 상태 응답 설계 (초간략)

### 문제

* annotation 개수(0 / 1 / 2)와
* 사용자 역할(reader / adjudicator / PM)에 따라
* **UI에 보여줄 상태 단계(색상)가 달라짐**

👉 이 로직을 **UI에서 계산하지 않게 하고 싶음**

---

### 해결 원칙

* **서버가 “의미 있는 상태(state)”를 계산해서 내려준다**
* UI는 상태를 그대로 렌더링만 한다

---

### 응답 구조 (extension 영역)

```json
"_ext": {
  "review": {
    "reviewStage": "IN_PROGRESS",
    "availableStages": ["NOT_STARTED", "IN_PROGRESS", "READY_FOR_ADJUDICATION"],
    "annotationSummary": {
      "reader1": true,
      "reader2": false
    }
  }
}
```

---

### 필드 설명

* `reviewStage`
  → 현재 Study의 단일 상태 값
* `availableStages`
  → **현재 사용자 역할 기준으로 UI에 보여줄 단계 목록**
* `annotationSummary`
  → 참고용 정보 (UI 디테일 표시용)

---

### 상태 정의

```text
NOT_STARTED            = annotation 0명
IN_PROGRESS            = annotation 1명
READY_FOR_ADJUDICATION = annotation 2명
```

---

### 역할별 차이

* reader: `availableStages`에 파랑 단계 제외
* adjudicator / PM: 파랑 단계 포함

👉 **같은 Study라도 role에 따라 availableStages만 달라짐**

---

### 핵심 한 줄

> **“annotation 개수·역할에 따른 판단은 서버에서 하고,
> UI는 reviewStage + availableStages만 보고 그린다.”**

---

이거면 서버가

* 왜 이렇게 하는지 이해하고
* 구현 포인트도 바로 잡을 수 있어.

원하면 **서버용 의사코드 5줄 버전**도 만들어줄까?
