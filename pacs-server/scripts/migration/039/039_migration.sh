#!/bin/bash
psql -h localhost -p 5456 -U pacs_extension_admin -d pacs_extension -f migrations/039_create_gc_deletion_log.sql