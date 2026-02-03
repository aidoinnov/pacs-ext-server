-- SW Information 테이블 (의료영상저장장치 소프트웨어 정보)
-- 화면: SW Information 모달 (품목, 모델명, 제조업자, UDI 등)

CREATE TABLE IF NOT EXISTS sw_information (
    id SERIAL PRIMARY KEY,
    product_item TEXT NOT NULL,
    model_name TEXT NOT NULL,
    sw_version TEXT,
    manufacturer TEXT NOT NULL,
    address TEXT NOT NULL,
    manufacturing_permit_number TEXT NOT NULL,
    manufacturing_year_month TEXT,
    serial_number TEXT,
    udi TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE sw_information IS 'SW Information (의료영상저장장치 소프트웨어 정보)';
COMMENT ON COLUMN sw_information.product_item IS '품목 (예: 의료영상저장장치소프트웨어)';
COMMENT ON COLUMN sw_information.model_name IS '모델명 (예: Aid-U)';
COMMENT ON COLUMN sw_information.sw_version IS 'SW Ver.';
COMMENT ON COLUMN sw_information.manufacturer IS '제조업자';
COMMENT ON COLUMN sw_information.address IS '주소';
COMMENT ON COLUMN sw_information.manufacturing_permit_number IS '제조허가번호';
COMMENT ON COLUMN sw_information.manufacturing_year_month IS '제조연월';
COMMENT ON COLUMN sw_information.serial_number IS '시리얼번호';
COMMENT ON COLUMN sw_information.udi IS 'UDI (Unique Device Identification)';

-- 화면 기준 샘플 데이터 삽입 (초기 실행 시 1건)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM sw_information) THEN
        INSERT INTO sw_information (
            product_item, model_name, sw_version, manufacturer, address,
            manufacturing_permit_number, manufacturing_year_month, serial_number, udi
        ) VALUES (
            '의료영상저장장치소프트웨어',
            'Aid-U',
            NULL,
            '(주)아이에이드',
            '서울특별시 동작구 상도로 398, 가나빌딩 7층',
            '제6816호',
            NULL,
            NULL,
            '(01) 08800080000004
(21) -
(8012) -'
        );
    END IF;
END $$;
