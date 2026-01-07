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
# Redis 설정 (Kubernetes 클러스터를 통한 접근)
KUBERNETES_HOST="${KUBERNETES_HOST:-192.168.0.202}"
KUBERNETES_USER="${KUBERNETES_USER:-dl-server102}"
KUBERNETES_PASSWORD="${KUBERNETES_PASSWORD:-d544}"  # 환경 변수로 오버라이드 가능
KUBERNETES_KEY="${KUBERNETES_KEY:-}"  # SSH 키가 있으면 사용, 없으면 비밀번호 사용
REDIS_NAMESPACE="${REDIS_NAMESPACE:-pacs}"
REDIS_SERVICE="${REDIS_SERVICE:-redis}"
REDIS_PORT="${REDIS_PORT:-6379}"
# KEY_PATH="~/.ssh/bastion-keypair.pem"
# 스크립트 디렉토리 기준으로 상대 경로 계산
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KEY_PATH="${SCRIPT_DIR}/../ssh/bastion-keypair.pem"
LOCAL_PORT_EXTENSION="5456"
LOCAL_PORT_POSTGRES="5457"
LOCAL_PORT_REDIS="6379"
LOG_LEVEL="ERROR"
TARGET="extension"  # extension, postgres, redis, both, all

# 포트 사용 확인 함수
port_in_use() {
    local port="$1"
    if command -v lsof >/dev/null 2>&1; then
        lsof -iTCP:$port -sTCP:LISTEN >/dev/null 2>&1
    elif command -v ss >/dev/null 2>&1; then
        ss -ltn "( sport = :$port )" 2>/dev/null | grep -q LISTEN
    elif command -v netstat >/dev/null 2>&1; then
        netstat -ltn 2>/dev/null | awk '{print $4}' | grep -Eq "[.:]$port\$"
    else
        # fallback: 시도 후 에러 코드로 판별
        (echo > /dev/tcp/127.0.0.1/$port) >/dev/null 2>&1
    fi
}

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
    
    if [ "$check_target" = "postgres" ] || [ "$check_target" = "both" ] || [ "$check_target" = "all" ]; then
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
    
    if [ "$check_target" = "redis" ] || [ "$check_target" = "all" ]; then
        local tunnels_redis=$(ps aux | grep "ssh.*-L.*${LOCAL_PORT_REDIS}:" | grep -v grep)
        if [ -n "$tunnels_redis" ]; then
            found=true
            echo -e "${GREEN}✅ Redis tunnel is running on port ${LOCAL_PORT_REDIS}${NC}"
            echo -e "${CYAN}📊 Active tunnels (Redis):${NC}"
            echo "$tunnels_redis" | while read line; do
                local pid=$(echo "$line" | awk '{print $2}')
                local user=$(echo "$line" | awk '{print $1}')
                local time=$(echo "$line" | awk '{print $9}')
                echo -e "   ${WHITE}PID: ${GREEN}${pid}${NC} | User: ${GREEN}${user}${NC} | Time: ${GREEN}${time}${NC}"
            done
            echo ""
        else
            echo -e "${RED}❌ No redis tunnel running on port ${LOCAL_PORT_REDIS}${NC}"
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
    
    if [ "$stop_target" = "postgres" ] || [ "$stop_target" = "both" ] || [ "$stop_target" = "all" ]; then
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
    
    if [ "$stop_target" = "redis" ] || [ "$stop_target" = "all" ]; then
        # SSH 터널 프로세스 종료
        local pids_redis=$(ps aux | grep "ssh.*-L.*${LOCAL_PORT_REDIS}:" | grep -v grep | awk '{print $2}')
        if [ -n "$pids_redis" ]; then
            echo -e "${YELLOW}🛑 Stopping redis tunnels on port ${LOCAL_PORT_REDIS}...${NC}"
            for pid in $pids_redis; do
                if kill "$pid" 2>/dev/null; then
                    echo -e "${GREEN}✅ Stopped redis tunnel PID: ${pid}${NC}"
                else
                    echo -e "${RED}❌ Failed to stop redis tunnel PID: ${pid}${NC}"
                fi
            done
        fi
        
        # expect 프로세스 종료
        local expect_pids=$(ps aux | grep "expect.*redis-tunnel-expect" | grep -v grep | awk '{print $2}')
        if [ -n "$expect_pids" ]; then
            for pid in $expect_pids; do
                kill "$pid" 2>/dev/null && echo -e "${GREEN}✅ Stopped expect process PID: ${pid}${NC}" || true
            done
        fi
        
        # kubectl port-forward 프로세스 종료 (원격 서버에서 실행 중일 수 있음)
        # 로컬에서 찾을 수 있는 경우만 종료
        local kubectl_pids=$(ps aux | grep "kubectl.*port-forward.*redis" | grep -v grep | awk '{print $2}')
        if [ -n "$kubectl_pids" ]; then
            for pid in $kubectl_pids; do
                kill "$pid" 2>/dev/null && echo -e "${GREEN}✅ Stopped kubectl process PID: ${pid}${NC}" || true
            done
        fi
        
        if [ -z "$pids_redis" ] && [ -z "$expect_pids" ]; then
            echo -e "${YELLOW}⚠️  No redis tunnels found on port ${LOCAL_PORT_REDIS}${NC}"
        fi
        
        # 임시 파일 정리
        rm -f /tmp/redis-tunnel-expect.* 2>/dev/null || true
    fi
}

