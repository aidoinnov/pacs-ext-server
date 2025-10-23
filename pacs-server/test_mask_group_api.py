#!/usr/bin/env python3
"""
Test script for Mask Group API
"""
import requests
import json
import uuid

# Server configuration
BASE_URL = "http://localhost:8080"
API_BASE = f"{BASE_URL}/api"

def test_health():
    """Test health endpoint"""
    print("🔍 Testing health endpoint...")
    response = requests.get(f"{BASE_URL}/health")
    print(f"Status: {response.status_code}")
    if response.status_code == 200:
        print(f"Response: {response.json()}")
        return True
    return False

def create_test_user():
    """Create a test user in the database"""
    print("👤 Creating test user...")
    
    # Use a unique username to avoid conflicts
    import time
    timestamp = int(time.time())
    return {
        "id": 1,
        "username": f"testuser_{timestamp}",
        "email": f"test_{timestamp}@example.com",
        "keycloak_id": str(uuid.uuid4())
    }

def test_login(user_data):
    """Test login endpoint"""
    print("🔐 Testing login endpoint...")
    
    login_data = {
        "keycloak_id": user_data["keycloak_id"],
        "username": user_data["username"],
        "email": user_data["email"]
    }
    
    response = requests.post(
        f"{API_BASE}/auth/login",
        json=login_data,
        headers={"Content-Type": "application/json"}
    )
    
    print(f"Status: {response.status_code}")
    print(f"Response: {response.text}")
    
    if response.status_code == 200:
        return response.json()
    return None

def test_create_annotation(token):
    """Test creating an annotation"""
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
    
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    
    response = requests.post(
        f"{API_BASE}/annotations",
        json=annotation_data,
        headers=headers
    )
    
    print(f"Status: {response.status_code}")
    print(f"Response: {response.text}")
    
    if response.status_code == 201:
        return response.json()
    return None

def test_create_mask_group(token, annotation_id):
    """Test creating a mask group"""
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
    
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    
    response = requests.post(
        f"{API_BASE}/annotations/{annotation_id}/mask-groups",
        json=mask_group_data,
        headers=headers
    )
    
    print(f"Status: {response.status_code}")
    print(f"Response: {response.text}")
    
    if response.status_code == 201:
        return response.json()
    return None

def test_list_mask_groups(token, annotation_id):
    """Test listing mask groups"""
    print("📋 Testing mask group listing...")
    
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    
    response = requests.get(
        f"{API_BASE}/annotations/{annotation_id}/mask-groups",
        headers=headers
    )
    
    print(f"Status: {response.status_code}")
    print(f"Response: {response.text}")
    
    if response.status_code == 200:
        return response.json()
    return None

def main():
    """Main test function"""
    print("🚀 Starting Mask Group API Test")
    print("=" * 50)
    
    # Test health
    if not test_health():
        print("❌ Health check failed")
        return
    
    # Create test user
    user_data = create_test_user()
    
    # Test login
    login_response = test_login(user_data)
    if not login_response:
        print("❌ Login failed")
        return
    
    token = login_response["token"]
    print(f"✅ Login successful, token: {token[:20]}...")
    
    # Test annotation creation
    annotation = test_create_annotation(token)
    if not annotation:
        print("❌ Annotation creation failed")
        return
    
    annotation_id = annotation["id"]
    print(f"✅ Annotation created with ID: {annotation_id}")
    
    # Test mask group creation
    mask_group = test_create_mask_group(token, annotation_id)
    if not mask_group:
        print("❌ Mask group creation failed")
        return
    
    print(f"✅ Mask group created with ID: {mask_group['id']}")
    
    # Test mask group listing
    mask_groups = test_list_mask_groups(token, annotation_id)
    if mask_groups:
        print(f"✅ Found {len(mask_groups)} mask groups")
    
    print("🎉 All tests completed successfully!")

if __name__ == "__main__":
    main()
