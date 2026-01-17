#!/usr/bin/env python3
"""
간단한 데모 테스트 - 설정 확인 및 기본 연결 테스트
"""
import sys
import logging
from config import TestConfig
from utils.api_client import APIClient

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def main():
    """데모 테스트 실행"""
    logger.info("="*80)
    logger.info("PACS Server E2E Test - Demo")
    logger.info("="*80)
    
    # 설정 로드
    config = TestConfig.from_env()
    logger.info(f"\n📋 Configuration:")
    logger.info(f"  Base URL: {config.base_url}")
    logger.info(f"  Admin Email: {config.admin_email}")
    logger.info(f"  Timeout: {config.timeout}s")
    
    # API 클라이언트 생성
    client = APIClient(config.base_url, config.timeout)
    
    try:
        # 1. Health Check
        logger.info(f"\n🏥 Testing health check...")
        try:
            response = client.get("/health")
            if response.status_code == 200:
                logger.info(f"  ✓ Server is healthy")
            else:
                logger.warning(f"  ⚠ Health check returned {response.status_code}")
        except Exception as e:
            logger.error(f"  ✗ Health check failed: {e}")
            logger.error(f"  Make sure the server is running at {config.base_url}")
            return False
        
        # 2. Login Test
        logger.info(f"\n🔐 Testing login...")
        try:
            login_data = client.login(config.admin_email, config.admin_password)
            logger.info(f"  ✓ Login successful")
            logger.info(f"  User ID: {login_data.get('user', {}).get('id')}")
            logger.info(f"  Email: {login_data.get('user', {}).get('email')}")
        except Exception as e:
            logger.error(f"  ✗ Login failed: {e}")
            logger.error(f"  Check your credentials in .env file")
            return False
        
        # 3. Get Current User
        logger.info(f"\n👤 Testing get current user...")
        try:
            response = client.get("/api/users/me")
            if response.status_code == 200:
                user_data = response.json()
                logger.info(f"  ✓ Got user info")
                logger.info(f"  Name: {user_data.get('name', 'N/A')}")
                logger.info(f"  Email: {user_data.get('email')}")
            else:
                logger.warning(f"  ⚠ Get user returned {response.status_code}")
        except Exception as e:
            logger.error(f"  ✗ Get user failed: {e}")
        
        # 4. List Projects
        logger.info(f"\n📁 Testing list projects...")
        try:
            response = client.get("/api/projects")
            if response.status_code == 200:
                data = response.json()
                projects = data if isinstance(data, list) else data.get("projects", [])
                logger.info(f"  ✓ Found {len(projects)} projects")
                
                if projects:
                    logger.info(f"  First project: {projects[0].get('name')}")
            else:
                logger.warning(f"  ⚠ List projects returned {response.status_code}")
        except Exception as e:
            logger.error(f"  ✗ List projects failed: {e}")
        
        # 5. Query Annotations
        logger.info(f"\n📝 Testing query annotations...")
        try:
            response = client.get("/api/annotations", params={"limit": 5})
            if response.status_code == 200:
                data = response.json()
                annotations = data if isinstance(data, list) else data.get("annotations", [])
                logger.info(f"  ✓ Found {len(annotations)} annotations (limited to 5)")
            else:
                logger.warning(f"  ⚠ Query annotations returned {response.status_code}")
        except Exception as e:
            logger.error(f"  ✗ Query annotations failed: {e}")
        
        logger.info("\n" + "="*80)
        logger.info("✅ Demo test completed successfully!")
        logger.info("="*80)
        logger.info("\nYou can now run the full test suite:")
        logger.info("  python run_all_tests.py")
        logger.info("\nOr run individual tests:")
        logger.info("  pytest test_01_auth.py -v -s")
        logger.info("  pytest test_02_project.py -v -s")
        logger.info("  pytest test_03_annotation.py -v -s")
        logger.info("="*80)
        
        return True
        
    except Exception as e:
        logger.error(f"\n❌ Demo test failed: {e}")
        return False
    
    finally:
        client.close()


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)

