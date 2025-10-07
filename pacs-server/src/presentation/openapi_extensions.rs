use utoipa::openapi::OpenApi;

pub fn extend_openapi(openapi: OpenApi) -> OpenApi {
    // 현재는 annotation 엔드포인트만 포함되어 있습니다
    // 나머지 엔드포인트는 태그와 설명으로 표시됩니다

    // Note: 다른 컨트롤러들은 제네릭 타입(<A: AuthService>, <P: ProjectService>)을
    // 사용하기 때문에 utoipa::path 매크로와 함께 사용하기 어렵습니다.
    //
    // 향후 문서화 방법:
    // 1. 각 컨트롤러 함수를 구체적인 타입으로 래핑하는 별도 함수 생성
    // 2. 수동으로 OpenAPI PathItem 구축 (복잡함)

    openapi
}
