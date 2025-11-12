#!/bin/bash

# ============================================================================
# 프로젝트 데이터 접근 제어 테스트 시나리오 구성 스크립트
# ============================================================================
# 
# 시나리오: 다기관 공동 연구 프로젝트
# 프로젝트: "심장질환 공동 연구" (project_id: 90)
#
# 참여자:
#   - Dr. Kim (101): 책임연구원 - 모든 데이터 접근 가능
#   - Dr. Lee (102): A병원 연구원 - A병원 데이터만 접근
#   - Dr. Park (103): B병원 연구원 - B병원 데이터만 접근
#   - Dr. Choi (104): 임시 협력자 - 특정 Study만 7일간 접근
#
# 데이터:
#   - Study 1, 2, 3: A병원 환자 데이터
#   - Study 4, 5, 6: B병원 환자 데이터
#   - Study 7: VIP 환자 (민감 데이터)
#
# ============================================================================

set -e  # 에러 발생 시 스크립트 중단

# 데이터베이스 접속 정보
export PGPASSWORD=PacsExtension2024
PSQL="psql -h localhost -p 5456 -U pacs_extension_admin -d pacs_extension"

echo "🎬 시나리오 구성 시작..."
echo ""

# ============================================================================
# 1. 프로젝트 생성
# ============================================================================
echo "📁 프로젝트 생성..."
$PSQL <<EOF
INSERT INTO security_project (name, description, status, is_active)
VALUES ('심장질환 공동 연구', '다기관 공동 연구 프로젝트', 'IN_PROGRESS', true)
ON CONFLICT (name) DO UPDATE SET
    description = EXCLUDED.description,
    status = EXCLUDED.status;
EOF

# 프로젝트 ID 조회
PROJECT_ID=$($PSQL -t -A <<'EOSQL'
SELECT id FROM security_project WHERE name = '심장질환 공동 연구';
EOSQL
)

echo "   프로젝트 ID: $PROJECT_ID"

# ============================================================================
# 2. 사용자 생성
# ============================================================================
echo "👥 사용자 생성..."
$PSQL <<EOF
-- 책임연구원 (Dr. Kim)
INSERT INTO security_user (keycloak_id, username, email, full_name, account_status, email_verified)
VALUES (gen_random_uuid(), 'dr_kim', 'kim@hospital.com', 'Dr. Kim (책임연구원)', 'ACTIVE', true)
ON CONFLICT (username) DO UPDATE SET full_name = EXCLUDED.full_name;

-- A병원 연구원 (Dr. Lee)
INSERT INTO security_user (keycloak_id, username, email, full_name, account_status, email_verified)
VALUES (gen_random_uuid(), 'dr_lee', 'lee@hospital-a.com', 'Dr. Lee (A병원)', 'ACTIVE', true)
ON CONFLICT (username) DO UPDATE SET full_name = EXCLUDED.full_name;

-- B병원 연구원 (Dr. Park)
INSERT INTO security_user (keycloak_id, username, email, full_name, account_status, email_verified)
VALUES (gen_random_uuid(), 'dr_park', 'park@hospital-b.com', 'Dr. Park (B병원)', 'ACTIVE', true)
ON CONFLICT (username) DO UPDATE SET full_name = EXCLUDED.full_name;

-- 임시 협력자 (Dr. Choi)
INSERT INTO security_user (keycloak_id, username, email, full_name, account_status, email_verified)
VALUES (gen_random_uuid(), 'dr_choi', 'choi@external.com', 'Dr. Choi (임시 협력자)', 'ACTIVE', true)
ON CONFLICT (username) DO UPDATE SET full_name = EXCLUDED.full_name;
EOF

# 사용자 ID 조회
USER_KIM=$($PSQL -t -A <<'EOSQL'
SELECT id FROM security_user WHERE username = 'dr_kim';
EOSQL
)
USER_LEE=$($PSQL -t -A <<'EOSQL'
SELECT id FROM security_user WHERE username = 'dr_lee';
EOSQL
)
USER_PARK=$($PSQL -t -A <<'EOSQL'
SELECT id FROM security_user WHERE username = 'dr_park';
EOSQL
)
USER_CHOI=$($PSQL -t -A <<'EOSQL'
SELECT id FROM security_user WHERE username = 'dr_choi';
EOSQL
)

