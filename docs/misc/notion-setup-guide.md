# Notion MCP 서버 설정 가이드

Claude Code와 Notion을 연동하여 작업 내용을 Notion에 자동으로 정리할 수 있습니다.

## 1. Notion Integration 생성

1. https://www.notion.so/my-integrations 접속
2. **"New integration"** 클릭
3. Integration 이름 입력 (예: "Claude Code")
4. **Associated workspace** 선택 (필수)
   - 드롭다운에서 사용할 워크스페이스를 반드시 선택해야 합니다
   - 개인 워크스페이스 또는 팀 워크스페이스 중 선택
5. **Capabilities** 확인 (기본값으로 두면 됩니다)
   - Read content
   - Update content
   - Insert content
6. **Submit** 클릭
7. **Internal Integration Secret** 복사 (나중에 사용)
   - 형식: `secret_XXXXXXXXXXXXXXXXXXXXXXXX`

## 2. Notion 페이지에 권한 부여

1. Notion에서 사용할 페이지 열기
2. 우측 상단 `...` (더보기) 클릭
3. **"Add connections"** 선택
4. 방금 생성한 Integration 선택

## 3. Notion MCP 서버 설치 (완료)

```bash
npm install -g @notionhq/notion-mcp-server
```

✅ 이미 설치 완료되었습니다.

## 4. Claude Desktop 설정

Claude Desktop의 설정 파일에 Notion MCP 서버를 추가해야 합니다.

**설정 파일 위치:**
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`

**설정 내용:**

```json
{
  "mcpServers": {
    "notion": {
      "command": "notion-mcp-server",
      "env": {
        "NOTION_API_KEY": "YOUR_NOTION_INTEGRATION_SECRET"
      }
    }
  }
}
```

**YOUR_NOTION_INTEGRATION_SECRET**를 1단계에서 복사한 실제 API 키로 교체하세요.

## 5. Claude Desktop 재시작

설정 파일을 수정한 후 Claude Desktop을 완전히 종료하고 다시 시작하세요.

## 6. 사용 방법

연동 후 Claude Code에서 다음과 같이 요청할 수 있습니다:

- "이 코드 변경사항을 Notion에 정리해줘"
- "오늘 작업한 내용을 Notion 페이지로 만들어줘"
- "이 성능 테스트 결과를 Notion 데이터베이스에 추가해줘"

## 주의사항

- Integration은 명시적으로 권한을 부여한 페이지만 접근 가능
- API 키는 절대 공개 저장소에 커밋하지 마세요
- 여러 workspace를 사용하는 경우 각각 별도의 Integration 필요

## 문제 해결

**MCP 서버가 작동하지 않는 경우:**
1. Claude Desktop 로그 확인: `~/Library/Logs/Claude/`
2. Notion API 키가 올바른지 확인
3. 페이지에 Integration 권한이 부여되었는지 확인
4. Claude Desktop을 완전히 재시작했는지 확인

**도움이 필요하면:**
- Notion MCP 서버 문서: https://github.com/notionhq/notion-mcp-server
- MCP 프로토콜 문서: https://modelcontextprotocol.io
