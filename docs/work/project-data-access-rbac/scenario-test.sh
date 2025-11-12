#!/bin/bash

# ============================================================================
# 프로젝트 데이터 접근 제어 테스트 스크립트
# ============================================================================
#
# 이 스크립트는 project_data_access 테이블 기반 접근 제어가
# 올바르게 작동하는지 확인합니다.
#
# 사전 요구사항:
#   - scenario-setup.sh 실행 완료
#   - 서버 실행 중 (http://localhost:8080)
#
# ============================================================================

set -e  # 에러 발생 시 스크립트 중단

# 데이터베이스 접속 정보
export PGPASSWORD=PacsExtension2024
PSQL="psql -h localhost -p 5456 -U pacs_extension_admin -d pacs_extension -t -A"

echo "🧪 DICOM API 접근 제어 테스트 시작..."
echo ""

# ============================================================================
# 1. 데이터베이스 직접 확인
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 데이터베이스 직접 확인"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Dr. Kim (책임연구원) - 전체 접근
echo "👤 Dr. Kim (책임연구원, user_id=101)"
echo "   예상: 전체 Study 접근 가능 (7개)"
echo ""

has_restrictions=$($PSQL <<EOF
SELECT EXISTS(
    SELECT 1 FROM project_data_access 
    WHERE user_id = 101 AND project_id = 90
);
EOF
)

if [ "$has_restrictions" = "f" ]; then
    echo "   ✅ 제약 없음 → 전체 접근 가능"
    total_studies=$($PSQL <<EOF
SELECT COUNT(*)
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
WHERE pd.project_id = 90;
EOF
)
    echo "   📊 접근 가능한 Study: $total_studies개"
else
    echo "   ❌ 제약 있음 (예상과 다름)"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Dr. Lee (A병원 연구원) - A병원 Study만
echo "👤 Dr. Lee (A병원 연구원, user_id=102)"
echo "   예상: A병원 Study만 접근 (3개)"
echo ""

lee_count=$($PSQL <<EOF
SELECT COUNT(*)
FROM project_data_access pda
INNER JOIN project_data_study pds ON pda.study_id = pds.id
WHERE pda.user_id = 102 
  AND pda.project_id = 90
  AND pda.status = 'APPROVED';
EOF
)

echo "   📊 접근 가능한 Study: $lee_count개"

if [ "$lee_count" = "3" ]; then
    echo "   ✅ 예상과 일치"
else
    echo "   ❌ 예상과 다름 (예상: 3개, 실제: $lee_count개)"
fi

echo ""
echo "   📋 Study 목록:"
$PSQL <<EOF | while read line; do echo "      - $line"; done
SELECT pds.study_uid || ' (' || pds.study_description || ')'
FROM project_data_access pda
INNER JOIN project_data_study pds ON pda.study_id = pds.id
WHERE pda.user_id = 102 
  AND pda.project_id = 90
  AND pda.status = 'APPROVED'
ORDER BY pds.id;
EOF

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Dr. Park (B병원 연구원) - B병원 Study만
echo "👤 Dr. Park (B병원 연구원, user_id=103)"
echo "   예상: B병원 Study만 접근 (3개)"
echo ""

park_count=$($PSQL <<EOF
SELECT COUNT(*)
FROM project_data_access pda
INNER JOIN project_data_study pds ON pda.study_id = pds.id
WHERE pda.user_id = 103 
  AND pda.project_id = 90
  AND pda.status = 'APPROVED';
EOF
)

echo "   📊 접근 가능한 Study: $park_count개"

if [ "$park_count" = "3" ]; then
    echo "   ✅ 예상과 일치"
else
    echo "   ❌ 예상과 다름 (예상: 3개, 실제: $park_count개)"
fi

echo ""
echo "   📋 Study 목록:"
$PSQL <<EOF | while read line; do echo "      - $line"; done
SELECT pds.study_uid || ' (' || pds.study_description || ')'
FROM project_data_access pda
INNER JOIN project_data_study pds ON pda.study_id = pds.id
WHERE pda.user_id = 103 
  AND pda.project_id = 90
  AND pda.status = 'APPROVED'
ORDER BY pds.id;
EOF

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Dr. Choi (임시 협력자) - Study 1만 (7일간)
echo "👤 Dr. Choi (임시 협력자, user_id=104)"
echo "   예상: Study 1001만 접근 (1개, 7일간, 읽기 전용)"
echo ""

choi_count=$($PSQL <<EOF
SELECT COUNT(*)
FROM project_data_access pda
INNER JOIN project_data_study pds ON pda.study_id = pds.id
WHERE pda.user_id = 104 
  AND pda.project_id = 90
  AND pda.status = 'APPROVED'
  AND (pda.expires_at IS NULL OR pda.expires_at > NOW());
EOF
)