echo "   Dr. Kim ID: $USER_KIM"
echo "   Dr. Lee ID: $USER_LEE"
echo "   Dr. Park ID: $USER_PARK"
echo "   Dr. Choi ID: $USER_CHOI"

# ============================================================================
# 3. 프로젝트에 사용자 할당 (USER 역할)
# ============================================================================
echo "🔗 프로젝트에 사용자 할당..."
$PSQL <<EOF
DO \$\$
DECLARE
    user_role_id INTEGER;
    v_project_id INTEGER;
    v_user_kim INTEGER;
    v_user_lee INTEGER;
    v_user_park INTEGER;
    v_user_choi INTEGER;
BEGIN
    -- 역할 ID 조회
    SELECT id INTO user_role_id FROM security_role WHERE name = 'USER' LIMIT 1;

    IF user_role_id IS NULL THEN
        RAISE EXCEPTION 'USER role not found';
    END IF;

    -- 프로젝트 ID 조회
    SELECT id INTO v_project_id FROM security_project WHERE name = '심장질환 공동 연구';

    -- 사용자 ID 조회
    SELECT id INTO v_user_kim FROM security_user WHERE username = 'dr_kim';
    SELECT id INTO v_user_lee FROM security_user WHERE username = 'dr_lee';
    SELECT id INTO v_user_park FROM security_user WHERE username = 'dr_park';
    SELECT id INTO v_user_choi FROM security_user WHERE username = 'dr_choi';

    -- 모든 사용자를 프로젝트에 할당
    INSERT INTO security_user_project (user_id, project_id, role_id, created_at)
    VALUES
        (v_user_kim, v_project_id, user_role_id, NOW()),
        (v_user_lee, v_project_id, user_role_id, NOW()),
        (v_user_park, v_project_id, user_role_id, NOW()),
        (v_user_choi, v_project_id, user_role_id, NOW())
    ON CONFLICT (user_id, project_id) DO NOTHING;

    RAISE NOTICE '✅ 사용자 4명을 프로젝트에 할당했습니다.';
END \$\$;
EOF

# ============================================================================
# 4. Study 데이터 생성
# ============================================================================
echo "📊 Study 데이터 생성..."
$PSQL <<EOF
-- A병원 Study 1, 2, 3
INSERT INTO project_data_study (study_uid, study_description, patient_id, patient_name, study_date)
VALUES
    ('1.2.840.113619.2.55.3.A.1', 'CT Chest - A병원 환자1', 'A-P001', '김철수', '2025-01-10'),
    ('1.2.840.113619.2.55.3.A.2', 'MRI Brain - A병원 환자2', 'A-P002', '이영희', '2025-01-11'),
    ('1.2.840.113619.2.55.3.A.3', 'CT Abdomen - A병원 환자3', 'A-P003', '박민수', '2025-01-12')
ON CONFLICT (study_uid) DO UPDATE SET
    study_description = EXCLUDED.study_description;

-- B병원 Study 4, 5, 6
INSERT INTO project_data_study (study_uid, study_description, patient_id, patient_name, study_date)
VALUES
    ('1.2.840.113619.2.55.3.B.1', 'CT Chest - B병원 환자1', 'B-P001', '최지훈', '2025-01-13'),
    ('1.2.840.113619.2.55.3.B.2', 'MRI Spine - B병원 환자2', 'B-P002', '정수진', '2025-01-14'),
    ('1.2.840.113619.2.55.3.B.3', 'CT Heart - B병원 환자3', 'B-P003', '강민호', '2025-01-15')
ON CONFLICT (study_uid) DO UPDATE SET
    study_description = EXCLUDED.study_description;

-- VIP 환자 Study 7
INSERT INTO project_data_study (study_uid, study_description, patient_id, patient_name, study_date)
VALUES
    ('1.2.840.113619.2.55.3.VIP.1', 'CT Full Body - VIP 환자', 'VIP-001', 'VIP 환자', '2025-01-16')
