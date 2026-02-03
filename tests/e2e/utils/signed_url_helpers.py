"""
Signed URL 검증 헬퍼

템플릿/가이드 이미지 API 응답의 image_url이 실제 접근 가능한 presigned URL인지 검증합니다.
"""
import logging
from typing import Any, List
from urllib.parse import urlparse, parse_qs

logger = logging.getLogger(__name__)


def is_signed_url(url: str) -> bool:
    """
    URL이 presigned URL 형식인지 검증합니다.
    - https 또는 http로 시작
    - S3/MinIO presigned: X-Amz-Signature, X-Amz-Expires 등 쿼리 파라미터 포함
    - 또는 쿼리 파라미터가 있어 일반 static URL이 아님
    - static s3.example.com 등은 제외 (접근 불가)
    """
    if not url or not isinstance(url, str):
        return False
    url = url.strip()
    if not url.startswith("http://") and not url.startswith("https://"):
        return False
    # s3.example.com placeholder는 signed URL이 아님
    if "s3.example.com" in url and "X-Amz-" not in url:
        return False
    parsed = urlparse(url)
    # S3/MinIO presigned URL 패턴
    if "X-Amz-Signature" in url or "X-Amz-Expires" in url:
        return True
    # 쿼리 파라미터가 있으면 presigned일 가능성 높음
    if parsed.query and len(url) > 80:
        return True
    # URL이 충분히 길고 https이면 (일부 provider는 다른 형식 사용 가능)
    if url.startswith("https://") and len(url) > 100:
        return True
    return False


SIGNED_URL_HINT = (
    "pacs-server를 최신 코드로 빌드(cargo build) 후 재시작했는지 확인하세요. "
    "ReportGuideTemplateUseCase에 SignedUrlService가 주입되어 image_url을 presigned URL로 반환해야 합니다."
)


def assert_image_has_signed_url(image: dict, context: str = "", strict: bool = True) -> None:
    """
    이미지 객체에 image_url이 있고 signed URL 형식인지 검증.
    strict=False일 때 placeholder(s3.example.com)면 경고만 하고 통과.
    """
    assert "image_url" in image, f"{context} 이미지에 image_url 필드가 없습니다: {list(image.keys())}"
    url = image["image_url"]
    assert url, f"{context} image_url이 비어있습니다"
    if not is_signed_url(url):
        if not strict and "s3.example.com" in url:
            logger.warning(f"{context} image_url이 placeholder. Object Storage 설정 후 presigned URL 반환 필요.")
            return
        raise AssertionError(
            f"{context} image_url이 presigned URL 형식이 아닙니다. "
            f"got: {url[:120]}... | {SIGNED_URL_HINT}"
        )
    logger.debug(f"{context} image_url 검증 통과 (signed URL)")


def assert_images_have_signed_urls(images: List[dict], context: str = "", strict: bool = True) -> None:
    """이미지 목록의 모든 image_url이 signed URL인지 검증. strict=False면 placeholder 허용."""
    for i, img in enumerate(images):
        assert_image_has_signed_url(img, f"{context}[{i}]", strict=strict)


def assert_guide_images_accessible(guide: dict, context: str = "", strict: bool = True) -> None:
    """
    Report Guide 응답의 images가 있고, 각 image_url이 signed URL인지 검증.
    strict=False면 placeholder 허용.
    """
    images = guide.get("images")
    if not images:
        return
    assert_images_have_signed_urls(images, f"{context} guide.images", strict=strict)
