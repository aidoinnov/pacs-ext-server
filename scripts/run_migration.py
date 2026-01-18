#!/usr/bin/env python3
"""
Migration Runner Script
Executes SQL migration files against PostgreSQL database
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
        
        print(f"✅ Connected successfully")
        
        # Execute migration
        print(f"🚀 Executing migration...")
        cursor.execute(sql_content)
        
        # Commit transaction
        conn.commit()
        print(f"✅ Migration executed successfully")
        
        # Verify tables created
        print(f"\n📊 Verifying tables...")
        cursor.execute("""
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name IN ('project_subject', 'subject_timepoint', 'subject_timepoint_study_map')
            ORDER BY table_name;
        """)
        tables = cursor.fetchall()
        
        if len(tables) == 3:
            print(f"✅ All 3 tables created:")
            for table in tables:
                print(f"   - {table[0]}")
        else:
            print(f"⚠️  Expected 3 tables, found {len(tables)}")
        
        # Verify indexes
        print(f"\n📊 Verifying indexes...")
        cursor.execute("""
            SELECT indexname 
            FROM pg_indexes 
            WHERE schemaname = 'public' 
            AND tablename IN ('project_subject', 'subject_timepoint', 'subject_timepoint_study_map')
            ORDER BY indexname;
        """)
        indexes = cursor.fetchall()
        print(f"✅ Created {len(indexes)} indexes")
        
        # Verify constraints
        print(f"\n📊 Verifying constraints...")
        cursor.execute("""
            SELECT conname, contype 
            FROM pg_constraint 
            WHERE conrelid IN (
                SELECT oid FROM pg_class 
                WHERE relname IN ('project_subject', 'subject_timepoint', 'subject_timepoint_study_map')
            )
            ORDER BY conname;
        """)
        constraints = cursor.fetchall()
        print(f"✅ Created {len(constraints)} constraints")
        
        cursor.close()
        conn.close()
        
        return True
        
    except psycopg2.Error as e:
        print(f"❌ Database error: {e}")
        if conn:
            conn.rollback()
            conn.close()
        return False
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        if conn:
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
    migration_file = "migrations/040_create_subject_timepoint.sql"
    
    print("=" * 60)
    print("🔧 PACS Extension Server - Migration Runner")
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