ON CONFLICT (study_uid) DO UPDATE SET
    study_description = EXCLUDED.study_description;
EOF

# Study ID 조회
STUDY_A1=$($PSQL -t -A <<'EOSQL'
SELECT id FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.A.1';
EOSQL
)
STUDY_A2=$($PSQL -t -A <<'EOSQL'
SELECT id FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.A.2';
EOSQL
)
STUDY_A3=$($PSQL -t -A <<'EOSQL'
SELECT id FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.A.3';
EOSQL
)
STUDY_B1=$($PSQL -t -A <<'EOSQL'
SELECT id FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.B.1';
EOSQL
)
STUDY_B2=$($PSQL -t -A <<'EOSQL'
SELECT id FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.B.2';
EOSQL
)
STUDY_B3=$($PSQL -t -A <<'EOSQL'
SELECT id FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.B.3';
EOSQL
)
STUDY_VIP=$($PSQL -t -A <<'EOSQL'
SELECT id FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.VIP.1';
EOSQL
)

echo "   A병원 Study IDs: $STUDY_A1, $STUDY_A2, $STUDY_A3"
echo "   B병원 Study IDs: $STUDY_B1, $STUDY_B2, $STUDY_B3"
echo "   VIP Study ID: $STUDY_VIP"

# ============================================================================
# 5. 프로젝트에 Study 할당
# ============================================================================
echo "🔗 프로젝트에 Study 할당..."
$PSQL <<EOF
DO \$\$
DECLARE
    v_project_id INTEGER;
    v_study_a1 INTEGER;
    v_study_a2 INTEGER;
    v_study_a3 INTEGER;
    v_study_b1 INTEGER;
    v_study_b2 INTEGER;
    v_study_b3 INTEGER;
    v_study_vip INTEGER;
BEGIN
    -- 프로젝트 ID 조회
    SELECT id INTO v_project_id FROM security_project WHERE name = '심장질환 공동 연구';

    -- Study ID 조회
    SELECT id INTO v_study_a1 FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.A.1';
    SELECT id INTO v_study_a2 FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.A.2';
    SELECT id INTO v_study_a3 FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.A.3';
    SELECT id INTO v_study_b1 FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.B.1';
    SELECT id INTO v_study_b2 FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.B.2';
    SELECT id INTO v_study_b3 FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.B.3';
    SELECT id INTO v_study_vip FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.VIP.1';

    -- 프로젝트에 Study 할당
    INSERT INTO project_data (project_id, resource_level, study_id)
    VALUES
        (v_project_id, 'STUDY', v_study_a1),
        (v_project_id, 'STUDY', v_study_a2),
        (v_project_id, 'STUDY', v_study_a3),
        (v_project_id, 'STUDY', v_study_b1),
        (v_project_id, 'STUDY', v_study_b2),
        (v_project_id, 'STUDY', v_study_b3),
        (v_project_id, 'STUDY', v_study_vip)
    ON CONFLICT (project_id, study_id, series_id, instance_id) DO NOTHING;
END \$\$;
EOF

# ============================================================================
# 6. project_data_access 설정 (접근 제어)
# ============================================================================
echo "🔒 접근 제어 설정..."
$PSQL <<EOF
DO \$\$
DECLARE
    v_project_id INTEGER;
    v_user_kim INTEGER;
    v_user_lee INTEGER;
    v_user_park INTEGER;
    v_user_choi INTEGER;
    v_study_a1 INTEGER;
    v_study_a2 INTEGER;
    v_study_a3 INTEGER;
    v_study_b1 INTEGER;
    v_study_b2 INTEGER;
    v_study_b3 INTEGER;
