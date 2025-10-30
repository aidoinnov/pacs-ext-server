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
RDS_ENDPOINT_EXTENSION="pacs-extension.ciyua2gsk8ke.ap-northeast-2.rds.amazonaws.com"
RDS_ENDPOINT_POSTGRES="pacs-postgres.ciyua2gsk8ke.ap-northeast-2.rds.amazonaws.com"
KEY_PATH="~/.ssh/bastion-keypair.pem"
LOCAL_PORT_EXTENSION="5456"
LOCAL_PORT_POSTGRES="5457"
LOG_LEVEL="ERROR"
TARGET="extension"  # extension, postgres, both

# 터널 상태 확인 함수
check_tunnel_status() {
    local check_target=${1:-"both"}
    local found=false
    
    if [ "$check_target" = "extension" ] || [ "$check_target" = "both" ]; then
        local tunnels_ext=$(ps aux | grep "ssh.*-L.*${LOCAL_PORT_EXTENSION}:" | grep -v grep)
        if [ -n "$tunnels_ext" ]; then
            found=true
            echo -e "${GREEN}✅ Extension tunnel is running on port ${LOCAL_PORT_EXTENSION}${NC}"
            echo -e "${CYAN}📊 Active tunnels (Extension):${NC}"
            echo "$tunnels_ext" | while read line; do
                local pid=$(echo "$line" | awk '{print $2}')
                local user=$(echo "$line" | awk '{print $1}')
                local time=$(echo "$line" | awk '{print $9}')
                echo -e "   ${WHITE}PID: ${GREEN}${pid}${NC} | User: ${GREEN}${user}${NC} | Time: ${GREEN}${time}${NC}"
            done
            echo ""
        else
            echo -e "${RED}❌ No extension tunnel running on port ${LOCAL_PORT_EXTENSION}${NC}"
        fi
    fi
    
    if [ "$check_target" = "postgres" ] || [ "$check_target" = "both" ]; then
        local tunnels_pg=$(ps aux | grep "ssh.*-L.*${LOCAL_PORT_POSTGRES}:" | grep -v grep)
        if [ -n "$tunnels_pg" ]; then
            found=true
            echo -e "${GREEN}✅ Postgres tunnel is running on port ${LOCAL_PORT_POSTGRES}${NC}"
            echo -e "${CYAN}📊 Active tunnels (Postgres):${NC}"
            echo "$tunnels_pg" | while read line; do
                local pid=$(echo "$line" | awk '{print $2}')
                local user=$(echo "$line" | awk '{print $1}')
                local time=$(echo "$line" | awk '{print $9}')
                echo -e "   ${WHITE}PID: ${GREEN}${pid}${NC} | User: ${GREEN}${user}${NC} | Time: ${GREEN}${time}${NC}"
            done
            echo ""
        else
            echo -e "${RED}❌ No postgres tunnel running on port ${LOCAL_PORT_POSTGRES}${NC}"
        fi
    fi
    
    if [ "$found" = true ]; then
        return 0
    else
        return 1
    fi
}

# 터널 종료 함수
stop_tunnel() {
    local stop_target=${1:-"both"}
    
    if [ "$stop_target" = "extension" ] || [ "$stop_target" = "both" ]; then
        local pids_ext=$(ps aux | grep "ssh.*-L.*${LOCAL_PORT_EXTENSION}:" | grep -v grep | awk '{print $2}')
        if [ -n "$pids_ext" ]; then
            echo -e "${YELLOW}🛑 Stopping extension tunnels on port ${LOCAL_PORT_EXTENSION}...${NC}"
            echo "$pids_ext" | while read pid; do
                if kill "$pid" 2>/dev/null; then
                    echo -e "${GREEN}✅ Stopped extension tunnel PID: ${pid}${NC}"
                else
                    echo -e "${RED}❌ Failed to stop extension tunnel PID: ${pid}${NC}"
                fi
            done
        else
            echo -e "${YELLOW}⚠️  No extension tunnels found on port ${LOCAL_PORT_EXTENSION}${NC}"
        fi
    fi
    
    if [ "$stop_target" = "postgres" ] || [ "$stop_target" = "both" ]; then
        local pids_pg=$(ps aux | grep "ssh.*-L.*${LOCAL_PORT_POSTGRES}:" | grep -v grep | awk '{print $2}')
        if [ -n "$pids_pg" ]; then
            echo -e "${YELLOW}🛑 Stopping postgres tunnels on port ${LOCAL_PORT_POSTGRES}...${NC}"
            echo "$pids_pg" | while read pid; do
                if kill "$pid" 2>/dev/null; then
                    echo -e "${GREEN}✅ Stopped postgres tunnel PID: ${pid}${NC}"
                else
                    echo -e "${RED}❌ Failed to stop postgres tunnel PID: ${pid}${NC}"
                fi
            done
        else
            echo -e "${YELLOW}⚠️  No postgres tunnels found on port ${LOCAL_PORT_POSTGRES}${NC}"
        fi
    fi
}

