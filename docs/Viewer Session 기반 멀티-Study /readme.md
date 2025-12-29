
# Viewer Session 기반 멀티-Study / 멀티-Series 뷰잉 설계 문서

## 1. 문서 목적 (Purpose)

본 문서는 PACS UI에서 **여러 Study에 속한 Series를 선택하여 Viewer에서 출력**해야 하는 요구사항을 충족하기 위해,
Viewer 진입 방식을 **Study 중심(Viewer-by-Study)** 에서
**Selection(Session) 중심(Viewer-by-Selection)** 으로 전환하는 설계의 배경, 구조, 동작 순서를 정의한다.

본 설계는 다음을 목표로 한다:

* 멀티 Study / 멀티 Series 뷰잉 지원
* Progressive DICOM Loading 구조와의 정합성
* URL 기반 상태 재현 (새로고침 / 공유 / 감사 로그)
* PACS와 Viewer 간 책임 분리 (Separation of Concerns)

---

## 2. 기존 문제점 (Problem Statement)

### 2.1 기존 Viewer 진입 방식의 한계

기존 Viewer는 다음과 같은 구조를 가정한다:

```
Patient → Study → Series → Instance
```

Viewer는 `Study UID`를 기준으로 열리며, 해당 Study에 속한 모든 Series를 표시한다.

그러나 실제 요구사항은 다음과 같다:

* PACS 리스트 화면에서

  * 여러 Study에 속한 Series를 선택
  * 선택된 Series만 Viewer에 표시
* Study 경계를 넘는 Series 선택 가능

이로 인해 다음 문제가 발생한다:

* 단일 Study path 기반 Viewer (`/viewer/studies/{studyUID}`)로는 표현 불가
* Query Parameter로 Series UID를 모두 전달할 경우 URL 복잡도 폭발
* Viewer 상태를 URL로 안정적으로 표현하기 어려움

---

## 3. 설계 원칙 (Design Principles)

### 3.1 Viewer는 “명령(Command)”이 아니라 “상태(View)”를 표현해야 한다

* Viewer는 “무엇을 여는가”가 아니라
* “어떤 상태를 재현하는가”를 기준으로 동작해야 한다

따라서 Viewer 진입은 다음 원칙을 따른다:

* Viewer Open = **Read (GET)**
* Viewer 상태 생성 = **Create (POST)**

---

### 3.2 복잡한 View 상태는 ID로 캡슐화한다

여러 Study / Series 선택 정보는 다음과 같은 특성을 가진다:

* 구조가 복잡함
* 길이가 가변적임
* URL에 직접 직렬화하기 부적합함

이에 따라 선택 상태를 **Selection(Session)** 이라는 개념으로 서버에 저장하고,
이를 대표하는 **Selection ID**를 Viewer의 진입점으로 사용한다.

---

## 4. 핵심 개념 정의 (Core Concepts)

### 4.1 View Selection (또는 View Session)

| 항목           | 설명                        |
| ------------ | ------------------------- |
| Selection    | Viewer에서 재현할 Series 선택 상태 |
| Selection ID | Selection을 식별하는 고유 ID     |
| 수명           | TTL 기반 (일시적 세션)           |

Selection은 다음 정보를 포함한다:

```json
{
  "selectionId": "sel_xxx",
  "series": [
    { "studyUID": "...", "seriesUID": "..." },
    { "studyUID": "...", "seriesUID": "..." }
  ]
}
```

---

### 4.2 Viewer의 역할 정의

Viewer는 다음 책임만을 가진다:

* Selection ID를 해석
* Selection에 포함된 Series를 로딩
* Progressive Loading으로 렌더링

Viewer는 **선택 로직을 가지지 않으며**,
선택 상태는 외부(PACS UI 또는 Selection API)에서 생성된다.

---

## 5. 전체 동작 순서 (End-to-End Flow)

### 5.1 Step 1: PACS UI에서 Series 선택

사용자는 PACS 리스트 화면에서:

* 여러 Study를 탐색
* 각 Study에서 하나 이상의 Series 선택

이 시점의 선택 상태는 **Client State**로만 존재한다.

---

### 5.2 Step 2: Selection 생성 (POST)

선택 완료 후, PACS UI는 Selection 생성 API를 호출한다.

```http
POST /api/v1/view-selections
Content-Type: application/json
```

```json
{
  "series": [
    { "studyUID": "1.2.3", "seriesUID": "1.2.3.4" },
    { "studyUID": "2.3.4", "seriesUID": "2.3.4.5" }
  ]
}
```

서버는 다음 작업을 수행한다:

* 선택된 Series에 대한 접근 권한 검증 (RBAC)
* Selection 저장 (DB 또는 Redis)
* Selection ID 생성

```json
{
  "selectionId": "sel_8f23ab"
}
```

---

### 5.3 Step 3: Viewer 오픈 (GET)

PACS UI는 반환받은 Selection ID를 사용하여 Viewer를 연다.

```text
GET /viewer/selections/sel_8f23ab
```

* 같은 탭 또는 새 탭 모두 가능
* URL은 Viewer 상태를 완전히 표현

---

### 5.4 Step 4: Viewer 초기화 및 Selection 조회

Viewer 앱은 로딩 시 Selection 정보를 조회한다.

```http
GET /api/v1/view-selections/sel_8f23ab
```

```json
{
  "selectionId": "sel_8f23ab",
  "series": [
    { "studyUID": "1.2.3", "seriesUID": "1.2.3.4" },
    { "studyUID": "2.3.4", "seriesUID": "2.3.4.5" }
  ]
}
```

---

### 5.5 Step 5: Progressive DICOM Loading

Viewer는 Selection에 포함된 Series를 기준으로 다음 순서로 로딩한다:

1. Thumbnail Server 조회
2. Preview(Rendered Image) 로딩
3. 사용자가 열람 시 Instance(WADO-RS) 로딩

Study 경계는 Viewer 로직에서 중요하지 않으며,
Series UID가 로딩의 최소 단위가 된다.

---

## 6. 왜 Viewer를 POST + Body로 열지 않는가

### 6.1 POST Viewer Open의 문제점

* 새로고침 시 상태 소실
* URL 공유 및 북마크 불가
* 감사 로그 및 재현성 저하
* HTTP 캐시 및 프록시 활용 불가

### 6.2 설계 결론

| 행위         | HTTP Method |
| ---------- | ----------- |
| View 상태 생성 | POST        |
| Viewer 진입  | GET         |

---

## 7. 저장 방식 및 운영 고려사항

### 7.1 Selection 저장소

| 방식          | 권장도 | 비고          |
| ----------- | --- | ----------- |
| Redis + TTL | 높음  | 세션성 데이터에 적합 |
| RDB         | 중간  | 감사/이력 필요 시  |
| In-memory   | 낮음  | 단일 서버 한정    |

### 7.2 TTL 정책

* 기본 TTL: 10~30분
* Viewer 접근 시 TTL 연장 가능
* 만료 시 Selection 재생성 필요

---

## 8. 설계 요약 (Summary)

본 설계는 다음을 만족한다:

* 멀티 Study / 멀티 Series Viewer 지원
* Viewer 상태의 URL 기반 재현
* Progressive Loading 아키텍처와의 정합성
* PACS와 Viewer 간 책임 분리
* 확장 가능성 (협업, 북마크, 비교 뷰)

---

## 9. 핵심 문장 (Design Statement)

> **“Viewer는 DICOM 데이터를 여는 애플리케이션이 아니라,
> 서버에 저장된 View Selection 상태를 재현하는 Renderer이다.”**

---