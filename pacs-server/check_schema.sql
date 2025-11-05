-- Check if project_data table exists and its structure
SELECT 
    table_name,
    column_name,
    data_type,
    is_nullable
FROM information_schema.columns
WHERE table_name IN ('project_data', 'project_data_study')
ORDER BY table_name, ordinal_position;

