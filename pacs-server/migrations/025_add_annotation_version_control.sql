-- Migration: Add version control to annotation_annotation table
-- Purpose: Implement Optimistic Locking for Version Conflict handling
-- Date: 2025-11-07

-- Add version column to annotation_annotation table
-- This column tracks the version of each annotation for optimistic locking
ALTER TABLE annotation_annotation
ADD COLUMN version INTEGER NOT NULL DEFAULT 1;

-- Create index on version column for efficient querying
CREATE INDEX idx_annotation_version ON annotation_annotation(id, version);

-- Add comment to explain the version column
COMMENT ON COLUMN annotation_annotation.version IS 'Version number for optimistic locking. Incremented on each update.';

-- Verify the migration
-- SELECT id, version, created_at, updated_at FROM annotation_annotation LIMIT 5;

