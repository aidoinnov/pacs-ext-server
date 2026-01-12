#!/usr/bin/env python3
"""
Annotation Snapshot Upload E2E Test
어노테이션 스냅샷 이미지 업로드 기능 전체 워크플로우 테스트
"""

import requests
import os
import io
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

BASE_URL = "http://localhost:8080"
USER_ID = 'iaid-pacs-admin'
USER_PASSWORD = 'Qlalfqjsgh1!'

def login():
    """로그인하여 JWT 토큰 획득"""
    print("🔐 로그인 중...")
    login_resp = requests.post(
        f"{BASE_URL}/api/auth/login",
        json={"username": USER_ID, "password": USER_PASSWORD},
        timeout=5
    )

    if login_resp.status_code != 200:
        print(f"❌ 로그인 실패: {login_resp.status_code}")
        print(login_resp.text)
        exit(1)

    token = login_resp.json()["token"]
    if token is None:
        print("❌ 토큰이 없습니다")
        exit(1)
    print(f"✅ 로그인 성공 (token length: {len(token)})\n")
    return token

def create_test_image():
    """테스트용 PNG 이미지 생성"""
    # 800x600 크기의 이미지 생성
    img = Image.new('RGB', (800, 600), color='white')
    draw = ImageDraw.Draw(img)
    
    # 배경 그라데이션
    for y in range(600):
        color = int(255 * (1 - y / 600))
        draw.rectangle([(0, y), (800, y+1)], fill=(color, color, 255))
    
    # 원 그리기 (어노테이션 시뮬레이션)
    draw.ellipse([200, 150, 400, 350], outline='red', width=3)
    
    # 사각형 그리기
    draw.rectangle([450, 200, 650, 400], outline='green', width=3)
    
    # 텍스트 추가
    try:
        # 시스템 폰트 사용 시도
        font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 24)
    except:
        # 폰트 로드 실패 시 기본 폰트 사용
        font = ImageFont.load_default()
    
    draw.text((250, 450), "Test Annotation Snapshot", fill='black', font=font)
    draw.text((250, 480), "Circle: Lesion Area", fill='red', font=font)
    draw.text((250, 510), "Rectangle: ROI", fill='green', font=font)
    
    # 바이트 스트림으로 변환
    img_byte_arr = io.BytesIO()
    img.save(img_byte_arr, format='PNG')
    img_byte_arr.seek(0)
    
    return img_byte_arr.getvalue()
