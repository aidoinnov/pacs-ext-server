#!/usr/bin/env python3
"""
Annotation Snapshot Upload E2E Test
어노테이션 스냅샷 이미지 업로드 기능 전체 워크플로우 테스트

테스트 구조:
1. 사전준비: 테스트 사용자 생성, 프로젝트 생성
2. 본 테스트: 스냅샷 업로드 및 조회
3. 클린업: 생성한 데이터 정리
"""

import requests
import os
import io
import sys
from pathlib import Path

# PIL 패키지 확인 및 설치 안내
try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    print("❌ PIL (Pillow) 패키지가 설치되지 않았습니다.")
    print("다음 명령어로 설치하세요:")
    print("  pip install Pillow")
    sys.exit(1)

from test_common import (
    BASE_URL,
    get_headers,
    get_admin_token,
    create_test_user,
    create_test_project,
    add_user_to_project,
    cleanup_project,
    cleanup_user,
    health_check
)

# 테스트 데이터 저장용
test_context = {
    "user": None,
    "project_id": None,
    "annotation_id": None,
    "admin_token": None
}

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
def test_annotation_snapshot_workflow(project_id: int, token: str) -> int:
    """스냅샷 업로드 전체 워크플로우 테스트

    Args:
        project_id: 테스트 프로젝트 ID
        token: JWT 토큰

    Returns:
        생성된 어노테이션 ID
    """
    headers = get_headers(token)

    print("=" * 70)
    print("📋 Annotation Snapshot Upload E2E Test")
    print("=" * 70)

    # 1. 어노테이션 생성
    print("\n1️⃣  어노테이션 생성 중...")
    annotation_data = {
        "project_id": project_id,
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

    return annotation_id


def setup():
    """사전준비: 테스트 사용자 및 프로젝트 생성"""
    print("\n" + "=" * 70)
    print("🔧 사전준비: 테스트 환경 설정")
    print("=" * 70)

    # 1. 헬스 체크
    print("\n1️⃣  서버 헬스 체크...")
    if not health_check():
        print("❌ 서버가 응답하지 않습니다.")
        sys.exit(1)
    print("✅ 서버 정상")

    # 2. 관리자 토큰 획득
    print("\n2️⃣  관리자 로그인...")
    admin_token = get_admin_token()
    if not admin_token:
        print("❌ 관리자 로그인 실패")
        sys.exit(1)
    print("✅ 관리자 로그인 성공")
    test_context["admin_token"] = admin_token

    # 3. 테스트 사용자 생성
    print("\n3️⃣  테스트 사용자 생성...")
    user = create_test_user("snapshot_test")
    if not user:
        print("❌ 테스트 사용자 생성 실패")
        sys.exit(1)
    print(f"✅ 테스트 사용자 생성 성공: {user['username']} (ID: {user['user_id']})")
    test_context["user"] = user

    # 4. 테스트 프로젝트 생성
    print("\n4️⃣  테스트 프로젝트 생성...")
    project_id = create_test_project(user["token"], "snapshot_test")
    if not project_id:
        print("❌ 테스트 프로젝트 생성 실패")
        sys.exit(1)
    print(f"✅ 테스트 프로젝트 생성 성공: ID {project_id}")
    test_context["project_id"] = project_id

    # 5. 사용자를 프로젝트에 추가
    print("\n5️⃣  사용자를 프로젝트에 추가...")
    if not add_user_to_project(user["user_id"], project_id, user["token"]):
        print("⚠️  사용자 추가 실패 (이미 멤버일 수 있음)")
    else:
        print("✅ 사용자 추가 성공")

    print("\n" + "=" * 70)
    print("✅ 사전준비 완료!")
    print("=" * 70)


def cleanup():
    """클린업: 생성한 데이터 정리"""
    print("\n" + "=" * 70)
    print("🧹 클린업: 테스트 데이터 정리")
    print("=" * 70)

    # 1. 프로젝트 삭제
    if test_context["project_id"] and test_context["user"]:
        print(f"\n1️⃣  프로젝트 삭제 (ID: {test_context['project_id']})...")
        if cleanup_project(test_context["project_id"], test_context["user"]["token"]):
            print("✅ 프로젝트 삭제 성공")
        else:
            print("⚠️  프로젝트 삭제 실패")

    # 2. 사용자 삭제
    if test_context["user"] and test_context["admin_token"]:
        print(f"\n2️⃣  사용자 삭제 (ID: {test_context['user']['user_id']})...")
        if cleanup_user(test_context["user"]["user_id"], test_context["admin_token"]):
            print("✅ 사용자 삭제 성공")
        else:
            print("⚠️  사용자 삭제 실패")

    print("\n" + "=" * 70)
    print("✅ 클린업 완료!")
    print("=" * 70)


if __name__ == '__main__':
    try:
        print("\n🚀 Annotation Snapshot E2E Test 시작...\n")

        # 사전준비
        setup()

        # 본 테스트
        annotation_id = test_annotation_snapshot_workflow(
            test_context["project_id"],
            test_context["user"]["token"]
        )
        test_context["annotation_id"] = annotation_id

        print("\n✅ 모든 테스트 통과!\n")

    except AssertionError as e:
        print(f"\n❌ 검증 실패: {e}\n")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ 테스트 실패: {e}\n")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    finally:
        # 클린업
        cleanup()