echo "   📊 접근 가능한 Study: $choi_count개"

if [ "$choi_count" = "1" ]; then
    echo "   ✅ 예상과 일치"
else
    echo "   ❌ 예상과 다름 (예상: 1개, 실제: $choi_count개)"
fi

echo ""
echo "   📋 Study 목록:"
$PSQL <<EOF | while read line; do echo "      - $line"; done
SELECT pds.study_uid || ' (' || pds.study_description || ')'
FROM project_data_access pda
INNER JOIN project_data_study pds ON pda.study_id = pds.id
WHERE pda.user_id = 104 
  AND pda.project_id = 90
  AND pda.status = 'APPROVED'
  AND (pda.expires_at IS NULL OR pda.expires_at > NOW())
ORDER BY pds.id;
EOF

echo ""
echo "   🕐 만료 시간:"
expires_at=$($PSQL <<EOF
SELECT TO_CHAR(expires_at, 'YYYY-MM-DD HH24:MI:SS')
FROM project_data_access
WHERE user_id = 104 AND project_id = 90
LIMIT 1;
EOF
)
echo "      $expires_at"

echo ""
echo "   🔒 접근 범위:"
access_scope=$($PSQL <<EOF
SELECT access_scope
FROM project_data_access
WHERE user_id = 104 AND project_id = 90
LIMIT 1;
EOF
)
echo "      $access_scope"

if [ "$access_scope" = "READ_ONLY" ]; then
    echo "      ✅ 읽기 전용 (예상과 일치)"
else
    echo "      ❌ 예상과 다름 (예상: READ_ONLY, 실제: $access_scope)"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# ============================================================================
# 2. 접근 제어 매트릭스 출력
# ============================================================================
echo "📊 접근 제어 매트릭스"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 헤더 출력
printf "%-20s" "사용자"
for study_id in 1001 1002 1003 1004 1005 1006 1007; do
    printf "%-10s" "S$study_id"
done
echo ""

# 구분선
printf "%-20s" "--------------------"
for i in {1..7}; do
    printf "%-10s" "----------"
done
echo ""

# 각 사용자별 접근 권한 출력
for user_id in 101 102 103 104; do
    # 사용자 이름
    user_name=$($PSQL <<EOF
SELECT name FROM security_user WHERE id = $user_id;
EOF
)
    printf "%-20s" "$user_name"
    
    # 각 Study별 접근 권한 확인
    for study_id in 1001 1002 1003 1004 1005 1006 1007; do
        # 제약이 있는지 확인
        has_restrictions=$($PSQL <<EOF
SELECT EXISTS(
    SELECT 1 FROM project_data_access 
    WHERE user_id = $user_id AND project_id = 90
);
EOF
)
        
        if [ "$has_restrictions" = "f" ]; then
            # 제약 없음 → 전체 접근
            printf "%-10s" "✅"
        else
            # 제약 있음 → 승인된 Study만 접근
            is_approved=$($PSQL <<EOF
SELECT EXISTS(
    SELECT 1 FROM project_data_access pda
    WHERE pda.user_id = $user_id
      AND pda.project_id = 90
      AND pda.study_id = $study_id
      AND pda.status = 'APPROVED'
      AND (pda.expires_at IS NULL OR pda.expires_at > NOW())
);
EOF
)
            
            if [ "$is_approved" = "t" ]; then
                # 읽기 전용인지 확인
                access_scope=$($PSQL <<EOF
SELECT access_scope
FROM project_data_access
WHERE user_id = $user_id AND project_id = 90 AND study_id = $study_id
LIMIT 1;
EOF
)
                
                if [ "$access_scope" = "READ_ONLY" ]; then
                    printf "%-10s" "👁️"
                else
                    printf "%-10s" "✅"
                fi
            else
                printf "%-10s" "❌"
            fi
        fi
    done
    echo ""
done

echo ""
echo "범례:"
echo "  ✅ = 전체 접근 가능"
echo "  👁️ = 읽기 전용 접근"
echo "  ❌ = 접근 불가"
echo ""

# ============================================================================
# 3. 테스트 결과 요약
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ 테스트 완료!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📋 테스트 요약:"
echo "  - Dr. Kim: 전체 접근 (제약 없음)"
echo "  - Dr. Lee: A병원 Study만 ($lee_count개)"
echo "  - Dr. Park: B병원 Study만 ($park_count개)"
echo "  - Dr. Choi: Study 1001만 ($choi_count개, 읽기 전용, 만료: $expires_at)"
echo ""
echo "🎯 다음 단계:"
echo "  1. DICOM API로 실제 조회 테스트"
echo "  2. 만료 시간 경과 후 재테스트"
echo "  3. 접근 범위 변경 후 재테스트"
echo ""