# 도움말 함수
show_help() {
    echo -e "${WHITE}🔗 PACS Database Tunnel Script${NC}"
    echo -e "${CYAN}Usage: $0 [OPTIONS]${NC}"
    echo ""
    echo -e "${YELLOW}Options:${NC}"
    echo -e "  ${GREEN}-h, --help${NC}              Show this help message"
    echo -e "  ${GREEN}-t, --target TARGET${NC}     Target database: extension, postgres, both (default: extension)"
    echo -e "  ${GREEN}-p, --port PORT${NC}         Local port for extension (default: 5456)"
    echo -e "  ${GREEN}-P, --port-postgres PORT${NC} Local port for postgres (default: 5457)"
    echo -e "  ${GREEN}-l, --log-level LEVEL${NC}   SSH log level (default: ERROR)"
    echo -e "  ${GREEN}-v, --verbose${NC}           Verbose output"
    echo -e "  ${GREEN}-q, --quiet${NC}             Quiet mode"
    echo -e "  ${GREEN}-s, --status${NC}            Check tunnel status"
    echo -e "  ${GREEN}-k, --kill${NC}              Stop all tunnels"
    echo ""
    echo -e "${YELLOW}Target Options:${NC}"
    echo -e "  ${GREEN}extension${NC}  - Connect to pacs-extension RDS (port: ${LOCAL_PORT_EXTENSION})"
    echo -e "  ${GREEN}postgres${NC}   - Connect to pacs-postgres RDS (port: ${LOCAL_PORT_POSTGRES})"
    echo -e "  ${GREEN}both${NC}       - Connect to both databases"
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
    echo -e "  ${CYAN}$0${NC}                        # Start extension tunnel (default)"
    echo -e "  ${CYAN}$0 -t postgres${NC}            # Start postgres tunnel"
    echo -e "  ${CYAN}$0 -t both${NC}                # Start both tunnels"
    echo -e "  ${CYAN}$0 -p 5433 -P 5434${NC}        # Custom ports"
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
        -t|--target)
            TARGET="$2"
            if [ "$TARGET" != "extension" ] && [ "$TARGET" != "postgres" ] && [ "$TARGET" != "both" ]; then
                echo -e "${RED}❌ Invalid target: $TARGET${NC}"
                echo -e "${YELLOW}Valid targets: extension, postgres, both${NC}"
                exit 1
            fi
            shift 2
            ;;
        -p|--port)
            LOCAL_PORT_EXTENSION="$2"
            shift 2
            ;;
        -P|--port-postgres)
            LOCAL_PORT_POSTGRES="$2"
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
    check_tunnel_status "$TARGET"
    exit $?
fi

# 터널 종료 모드
if [ "$KILL_TUNNELS" = true ]; then
    stop_tunnel "$TARGET"
    exit 0
fi

# 조용한 모드가 아닌 경우에만 출력
if [ "$QUIET" = false ]; then
    echo -e "${WHITE}============================================================${NC}"
    echo -e "${WHITE}🔗 PACS Database Tunnel${NC}"
    echo -e "${WHITE}============================================================${NC}"
    echo -e "${BLUE}📡 Bastion Host:${NC} ${GREEN}${BASTION_HOST}${NC}"
    echo -e "${BLUE}🎯 Target:${NC} ${GREEN}${TARGET}${NC}"
    if [ "$TARGET" = "extension" ] || [ "$TARGET" = "both" ]; then
        echo -e "${BLUE}🗄️  Extension RDS:${NC} ${GREEN}${RDS_ENDPOINT_EXTENSION}${NC}"
        echo -e "${BLUE}🔌 Extension Port:${NC} ${GREEN}${LOCAL_PORT_EXTENSION}${NC}"
    fi
    if [ "$TARGET" = "postgres" ] || [ "$TARGET" = "both" ]; then
        echo -e "${BLUE}🗄️  Postgres RDS:${NC} ${GREEN}${RDS_ENDPOINT_POSTGRES}${NC}"
        echo -e "${BLUE}🔌 Postgres Port:${NC} ${GREEN}${LOCAL_PORT_POSTGRES}${NC}"
    fi
    echo -e "${BLUE}📝 Log Level:${NC} ${GREEN}${LOG_LEVEL}${NC}"
    echo -e "${BLUE}🔑 Key Path:${NC} ${GREEN}${KEY_PATH}${NC}"
    echo -e "${WHITE}============================================================${NC}"
    
    if [ "$VERBOSE" = true ]; then
        echo -e "${YELLOW}🔍 Verbose mode enabled${NC}"
    fi
    
    echo -e "${PURPLE}🚀 Starting tunnel(s)...${NC}"
