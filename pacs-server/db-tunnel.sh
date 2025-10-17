#!/bin/bash

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# 기본 설정
BASTION_HOST="13.125.228.206"
RDS_ENDPOINT="pacs-extension.ciyua2gsk8ke.ap-northeast-2.rds.amazonaws.com"
KEY_PATH="~/.ssh/bastion-keypair.pem"
LOCAL_PORT="5456"
LOG_LEVEL="ERROR"

# 터널 상태 확인 함수
check_tunnel_status() {
    local port=$1
    local tunnels=$(ps aux | grep "ssh.*-L.*${port}:" | grep -v grep)
    
    if [ -n "$tunnels" ]; then
        echo -e "${GREEN}✅ Tunnel is running on port ${port}${NC}"
        echo -e "${CYAN}📊 Active tunnels:${NC}"
        echo "$tunnels" | while read line; do
            local pid=$(echo "$line" | awk '{print $2}')
            local user=$(echo "$line" | awk '{print $1}')
            local time=$(echo "$line" | awk '{print $9}')
            echo -e "   ${WHITE}PID: ${GREEN}${pid}${NC} | User: ${GREEN}${user}${NC} | Time: ${GREEN}${time}${NC}"
        done
        return 0
    else
        echo -e "${RED}❌ No tunnel running on port ${port}${NC}"
        return 1
    fi
}

# 터널 종료 함수
stop_tunnel() {
    local port=$1
    local pids=$(ps aux | grep "ssh.*-L.*${port}:" | grep -v grep | awk '{print $2}')
    
    if [ -n "$pids" ]; then
        echo -e "${YELLOW}🛑 Stopping tunnels on port ${port}...${NC}"
        echo "$pids" | while read pid; do
            if kill "$pid" 2>/dev/null; then
                echo -e "${GREEN}✅ Stopped tunnel PID: ${pid}${NC}"
            else
                echo -e "${RED}❌ Failed to stop tunnel PID: ${pid}${NC}"
            fi
        done
    else
        echo -e "${YELLOW}⚠️  No tunnels found on port ${port}${NC}"
    fi
}

# 도움말 함수
show_help() {
    echo -e "${WHITE}🔗 PACS Database Tunnel Script${NC}"
    echo -e "${CYAN}Usage: $0 [OPTIONS]${NC}"
    echo ""
    echo -e "${YELLOW}Options:${NC}"
    echo -e "  ${GREEN}-h, --help${NC}              Show this help message"
    echo -e "  ${GREEN}-p, --port PORT${NC}         Local port (default: 5432)"
    echo -e "  ${GREEN}-l, --log-level LEVEL${NC}   SSH log level (default: ERROR)"
    echo -e "  ${GREEN}-v, --verbose${NC}           Verbose output"
    echo -e "  ${GREEN}-q, --quiet${NC}             Quiet mode"
    echo -e "  ${GREEN}-s, --status${NC}            Check tunnel status"
    echo -e "  ${GREEN}-k, --kill${NC}              Stop all tunnels"
    echo ""
    echo -e "${YELLOW}Log Levels:${NC}"
    echo -e "  ${GREEN}QUIET${NC}     - No output"
    echo -e "  ${GREEN}FATAL${NC}     - Fatal errors only"
    echo -e "  ${GREEN}ERROR${NC}     - Error messages (default)"
    echo -e "  ${GREEN}INFO${NC}      - Informational messages"
    echo -e "  ${GREEN}VERBOSE${NC}   - Verbose output"
    echo -e "  ${GREEN}DEBUG1${NC}    - Debug level 1"
    echo -e "  ${GREEN}DEBUG2${NC}    - Debug level 2"
    echo -e "  ${GREEN}DEBUG3${NC}    - Debug level 3"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo -e "  ${CYAN}$0${NC}                        # Start tunnel (default settings)"
    echo -e "  ${CYAN}$0 -p 5433${NC}               # Use port 5433"
    echo -e "  ${CYAN}$0 -l INFO -v${NC}             # Verbose with INFO level"
    echo -e "  ${CYAN}$0 -q${NC}                     # Quiet mode"
    echo -e "  ${CYAN}$0 -s${NC}                     # Check status"
    echo -e "  ${CYAN}$0 -k${NC}                     # Stop all tunnels"
}

