# 📄 기술 정책 문서

## Image / Annotation 정합성 정책

---

## 1. 문서 목적

본 문서는 **Annotation 도메인 데이터와 그에 종속된 이미지 자산(Snapshot / Mask) 간의 정합성을 보장하기 위한 정책**을 정의한다.

이 정책의 목적은 다음과 같다.

* Annotation과 이미지 간 **불일치 상태 방지**
* 업로드 실패, 재시도, 앱 재시작 상황에서도 **일관된 판단 기준 제공**
* 의료/진단 데이터로서의 **신뢰성 확보**
* 개발·운영·감사 시 **명확한 판단 근거 제공**

---

## 2. 핵심 개념 정의

### 2.1 Annotation

* 시스템의 **단일 진실 소스(Source of Truth)**
* 구조화된 도메인 데이터
* `version`을 통해 변경 이력 관리

---

### 2.2 Snapshot Image

* 특정 Annotation **version**의 시각적 표현
* DICOM + Annotation overlay 결과물
* **증거물(Evidence)** 성격

---

### 2.3 Mask

* Annotation에 종속된 분석/알고리즘용 자산
* Snapshot과 동일한 생명주기 원칙을 따름

---

## 3. 정합성의 기본 원칙 (절대 규칙)

### 원칙 1. Annotation 우선 원칙

> **Annotation이 존재하지 않으면,
> 그에 종속된 모든 이미지(Snapshot / Mask)는 의미가 없다.**

* Annotation = 루트 엔티티
* Image = 파생 자산

---

### 원칙 2. 버전 귀속 원칙

> **모든 Snapshot / Mask는 반드시 특정 Annotation version에 귀속된다.**

* `annotation_id`만으로는 충분하지 않음
* `(annotation_id, annotation_version)` 쌍이 기준

---

### 원칙 3. 최신성 원칙

> **최신 Annotation version과 일치하지 않는 이미지는
> “현재 상태를 대표하지 않는다”.**

* 표시/공유/진단에 사용 불가
* GC 대상 후보

---

### 원칙 4. 실패 허용 원칙

> **이미지 업로드 실패는 시스템 오류가 아니라 정상 상태이다.**

* 실패 ≠ 롤백
* 재시도 가능 상태로 유지

---

## 4. 정합성 상태 정의

### 4.1 Annotation 기준 상태

| 상태               | 의미                          |
| ---------------- | --------------------------- |
| version          | 현재 Annotation의 최신 버전        |
| snapshot_status  | Snapshot 업로드 상태             |
| snapshot_version | Snapshot이 생성된 Annotation 버전 |

---

### 4.2 Snapshot 상태 (`snapshot_status`)

| 상태      | 의미          |
| ------- | ----------- |
| NONE    | Snapshot 없음 |
| PENDING | 업로드 예정 / 대기 |
| READY   | 정합성 충족      |
| FAILED  | 업로드 실패      |

---

## 5. Snapshot 정합성 판정 기준

### Snapshot이 **유효(Valid)** 한 조건

```text
snapshot_status = 'READY'
AND snapshot_image_key IS NOT NULL
AND snapshot_version = annotation.version
```

👉 위 조건을 **모두 만족하지 않으면 Invalid**

---

### Snapshot이 **무효(Invalid)** 인 경우

| 상황              | 이유      |
| --------------- | ------- |
| version 불일치     | 구버전 이미지 |
| status != READY | 업로드 미완료 |
| key 없음          | 자산 부재   |

---

## 6. Annotation 변경 시 정책

### 6.1 Annotation 수정 (PATCH)

#### 처리 규칙

```
annotation.version += 1
snapshot_status = PENDING
snapshot_image_key = NULL
snapshot_version = NULL
```

#### 정책 의미

* 기존 Snapshot은 **자동으로 구버전**
* 새 Annotation 상태에 대한 Snapshot 필요

---

### 6.2 Annotation 삭제 (DELETE)

* 해당 Annotation에 종속된 모든 Snapshot / Mask는 **무효**
* GC 정책에 따라 즉시 또는 지연 삭제

---

## 7. Snapshot 업로드 정책

### 7.1 업로드 허용 조건

* 요청한 `annotationVersion`이

  * 서버의 `annotation.version`과 **일치**해야 함

불일치 시:

```
409 VERSION_MISMATCH
```

---

### 7.2 업로드 성공 시

```text
snapshot_status = READY
snapshot_version = annotation.version
snapshot_image_key = issued_key
```

---

### 7.3 업로드 실패 시

```text
snapshot_status = FAILED
(snapshot_version / key 유지하지 않음)
```

* Annotation 데이터에는 영향 없음
* 재시도 가능

---

## 8. 재시도 및 앱 재시작 정책

### 8.1 로컬 업로드 큐 기준

로컬 큐에 반드시 포함해야 할 정보:

```text
annotation_id
annotation_version
image_data
```

---

### 8.2 재시도 판단 기준

| 조건                              | 처리       |
| ------------------------------- | -------- |
| local.version == server.version | 업로드 재시도  |
| local.version != server.version | 폐기 + 재캡쳐 |

---

## 9. Snapshot / Mask 공통 정책

| 항목     | 정책                 |
| ------ | ------------------ |
| 루트     | Annotation         |
| 귀속 기준  | Annotation version |
| 최신성 판단 | version 비교         |
| 실패 처리  | 재시도 가능             |
| 삭제 판단  | Annotation 상태 기반   |

---

## 10. 표시 / 사용 정책 (Viewer 기준)

### Viewer에서 Snapshot을 표시할 수 있는 조건

* Snapshot이 **정합성 조건 충족**
* 그렇지 않으면:

  * Placeholder 표시
  * “이미지 생성 필요” 상태 안내

---

## 11. GC 정책과의 연계

### GC 대상이 되는 경우

| 조건                                    | 사유     |
| ------------------------------------- | ------ |
| annotation 없음                         | orphan |
| snapshot_version < annotation.version | 구버전    |
| FAILED 상태 장기 유지                       | 미사용    |

GC는 **정합성 정책의 결과**를 기반으로 수행된다.

---

## 12. 금지 사항 (명시적)

❌ 구버전 Snapshot을 최신 Annotation에 연결
❌ version mismatch 무시하고 업로드 허용
❌ Snapshot을 Annotation 없이 단독 취급
❌ 서버가 이미지 바이너리를 직접 보관

---

## 13. 정책의 효과

* Annotation–Image 불일치 원천 차단
* 장애/재시작/네트워크 실패에 강함
* 감사·법적 분쟁 시 명확한 근거 제공
* Snapshot / Mask 확장에도 동일 규칙 적용 가능

---

## 14. 결론

본 정합성 정책은 단순한 기술 규칙이 아니라,
**의료 Annotation 데이터의 신뢰성과 재현성을 보장하기 위한 최소 기준**이다.

> **Annotation은 사실(Fact)이고,
> Snapshot은 그 사실의 특정 시점을 증명하는 증거물이다.
> 증거물은 반드시 시점(version)에 묶여야 한다.**

---

### 다음으로 정리하면 완성되는 문서 세트

* 📄 API 명세서 (완)
* 📄 DB 마이그레이션 문서 (완)
* 📄 Image/Annotation 정합성 정책 (본 문서)
* 📄 GC 정책 (완)

필요하면 이걸 **ADR 형식**이나 **Notion 템플릿**으로도 바로 변환해줄게.
