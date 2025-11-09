-- Migration: Add ANNOTATION:READ_ALL permission
-- Created: 2025-01-07
-- Description: Add READ_ALL action for ANNOTATION resource type to support permission-based filtering
--              READ_ALL allows users to view all annotations in a project, not just their own

-- Add ANNOTATION:READ_ALL permission
INSERT INTO security_permission (category, resource_type, action) VALUES
    ('어노테이션 관리', 'ANNOTATION', 'READ_ALL')
ON CONFLICT (resource_type, action) DO NOTHING;

-- Assign ANNOTATION:READ_ALL to SUPER_ADMIN role
INSERT INTO security_role_permission (role_id, permission_id)
SELECT r.id, p.id
FROM security_role r
CROSS JOIN security_permission p
WHERE r.name = 'SUPER_ADMIN'
  AND p.resource_type = 'ANNOTATION' 
  AND p.action = 'READ_ALL'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Assign ANNOTATION:READ_ALL to PROJECT_ADMIN role
INSERT INTO security_role_permission (role_id, permission_id)
SELECT r.id, p.id
FROM security_role r
CROSS JOIN security_permission p
WHERE r.name = 'PROJECT_ADMIN'
  AND p.resource_type = 'ANNOTATION' 
  AND p.action = 'READ_ALL'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Note: RESEARCHER, ANNOTATOR, VIEWER roles only have READ permission (own annotations)
-- They do NOT get READ_ALL permission by default

