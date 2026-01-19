#!/usr/bin/env python3
"""
마스크 CRUD API 테스트 스크립트
"""

import requests
import json
import time

BASE_URL = "http://localhost:8080"

def test_health():
    """헬스 체크 테스트"""
    print("🔍 Testing health endpoint...")
    try:
        response = requests.get(f"{BASE_URL}/health")
        print(f"Status: {response.status_code}")
        print(f"Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"❌ Health check failed: {e}")
        return False

def create_test_user():
    """테스트 사용자 생성"""
    print("👤 Creating test user...")
    username = f"testuser_{int(time.time())}"
    email = f"test_{int(time.time())}@example.com"
    
    user_data = {
        "keycloak_id": f"test-keycloak-{int(time.time())}",
        "username": username,
        "email": email
    }
    
    try:
        response = requests.post(f"{BASE_URL}/api/users", json=user_data)
        if response.status_code == 201:
            print(f"✅ User created: {username}")
            return response.json()
        else:
            print(f"❌ User creation failed: {response.status_code} - {response.text}")
            return None
    except Exception as e:
        print(f"❌ User creation error: {e}")
        return None

def login():
    """로그인 테스트"""
    print("🔐 Testing login endpoint...")
    login_data = {
        "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
        "username": "TestUser2",
        "email": "user2@example.com"
    }
    
    try:
        response = requests.post(f"{BASE_URL}/api/auth/login", json=login_data)
        print(f"Status: {response.status_code}")
        print(f"Response: {response.text}")
        
        if response.status_code == 200:
            token = response.json()["token"]
            print(f"✅ Login successful, token: {token[:20]}...")
            return token
        else:
            print("❌ Login failed")
            return None
    except Exception as e:
        print(f"❌ Login error: {e}")
        return None

def create_annotation(token):
    """어노테이션 생성"""
    print("📝 Testing annotation creation...")
    
    annotation_data = {
        "project_id": 1,
        "study_instance_uid": "1.2.3.4.5.6.7.8.9.10",
        "series_instance_uid": "1.2.3.4.5.6.7.8.9.11",
        "sop_instance_uid": "1.2.3.4.5.6.7.8.9.12",
        "annotation_data": {
            "type": "point",
            "coordinates": [100, 200],
            "label": "Test annotation"
        },
        "is_shared": False,
        "viewer_software": "ohif",
        "measurement_values": [
            {
                "id": "m1",
                "type": "raw",
                "values": [42.3, 18.7],
                "unit": "mm"
            }
        ]
    }
    
    headers = {"Authorization": f"Bearer {token}"}
    
    try:
        response = requests.post(f"{BASE_URL}/api/annotations", json=annotation_data, headers=headers)
        print(f"Status: {response.status_code}")
        print(f"Response: {response.json()}")
        
        if response.status_code == 201:
            annotation_id = response.json()["id"]
            print(f"✅ Annotation created with ID: {annotation_id}")
            return annotation_id
        else:
            print("❌ Annotation creation failed")
            return None
    except Exception as e:
        print(f"❌ Annotation creation error: {e}")
        return None

def create_mask_group(annotation_id, token):
    """마스크 그룹 생성"""
    print("🎭 Testing mask group creation...")
    
    mask_group_data = {
        "group_name": "Test Mask Group",
        "model_name": "Test Model",
        "version": "1.0.0",
        "modality": "CT",
        "slice_count": 10,
        "mask_type": "segmentation",
        "description": "Test mask group for API testing"
    }
    
    headers = {"Authorization": f"Bearer {token}"}
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/annotations/{annotation_id}/mask-groups",
            json=mask_group_data,
            headers=headers
        )
        print(f"Status: {response.status_code}")
        print(f"Response: {response.json()}")
        
        if response.status_code == 201:
            group_id = response.json()["id"]
            print(f"✅ Mask group created with ID: {group_id}")
            return group_id
        else:
            print("❌ Mask group creation failed")
            return None
    except Exception as e:
        print(f"❌ Mask group creation error: {e}")
        return None

def create_mask(annotation_id, group_id, token):
    """마스크 생성"""
    print("🎨 Testing mask creation...")
    
    mask_data = {
        "mask_group_id": group_id,
        "slice_index": 1,
        "sop_instance_uid": "1.2.3.4.5.6.7.8.9.13",
        "label_name": "liver",
        "file_path": "masks/group1/slice_001.png",
        "mime_type": "image/png",
        "file_size": 1024000,
        "checksum": "sha256:abc123def456",
        "width": 512,
        "height": 512
    }
    
    headers = {"Authorization": f"Bearer {token}"}
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/annotations/{annotation_id}/mask-groups/{group_id}/masks",
            json=mask_data,
            headers=headers
        )
        print(f"Status: {response.status_code}")
        print(f"Response: {response.json()}")
        
        if response.status_code == 201:
            mask_id = response.json()["id"]
            print(f"✅ Mask created with ID: {mask_id}")
            return mask_id
        else:
            print("❌ Mask creation failed")
            return None
    except Exception as e:
        print(f"❌ Mask creation error: {e}")
        return None

