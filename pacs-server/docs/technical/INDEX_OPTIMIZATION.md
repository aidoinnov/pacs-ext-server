# Index Optimization Notes

This document records core indexes/constraints introduced in `018_core_indices.sql` to accelerate:
- UID→ID mapping (study/series/instance)
- Gateway QIDO-mapped filters (StudyDate/PatientID/Modality)
- Explicit access checks and membership

Validation checklist:
- EXPLAIN UID mapping queries
- EXPLAIN Study filters (project_id + study_date, project_id + patient_id)
- EXPLAIN Series filters (study_id + modality)
- EXPLAIN explicit access scans (user_id, project_id, resource_level)

Rollback: use `DROP INDEX IF EXISTS` and `DROP CONSTRAINT IF EXISTS`.
