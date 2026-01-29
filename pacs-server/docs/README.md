# PACS Extension Server - Documentation

## 📚 문서 목록

### View Selection API

View Selection API는 DICOM Viewer에서 여러 Study/Series를 선택하여 세션 상태를 저장하고 공유할 수 있는 기능을 제공합니다.

- **[Quick Start Guide](./VIEW_SELECTION_QUICK_START.md)** - 5분 안에 시작하기
- **[API Guide (English)](./VIEW_SELECTION_API_GUIDE.md)** - 전체 API 가이드 (영문)
- **[API Guide (한국어)](./VIEW_SELECTION_API_GUIDE_KR.md)** - 전체 API 가이드 (한국어)

### Architecture

- **[Architecture Overview](./architecture/)** - 시스템 아키텍처 개요

### Authentication

- **[Authentication Guide](./auth/)** - 인증 및 권한 관리

## 🚀 Quick Links

- **Swagger UI**: `http://localhost:8080/swagger-ui/`
- **Health Check**: `http://localhost:8080/health`
- **E2E Tests**: `../e2e/`

## 📖 주요 기능

### View Selection API

```bash
# Selection 생성
curl -X POST http://localhost:8080/api/v1/view-selections \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{
    "series": [
      {
        "study_uid": "1.2.840.113619.2.55.3.604641477.123",
        "series_uid": "1.2.840.113619.2.55.3.604641477.124"
      }
    ],
    "layout": {
      "rows": 2,
      "cols": 2
    },
    "initial_views": [
      {
        "row": 0,
        "col": 0,
        "study_uid": "1.2.840.113619.2.55.3.604641477.123",
        "series_uid": "1.2.840.113619.2.55.3.604641477.124",
        "sop_uid": "1.2.840.113619.2.55.3.604641477.126"
      }
    ]
  }'
```

**주요 기능**:
- ✅ 멀티 Study/Series 선택
- ✅ Viewport Layout 설정 (그리드 기반)
- ✅ Initial Views 설정 (각 Viewport의 초기 이미지)
- ✅ 자동 TTL 연장 (조회 시마다)
- ✅ URL 공유 (Selection ID 기반)
- ✅ Redis/In-memory 지원

## 🧪 테스트

### E2E 테스트 실행

```bash
# View Selection API 테스트
cd e2e
python3 test_view_selection_e2e.py
```

### 테스트 결과 예시

```
============================================================
📊 테스트 결과 요약
============================================================
✅ 통과: 79
❌ 실패: 0
📝 총계: 79

🎉 모든 테스트 통과!
```

## 🔧 설정

### Redis 설정 (권장)

```toml
# config/default.toml
[redis]
url = "redis://localhost:6379"
view_selection_ttl_sec = 1800  # 30분
```

### In-memory Fallback

Redis가 연결되지 않으면 자동으로 in-memory 저장소를 사용합니다.

**경고**: 
- ❌ 서버 재시작 시 모든 데이터 삭제
- ❌ 여러 서버 인스턴스 간 공유 불가
- ✅ 단일 서버 개발 환경에서는 사용 가능

## 📝 기여 가이드

문서 작성 시 다음 규칙을 따라주세요:

1. **명확성**: 기술적 용어는 설명과 함께 사용
2. **예제**: 모든 API는 curl/JavaScript/Python 예제 포함
3. **에러 처리**: 가능한 에러 케이스와 해결 방법 명시
4. **다국어**: 중요한 문서는 영문/한글 버전 모두 제공

## 📞 문의

- **이슈 트래커**: GitHub Issues
- **이메일**: support@example.com

