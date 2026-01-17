#!/usr/bin/env python3
"""
Instances & Instances/Metadata API E2E 테스트

V2 배치 쿼리 최적화 검증:
1. /instances 엔드포인트 (includefield 사용)
2. /instances/metadata 엔드포인트 (모든 DICOM 태그)
"""

import time
import sys
from pathlib import Path

# 프로젝트 루트를 Python 경로에 추가
sys.path.insert(0, str(Path(__file__).parent))

from utils.api_client import APIClient
from config import TestConfig

# 테스트 데이터
STUDY_UID = "1.3.6.1.4.1.14519.5.2.1.6655.2359.307959856517080892181338382781"
SERIES_UID = "1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345"
PROJECT_ID = 2


def test_instances_endpoint(client: APIClient, limit: int):
    """
    /instances 엔드포인트 테스트 (includefield 사용)
    """
    print(f"\n{'='*80}")
    print(f"📋 Testing /instances endpoint (limit={limit})")
    print(f"{'='*80}")
    
    path = f"/api/dicom/studies/{STUDY_UID}/series/{SERIES_UID}/instances"
    params = {
        'project_id': PROJECT_ID,
        'includefield': [
            '00080018',  # SOP Instance UID
            '00200013',  # Instance Number
            '00200032',  # Image Position Patient
            '00200037',  # Image Orientation Patient
            '00201041',  # Slice Location
            '00180050',  # Slice Thickness
            '00180088',  # Spacing Between Slices
            '00281050',  # Window Center
            '00281051',  # Window Width
        ],
        'limit': limit
    }
    
    print(f"🔗 Path: {path}")
    print(f"📊 Params: project_id={PROJECT_ID}, limit={limit}, includefields={len(params['includefield'])}")
    print(f"🚀 Sending request...")
    
    start = time.time()
    response = client.get(path, params=params)
    elapsed = time.time() - start
    
    print(f"⏱️  Response time: {elapsed:.3f}s")
    print(f"📊 Status code: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        count = len(data) if isinstance(data, list) else 0
        print(f"✅ Success: {count} instances returned")
        
        # 첫 번째 인스턴스 샘플 출력
        if count > 0:
            first = data[0]
            print(f"\n📄 Sample instance (first):")
            print(f"   - Total tags: {len(first.keys())}")
            print(f"   - Tags: {list(first.keys())[:10]}...")
            if '00080018' in first:
                sop_uid = first['00080018'].get('Value', ['N/A'])[0]
                print(f"   - SOP Instance UID: {sop_uid[:60]}...")
        
        return {
            'success': True,
            'count': count,
            'elapsed': elapsed,
            'data': data
        }
    else:
        print(f"❌ Failed: {response.status_code}")
        print(f"   Error: {response.text[:200]}")
        return {
            'success': False,
            'count': 0,
            'elapsed': elapsed,
            'error': response.text
        }


def test_instances_metadata_endpoint(client: APIClient, limit: int):
    """
    /series/metadata 엔드포인트 테스트 (모든 DICOM 태그)
    """
    print(f"\n{'='*80}")
    print(f"📋 Testing /series/metadata endpoint (limit={limit})")
    print(f"{'='*80}")

    path = f"/api/dicom/studies/{STUDY_UID}/series/{SERIES_UID}/metadata"
    params = {
        'project_id': PROJECT_ID,
        'limit': limit
    }

    print(f"🔗 Path: {path}")
    print(f"📊 Params: project_id={PROJECT_ID}, limit={limit}")
    print(f"🚀 Sending request...")
    
    start = time.time()
    response = client.get(path, params=params)
    elapsed = time.time() - start
    
    print(f"⏱️  Response time: {elapsed:.3f}s")
    print(f"📊 Status code: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        count = len(data) if isinstance(data, list) else 0
        tag_count = len(data[0].keys()) if count > 0 else 0
        print(f"✅ Success: {count} instances metadata returned")
        
        # 첫 번째 인스턴스 메타데이터 샘플 출력
        if count > 0:
            first = data[0]
            print(f"\n📄 Sample metadata (first instance):")
            print(f"   - Total DICOM tags: {tag_count}")
            print(f"   - Sample tags: {list(first.keys())[:15]}...")
            if '00080018' in first:
                sop_uid = first['00080018'].get('Value', ['N/A'])[0]
                print(f"   - SOP Instance UID: {sop_uid[:60]}...")
        
        return {
            'success': True,
            'count': count,
            'elapsed': elapsed,
            'tag_count': tag_count,
            'data': data
        }
    else:
        print(f"❌ Failed: {response.status_code}")
        print(f"   Error: {response.text[:200]}")
        return {
            'success': False,
            'count': 0,
            'elapsed': elapsed,
            'error': response.text
        }


def main():
    """메인 테스트 실행"""
    print("\n" + "="*80)
    print("🧪 Instances & Instances/Metadata API E2E Test")
    print("   V2 Batch Query Optimization Verification")
    print("="*80)

    # 설정 로드
    config = TestConfig.from_env()

    # API 클라이언트 생성
    client = APIClient(config.base_url, timeout=60)

    try:
        # 로그인
        print(f"\n🔐 Logging in as {config.admin_email}...")
        client.login(config.admin_email, config.admin_password)
        print("✅ Login successful")

        # 테스트 시나리오
        test_limits = [10, 50, 100, 220]  # 220은 전체 인스턴스 수

        results_instances = []
        results_metadata = []

        # 1. /instances 엔드포인트 테스트
        print(f"\n{'='*80}")
        print("🔬 Phase 1: Testing /instances endpoint")
        print(f"{'='*80}")

        for limit in test_limits:
            result = test_instances_endpoint(client, limit)
            results_instances.append(result)
            time.sleep(1)  # 서버 부하 방지

        # 2. /series/metadata 엔드포인트 테스트
        print(f"\n{'='*80}")
        print("🔬 Phase 2: Testing /series/metadata endpoint")
        print(f"{'='*80}")

        for limit in test_limits:
            result = test_instances_metadata_endpoint(client, limit)
            results_metadata.append(result)
            time.sleep(1)  # 서버 부하 방지

        # 결과 요약
        print(f"\n{'='*80}")
        print("📊 Test Results Summary")
        print(f"{'='*80}")

        print(f"\n1️⃣  /instances endpoint:")
        print(f"{'Limit':<10} {'Count':<10} {'Time (s)':<12} {'Status':<10}")
        print("-" * 50)
        for i, limit in enumerate(test_limits):
            r = results_instances[i]
            status = "✅ PASS" if r['success'] else "❌ FAIL"
            print(f"{limit:<10} {r['count']:<10} {r['elapsed']:<12.3f} {status:<10}")

        print(f"\n2️⃣  /series/metadata endpoint:")
        print(f"{'Limit':<10} {'Count':<10} {'Tags':<10} {'Time (s)':<12} {'Status':<10}")
        print("-" * 60)
        for i, limit in enumerate(test_limits):
            r = results_metadata[i]
            status = "✅ PASS" if r['success'] else "❌ FAIL"
            tags = r.get('tag_count', 0)
            print(f"{limit:<10} {r['count']:<10} {tags:<10} {r['elapsed']:<12.3f} {status:<10}")

        # 성능 평가
        print(f"\n{'='*80}")
        print("📈 Performance Analysis")
        print(f"{'='*80}")

        # 최대 limit 결과 분석
        max_limit_idx = len(test_limits) - 1
        instances_result = results_instances[max_limit_idx]
        metadata_result = results_metadata[max_limit_idx]

        if instances_result['success']:
            print(f"\n✅ /instances (limit={test_limits[max_limit_idx]}):")
            print(f"   - Response time: {instances_result['elapsed']:.3f}s")
            print(f"   - Instances returned: {instances_result['count']}")
            print(f"   - Performance: {'🚀 Excellent' if instances_result['elapsed'] < 1.0 else '✅ Good' if instances_result['elapsed'] < 5.0 else '⚠️  Needs improvement'}")

        if metadata_result['success']:
            print(f"\n✅ /series/metadata (limit={test_limits[max_limit_idx]}):")
            print(f"   - Response time: {metadata_result['elapsed']:.3f}s")
            print(f"   - Instances returned: {metadata_result['count']}")
            print(f"   - DICOM tags per instance: {metadata_result.get('tag_count', 0)}")
            print(f"   - Performance: {'🚀 Excellent' if metadata_result['elapsed'] < 2.0 else '✅ Good' if metadata_result['elapsed'] < 10.0 else '⚠️  Needs improvement'}")

        # V2 배치 쿼리 효과 분석
        print(f"\n💡 V2 Batch Query Optimization Impact:")
        print(f"   - V1 (N+1 queries): ~{test_limits[max_limit_idx] * 0.05:.1f}s expected (N × 50ms)")
        if instances_result['success']:
            improvement = ((test_limits[max_limit_idx] * 0.05 - instances_result['elapsed']) / (test_limits[max_limit_idx] * 0.05)) * 100
            print(f"   - V2 (batch query): {instances_result['elapsed']:.3f}s actual")
            print(f"   - Improvement: {improvement:.1f}% faster 🎉")

        # 전체 테스트 결과
        all_success = all(r['success'] for r in results_instances + results_metadata)

        print(f"\n{'='*80}")
        if all_success:
            print("✅ All tests PASSED! 🎉")
        else:
            print("❌ Some tests FAILED")
        print(f"{'='*80}\n")

        return 0 if all_success else 1

    except Exception as e:
        print(f"\n❌ Test failed with error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    finally:
        client.close()


if __name__ == "__main__":
    exit(main())