def test_annotation_snapshot_workflow(token: str):
    """스냅샷 업로드 전체 워크플로우 테스트"""
    headers = {"Authorization": f"Bearer {token}"}

    print("=" * 70)
    print("📋 Annotation Snapshot Upload E2E Test")
    print("=" * 70)

    # 1. 어노테이션 생성
    print("\n1️⃣  어노테이션 생성 중...")
    annotation_data = {
        "project_id": 2,
        "study_instance_uid": "1.3.6.1.4.1.14519.5.2.1.6655.2359.321111757620390201880556376661",
        "series_instance_uid": "1.3.6.1.4.1.14519.5.2.1.6655.2359.260616660471925521837323152953",
        "sop_instance_uid": "1.3.6.1.4.1.14519.5.2.1.6655.2359.217230834888240455035945707219",
        "annotation_data": {
            "type": "circle",
            "x": 300,
            "y": 250,
            "radius": 100,
            "color": "#FF0000",
            "label": "Lesion Area"
        },
        "tool_name": "Circle Tool",
        "tool_version": "2.1.0",
        "viewer_software": "OHIF Viewer",
        "description": "테스트용 병변 영역 어노테이션",
        "label": "Tumor",
        "measurement_values": [
            {"id": "m1", "type": "diameter", "values": [42.3], "unit": "mm"}
        ]
    }

    # 개발 모드에서 user_id 헤더 추가
    dev_headers = headers.copy()
    dev_headers["X-User-ID"] = "1"  # iaid-pacs-admin의 user_id

    response = requests.post(
        f"{BASE_URL}/api/annotations",
        json=annotation_data,
        headers=dev_headers,
        timeout=30
    )

    print(f"   Status: {response.status_code}")
    if response.status_code != 201:
        print(f"   ❌ 어노테이션 생성 실패")
        print(f"   Response: {response.text}")
        exit(1)

    annotation = response.json()
    annotation_id = annotation["id"]
    print(f"   ✅ 어노테이션 생성 성공!")
    print(f"   - Annotation ID: {annotation_id}")
    print(f"   - Label: {annotation.get('label', 'N/A')}")
    print(f"   - Tool: {annotation.get('tool_name', 'N/A')}")

    # 2. 스냅샷 업로드 URL 요청
    print("\n2️⃣  스냅샷 업로드 URL 요청 중...")
    upload_request = {
        "filename": "snapshot_test_annotation.png",
        "mime_type": "image/png",
        "ttl_seconds": 600
    }

    response = requests.post(
        f"{BASE_URL}/api/annotations/{annotation_id}/snapshot/upload-url",
        json=upload_request,
        headers=dev_headers,
        timeout=30
    )

    print(f"   Status: {response.status_code}")
    if response.status_code != 200:
        print(f"   ❌ 업로드 URL 생성 실패")
        print(f"   URL: {BASE_URL}/api/annotations/{annotation_id}/snapshot/upload-url")
        print(f"   Response: {response.text}")
        print(f"   Headers: {response.headers}")
        exit(1)

    upload_data = response.json()
    print(f"   ✅ 업로드 URL 생성 성공!")
    print(f"   - Upload URL: {upload_data['upload_url'][:80]}...")
    print(f"   - Download URL: {upload_data['download_url'][:80]}...")
    print(f"   - Image Key: {upload_data['image_key']}")
    print(f"   - Expires in: {upload_data['expires_in']}s")
    print(f"   - Expires at: {upload_data['expires_at']}")

    # 3. 테스트 이미지 생성
    print("\n3️⃣  테스트 이미지 생성 중...")
    image_data = create_test_image()
    image_size = len(image_data)
    print(f"   ✅ 이미지 생성 완료!")
    print(f"   - Size: {image_size:,} bytes ({image_size / 1024:.2f} KB)")
    print(f"   - Format: PNG")
    print(f"   - Dimensions: 800x600")

    # 4. S3에 이미지 업로드
    print("\n4️⃣  S3에 이미지 업로드 중...")
    upload_response = requests.put(
        upload_data["upload_url"],
        data=image_data,
        headers={"Content-Type": "image/png"},
        timeout=30
    )

    print(f"   Status: {upload_response.status_code}")
    if upload_response.status_code not in [200, 204]:
        print(f"   ❌ S3 업로드 실패")
        print(f"   Response: {upload_response.text}")
        exit(1)

    print(f"   ✅ S3 업로드 성공!")

    # 5. 업로드 완료 알림
    print("\n5️⃣  업로드 완료 알림 중...")
    complete_request = {
        "image_key": upload_data["image_key"],
        "success": True
    }

    response = requests.post(
        f"{BASE_URL}/api/annotations/{annotation_id}/snapshot/complete-upload",
        json=complete_request,
        headers=dev_headers,
        timeout=60  # S3 다운로드 시간을 고려하여 60초로 증가
    )

    print(f"   Status: {response.status_code}")
    if response.status_code != 200:
        print(f"   ❌ 업로드 완료 처리 실패")
        print(f"   Response: {response.text}")
        exit(1)

    updated_annotation = response.json()
    print(f"   ✅ 업로드 완료 처리 성공!")
    print(f"   - Snapshot Image Key: {updated_annotation.get('snapshot_image_key', 'N/A')}")
    print(f"   - Snapshot Status: {updated_annotation.get('snapshot_status', 'N/A')}")
    print(f"   - Snapshot Uploaded At: {updated_annotation.get('snapshot_uploaded_at', 'N/A')}")

    # 6. 스냅샷 상태 조회
    print("\n6️⃣  스냅샷 상태 조회 중...")
    response = requests.get(
        f"{BASE_URL}/api/annotations/{annotation_id}/snapshot/status",
        headers=headers,
        timeout=30
    )

    print(f"   Status: {response.status_code}")
    if response.status_code != 200:
        print(f"   ⚠️  상태 조회 실패 (optional)")
        print(f"   Response: {response.text}")
    else:
        status_data = response.json()
        print(f"   ✅ 상태 조회 성공!")
        print(f"   - Annotation ID: {status_data.get('annotation_id', 'N/A')}")
        print(f"   - Image Key: {status_data.get('image_key', 'N/A')}")
        print(f"   - Status: {status_data.get('status', 'N/A')}")
        print(f"   - Uploaded At: {status_data.get('uploaded_at', 'N/A')}")

    # 7. 어노테이션 조회하여 스냅샷 정보 확인
    print("\n7️⃣  어노테이션 조회하여 스냅샷 정보 확인 중...")
    response = requests.get(
        f"{BASE_URL}/api/annotations/{annotation_id}",
        headers=headers,
        timeout=30
    )

    print(f"   Status: {response.status_code}")
    if response.status_code != 200:
        print(f"   ❌ 어노테이션 조회 실패")
        print(f"   Response: {response.text}")
        exit(1)

    final_annotation = response.json()
    print(f"   ✅ 어노테이션 조회 성공!")
    print(f"   - ID: {final_annotation.get('id', 'N/A')}")
    print(f"   - Label: {final_annotation.get('label', 'N/A')}")
    print(f"   - Snapshot Image Key: {final_annotation.get('snapshot_image_key', 'N/A')}")
    print(f"   - Snapshot Status: {final_annotation.get('snapshot_status', 'N/A')}")

    # 검증
    assert final_annotation.get('snapshot_image_key') == upload_data['image_key'], \
        "스냅샷 이미지 키가 일치하지 않습니다"
    assert final_annotation.get('snapshot_status') == 'completed', \
        f"스냅샷 상태가 'completed'가 아닙니다: {final_annotation.get('snapshot_status')}"

    print("\n" + "=" * 70)
    print("🎉 모든 테스트 통과!")
    print("=" * 70)
    print(f"\n📊 테스트 요약:")
    print(f"   - Annotation ID: {annotation_id}")
    print(f"   - Image Key: {upload_data['image_key']}")
    print(f"   - Image Size: {image_size:,} bytes")
    print(f"   - Status: {final_annotation.get('snapshot_status')}")
    print(f"   - Uploaded At: {final_annotation.get('snapshot_uploaded_at', 'N/A')}")
    print()


if __name__ == '__main__':
    try:
        print("\n🚀 Annotation Snapshot E2E Test 시작...\n")
        token = login()
        test_annotation_snapshot_workflow(token)
        print("✅ 테스트 완료!\n")
    except AssertionError as e:
        print(f"\n❌ 검증 실패: {e}\n")
        exit(1)
    except Exception as e:
        print(f"\n❌ 테스트 실패: {e}\n")
        import traceback
        traceback.print_exc()
        exit(1)
