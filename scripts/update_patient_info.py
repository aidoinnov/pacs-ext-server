#!/usr/bin/env python3
"""
Update patient_id and patient_name in project_data_study from dcm4chee database
"""

import psycopg2
import sys

# Database configurations
RBAC_DB = {
    'host': 'localhost',
    'port': 5456,
    'user': 'pacs_extension_admin',
    'password': 'PacsExtension2024',
    'database': 'pacs_extension'
}

DCM4CHEE_DB = {
    'host': 'localhost',
    'port': 5457,
    'user': 'pacsadmin',
    'password': 'HhL}qb(tl}?zJ4}(',
    'database': 'postgres'
}

def main():
    print("=" * 80)
    print("🔄 Updating patient_id and patient_name from dcm4chee")
    print("=" * 80)
    
    # Connect to dcm4chee
    print("\n📡 Connecting to dcm4chee database...")
    dcm_conn = psycopg2.connect(**DCM4CHEE_DB)
    dcm_cur = dcm_conn.cursor()
    
    # Connect to RBAC DB
    print("📡 Connecting to RBAC database...")
    rbac_conn = psycopg2.connect(**RBAC_DB)
    rbac_cur = rbac_conn.cursor()
    
    try:
        # Get all studies from dcm4chee with patient info
        print("\n🔍 Fetching studies from dcm4chee...")
        dcm_cur.execute("""
            SELECT 
                st.study_iuid,
                pid.pat_id AS patient_id,
                pn.alphabetic_name AS patient_name
            FROM study st
            LEFT JOIN patient pt ON st.patient_fk = pt.pk
            LEFT JOIN patient_id pid ON pt.patient_id_fk = pid.pk
            LEFT JOIN person_name pn ON pt.pat_name_fk = pn.pk
            WHERE st.study_iuid IS NOT NULL
        """)
        
        studies = dcm_cur.fetchall()
        print(f"✅ Found {len(studies)} studies in dcm4chee")
        
        # Update each study in RBAC DB
        print("\n🔄 Updating project_data_study...")
        updated_count = 0
        skipped_count = 0
        
        for study_uid, patient_id, patient_name in studies:
            if not patient_id:
                skipped_count += 1
                continue
                
            rbac_cur.execute("""
                UPDATE project_data_study
                SET patient_id = %s,
                    patient_name = %s,
                    updated_at = CURRENT_TIMESTAMP
                WHERE study_uid = %s
            """, (patient_id, patient_name, study_uid))
            
            if rbac_cur.rowcount > 0:
                updated_count += 1
        
        rbac_conn.commit()
        
        print(f"\n✅ Update completed!")
        print(f"   - Updated: {updated_count} studies")
        print(f"   - Skipped (no patient_id): {skipped_count} studies")
        
        # Verify results
        print("\n🔍 Verifying results...")
        rbac_cur.execute("""
            SELECT 
                COUNT(*) FILTER (WHERE patient_id IS NOT NULL) as with_patient,
                COUNT(*) FILTER (WHERE patient_id IS NULL) as without_patient,
                COUNT(*) as total
            FROM project_data_study
        """)
        
        with_patient, without_patient, total = rbac_cur.fetchone()
        print(f"   - Total studies: {total}")
        print(f"   - With patient_id: {with_patient}")
        print(f"   - Without patient_id: {without_patient}")
        
    except Exception as e:
        print(f"\n❌ Error: {e}")
        rbac_conn.rollback()
        sys.exit(1)
    finally:
        dcm_cur.close()
        dcm_conn.close()
        rbac_cur.close()
        rbac_conn.close()
    
    print("\n" + "=" * 80)
    print("✅ Done!")
    print("=" * 80)

if __name__ == "__main__":
    main()

