#!/bin/bash

# Subject 생성 스크립트
# 프로젝트에 할당된 Study들에 대해 Subject를 자동으로 생성합니다.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 색상 정의
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 도움말 출력
show_help() {
    cat << EOF
Subject 생성 스크립트

Usage:
    $0 [OPTIONS]

Options:
    -p, --project-id ID     특정 프로젝트 ID에 대해 Subject 생성
    -a, --all-projects      모든 활성 프로젝트에 대해 Subject 생성
    -d, --dry-run           실제 생성하지 않고 시뮬레이션만 수행
    -h, --help              이 도움말 출력

Examples:
    # 특정 프로젝트만 Subject 생성
    $0 --project-id 1

    # 모든 프로젝트에 Subject 생성
    $0 --all-projects

    # Dry-run 모드 (실제 생성 안 함)
    $0 --project-id 1 --dry-run

    # 모든 프로젝트 Dry-run
    $0 --all-projects --dry-run

EOF
}

# 인자 파싱
PROJECT_ID=""
ALL_PROJECTS=false
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--project-id)
            PROJECT_ID="$2"
            shift 2
            ;;
        -a|--all-projects)
            ALL_PROJECTS=true
            shift
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# 인자 검증
if [[ -z "$PROJECT_ID" && "$ALL_PROJECTS" == false ]]; then
    echo -e "${RED}Error: --project-id 또는 --all-projects 중 하나를 지정해야 합니다${NC}"
    show_help
    exit 1
fi

if [[ -n "$PROJECT_ID" && "$ALL_PROJECTS" == true ]]; then
    echo -e "${RED}Error: --project-id와 --all-projects를 동시에 사용할 수 없습니다${NC}"
    show_help
    exit 1
fi

# Python 가상환경 확인 및 활성화
if [[ -d "$PROJECT_ROOT/venv" ]]; then
    echo -e "${GREEN}✓ Activating virtual environment...${NC}"
    source "$PROJECT_ROOT/venv/bin/activate"
elif [[ -d "$PROJECT_ROOT/.venv" ]]; then
    echo -e "${GREEN}✓ Activating virtual environment...${NC}"
    source "$PROJECT_ROOT/.venv/bin/activate"
fi

# 필요한 패키지 확인
if ! python3 -c "import psycopg2" 2>/dev/null; then
    echo -e "${YELLOW}⚠ Installing required packages...${NC}"
    pip install psycopg2-binary
fi

# Python 스크립트 실행
echo -e "${GREEN}Starting Subject creation...${NC}"
echo ""

PYTHON_ARGS=()

if [[ -n "$PROJECT_ID" ]]; then
    PYTHON_ARGS+=("--project-id" "$PROJECT_ID")
fi

if [[ "$ALL_PROJECTS" == true ]]; then
    PYTHON_ARGS+=("--all-projects")
fi

if [[ "$DRY_RUN" == true ]]; then
    PYTHON_ARGS+=("--dry-run")
    echo -e "${YELLOW}🔍 DRY-RUN MODE: 실제 생성하지 않습니다${NC}"
    echo ""
fi

# Python 스크립트 실행
python3 "$SCRIPT_DIR/migrate_subjects.py" "${PYTHON_ARGS[@]}"

echo ""
echo -e "${GREEN}✓ Done!${NC}"

