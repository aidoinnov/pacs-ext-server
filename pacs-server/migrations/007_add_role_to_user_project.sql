-- Add role_id column to security_user_project table
-- This allows each user to have a specific role within a project

-- Add role_id column to security_user_project
ALTER TABLE security_user_project 
ADD COLUMN role_id INTEGER REFERENCES security_role(id) ON DELETE SET NULL;

-- Create index for role queries
CREATE INDEX idx_user_project_role ON security_user_project(role_id);

-- Add comment
COMMENT ON COLUMN security_user_project.role_id IS 'User role within the project';

-- Add comment to the table
COMMENT ON TABLE security_user_project IS 'User membership in projects with optional role assignment';
