#!/usr/bin/env python3
"""
Migration 042 Runner Script
Executes SQL migration file for RECIST Lesion simplification
"""

import sys
import os
from pathlib import Path

try:
    import psycopg2
except ImportError:
    print("❌ psycopg2 not installed. Installing...")
    os.system(f"{sys.executable} -m pip install psycopg2-binary")
    import psycopg2

def run_migration(migration_file: str, db_config: dict):
    """Run a single migration file"""
    
    # Read migration file
    migration_path = Path(migration_file)
    if not migration_path.exists():
        print(f"❌ Migration file not found: {migration_file}")
        return False
    
    print(f"📄 Reading migration: {migration_path.name}")
    with open(migration_path, 'r', encoding='utf-8') as f:
        sql_content = f.read()
    
    # Connect to database
    try:
        print(f"🔌 Connecting to database: {db_config['database']}@{db_config['host']}:{db_config['port']}")
        conn = psycopg2.connect(
            host=db_config['host'],
            port=db_config['port'],
            database=db_config['database'],
            user=db_config['user'],
            password=db_config['password']
        )
        conn.autocommit = False
        cursor = conn.cursor()
        
        print("✅ Connected to database")
        print()
        
        # Execute migration
        print("🚀 Executing migration...")
        print("-" * 60)
        cursor.execute(sql_content)
        
        # Commit transaction
        conn.commit()
        print("-" * 60)
        print("✅ Migration executed successfully")
        
        # Close connection
        cursor.close()
        conn.close()
        
        return True
        
    except psycopg2.Error as e:
        print(f"❌ Database error: {e}")
        if 'conn' in locals():
            conn.rollback()
            conn.close()
        return False
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        if 'conn' in locals():
            conn.rollback()
            conn.close()
        return False

if __name__ == "__main__":
    # Database configuration from .env
    db_config = {
        'host': 'localhost',
        'port': 5456,
        'database': 'pacs_extension',
        'user': 'pacs_extension_admin',
        'password': 'PacsExtension2024'
    }
    
    # Migration file
    migration_file = "pacs-server/migrations/042_simplify_recist_lesion.sql"
    
    print("=" * 60)
    print("🔧 PACS Extension Server - Migration 042 Runner")
    print("=" * 60)
    print()
    
    success = run_migration(migration_file, db_config)
    
    print()
    print("=" * 60)
    if success:
        print("✅ Migration completed successfully!")
    else:
        print("❌ Migration failed!")
        sys.exit(1)
    print("=" * 60)

