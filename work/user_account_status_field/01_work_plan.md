# 사용자 목록 응답에 account_status 필드 추가

## 작업 개요

사용자 목록 API 응답에 계정 상태 필드를 추가하여 활성화 여부를 확인할 수 있도록 합니다.

## 작업 목표

1. UserResponse DTO에 account_status 필드 추가
2. UserResponse DTO에 email_verified 필드 추가
3. From 트레이트 구현 수정

## 작업 범위

### 1. DTO 수정

- `account_status: String` 필드 추가
- `email_verified: bool` 필드 추가
- From 트레이트 구현 수정

### 2. 응답 확인

- 사용자 목록 API 응답에 필드 포함
- 활성화 여부 확인 가능

## 예상 작업 시간

- 개발: 30분
- 테스트: 30분
- **총 예상 시간: 1시간**

## 우선순위

**중간** - 사용자 관리 UI 개선

