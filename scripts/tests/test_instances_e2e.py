#!/usr/bin/env python3
"""
E2E Test: Series Instances 조회 API
- 로그인 후 JWT 토큰 획득
- Instance 목록 조회 (InstanceNumber 정렬)
- 응답 검증
"""

import requests
import json
import sys
from typing import Optional

# 설정
BASE_URL = "http://localhost:8080"
LOGIN_ENDPOINT = f"{BASE_URL}/api/auth/login"
INSTANCES_ENDPOINT = f"{BASE_URL}/api/me/dicom/studies/1.2.410.200017.0.1.2.7.2780199001.0/series/1.2.410.200017.0.1.3.7.2780199001.3/instances"

# 테스트 계정
USERNAME = "iaid-pacs-admin"
PASSWORD = "Qlalfqjsgh1!"
PROJECT_ID = 2


def print_section(title: str):
    """섹션 제목 출력"""
    print(f"\n{'=' * 80}")
    print(f"  {title}")
    print(f"{'=' * 80}\n")


def login(username: str, password: str) -> Optional[str]:
    """로그인하여 JWT 토큰 획득"""
    print_section("1️⃣  로그인")

    payload = {
        "username": username,
        "password": password
    }

    print(f"📤 요청: POST {LOGIN_ENDPOINT}")
    print(f"   Username: {username}")
    print(f"   Password: {'*' * len(password)}")

    try:
        response = requests.post(LOGIN_ENDPOINT, json=payload)

        print(f"\n📥 응답: {response.status_code}")

        if response.status_code == 200:
            data = response.json()
            token = data.get("keycloak_access_token")

            if token:
                print(f"✅ 로그인 성공!")
                print(f"   Token: {token[:50]}...")
                return token
            else:
                print(f"❌ 토큰이 응답에 없습니다")
                print(f"   응답: {json.dumps(data, indent=2, ensure_ascii=False)}")
                return None
        else:
            print(f"❌ 로그인 실패: {response.status_code}")
            print(f"   응답: {response.text}")
            return None

    except Exception as e:
        print(f"❌ 로그인 중 오류 발생: {e}")
        return None


def get_instances(token: str, project_id: int) -> bool:
    """Instance 목록 조회"""
    print_section("2️⃣  Instance 목록 조회")
    
    headers = {
        "Authorization": f"Bearer {token}"
    }
    
    params = {
        "project_id": project_id,
        "orderby": "InstanceNumber"
    }
    
    print(f"📤 요청: GET {INSTANCES_ENDPOINT}")
    print(f"   Project ID: {project_id}")
    print(f"   Order By: InstanceNumber")
    print(f"   Authorization: Bearer {token[:30]}...")
    
    try:
        response = requests.get(INSTANCES_ENDPOINT, headers=headers, params=params)
        
        print(f"\n📥 응답: {response.status_code}")
        print(f"   Content-Type: {response.headers.get('Content-Type')}")
        print(f"   Content-Length: {response.headers.get('Content-Length')} bytes")
        
        if response.status_code == 200:
            instances = response.json()
            
            print(f"\n✅ Instance 조회 성공!")
            print(f"   총 Instance 개수: {len(instances)}")
            
            if len(instances) > 0:
                print(f"\n📊 Instance 정보:")
                for i, instance in enumerate(instances[:5], 1):  # 처음 5개만 출력
                    instance_uid = instance.get("00080018", {}).get("Value", ["N/A"])[0]
                    instance_number = instance.get("00200013", {}).get("Value", ["N/A"])[0]
                    rows = instance.get("00280010", {}).get("Value", ["N/A"])[0]
                    columns = instance.get("00280011", {}).get("Value", ["N/A"])[0]
                    
                    print(f"   [{i}] Instance UID: {instance_uid}")
                    print(f"       Instance Number: {instance_number}")
                    print(f"       Image Size: {columns}x{rows}")
                
                if len(instances) > 5:
                    print(f"   ... 외 {len(instances) - 5}개")
                
                # 전체 응답 저장
                output_file = "instances_response.json"
                with open(output_file, "w", encoding="utf-8") as f:
                    json.dump(instances, f, indent=2, ensure_ascii=False)
                print(f"\n💾 전체 응답 저장: {output_file}")
                
            else:
                print(f"\n⚠️  Instance가 없습니다")
            
            return True
            
        elif response.status_code == 403:
            print(f"❌ 접근 권한 없음 (403 Forbidden)")
            print(f"   응답: {response.text}")
            return False
            
        elif response.status_code == 404:
            print(f"❌ Series를 찾을 수 없음 (404 Not Found)")
            print(f"   응답: {response.text}")
            return False
            
        else:
            print(f"❌ Instance 조회 실패: {response.status_code}")
            print(f"   응답: {response.text}")
            return False
            
    except Exception as e:
        print(f"❌ Instance 조회 중 오류 발생: {e}")
        return False


def main():
    """메인 함수"""
    print_section("🧪 E2E Test: Series Instances 조회")

    # 1. 로그인
    token = login(USERNAME, PASSWORD)
    if not token:
        print("\n❌ 테스트 실패: 로그인 실패")
        sys.exit(1)

    # 2. Instance 조회
    success = get_instances(token, PROJECT_ID)

    # 결과
    print_section("📋 테스트 결과")
    if success:
        print("✅ 모든 테스트 통과!")
        sys.exit(0)
    else:
        print("❌ 테스트 실패")
        sys.exit(1)


if __name__ == "__main__":
    main()