BEGIN
    -- 프로젝트 ID 조회
    SELECT id INTO v_project_id FROM security_project WHERE name = '심장질환 공동 연구';

    -- 사용자 ID 조회
    SELECT id INTO v_user_kim FROM security_user WHERE username = 'dr_kim';
    SELECT id INTO v_user_lee FROM security_user WHERE username = 'dr_lee';
    SELECT id INTO v_user_park FROM security_user WHERE username = 'dr_park';
    SELECT id INTO v_user_choi FROM security_user WHERE username = 'dr_choi';

    -- Study ID 조회
    SELECT id INTO v_study_a1 FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.A.1';
    SELECT id INTO v_study_a2 FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.A.2';
    SELECT id INTO v_study_a3 FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.A.3';
    SELECT id INTO v_study_b1 FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.B.1';
    SELECT id INTO v_study_b2 FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.B.2';
    SELECT id INTO v_study_b3 FROM project_data_study WHERE study_uid = '1.2.840.113619.2.55.3.B.3';

    -- 기존 레코드 삭제 (재실행 시 깨끗하게 시작)
    DELETE FROM project_data_access WHERE project_id = v_project_id;

    -- 책임연구원 (Dr. Kim): 레코드 없음 → 전체 접근 가능
    -- (아무것도 추가하지 않음)

    -- A병원 연구원 (Dr. Lee): A병원 Study만 접근
    INSERT INTO project_data_access (user_id, project_id, resource_level, study_id, status, access_scope)
    VALUES
        (v_user_lee, v_project_id, 'STUDY', v_study_a1, 'APPROVED', 'FULL'),
        (v_user_lee, v_project_id, 'STUDY', v_study_a2, 'APPROVED', 'FULL'),
        (v_user_lee, v_project_id, 'STUDY', v_study_a3, 'APPROVED', 'FULL');

    -- B병원 연구원 (Dr. Park): B병원 Study만 접근
    INSERT INTO project_data_access (user_id, project_id, resource_level, study_id, status, access_scope)
    VALUES
        (v_user_park, v_project_id, 'STUDY', v_study_b1, 'APPROVED', 'FULL'),
        (v_user_park, v_project_id, 'STUDY', v_study_b2, 'APPROVED', 'FULL'),
        (v_user_park, v_project_id, 'STUDY', v_study_b3, 'APPROVED', 'FULL');

    -- 임시 협력자 (Dr. Choi): Study 1만 7일간 읽기 전용 접근
    INSERT INTO project_data_access (user_id, project_id, resource_level, study_id, status, access_scope, expires_at)
    VALUES
        (v_user_choi, v_project_id, 'STUDY', v_study_a1, 'APPROVED', 'READ_ONLY', NOW() + INTERVAL '7 days');

    RAISE NOTICE '✅ 접근 제어 설정 완료';
END \$\$;
EOF

# ============================================================================
# 7. 결과 확인
# ============================================================================
echo ""
echo "✅ 시나리오 구성 완료!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 구성된 데이터 요약"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📁 프로젝트:"
echo "  - ID: 90"
echo "  - 이름: 심장질환 공동 연구"
echo "  - 설명: 다기관 공동 연구 프로젝트"
echo ""
echo "👥 사용자 (4명):"
echo "  - Dr. Kim (101): 책임연구원"
echo "  - Dr. Lee (102): A병원 연구원"
echo "  - Dr. Park (103): B병원 연구원"
echo "  - Dr. Choi (104): 임시 협력자"
echo ""
echo "📊 Study 데이터 (7개):"
echo "  - A병원: Study 1001, 1002, 1003"
echo "  - B병원: Study 1004, 1005, 1006"
echo "  - VIP: Study 1007"
echo ""
echo "🔒 접근 제어 설정:"
echo "  - Dr. Kim (101): 전체 접근 ✅ (레코드 없음)"
echo "  - Dr. Lee (102): A병원 Study만 (1001, 1002, 1003)"
echo "  - Dr. Park (103): B병원 Study만 (1004, 1005, 1006)"
echo "  - Dr. Choi (104): Study 1001만 (7일간, 읽기 전용)"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🧪 다음 단계:"
echo "  1. ./scenario-test.sh 실행하여 접근 제어 테스트"
echo "  2. 데이터베이스에서 직접 확인:"
echo "     psql -h localhost -p 5456 -U pacs_extension_admin -d pacs_extension"
echo "     SELECT * FROM project_data_access WHERE project_id = 90;"
echo ""