fi

# 터널 시작 함수
start_tunnel() {
    local endpoint=$1
    local local_port=$2
    local name=$3
    
    if [ "$QUIET" = false ]; then
        echo -e "${CYAN}🔗 Starting ${name} tunnel on port ${local_port}...${NC}"
    fi
    
    ssh -i ${KEY_PATH} \
        -L ${local_port}:${endpoint}:5432 \
        ec2-user@${BASTION_HOST} \
        -N \
        -o StrictHostKeyChecking=no \
        -o UserKnownHostsFile=/dev/null \
        -o LogLevel=${LOG_LEVEL} &
    
    local tunnel_pid=$!
    sleep 1
    
    if kill -0 "$tunnel_pid" 2>/dev/null; then
        if [ "$QUIET" = false ]; then
            echo -e "${GREEN}✅ ${name} tunnel started successfully!${NC}"
            echo -e "${CYAN}   Process ID: ${WHITE}${tunnel_pid}${NC}"
            echo -e "${CYAN}   Connect to: ${WHITE}localhost:${local_port}${NC}"
            echo ""
        fi
        return 0
    else
        if [ "$QUIET" = false ]; then
            echo -e "${RED}❌ Failed to start ${name} tunnel${NC}"
        fi
        return 1
    fi
}

# 터널 시작
SUCCESS=true

if [ "$TARGET" = "extension" ] || [ "$TARGET" = "both" ]; then
    if ! start_tunnel "$RDS_ENDPOINT_EXTENSION" "$LOCAL_PORT_EXTENSION" "Extension"; then
        SUCCESS=false
    fi
fi

if [ "$TARGET" = "postgres" ] || [ "$TARGET" = "both" ]; then
    if ! start_tunnel "$RDS_ENDPOINT_POSTGRES" "$LOCAL_PORT_POSTGRES" "Postgres"; then
        SUCCESS=false
    fi
fi

# 조용한 모드가 아닌 경우에만 결과 출력
if [ "$QUIET" = false ]; then
    if [ "$SUCCESS" = true ]; then
        echo ""
        echo -e "${GREEN}🎉 All tunnels are ready!${NC}"
        echo ""
        echo -e "${YELLOW}💡 DBeaver Connection Examples:${NC}"
        if [ "$TARGET" = "extension" ] || [ "$TARGET" = "both" ]; then
            echo -e "${CYAN}   Extension:${NC}"
            echo -e "      ${WHITE}Host:${NC} localhost"
            echo -e "      ${WHITE}Port:${NC} ${LOCAL_PORT_EXTENSION}"
            echo -e "      ${WHITE}Database:${NC} pacs_db"
            echo ""
        fi
        if [ "$TARGET" = "postgres" ] || [ "$TARGET" = "both" ]; then
            echo -e "${CYAN}   Postgres:${NC}"
            echo -e "      ${WHITE}Host:${NC} localhost"
            echo -e "      ${WHITE}Port:${NC} ${LOCAL_PORT_POSTGRES}"
            echo -e "      ${WHITE}Database:${NC} (your database name)"
            echo ""
        fi
        echo -e "${YELLOW}🛑 Stop tunnels:${NC}"
        if [ "$TARGET" = "both" ]; then
            echo -e "   ${WHITE}$0 -k${NC} or ${WHITE}$0 -k -t both${NC}"
        else
            echo -e "   ${WHITE}$0 -k -t ${TARGET}${NC}"
        fi
    else
        echo -e "${RED}❌ Some tunnels failed to start${NC}"
        exit 1
    fi
fi