# 파라미터 파싱
VERBOSE=false
QUIET=false
CHECK_STATUS=false
KILL_TUNNELS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -p|--port)
            LOCAL_PORT="$2"
            shift 2
            ;;
        -l|--log-level)
            LOG_LEVEL="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            LOG_LEVEL="INFO"
            shift
            ;;
        -q|--quiet)
            QUIET=true
            LOG_LEVEL="QUIET"
            shift
            ;;
        -s|--status)
            CHECK_STATUS=true
            shift
            ;;
        -k|--kill)
            KILL_TUNNELS=true
            shift
            ;;
        *)
            echo -e "${RED}❌ Unknown option: $1${NC}"
            echo -e "${YELLOW}Use -h or --help for usage information${NC}"
            exit 1
            ;;
    esac
done

# 상태 확인 모드
if [ "$CHECK_STATUS" = true ]; then
    check_tunnel_status "$LOCAL_PORT"
    exit $?
fi

# 터널 종료 모드
if [ "$KILL_TUNNELS" = true ]; then
    stop_tunnel "$LOCAL_PORT"
    exit 0
fi

# 조용한 모드가 아닌 경우에만 출력
if [ "$QUIET" = false ]; then
    echo -e "${WHITE}============================================================${NC}"
    echo -e "${WHITE}🔗 PACS Database Tunnel${NC}"
    echo -e "${WHITE}============================================================${NC}"
    echo -e "${BLUE}📡 Bastion Host:${NC} ${GREEN}${BASTION_HOST}${NC}"
    echo -e "${BLUE}🗄️  RDS Endpoint:${NC} ${GREEN}${RDS_ENDPOINT}${NC}"
    echo -e "${BLUE}🔌 Local Port:${NC} ${GREEN}${LOCAL_PORT}${NC}"
    echo -e "${BLUE}📝 Log Level:${NC} ${GREEN}${LOG_LEVEL}${NC}"
    echo -e "${BLUE}🔑 Key Path:${NC} ${GREEN}${KEY_PATH}${NC}"
    echo -e "${WHITE}${'='*60}${NC}"
    
    if [ "$VERBOSE" = true ]; then
        echo -e "${YELLOW}🔍 Verbose mode enabled${NC}"
    fi
    
    echo -e "${PURPLE}🚀 Starting tunnel...${NC}"
fi

# SSH 터널 시작
ssh -i ${KEY_PATH} \
    -L ${LOCAL_PORT}:${RDS_ENDPOINT}:5432 \
    ec2-user@${BASTION_HOST} \
    -N \
    -o StrictHostKeyChecking=no \
    -o UserKnownHostsFile=/dev/null \
    -o LogLevel=${LOG_LEVEL} &

# 백그라운드 프로세스 ID 저장
TUNNEL_PID=$!

# 조용한 모드가 아닌 경우에만 결과 출력
if [ "$QUIET" = false ]; then
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Tunnel started successfully!${NC}"
        echo -e "${CYAN}📊 Process ID: ${WHITE}${TUNNEL_PID}${NC}"
        echo -e "${CYAN}🔌 Connect to: ${WHITE}localhost:${LOCAL_PORT}${NC}"
        echo -e "${CYAN}📝 Stop tunnel: ${WHITE}kill ${TUNNEL_PID}${NC}"
        echo -e "${CYAN}🛑 Or kill all: ${WHITE}pkill -f 'ssh.*${LOCAL_PORT}'${NC}"
        echo ""
        echo -e "${YELLOW}💡 DBeaver Connection:${NC}"
        echo -e "   ${WHITE}Host:${NC} localhost"
        echo -e "   ${WHITE}Port:${NC} ${LOCAL_PORT}"
        echo -e "   ${WHITE}Database:${NC} pacs_db"
        echo ""
        echo -e "${GREEN}🎉 Ready to connect!${NC}"
    else
        echo -e "${RED}❌ Failed to start tunnel${NC}"
        exit 1
    fi
fi

