#!/usr/bin/env python3
"""project_data 테이블 확인"""
import psycopg2
import os

# 환경 변수에서 가져오기
db_host = os.getenv('APP_DATABASE__HOST', 'localhost')
db_port = int(os.getenv('APP_DATABASE__PORT', '5456'))
db_user = os.getenv('APP_DATABASE__USERNAME', 'admin')
db_pass = os.getenv('APP_DATABASE__PASSWORD', 'admin123')
db_name = os.getenv('APP_DATABASE__DATABASE', 'pacs_db')

print(f'Connecting to {db_host}:{db_port}/{db_name} as {db_user}')

try:
    conn = psycopg2.connect(
        host='127.0.0.1' if db_host == 'localhost' else db_host,
        port=db_port,
        user=db_user,
        password=db_pass,
        database=db_name
    )
    cur = conn.cursor()

    # project_data 확인
    cur.execute('SELECT COUNT(*) FROM project_data WHERE project_id = 2')
    count = cur.fetchone()[0]
    print(f'✅ project_data (project_id=2) 개수: {count}')

    # project_data_study 확인
    cur.execute('SELECT COUNT(*) FROM project_data_study')
    study_count = cur.fetchone()[0]
    print(f'✅ project_data_study 개수: {study_count}')

    # project_data_series 확인
    cur.execute('SELECT COUNT(*) FROM project_data_series')
    series_count = cur.fetchone()[0]
    print(f'✅ project_data_series 개수: {series_count}')

    # project_data 상세 확인
    cur.execute('''
        SELECT pd.project_id, pd.study_id, pd.series_id, pd.resource_level, 
               pds.study_uid, pdser.series_uid
        FROM project_data pd
        LEFT JOIN project_data_study pds ON pd.study_id = pds.id
        LEFT JOIN project_data_series pdser ON pd.series_id = pdser.id
        WHERE pd.project_id = 2
        LIMIT 5
    ''')
    rows = cur.fetchall()
    print(f'\n✅ project_data 샘플 (최대 5개):')
    for row in rows:
        print(f'  project_id={row[0]}, study_id={row[1]}, series_id={row[2]}, level={row[3]}, study_uid={row[4]}, series_uid={row[5]}')

    cur.close()
    conn.close()
except Exception as e:
    print(f'❌ Error: {e}')

