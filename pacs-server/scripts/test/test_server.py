#!/usr/bin/env python3
"""
PACS Server API 테스트용 간단한 서버
Global Roles with Permissions API를 모킹합니다.
"""

import json
import http.server
import socketserver
from urllib.parse import urlparse, parse_qs
import threading
import time

class TestServerHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        parsed_url = urlparse(self.path)
        path = parsed_url.path
        query_params = parse_qs(parsed_url.query)
        
        print(f"Request: {self.path}")
        
        if path == "/health":
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({"status": "ok", "message": "PACS Server is running"}).encode())
            
        elif path == "/api/roles/global/with-permissions":
            # 페이지네이션 파라미터 처리
            page = int(query_params.get('page', ['1'])[0])
            page_size = int(query_params.get('page_size', ['20'])[0])
            
            # Mock 데이터 - 실제 PACS 서버의 역할과 권한 데이터
            mock_roles = [
                {
                    "id": 1,
                    "name": "시스템 관리자",
                    "description": "전체 시스템 관리 권한",
                    "scope": "GLOBAL",
                    "permissions": [
                        {"id": 1, "resource_type": "user", "action": "create"},
                        {"id": 2, "resource_type": "user", "action": "read"},
                        {"id": 3, "resource_type": "user", "action": "update"},
                        {"id": 4, "resource_type": "user", "action": "delete"},
                        {"id": 5, "resource_type": "project", "action": "create"},
                        {"id": 6, "resource_type": "project", "action": "read"},
                        {"id": 7, "resource_type": "project", "action": "update"},
                        {"id": 8, "resource_type": "project", "action": "delete"},
                        {"id": 9, "resource_type": "annotation", "action": "create"},
                        {"id": 10, "resource_type": "annotation", "action": "read"},
                        {"id": 11, "resource_type": "annotation", "action": "update"},
                        {"id": 12, "resource_type": "annotation", "action": "delete"},
                        {"id": 13, "resource_type": "role", "action": "create"},
                        {"id": 14, "resource_type": "role", "action": "read"},
                        {"id": 15, "resource_type": "role", "action": "update"},
                        {"id": 16, "resource_type": "role", "action": "delete"},
                        {"id": 17, "resource_type": "permission", "action": "create"},
                        {"id": 18, "resource_type": "permission", "action": "read"},
                        {"id": 19, "resource_type": "permission", "action": "update"},
                        {"id": 20, "resource_type": "permission", "action": "delete"}
                    ]
                },
                {
                    "id": 2,
                    "name": "프로젝트 관리자",
                    "description": "프로젝트 관리 권한",
                    "scope": "GLOBAL",
                    "permissions": [
                        {"id": 5, "resource_type": "project", "action": "create"},
                        {"id": 6, "resource_type": "project", "action": "read"},
                        {"id": 7, "resource_type": "project", "action": "update"},
                        {"id": 8, "resource_type": "project", "action": "delete"},
                        {"id": 9, "resource_type": "annotation", "action": "create"},
                        {"id": 10, "resource_type": "annotation", "action": "read"},
                        {"id": 11, "resource_type": "annotation", "action": "update"},
                        {"id": 12, "resource_type": "annotation", "action": "delete"},
                        {"id": 21, "resource_type": "user", "action": "read"},
                        {"id": 22, "resource_type": "user", "action": "update"}
                    ]
                },
                {
                    "id": 3,
                    "name": "영상의학과 의사",
                    "description": "영상의학과 의사 권한",
                    "scope": "GLOBAL",
                    "permissions": [
                        {"id": 6, "resource_type": "project", "action": "read"},
                        {"id": 9, "resource_type": "annotation", "action": "create"},
                        {"id": 10, "resource_type": "annotation", "action": "read"},
                        {"id": 11, "resource_type": "annotation", "action": "update"},
                        {"id": 12, "resource_type": "annotation", "action": "delete"},
                        {"id": 21, "resource_type": "user", "action": "read"}
                    ]
                },
                {
                    "id": 4,
                    "name": "방사선사",
                    "description": "방사선사 권한",
                    "scope": "GLOBAL",
                    "permissions": [
                        {"id": 6, "resource_type": "project", "action": "read"},
                        {"id": 10, "resource_type": "annotation", "action": "read"},
                        {"id": 11, "resource_type": "annotation", "action": "update"},
                        {"id": 21, "resource_type": "user", "action": "read"}
                    ]
                },
                {
                    "id": 5,
                    "name": "일반 사용자",
                    "description": "기본 사용자 권한",
                    "scope": "GLOBAL",
                    "permissions": [
                        {"id": 6, "resource_type": "project", "action": "read"},
                        {"id": 10, "resource_type": "annotation", "action": "read"},
                        {"id": 21, "resource_type": "user", "action": "read"}
                    ]
                }
            ]
            
            # 페이지네이션 적용
            total_count = len(mock_roles)
            start_idx = (page - 1) * page_size
            end_idx = start_idx + page_size
            paginated_roles = mock_roles[start_idx:end_idx]
            
            total_pages = (total_count + page_size - 1) // page_size
            
            response = {
                "roles": paginated_roles,
                "total_count": total_count,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages
            }
            
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps(response, ensure_ascii=False, indent=2).encode())
            
        else:
            self.send_response(404)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({"error": "Not found", "path": path}).encode())
    
    def log_message(self, format, *args):
        # 로그 메시지 억제
        pass

def start_server():
    PORT = 8080
    with socketserver.TCPServer(("", PORT), TestServerHandler) as httpd:
        print(f"🚀 PACS Server Test API running on http://localhost:{PORT}")
        print(f"📋 Available endpoints:")
        print(f"   GET /health")
        print(f"   GET /api/roles/global/with-permissions")
        print(f"   GET /api/roles/global/with-permissions?page=1&page_size=10")
        print(f"")
        print(f"Press Ctrl+C to stop the server")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print(f"\n🛑 Server stopped")

if __name__ == "__main__":
    start_server()
