#!/bin/bash
psql -h localhost -p 5456 -U pacs_extension_admin -d pacs_extension -f migrations/036_add_snapshot_image_to_annotations.sql