def list_masks(annotation_id, group_id, token):
    """마스크 목록 조회"""
    print("📋 Testing mask listing...")
    
    headers = {"Authorization": f"Bearer {token}"}
    
    try:
        response = requests.get(
            f"{BASE_URL}/api/annotations/{annotation_id}/mask-groups/{group_id}/masks",
            headers=headers
        )
        print(f"Status: {response.status_code}")
        print(f"Response: {response.text}")
        
        if response.status_code == 200:
            masks = response.json()["masks"]
            print(f"✅ Found {len(masks)} masks")
            return masks
        else:
            print("❌ Mask listing failed")
            return []
    except Exception as e:
        print(f"❌ Mask listing error: {e}")
        return []

def get_mask(annotation_id, group_id, mask_id, token):
    """마스크 상세 조회"""
    print("🔍 Testing mask detail retrieval...")
    
    headers = {"Authorization": f"Bearer {token}"}
    
    try:
        response = requests.get(
            f"{BASE_URL}/api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
            headers=headers
        )
        print(f"Status: {response.status_code}")
        print(f"Response: {response.text}")
        
        if response.status_code == 200:
            print("✅ Mask detail retrieved successfully")
            return response.json()
        else:
            print("❌ Mask detail retrieval failed")
            return None
    except Exception as e:
        print(f"❌ Mask detail retrieval error: {e}")
        return None

def update_mask(annotation_id, group_id, mask_id, token):
    """마스크 수정"""
    print("✏️ Testing mask update...")
    
    update_data = {
        "label_name": "liver_updated",
        "file_size": 2048000,
        "width": 1024,
        "height": 1024
    }
    
    headers = {"Authorization": f"Bearer {token}"}
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
            json=update_data,
            headers=headers
        )
        print(f"Status: {response.status_code}")
        print(f"Response: {response.text}")
        
        if response.status_code == 200:
            print("✅ Mask updated successfully")
            return response.json()
        else:
            print("❌ Mask update failed")
            return None
    except Exception as e:
        print(f"❌ Mask update error: {e}")
        return None

def generate_download_url(annotation_id, group_id, mask_id, token):
    """다운로드 URL 생성"""
    print("🔗 Testing download URL generation...")
    
    download_data = {
        "mask_id": mask_id,
        "file_path": "masks/group1/slice_001.png",
        "expires_in": 3600
    }
    
    headers = {"Authorization": f"Bearer {token}"}
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}/download-url",
            json=download_data,
            headers=headers
        )
        print(f"Status: {response.status_code}")
        print(f"Response: {response.text}")
        
        if response.status_code == 200:
            print("✅ Download URL generated successfully")
            return response.json()
        else:
            print("❌ Download URL generation failed")
            return None
    except Exception as e:
        print(f"❌ Download URL generation error: {e}")
        return None

def delete_mask(annotation_id, group_id, mask_id, token):
    """마스크 삭제"""
    print("🗑️ Testing mask deletion...")
    
    headers = {"Authorization": f"Bearer {token}"}
    
    try:
        response = requests.delete(
            f"{BASE_URL}/api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
            headers=headers
        )
        print(f"Status: {response.status_code}")
        
        if response.status_code == 204:
            print("✅ Mask deleted successfully")
            return True
        else:
            print("❌ Mask deletion failed")
            return False
    except Exception as e:
        print(f"❌ Mask deletion error: {e}")
        return False

def main():
    """메인 테스트 함수"""
    print("🚀 Starting Mask CRUD API Test")
    print("=" * 50)
    
    # 1. 헬스 체크
    if not test_health():
        print("❌ Health check failed, exiting")
        return
    
    # 2. 로그인
    token = login()
    if not token:
        print("❌ Login failed, exiting")
        return
    
    # 3. 어노테이션 생성
    annotation_id = create_annotation(token)
    if not annotation_id:
        print("❌ Annotation creation failed, exiting")
        return
    
    # 4. 마스크 그룹 생성
    group_id = create_mask_group(annotation_id, token)
    if not group_id:
        print("❌ Mask group creation failed, exiting")
        return
    
    # 5. 마스크 생성
    mask_id = create_mask(annotation_id, group_id, token)
    if not mask_id:
        print("❌ Mask creation failed, exiting")
        return
    
    # 6. 마스크 목록 조회
    masks = list_masks(annotation_id, group_id, token)
    
    # 7. 마스크 상세 조회
    mask_detail = get_mask(annotation_id, group_id, mask_id, token)
    
    # 8. 마스크 수정
    updated_mask = update_mask(annotation_id, group_id, mask_id, token)
    
    # 9. 다운로드 URL 생성
    download_url = generate_download_url(annotation_id, group_id, mask_id, token)
    
    # 10. 마스크 삭제
    delete_success = delete_mask(annotation_id, group_id, mask_id, token)
    
    print("\n🎉 All tests completed!")
    print("=" * 50)

if __name__ == "__main__":
    main()
