#!/usr/bin/env python3
"""
마스크 업로드 워크플로우 테스트 스크립트
"""

import requests
import json
import time
import hashlib
import os

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
        print(f"Response: {response.text}")
        
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
        "group_name": "Upload Test Mask Group",
        "model_name": "Test Model",
        "version": "1.0.0",
        "modality": "CT",
        "slice_count": 3,
        "mask_type": "segmentation",
        "description": "Test mask group for upload workflow testing"
    }
    
    headers = {"Authorization": f"Bearer {token}"}
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/annotations/{annotation_id}/mask-groups",
            json=mask_group_data,
            headers=headers
        )
        print(f"Status: {response.status_code}")
        print(f"Response: {response.text}")
        
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

def generate_upload_url(annotation_id, group_id, token):
    """업로드 URL 생성"""
    print("🔗 Testing upload URL generation...")
    
    upload_data = {
        "mask_group_id": group_id,
        "filename": "test_mask_slice_001.png",
        "mime_type": "image/png",
        "expires_in": 3600
    }
    
    headers = {"Authorization": f"Bearer {token}"}
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/annotations/{annotation_id}/mask-groups/{group_id}/upload-url",
            json=upload_data,
            headers=headers
        )
        print(f"Status: {response.status_code}")
        print(f"Response: {response.text}")
        
        if response.status_code == 200:
            upload_info = response.json()
            print(f"✅ Upload URL generated successfully")
            print(f"   File Key: {upload_info.get('file_key', 'N/A')}")
            print(f"   Expires At: {upload_info.get('expires_at', 'N/A')}")
            return upload_info
        else:
            print("❌ Upload URL generation failed")
            return None
    except Exception as e:
        print(f"❌ Upload URL generation error: {e}")
        return None

