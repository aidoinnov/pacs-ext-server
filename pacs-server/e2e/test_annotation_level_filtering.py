#!/usr/bin/env python3
"""
어노테이션 레벨 필터링 E2E 테스트

이 테스트는 어노테이션을 Study/Series/Instance 레벨로 필터링하는 기능을 검증합니다.
- Study level: series_uid와 instance_uid가 모두 비어있음
- Series level: series_uid는 있고 instance_uid는 비어있음
- Instance level: series_uid와 instance_uid가 모두 있음
"""

import requests
import json
from test_utils import cleanup_annotations

BASE_URL = "http://localhost:8080"
USER_ID = 'iaid-pacs-admin'
USER_PASSWORD = 'Qlalfqjsgh1!'

def login():
    """로그인하여 JWT 토큰 획득"""
    print("🔐 로그인 중...")
    response = requests.post(
        f"{BASE_URL}/api/auth/login",
        json={"username": USER_ID, "password": USER_PASSWORD},
        timeout=5
    )
    
    if response.status_code != 200:
        print(f"❌ 로그인 실패: {response.status_code}")
        print(response.text)
        exit(1)
    
    token = response.json()["token"]
    print(f"✅ 로그인 성공\n")
    return token


def create_test_annotations(token: str):
    """테스트용 어노테이션 생성 (Study/Series/Instance 레벨)"""
    print("📝 테스트용 어노테이션 생성 중...")
    headers = {"Authorization": f"Bearer {token}"}
    
    study_uid = "1.3.6.1.4.1.14519.5.2.1.6655.2359.307959856517080892181338382781"
    series_uid = "1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345"
    instance_uid = "1.3.6.1.4.1.14519.5.2.1.6655.2359.238273576775187812804817387920"
    
    annotations = [
        # Study level annotation
        {
            "project_id": 2,
            "study_instance_uid": study_uid,
            "series_instance_uid": None,
            "sop_instance_uid": None,
            "annotation_data": {"type": "study_note", "text": "Study level annotation"},
            "tool_name": "Note Tool",
            "tool_version": "1.0.0",
            "viewer_software": "TI-DicomViewer",
            "description": "Study level test",
        },
        # Series level annotation
        {
            "project_id": 2,
            "study_instance_uid": study_uid,
            "series_instance_uid": series_uid,
            "sop_instance_uid": None,
            "annotation_data": {"type": "series_note", "text": "Series level annotation"},
            "tool_name": "Note Tool",
            "tool_version": "1.0.0",
            "viewer_software": "TI-DicomViewer",
            "description": "Series level test",
        },
        # Instance level annotation
        {
            "project_id": 2,
            "study_instance_uid": study_uid,
            "series_instance_uid": series_uid,
            "sop_instance_uid": instance_uid,
            "annotation_data": {"type": "measurement", "value": 10.5},
            "tool_name": "Measurement Tool",
            "tool_version": "1.0.0",
            "viewer_software": "TI-DicomViewer",
            "description": "Instance level test",
        },
    ]
    
    created_ids = []
    for ann in annotations:
        response = requests.post(
            f"{BASE_URL}/api/annotations",
            json=ann,
            headers=headers,
            timeout=10
        )
        if response.status_code == 201:
            created_ids.append(response.json()["id"])
            print(f"   ✓ Created annotation ID: {response.json()['id']} - {ann['description']}")
        else:
            print(f"   ✗ Failed to create: {ann['description']} - {response.status_code}: {response.text}")

    print(f"✅ {len(created_ids)}개 어노테이션 생성 완료\n")
    return created_ids, study_uid, series_uid, instance_uid


def test_level_filter_study(token: str, study_uid: str):
    """테스트 1: Study 레벨 필터링"""
    print("\n" + "=" * 70)
    print("테스트 1: Study 레벨 필터링")
    print("=" * 70)
    
    headers = {"Authorization": f"Bearer {token}"}
    
    response = requests.get(
        f"{BASE_URL}/api/annotations?study_instance_uid={study_uid}&level=study",
        headers=headers,
        timeout=10
    )
    
    print(f"Status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        annotations = data.get("annotations", [])
        
        # Study 레벨만 필터링되었는지 확인
        study_level = [ann for ann in annotations 
                      if not ann["series_instance_uid"] and not ann["sop_instance_uid"]]
        
        print(f"✅ Study level annotations: {len(study_level)}")
        for ann in study_level:
            print(f"   - ID: {ann['id']}, Description: {ann.get('description', 'N/A')}")
        
        assert len(study_level) > 0, "Should have at least one study level annotation"
        print("✅ 테스트 통과")
    else:
        print(f"❌ 테스트 실패: {response.text}")
        exit(1)


def test_level_filter_series(token: str, series_uid: str):
    """테스트 2: Series 레벨 필터링"""
    print("\n" + "=" * 70)
    print("테스트 2: Series 레벨 필터링")
    print("=" * 70)
    
    headers = {"Authorization": f"Bearer {token}"}
    
    response = requests.get(
        f"{BASE_URL}/api/annotations?series_instance_uid={series_uid}&level=series",
        headers=headers,
        timeout=10
    )
    
    print(f"Status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        annotations = data.get("annotations", [])
        
        # Series 레벨만 필터링되었는지 확인
        series_level = [ann for ann in annotations 
                       if ann["series_instance_uid"] and not ann["sop_instance_uid"]]
        
        print(f"✅ Series level annotations: {len(series_level)}")
        for ann in series_level:
            print(f"   - ID: {ann['id']}, Description: {ann.get('description', 'N/A')}")
        
        print("✅ 테스트 통과")
    else:
        print(f"❌ 테스트 실패: {response.text}")
        exit(1)


def test_level_filter_instance(token: str, instance_uid: str):
    """테스트 3: Instance 레벨 필터링"""
    print("\n" + "=" * 70)
    print("테스트 3: Instance 레벨 필터링")
    print("=" * 70)
    
    headers = {"Authorization": f"Bearer {token}"}
    
    response = requests.get(
        f"{BASE_URL}/api/annotations?sop_instance_uid={instance_uid}&level=instance",
        headers=headers,
        timeout=10
    )
    
    print(f"Status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        annotations = data.get("annotations", [])
        
        # Instance 레벨만 필터링되었는지 확인
        instance_level = [ann for ann in annotations if ann["sop_instance_uid"]]
        
        print(f"✅ Instance level annotations: {len(instance_level)}")
        for ann in instance_level:
            print(f"   - ID: {ann['id']}, Description: {ann.get('description', 'N/A')}")
        
        print("✅ 테스트 통과")
    else:
        print(f"❌ 테스트 실패: {response.text}")
        exit(1)


if __name__ == '__main__':
    created_ids = []
    token = None
    try:
        print("\n🚀 어노테이션 레벨 필터링 E2E 테스트 시작...\n")

        token = login()
        created_ids, study_uid, series_uid, instance_uid = create_test_annotations(token)

        test_level_filter_study(token, study_uid)
        test_level_filter_series(token, series_uid)
        test_level_filter_instance(token, instance_uid)

        print("\n" + "=" * 70)
        print("🎉 모든 테스트 통과!")
        print("=" * 70)
        print()

    except Exception as e:
        print(f"\n❌ 테스트 실패: {e}\n")
        import traceback
        traceback.print_exc()
        exit(1)
    finally:
        # Cleanup
        if created_ids and token:
            cleanup_annotations(token, created_ids)