# 도움말 함수
show_help() {
    echo -e "${WHITE}🔗 PACS Database Tunnel Script${NC}"
    echo -e "${CYAN}Usage: $0 [OPTIONS]${NC}"
    echo ""
    echo -e "${YELLOW}Options:${NC}"
    echo -e "  ${GREEN}-h, --help${NC}              Show this help message"
    echo -e "  ${GREEN}-t, --target TARGET${NC}     Target database: extension, postgres, redis, both, all (default: extension)"
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
    echo -e "  ${GREEN}redis${NC}      - Connect to Redis (port: ${LOCAL_PORT_REDIS})"
    echo -e "  ${GREEN}both${NC}       - Connect to both databases (extension + postgres)"
    echo -e "  ${GREEN}all${NC}        - Connect to all (extension + postgres + redis)"
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
            if [ "$TARGET" != "extension" ] && [ "$TARGET" != "postgres" ] && [ "$TARGET" != "redis" ] && [ "$TARGET" != "both" ] && [ "$TARGET" != "all" ]; then
                echo -e "${RED}❌ Invalid target: $TARGET${NC}"
                echo -e "${YELLOW}Valid targets: extension, postgres, redis, both, all${NC}"
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
    if [ "$TARGET" = "extension" ] || [ "$TARGET" = "both" ] || [ "$TARGET" = "all" ]; then
        echo -e "${BLUE}🗄️  Extension RDS:${NC} ${GREEN}${RDS_ENDPOINT_EXTENSION}${NC}"
        echo -e "${BLUE}🔌 Extension Port:${NC} ${GREEN}${LOCAL_PORT_EXTENSION}${NC}"
    fi
    if [ "$TARGET" = "postgres" ] || [ "$TARGET" = "both" ] || [ "$TARGET" = "all" ]; then
        echo -e "${BLUE}🗄️  Postgres RDS:${NC} ${GREEN}${RDS_ENDPOINT_POSTGRES}${NC}"
        echo -e "${BLUE}🔌 Postgres Port:${NC} ${GREEN}${LOCAL_PORT_POSTGRES}${NC}"
    fi
    if [ "$TARGET" = "redis" ] || [ "$TARGET" = "all" ]; then
        echo -e "${BLUE}🗄️  Kubernetes Host:${NC} ${GREEN}${KUBERNETES_HOST}${NC}"
        echo -e "${BLUE}🔌 Redis Port:${NC} ${GREEN}${LOCAL_PORT_REDIS}${NC}"
        echo -e "${BLUE}📦 Namespace:${NC} ${GREEN}${REDIS_NAMESPACE}${NC}"
        echo -e "${BLUE}🔴 Service:${NC} ${GREEN}${REDIS_SERVICE}${NC}"
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
    local remote_port=$3
    local name=$4
    
    if [ "$QUIET" = false ]; then
        echo -e "${CYAN}🔗 Starting ${name} tunnel on port ${local_port}...${NC}"
    fi
    
    ssh -i ${KEY_PATH} \
        -L ${local_port}:${endpoint}:${remote_port} \
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

# Redis 터널 시작 함수 (Kubernetes 클러스터를 통한 접근)
start_redis_tunnel() {
    if [ "$QUIET" = false ]; then
        echo -e "${CYAN}🔗 Starting Redis tunnel via Kubernetes cluster...${NC}"
        echo -e "${CYAN}   Kubernetes Host: ${KUBERNETES_HOST}${NC}"
        echo -e "${CYAN}   Namespace: ${REDIS_NAMESPACE}${NC}"
        echo -e "${CYAN}   Service: ${REDIS_SERVICE}${NC}"
    fi
    
    # SSH 터널을 만들면서 원격에서 kubectl port-forward 실행
    # expect를 사용하여 비밀번호 자동 입력
    local expect_script=$(cat <<'EXPECT_EOF'
#!/usr/bin/expect -f
set timeout 30
set password [lindex $argv 0]
set user [lindex $argv 1]
set host [lindex $argv 2]
set local_port [lindex $argv 3]
set remote_port [lindex $argv 4]
set namespace [lindex $argv 5]
set service [lindex $argv 6]

# 원격 서버에서 사용할 임시 포트 (로컬 6379와 충돌 방지, 매우 높은 포트 사용)
# 60000 이상의 포트를 사용하여 충돌 가능성 최소화
# 동적으로 사용 가능한 포트를 찾기 위해 타임스탬프 기반 포트 사용
set remote_temp_port [expr {60000 + ([clock seconds] % 1000)}]
spawn ssh -L $local_port:localhost:$remote_temp_port $user@$host -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -o ConnectTimeout=10 "kubectl port-forward -n $namespace svc/$service $remote_temp_port:$remote_port"

expect {
    "password:" {
        send "$password\r"
        exp_continue
    }
    "(yes/no)?" {
        send "yes\r"
        exp_continue
    }
    "Are you sure you want to continue connecting" {
        send "yes\r"
        exp_continue
    }
    "Forwarding from" {
        # 포트포워딩 시작되면 무한 대기 (백그라운드에서 interact 대신 사용)
        set timeout -1
        expect eof
    }
    timeout {
        exit 1
    }
    eof {
        exit 1
    }
}
EXPECT_EOF
)
    
    # 기존 임시 파일 정리
    rm -f /tmp/redis-tunnel-expect.* 2>/dev/null || true
    
    # expect 스크립트를 임시 파일로 저장하고 실행
    local expect_file=$(mktemp /tmp/redis-tunnel-expect.XXXXXX) || {
        if [ "$QUIET" = false ]; then
            echo -e "${RED}❌ Failed to create temporary file${NC}"
        fi
        return 1
    }
    echo "$expect_script" > "$expect_file"
    chmod +x "$expect_file"
    
    # 백그라운드로 실행 (로그는 임시 파일로 저장)
    # nohup으로 부모 프로세스 종료 후에도 터널 유지
    local log_file="${expect_file}.log"
    nohup $expect_file "$KUBERNETES_PASSWORD" "$KUBERNETES_USER" "$KUBERNETES_HOST" "$LOCAL_PORT_REDIS" "$REDIS_PORT" "$REDIS_NAMESPACE" "$REDIS_SERVICE" >"$log_file" 2>&1 &

    local expect_pid=$!
    disown $expect_pid 2>/dev/null || true
    sleep 5  # expect 스크립트가 SSH 연결하고 비밀번호 입력하는 시간 확보
    
    # 포트가 실제로 열렸는지 확인 (최대 15초 대기)
    for i in {1..30}; do
        if port_in_use "$LOCAL_PORT_REDIS"; then
            if [ "$QUIET" = false ]; then
                echo -e "${GREEN}✅ Redis tunnel started successfully!${NC}"
                echo -e "${CYAN}   Process ID: ${WHITE}${expect_pid}${NC}"
                echo -e "${CYAN}   Connect to: ${WHITE}localhost:${LOCAL_PORT_REDIS}${NC}"
                echo -e "${CYAN}   Method: SSH → kubectl port-forward${NC}"
                echo ""
            fi
            return 0
        fi
        # expect 프로세스가 살아있는지 확인
        if ! kill -0 "$expect_pid" 2>/dev/null; then
            # 프로세스가 종료되었는데 포트가 열리지 않았다면 실패
            if [ "$QUIET" = false ]; then
                echo -e "${YELLOW}⚠️  Expect 프로세스가 종료되었습니다.${NC}"
                if [ -f "$log_file" ] && [ -s "$log_file" ]; then
                    echo -e "${YELLOW}   로그 (마지막 5줄):${NC}"
                    tail -5 "$log_file" | sed 's/^/   /'
                fi
            fi
            [ -f "$log_file" ] && rm -f "$log_file"
            break
        fi
        sleep 0.5
    done
    
    if [ "$QUIET" = false ]; then
        echo -e "${RED}❌ Failed to start Redis tunnel${NC}"
        echo -e "${YELLOW}   Check if kubectl is available on ${KUBERNETES_HOST}${NC}"
        echo -e "${YELLOW}   Check if Redis service exists: kubectl -n ${REDIS_NAMESPACE} get svc ${REDIS_SERVICE}${NC}"
        if [ -f "$log_file" ] && [ -s "$log_file" ]; then
            echo -e "${YELLOW}   로그 파일: ${log_file}${NC}"
        fi
    fi
    [ -f "$log_file" ] && rm -f "$log_file"
    return 1
}

# 터널 시작
SUCCESS=true

if [ "$TARGET" = "extension" ] || [ "$TARGET" = "both" ] || [ "$TARGET" = "all" ]; then
    if ! start_tunnel "$RDS_ENDPOINT_EXTENSION" "$LOCAL_PORT_EXTENSION" "5432" "Extension"; then
        SUCCESS=false
    fi
fi

if [ "$TARGET" = "postgres" ] || [ "$TARGET" = "both" ] || [ "$TARGET" = "all" ]; then
    if ! start_tunnel "$RDS_ENDPOINT_POSTGRES" "$LOCAL_PORT_POSTGRES" "5432" "Postgres"; then
        SUCCESS=false
    fi
fi

if [ "$TARGET" = "redis" ] || [ "$TARGET" = "all" ]; then
    if ! start_redis_tunnel; then
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
        if [ "$TARGET" = "extension" ] || [ "$TARGET" = "both" ] || [ "$TARGET" = "all" ]; then
            echo -e "${CYAN}   Extension:${NC}"
            echo -e "      ${WHITE}Host:${NC} localhost"
            echo -e "      ${WHITE}Port:${NC} ${LOCAL_PORT_EXTENSION}"
            echo -e "      ${WHITE}Database:${NC} pacs_db"
            echo ""
        fi
        if [ "$TARGET" = "postgres" ] || [ "$TARGET" = "both" ] || [ "$TARGET" = "all" ]; then
            echo -e "${CYAN}   Postgres:${NC}"
            echo -e "      ${WHITE}Host:${NC} localhost"
            echo -e "      ${WHITE}Port:${NC} ${LOCAL_PORT_POSTGRES}"
            echo -e "      ${WHITE}Database:${NC} (your database name)"
            echo ""
        fi
        if [ "$TARGET" = "redis" ] || [ "$TARGET" = "all" ]; then
            echo -e "${CYAN}   Redis:${NC}"
            echo -e "      ${WHITE}Host:${NC} localhost"
            echo -e "      ${WHITE}Port:${NC} ${LOCAL_PORT_REDIS}"
            echo -e "      ${WHITE}URL:${NC} redis://localhost:${LOCAL_PORT_REDIS}"
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

