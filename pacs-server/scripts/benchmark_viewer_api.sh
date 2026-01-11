#!/bin/bash

# Viewer API 성능 벤치마크 스크립트
# 
# 사용법:
#   ./scripts/benchmark_viewer_api.sh <JWT_TOKEN>

set -e

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 로그 함수
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_perf() {
    echo -e "${CYAN}⏱️  $1${NC}"
}

# JWT 토큰 확인
if [ -z "$1" ]; then
    log_error "JWT token is required"
    echo "Usage: $0 <JWT_TOKEN>"
    exit 1
fi

JWT_TOKEN="$1"
BASE_URL="http://localhost:8080"

log_info "Viewer API 성능 벤치마크 시작"
echo ""

# 성능 측정 함수
measure_performance() {
    local test_name="$1"
    local endpoint="$2"
    local payload="$3"
    local expected_time="$4"
    
    log_info "테스트: $test_name"
    
    local start_time=$(date +%s.%N)
    
    local response=$(curl -s -w "\n%{http_code}\n%{time_total}" -X POST "${BASE_URL}${endpoint}" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer ${JWT_TOKEN}" \
        -d "$payload")
    
    local end_time=$(date +%s.%N)
    local elapsed=$(echo "$end_time - $start_time" | bc)
    local http_code=$(echo "$response" | tail -n 2 | head -n 1)
    local curl_time=$(echo "$response" | tail -n 1)
    
    log_perf "응답 시간: ${curl_time}초 (총 ${elapsed}초)"
    echo "  HTTP 상태: $http_code"
    
    # 성능 기준 체크
    local time_check=$(echo "$curl_time < $expected_time" | bc -l)
    if [ "$time_check" -eq 1 ]; then
        log_success "성능 기준 통과 (< ${expected_time}초)"
    else
        log_warning "성능 기준 초과 (예상: < ${expected_time}초)"
    fi
    
    echo ""
}

# 테스트 1: 10개 Study UID
log_info "=== 테스트 1: 10개 Study UID ==="
echo ""

STUDY_UIDS_10='["1.2.840.113619.2.55.3.604688433.1","1.2.840.113619.2.55.3.604688433.2","1.2.840.113619.2.55.3.604688433.3","1.2.840.113619.2.55.3.604688433.4","1.2.840.113619.2.55.3.604688433.5","1.2.840.113619.2.55.3.604688433.6","1.2.840.113619.2.55.3.604688433.7","1.2.840.113619.2.55.3.604688433.8","1.2.840.113619.2.55.3.604688433.9","1.2.840.113619.2.55.3.604688433.10"]'

measure_performance \
    "10개 Study UID 조회" \
    "/api/v1/viewer/studies/meta" \
    "{\"study_uids\": $STUDY_UIDS_10, \"max_count\": 20}" \
    "5.0"

# 테스트 2: 50개 Study UID
log_info "=== 테스트 2: 50개 Study UID ==="
echo ""

# 50개 UID 생성
STUDY_UIDS_50=$(seq 1 50 | jq -R . | jq -s 'map("1.2.840.113619.2.55.3.604688433.\(.)")')

measure_performance \
    "50개 Study UID 조회" \
    "/api/v1/viewer/studies/meta" \
    "{\"study_uids\": $STUDY_UIDS_50, \"max_count\": 100}" \
    "15.0"

# 테스트 3: 100개 Study UID
log_info "=== 테스트 3: 100개 Study UID ==="
echo ""

STUDY_UIDS_100=$(seq 1 100 | jq -R . | jq -s 'map("1.2.840.113619.2.55.3.604688433.\(.)")')

measure_performance \
    "100개 Study UID 조회" \
    "/api/v1/viewer/studies/meta" \
    "{\"study_uids\": $STUDY_UIDS_100, \"max_count\": 100}" \
    "30.0"

# 테스트 4: 50개 Series UID
log_info "=== 테스트 4: 50개 Series UID ==="
echo ""

SERIES_UIDS_50=$(seq 1 50 | jq -R . | jq -s 'map("1.2.840.113619.2.55.3.604688433.1234.\(.)")')

measure_performance \
    "50개 Series UID 조회" \
    "/api/v1/viewer/series/meta" \
    "{\"series_uids\": $SERIES_UIDS_50, \"max_count\": 100}" \
    "15.0"

# 테스트 5: 200개 Series UID
log_info "=== 테스트 5: 200개 Series UID ==="
echo ""

SERIES_UIDS_200=$(seq 1 200 | jq -R . | jq -s 'map("1.2.840.113619.2.55.3.604688433.1234.\(.)")')

measure_performance \
    "200개 Series UID 조회" \
    "/api/v1/viewer/series/meta" \
    "{\"series_uids\": $SERIES_UIDS_200, \"max_count\": 200}" \
    "60.0"

log_success "벤치마크 완료!"

