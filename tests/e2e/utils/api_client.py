"""
API 클라이언트 유틸리티
"""
import requests
from typing import Dict, Any, Optional
import logging

logger = logging.getLogger(__name__)


class APIClient:
    """PACS 서버 API 클라이언트"""
    
    def __init__(self, base_url: str, timeout: int = 30):
        self.base_url = base_url.rstrip('/')
        self.timeout = timeout
        self.token: Optional[str] = None
        self.session = requests.Session()
    
    def login(self, email: str, password: str) -> Dict[str, Any]:
        """로그인하고 토큰 저장"""
        response = self.session.post(
            f"{self.base_url}/api/auth/login",
            json={"username": email, "password": password},
            timeout=self.timeout
        )
        response.raise_for_status()
        data = response.json()
        self.token = data.get('access_token') or data.get('token')
        logger.info(f"Logged in as {email}")
        return data
    
    def _get_headers(self, extra_headers: Optional[Dict[str, str]] = None) -> Dict[str, str]:
        """요청 헤더 생성"""
        headers = {"Content-Type": "application/json"}
        if self.token:
            headers["Authorization"] = f"Bearer {self.token}"
        if extra_headers:
            headers.update(extra_headers)
        return headers
    
    def get(self, path: str, params: Optional[Dict[str, Any]] = None) -> requests.Response:
        """GET 요청"""
        url = f"{self.base_url}{path}"
        response = self.session.get(
            url,
            params=params,
            headers=self._get_headers(),
            timeout=self.timeout
        )
        return response
    
    def post(self, path: str, json: Optional[Dict[str, Any]] = None) -> requests.Response:
        """POST 요청"""
        url = f"{self.base_url}{path}"
        response = self.session.post(
            url,
            json=json,
            headers=self._get_headers(),
            timeout=self.timeout
        )
        return response
    
    def put(self, path: str, json: Optional[Dict[str, Any]] = None) -> requests.Response:
        """PUT 요청"""
        url = f"{self.base_url}{path}"
        response = self.session.put(
            url,
            json=json,
            headers=self._get_headers(),
            timeout=self.timeout
        )
        return response
    
    def delete(self, path: str, json: Optional[Dict[str, Any]] = None) -> requests.Response:
        """DELETE 요청"""
        url = f"{self.base_url}{path}"
        response = self.session.delete(
            url,
            json=json,
            headers=self._get_headers(),
            timeout=self.timeout
        )
        return response
    
    def close(self):
        """세션 종료"""
        self.session.close()

