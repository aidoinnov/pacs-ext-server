# GitHub MCP 연동 가이드

## 개요
GitHub MCP(Model Context Protocol) 서버를 통해 Claude Desktop에서 GitHub 저장소, 이슈, PR 등을 직접 관리할 수 있습니다.

## 사전 준비사항

### 1. GitHub Personal Access Token 생성
1. GitHub 설정 페이지로 이동: https://github.com/settings/tokens
2. "Generate new token" → "Generate new token (classic)" 선택
3. 토큰 이름 설정 (예: "Claude MCP Token")
4. 필요한 권한 선택:
   - `repo` - 전체 저장소 접근
   - `read:org` - 조직 정보 읽기
   - `user` - 사용자 정보 읽기
5. "Generate token" 클릭
6. 생성된 토큰 복사 (다시 볼 수 없으니 안전한 곳에 보관)

### 2. GitHub MCP 서버 설치

```bash
npm install -g @modelcontextprotocol/server-github
```

또는

```bash
npx -y @modelcontextprotocol/server-github
```

## 설정 방법

### Claude Desktop 설정 파일 수정

#### macOS
파일 위치: `~/Library/Application Support/Claude/claude_desktop_config.json`

#### Windows
파일 위치: `%APPDATA%\Claude\claude_desktop_config.json`

#### Linux
파일 위치: `~/.config/Claude/claude_desktop_config.json`

### 설정 예시

```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "여기에_토큰_입력"
      }
    }
  }
}
```

### 여러 MCP 서버 동시 사용 (GitHub + Notion 예시)

```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_xxxxxxxxxxxxx"
      }
    },
    "notion": {
      "command": "npx",
      "args": ["-y", "@notionhq/client"],
      "env": {
        "NOTION_API_KEY": "ntn_xxxxxxxxxxxxx"
      }
    }
  }
}
```

## 사용 방법

### 1. Claude Desktop 재시작
설정 파일 수정 후 Claude Desktop을 완전히 종료하고 다시 시작합니다.

### 2. 연결 확인
Claude에게 다음과 같이 요청하여 연결을 테스트할 수 있습니다:
- "내 GitHub 저장소 목록을 보여줘"
- "특정 저장소의 이슈 목록을 가져와줘"
- "새로운 이슈를 생성해줘"

### 3. 주요 기능
- **저장소 관리**: 저장소 검색, 정보 조회, 파일 읽기
- **이슈 관리**: 이슈 생성, 조회, 수정, 닫기
- **Pull Request**: PR 생성, 조회, 머지
- **브랜치 관리**: 브랜치 생성, 조회
- **커밋**: 커밋 히스토리 조회

## 문제 해결

### 연결이 안 될 때
1. **토큰 확인**: GitHub Personal Access Token이 올바른지 확인
2. **권한 확인**: 토큰에 필요한 권한이 부여되었는지 확인
3. **재시작**: Claude Desktop 완전 종료 후 재시작
4. **로그 확인**: Claude Desktop 개발자 도구에서 에러 로그 확인

### 토큰이 만료되었을 때
1. GitHub에서 새 토큰 생성
2. 설정 파일의 `GITHUB_PERSONAL_ACCESS_TOKEN` 업데이트
3. Claude Desktop 재시작

## 보안 주의사항

1. **토큰 보안**: Personal Access Token을 절대 공개 저장소에 커밋하지 마세요
2. **최소 권한**: 필요한 최소한의 권한만 부여하세요
3. **토큰 주기적 갱신**: 보안을 위해 토큰을 주기적으로 갱신하세요
4. **환경 변수 사용**: 가능하면 환경 변수로 토큰을 관리하세요

## 추가 자료
- [MCP 공식 문서](https://modelcontextprotocol.io)
- [GitHub MCP 서버 저장소](https://github.com/modelcontextprotocol/servers)
- [Claude Desktop 문서](https://claude.ai/docs)
