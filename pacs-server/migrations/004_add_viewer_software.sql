-- annotation_annotation 테이블에 viewer_software 컬럼 추가
ALTER TABLE annotation_annotation 
ADD COLUMN viewer_software VARCHAR(100) DEFAULT NULL;

-- viewer_software 컬럼에 대한 인덱스 추가
CREATE INDEX idx_annotation_viewer_software ON annotation_annotation (viewer_software);

-- 문서화를 위한 주석 추가
COMMENT ON COLUMN annotation_annotation.viewer_software IS 
'뷰어 소프트웨어 정보 (예: "OHIF", "Cornerstone", "DICOM.js")';
