#!/usr/bin/env python3
"""
스냅샷 URL 반환 테스트
"""
import sys
import logging
from utils.api_client import APIClient
from config import TestConfig

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def test_snapshot_url_in_annotations():
    """어노테이션 조회 시 snapshot_image_url이 반환되는지 테스트"""
    
    config = TestConfig.from_env()
    client = APIClient(config.base_url, config.timeout)
    
    try:
        # 로그인
        logger.info("로그인 중...")
        client.login(config.admin_email, config.admin_password)
        logger.info("✓ 로그인 성공")
        
        # 어노테이션 조회
        logger.info("\n어노테이션 조회 중...")
        response = client.get("/api/annotations", params={"limit": 10})
        
        if response.status_code != 200:
            logger.error(f"어노테이션 조회 실패: {response.status_code}")
            return False
        
        data = response.json()
        annotations = data if isinstance(data, list) else data.get("annotations", [])
        
        logger.info(f"총 {len(annotations)}개의 어노테이션 조회됨")
        
        # 스냅샷이 있는 어노테이션 찾기
        snapshot_annotations = [
            ann for ann in annotations 
            if ann.get("snapshot_image_key")
        ]
        
        logger.info(f"스냅샷이 있는 어노테이션: {len(snapshot_annotations)}개")
        
        if not snapshot_annotations:
            logger.warning("⚠️  스냅샷이 있는 어노테이션이 없습니다")
            return True
        
        # 각 어노테이션 검사
        print("\n" + "="*80)
        print("스냅샷 URL 검사 결과")
        print("="*80)
        
        has_url_count = 0
        no_url_count = 0
        
        for i, ann in enumerate(snapshot_annotations[:5], 1):
            print(f"\n[어노테이션 {i}]")
            print(f"  ID: {ann.get('id')}")
            print(f"  Tool: {ann.get('tool_name')}")
            print(f"  Snapshot Key: {ann.get('snapshot_image_key')}")
            print(f"  Snapshot Status: {ann.get('snapshot_status')}")
            
            snapshot_url = ann.get('snapshot_image_url')
            if snapshot_url:
                print(f"  ✅ Snapshot URL: {snapshot_url[:80]}...")
                has_url_count += 1
            else:
                print(f"  ❌ Snapshot URL: None")
                no_url_count += 1
        
        print("\n" + "="*80)
        print(f"요약:")
        print(f"  - URL 있음: {has_url_count}개")
        print(f"  - URL 없음: {no_url_count}개")
        print("="*80)
        
        if no_url_count > 0:
            print("\n⚠️  문제 발견: snapshot_image_url이 None으로 반환됨")
            print("\n원인:")
            print("  - annotation_use_case.rs에서 snapshot_image_url을 None으로 설정")
            print("  - Controller에서 URL을 생성하지 않음")
            print("\n해결 방법:")
            print("  1. Controller에서 응답 후처리로 URL 생성")
            print("  2. 또는 UseCase에서 SignedUrlService를 주입받아 URL 생성")
            return False
        else:
            print("\n✅ 모든 스냅샷에 URL이 있습니다!")
            return True
            
    except Exception as e:
        logger.error(f"테스트 실패: {e}")
        import traceback
        traceback.print_exc()
        return False
    finally:
        client.close()


def test_individual_annotation_with_snapshot():
    """개별 어노테이션 조회 시 snapshot_image_url 테스트"""
    
    config = TestConfig.from_env()
    client = APIClient(config.base_url, config.timeout)
    
    try:
        # 로그인
        client.login(config.admin_email, config.admin_password)
        
        # 먼저 스냅샷이 있는 어노테이션 ID 찾기
        response = client.get("/api/annotations", params={"limit": 10})
        data = response.json()
        annotations = data if isinstance(data, list) else data.get("annotations", [])
        
        snapshot_ann = next(
            (ann for ann in annotations if ann.get("snapshot_image_key")),
            None
        )
        
        if not snapshot_ann:
            logger.warning("스냅샷이 있는 어노테이션을 찾을 수 없습니다")
            return True
        
        ann_id = snapshot_ann["id"]
        logger.info(f"\n개별 어노테이션 조회 (ID: {ann_id})...")
        
        # 개별 조회
        response = client.get(f"/api/annotations/{ann_id}")
        
        if response.status_code != 200:
            logger.error(f"개별 조회 실패: {response.status_code}")
            return False
        
        ann = response.json()
        
        print("\n" + "="*80)
        print("개별 어노테이션 조회 결과")
        print("="*80)
        print(f"ID: {ann.get('id')}")
        print(f"Snapshot Key: {ann.get('snapshot_image_key')}")
        print(f"Snapshot URL: {ann.get('snapshot_image_url')}")
        print(f"Snapshot Status: {ann.get('snapshot_status')}")
        print("="*80)
        
        if ann.get('snapshot_image_url'):
            print("✅ 개별 조회에서도 snapshot_image_url이 반환됨!")
            return True
        else:
            print("❌ 개별 조회에서도 snapshot_image_url이 None")
            return False
            
    except Exception as e:
        logger.error(f"테스트 실패: {e}")
        return False
    finally:
        client.close()


if __name__ == "__main__":
    print("\n" + "="*80)
    print("스냅샷 URL 반환 테스트")
    print("="*80)
    
    result1 = test_snapshot_url_in_annotations()
    result2 = test_individual_annotation_with_snapshot()
    
    if result1 and result2:
        print("\n✅ 모든 테스트 통과!")
        sys.exit(0)
    else:
        print("\n❌ 일부 테스트 실패")
        sys.exit(1)