def simulate_file_upload(upload_info):
    """파일 업로드 시뮬레이션"""
    print("📤 Simulating file upload to S3...")
    
    if not upload_info or 'upload_url' not in upload_info:
        print("❌ No upload URL available")
        return None
    
    # 가짜 PNG 파일 데이터 생성 (실제로는 S3에 업로드)
    fake_png_data = b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x02\x00\x00\x00\x02\x00\x08\x02\x00\x00\x00\xff\x80\x02\x00\x00\x00\x00IEND\xaeB`\x82'
    
    try:
        # 실제로는 S3에 PUT 요청을 보내야 하지만, 여기서는 시뮬레이션
        print(f"   Uploading to: {upload_info['upload_url'][:50]}...")
        print(f"   File size: {len(fake_png_data)} bytes")
        
        # 파일 해시 계산
        file_hash = hashlib.sha256(fake_png_data).hexdigest()
        print(f"   File hash: {file_hash}")
        
        return {
            "file_key": upload_info.get('file_path'),
            "file_size": len(fake_png_data),
            "checksum": f"sha256:{file_hash}",
            "upload_success": True
        }
    except Exception as e:
        print(f"❌ File upload simulation error: {e}")
        return None

def complete_upload(annotation_id, group_id, upload_result, token):
    """업로드 완료 확인"""
    print("✅ Testing upload completion...")
    
    if not upload_result or not upload_result.get('upload_success'):
        print("❌ No upload result available")
        return None
    
    complete_data = {
        "mask_group_id": group_id,
        "slice_count": 1,
        "labels": ["liver"],
        "uploaded_files": [upload_result['file_key']]
    }
    
    headers = {"Authorization": f"Bearer {token}"}
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/annotations/{annotation_id}/mask-groups/{group_id}/complete-upload",
            json=complete_data,
            headers=headers
        )
        print(f"Status: {response.status_code}")
        print(f"Response: {response.text}")
        
        if response.status_code == 200:
            print("✅ Upload completion confirmed successfully")
            return response.json()
        else:
            print("❌ Upload completion failed")
            return None
    except Exception as e:
        print(f"❌ Upload completion error: {e}")
        return None

def create_mask_from_upload(annotation_id, group_id, upload_result, token):
    """업로드된 파일로 마스크 생성"""
    print("🎨 Creating mask from uploaded file...")
    
    if not upload_result:
        print("❌ No upload result available")
        return None
    
    mask_data = {
        "mask_group_id": group_id,
        "slice_index": 1,
        "sop_instance_uid": "1.2.3.4.5.6.7.8.9.13",
        "label_name": "liver",
        "file_path": upload_result['file_key'],
        "mime_type": "image/png",
        "file_size": upload_result['file_size'],
        "checksum": upload_result['checksum'],
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
        print(f"Response: {response.text}")
        
        if response.status_code == 201:
            mask_id = response.json()["id"]
            print(f"✅ Mask created from upload with ID: {mask_id}")
            return mask_id
        else:
            print("❌ Mask creation from upload failed")
            return None
    except Exception as e:
        print(f"❌ Mask creation from upload error: {e}")
        return None

def test_batch_upload_workflow(annotation_id, group_id, token):
    """배치 업로드 워크플로우 테스트"""
    print("📦 Testing batch upload workflow...")
    
    # 여러 파일에 대한 업로드 URL 생성
    files_to_upload = [
        {"filename": "slice_001.png", "mime_type": "image/png"},
        {"filename": "slice_002.png", "mime_type": "image/png"},
        {"filename": "slice_003.png", "mime_type": "image/png"}
    ]
    
    upload_urls = []
    for i, file_info in enumerate(files_to_upload):
        print(f"   Generating upload URL for {file_info['filename']}...")
        
        upload_data = {
            "mask_group_id": group_id,
            "filename": file_info["filename"],
            "mime_type": file_info["mime_type"],
            "expires_in": 3600
        }
        
        headers = {"Authorization": f"Bearer {token}"}
        
        try:
            response = requests.post(
                f"{BASE_URL}/api/annotations/{annotation_id}/mask-groups/{group_id}/upload-url",
                json=upload_data,
                headers=headers
            )
            
            if response.status_code == 200:
                upload_info = response.json()
                upload_urls.append(upload_info)
                print(f"   ✅ Upload URL generated for {file_info['filename']}")
            else:
                print(f"   ❌ Failed to generate upload URL for {file_info['filename']}")
        except Exception as e:
            print(f"   ❌ Error generating upload URL for {file_info['filename']}: {e}")
    
    print(f"✅ Generated {len(upload_urls)} upload URLs")
    return upload_urls

def main():
    """메인 테스트 함수"""
    print("🚀 Starting Mask Upload Workflow Test")
    print("=" * 60)
    
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
    
    # 5. 업로드 URL 생성
    upload_info = generate_upload_url(annotation_id, group_id, token)
    if not upload_info:
        print("❌ Upload URL generation failed, exiting")
        return
    
    # 6. 파일 업로드 시뮬레이션
    upload_result = simulate_file_upload(upload_info)
    if not upload_result:
        print("❌ File upload simulation failed, exiting")
        return
    
    # 7. 업로드 완료 확인
    complete_result = complete_upload(annotation_id, group_id, upload_result, token)
    if not complete_result:
        print("❌ Upload completion failed, exiting")
        return
    
    # 8. 업로드된 파일로 마스크 생성
    mask_id = create_mask_from_upload(annotation_id, group_id, upload_result, token)
    if not mask_id:
        print("❌ Mask creation from upload failed, exiting")
        return
    
    # 9. 배치 업로드 워크플로우 테스트
    batch_upload_urls = test_batch_upload_workflow(annotation_id, group_id, token)
    
    print("\n🎉 All upload workflow tests completed!")
    print("=" * 60)
    print(f"📊 Summary:")
    print(f"   - Annotation ID: {annotation_id}")
    print(f"   - Mask Group ID: {group_id}")
    print(f"   - Created Mask ID: {mask_id}")
    print(f"   - Batch Upload URLs: {len(batch_upload_urls)}")

if __name__ == "__main__":
    main()
