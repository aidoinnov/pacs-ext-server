#!/usr/bin/env python3
"""
QIDO-RS Series 조회 시 Study Description 포함 여부 확인
"""

import requests
import json
from requests.auth import HTTPBasicAuth

# 실제 PACS 서버 설정
QIDO_BASE_URL = "https://archive.pacs.ai-do.co.kr/iaid-pacs/aets/iAID_PACS/rs"
USERNAME = "pacsadmin"
PASSWORD = "HhL}qb(tl}?zJ4}("

def test_series_query_with_study_uid():
    """
    Study UID와 Series UID를 모두 파라미터로 전달했을 때
    응답에 StudyDescription이 포함되는지 확인
    """
    print("=" * 80)
    print("테스트 1: /studies/{studyUID}/series 엔드포인트로 조회")
    print("=" * 80)

    # 먼저 실제 데이터 하나 가져오기
    print("\n📋 Step 1: 실제 Study 하나 조회...")
    study_list_url = f"{QIDO_BASE_URL}/studies"
    study_list_params = {"limit": 1}

    try:
        resp = requests.get(
            study_list_url,
            params=study_list_params,
            auth=HTTPBasicAuth(USERNAME, PASSWORD),
            headers={"Accept": "application/dicom+json"},
            verify=False
        )

        if resp.status_code != 200:
            print(f"❌ Study 목록 조회 실패: {resp.status_code}")
            return

        studies = resp.json()
        if not studies:
            print("❌ Study가 없습니다")
            return

        study = studies[0]
        study_uid = study.get("0020000D", {}).get("Value", [None])[0]
        study_desc = study.get("00081030", {}).get("Value", [None])[0]

        print(f"✅ Study UID: {study_uid}")
        print(f"✅ Study Description: {study_desc}")

    except Exception as e:
        print(f"❌ 에러: {e}")
        return

    # 이제 해당 Study의 Series 조회
    print(f"\n📋 Step 2: Study의 Series 목록 조회...")
    series_list_url = f"{QIDO_BASE_URL}/studies/{study_uid}/series"
    params = {"limit": 1}
    
    print(f"\n📡 요청 URL: {series_list_url}")
    print(f"📋 파라미터: {params}")

    try:
        response = requests.get(
            series_list_url,
            params=params,
            auth=HTTPBasicAuth(USERNAME, PASSWORD),
            headers={"Accept": "application/dicom+json"},
            verify=False
        )

        print(f"\n✅ 응답 상태: {response.status_code}")

        if response.status_code == 200:
            data = response.json()

            if data and len(data) > 0:
                series = data[0]
                series_uid = series.get("0020000E", {}).get("Value", [None])[0]

                print(f"\n📊 Series UID: {series_uid}")
                print("\n📊 응답에 포함된 태그들:")
                for tag in sorted(series.keys())[:30]:
                    value = series[tag].get("Value", [None])[0] if "Value" in series[tag] else None
                    print(f"  {tag}: {value}")

                # StudyDescription (0008,1030) 확인
                study_desc_tag = "00081030"
                if study_desc_tag in series:
                    series_study_desc = series[study_desc_tag].get("Value", [None])[0]
                    print(f"\n✅✅✅ StudyDescription 포함됨: {series_study_desc}")
                    print(f"원본 Study Description: {study_desc}")
                    print(f"일치 여부: {series_study_desc == study_desc}")
                else:
                    print(f"\n❌❌❌ StudyDescription (0008,1030) 없음!")
                    print("→ /studies/{{uid}}/series 엔드포인트도 Study 속성을 포함하지 않음")
            else:
                print("\n⚠️  응답 데이터가 비어있음")
        else:
            print(f"\n❌ 요청 실패: {response.text}")

    except Exception as e:
        print(f"\n❌ 에러 발생: {e}")


def test_series_query_without_study_uid():
    """
    Series UID만으로 조회했을 때
    응답에 StudyDescription이 포함되는지 확인
    """
    print("\n" + "=" * 80)
    print("테스트 2: /series 엔드포인트로 조회 (Study UID 없이)")
    print("=" * 80)

    url = f"{QIDO_BASE_URL}/series"
    params = {"limit": 1}
    
    print(f"\n📡 요청 URL: {url}")
    print(f"📋 파라미터: {params}")
    
    try:
        response = requests.get(
            url,
            params=params,
            auth=HTTPBasicAuth(USERNAME, PASSWORD),
            headers={"Accept": "application/dicom+json"},
            verify=False
        )

        print(f"\n✅ 응답 상태: {response.status_code}")

        if response.status_code == 200:
            data = response.json()

            if data and len(data) > 0:
                series = data[0]
                series_uid = series.get("0020000E", {}).get("Value", [None])[0]

                print(f"\n📊 Series UID: {series_uid}")

                # StudyDescription (0008,1030) 확인
                study_desc_tag = "00081030"
                if study_desc_tag in series:
                    study_desc = series[study_desc_tag].get("Value", [None])[0]
                    print(f"\n✅✅✅ StudyDescription 포함됨: {study_desc}")
                else:
                    print(f"\n❌❌❌ StudyDescription (0008,1030) 없음!")
                    print("→ /series 엔드포인트는 Series 레벨 속성만 반환")

                # StudyInstanceUID 확인
                study_uid_tag = "0020000D"
                if study_uid_tag in series:
                    study_uid = series[study_uid_tag].get("Value", [None])[0]
                    print(f"📌 StudyInstanceUID: {study_uid}")

            else:
                print("\n⚠️  응답 데이터가 비어있음")
        else:
            print(f"\n❌ 요청 실패: {response.text}")

    except Exception as e:
        print(f"\n❌ 에러 발생: {e}")


def test_includefield_parameter():
    """
    includefield 파라미터로 StudyDescription 요청
    """
    print("\n" + "=" * 80)
    print("테스트 3: includefield 파라미터 사용")
    print("=" * 80)

    url = f"{QIDO_BASE_URL}/series"
    params = {
        "limit": 1,
        "includefield": "00081030"  # StudyDescription
    }

    print(f"\n📡 요청 URL: {url}")
    print(f"📋 파라미터: {params}")

    try:
        response = requests.get(
            url,
            params=params,
            auth=HTTPBasicAuth(USERNAME, PASSWORD),
            headers={"Accept": "application/dicom+json"},
            verify=False
        )

        print(f"\n✅ 응답 상태: {response.status_code}")

        if response.status_code == 200:
            data = response.json()

            if data and len(data) > 0:
                series = data[0]

                # StudyDescription (0008,1030) 확인
                study_desc_tag = "00081030"
                if study_desc_tag in series:
                    study_desc = series[study_desc_tag].get("Value", [None])[0]
                    print(f"\n✅✅✅ includefield 작동! StudyDescription: {study_desc}")
                else:
                    print(f"\n❌❌❌ includefield 파라미터가 작동하지 않음")

            else:
                print("\n⚠️  응답 데이터가 비어있음")
        else:
            print(f"\n❌ 요청 실패: {response.text}")

    except Exception as e:
        print(f"\n❌ 에러 발생: {e}")


if __name__ == "__main__":
    import urllib3
    urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

    print("\n🔍 QIDO-RS Series 조회 시 Study Description 포함 여부 테스트\n")
    print("🌐 PACS 서버: https://archive.pacs.ai-do.co.kr\n")

    test_series_query_with_study_uid()
    test_series_query_without_study_uid()
    test_includefield_parameter()

    print("\n" + "=" * 80)
    print("📝 최종 결론")
    print("=" * 80)

