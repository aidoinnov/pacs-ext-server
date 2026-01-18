#!/usr/bin/env python3
"""
Migration Verification Script
Verifies the Subject & TimePoint schema
"""

import sys
try:
    import psycopg2
    from psycopg2.extras import RealDictCursor
except ImportError:
    print("Installing psycopg2...")
    import os
    os.system(f"{sys.executable} -m pip install psycopg2-binary")
    import psycopg2
    from psycopg2.extras import RealDictCursor

def verify_schema(db_config: dict):
    """Verify the migration schema"""
    
    try:
        conn = psycopg2.connect(
            host=db_config['host'],
            port=db_config['port'],
            database=db_config['database'],
            user=db_config['user'],
            password=db_config['password'],
            cursor_factory=RealDictCursor
        )
        cursor = conn.cursor()
        
        print("=" * 80)
        print("📊 SCHEMA VERIFICATION REPORT")
        print("=" * 80)
        print()
        
        # 1. Tables
        print("1️⃣  TABLES")
        print("-" * 80)
        cursor.execute("""
            SELECT 
                table_name,
                (SELECT COUNT(*) FROM information_schema.columns 
                 WHERE table_schema = 'public' AND table_name = t.table_name) as column_count
            FROM information_schema.tables t
            WHERE table_schema = 'public' 
            AND table_name IN ('project_subject', 'subject_timepoint', 'subject_timepoint_study_map')
            ORDER BY table_name;
        """)
        tables = cursor.fetchall()
        for table in tables:
            print(f"   ✅ {table['table_name']:<40} ({table['column_count']} columns)")
        print()
        
        # 2. Columns for each table
        for table_name in ['project_subject', 'subject_timepoint', 'subject_timepoint_study_map']:
            print(f"2️⃣  COLUMNS: {table_name}")
            print("-" * 80)
            cursor.execute("""
                SELECT 
                    column_name,
                    data_type,
                    character_maximum_length,
                    is_nullable,
                    column_default
                FROM information_schema.columns
                WHERE table_schema = 'public' AND table_name = %s
                ORDER BY ordinal_position;
            """, (table_name,))
            columns = cursor.fetchall()
            for col in columns:
                nullable = "NULL" if col['is_nullable'] == 'YES' else "NOT NULL"
                data_type = col['data_type']
                if col['character_maximum_length']:
                    data_type += f"({col['character_maximum_length']})"
                default = f" DEFAULT {col['column_default']}" if col['column_default'] else ""
                print(f"   - {col['column_name']:<30} {data_type:<20} {nullable}{default}")
            print()
        
        # 3. Indexes
        print("3️⃣  INDEXES")
        print("-" * 80)
        cursor.execute("""
            SELECT 
                tablename,
                indexname,
                indexdef
            FROM pg_indexes
            WHERE schemaname = 'public' 
            AND tablename IN ('project_subject', 'subject_timepoint', 'subject_timepoint_study_map')
            ORDER BY tablename, indexname;
        """)
        indexes = cursor.fetchall()
        current_table = None
        for idx in indexes:
            if idx['tablename'] != current_table:
                if current_table:
                    print()
                print(f"   📌 {idx['tablename']}")
                current_table = idx['tablename']
            print(f"      - {idx['indexname']}")
        print()
        
        # 4. Constraints
        print("4️⃣  CONSTRAINTS")
        print("-" * 80)
        cursor.execute("""
            SELECT 
                tc.table_name,
                tc.constraint_name,
                tc.constraint_type,
                kcu.column_name
            FROM information_schema.table_constraints tc
            LEFT JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name
            WHERE tc.table_schema = 'public'
            AND tc.table_name IN ('project_subject', 'subject_timepoint', 'subject_timepoint_study_map')
            ORDER BY tc.table_name, tc.constraint_type, tc.constraint_name;
        """)
        constraints = cursor.fetchall()
        current_table = None
        for con in constraints:
            if con['table_name'] != current_table:
                if current_table:
                    print()
                print(f"   🔒 {con['table_name']}")
                current_table = con['table_name']
            con_type = con['constraint_type'].replace('_', ' ')
            print(f"      - {con['constraint_name']:<40} ({con_type})")
        print()
        
        # 5. Foreign Keys
        print("5️⃣  FOREIGN KEY RELATIONSHIPS")
        print("-" * 80)
        cursor.execute("""
            SELECT
                tc.table_name,
                kcu.column_name,
                ccu.table_name AS foreign_table_name,
                ccu.column_name AS foreign_column_name,
                rc.delete_rule
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name
            JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = tc.constraint_name
            JOIN information_schema.referential_constraints AS rc
                ON rc.constraint_name = tc.constraint_name
            WHERE tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_name IN ('project_subject', 'subject_timepoint', 'subject_timepoint_study_map')
            ORDER BY tc.table_name, kcu.column_name;
        """)
        fks = cursor.fetchall()
        for fk in fks:
            print(f"   {fk['table_name']}.{fk['column_name']}")
            print(f"      → {fk['foreign_table_name']}.{fk['foreign_column_name']} (ON DELETE {fk['delete_rule']})")
        print()
        
        cursor.close()
        conn.close()
        
        print("=" * 80)
        print("✅ VERIFICATION COMPLETE")
        print("=" * 80)
        
    except Exception as e:
        print(f"❌ Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    db_config = {
        'host': 'localhost',
        'port': 5456,
        'database': 'pacs_extension',
        'user': 'pacs_extension_admin',
        'password': 'PacsExtension2024'
    }
    
    verify_schema(db_config)

