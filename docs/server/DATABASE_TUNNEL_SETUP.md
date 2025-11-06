# 데이터베이스 터널 설정 가이드

## 개요

PACS Extension Server에서 RDS(Amazon Relational Database Service)에 안전하게 연결하기 위한 SSH 터널 설정 및 관리에 대한 상세한 기술 문서입니다.

---

## 1. 아키텍처 개요

### 1.1 네트워크 구성도
```
[개발자 로컬] 
    ↓ SSH 터널 (포트 5432)
[Bastion Host (EC2)]
    ↓ 내부 네트워크
[RDS Instance]
```

### 1.2 보안 고려사항
- **Bastion Host**: RDS에 대한 유일한 접근점
- **SSH 키 인증**: 패스워드 대신 SSH 키 사용
- **포트 포워딩**: 로컬 포트를 RDS 포트로 전달
- **네트워크 격리**: RDS는 Private 서브넷에 위치

---

## 2. 사전 준비사항

### 2.1 필요한 정보
- **Bastion Host IP**: `13.125.228.206`
- **RDS Endpoint**: `pacs-extension.ciyua2gsk8ke.ap-northeast-2.rds.amazonaws.com`
- **SSH 키**: `~/.ssh/bastion-keypair.pem`
- **RDS 포트**: `5432`
- **로컬 포트**: `5432` (기본값)

### 2.2 시스템 요구사항
- SSH 클라이언트 설치
- PostgreSQL 클라이언트 (psql)
- 적절한 네트워크 접근 권한

---

## 3. SSH 키 설정

### 3.1 키 파일 권한 설정
```bash
# SSH 키 파일 권한 설정 (중요!)
chmod 600 ~/.ssh/bastion-keypair.pem

# 권한 확인
ls -la ~/.ssh/bastion-keypair.pem
# -rw------- 1 user user 1674 Dec 15 10:30 bastion-keypair.pem
```

### 3.2 SSH 키 검증
```bash
# SSH 키 형식 확인
file ~/.ssh/bastion-keypair.pem
# bastion-keypair.pem: PEM RSA private key

# SSH 연결 테스트
ssh -i ~/.ssh/bastion-keypair.pem ec2-user@13.125.228.206 -o ConnectTimeout=10
```

---

## 4. 기본 터널 설정

### 4.1 수동 SSH 터널 생성
```bash
# 기본 터널 생성
ssh -i ~/.ssh/bastion-keypair.pem \
    -L 5432:pacs-extension.ciyua2gsk8ke.ap-northeast-2.rds.amazonaws.com:5432 \
    ec2-user@13.125.228.206 \
    -N

# 백그라운드 실행
ssh -i ~/.ssh/bastion-keypair.pem \
    -L 5432:pacs-extension.ciyua2gsk8ke.ap-northeast-2.rds.amazonaws.com:5432 \
    ec2-user@13.125.228.206 \
    -N &
```

### 4.2 터널 상태 확인
```bash
# 실행 중인 SSH 터널 확인
ps aux | grep "ssh.*-L.*5432:"

# 포트 사용 확인
netstat -tlnp | grep 5432
# tcp 0 0 127.0.0.1:5432 0.0.0.0:* LISTEN 12345/ssh
```

---

## 5. 고급 터널 관리 스크립트

### 5.1 db-tunnel.sh 스크립트 기능

#### 5.1.1 기본 사용법
```bash
# 기본 터널 시작
./db-tunnel.sh

# 특정 포트 사용
./db-tunnel.sh -p 5433

# 상세 로그와 함께
./db-tunnel.sh -v

# 조용한 모드
./db-tunnel.sh -q
```

#### 5.1.2 상태 관리
```bash
# 터널 상태 확인
./db-tunnel.sh -s

# 모든 터널 종료
./db-tunnel.sh -k

# 도움말 보기
./db-tunnel.sh -h
```

### 5.2 스크립트 주요 기능

#### 5.2.1 색상 출력
```bash
# 성공 메시지
echo -e "${GREEN}✅ Tunnel started successfully!${NC}"

# 에러 메시지
echo -e "${RED}❌ Failed to start tunnel${NC}"

# 정보 메시지
echo -e "${BLUE}📡 Bastion Host:${NC} ${GREEN}${BASTION_HOST}${NC}"
```

#### 5.2.2 로깅 레벨 제어
```bash
# 사용 가능한 로깅 레벨
QUIET, FATAL, ERROR, INFO, VERBOSE, DEBUG1, DEBUG2, DEBUG3

# 예시
./db-tunnel.sh -l INFO -v
```

#### 5.2.3 프로세스 관리
```bash
# 터널 상태 확인 함수
check_tunnel_status() {
    local port=$1
    local tunnels=$(ps aux | grep "ssh.*-L.*${port}:" | grep -v grep)
    # ... 상태 확인 로직
}

# 터널 종료 함수
stop_tunnel() {
    local port=$1
    local pids=$(ps aux | grep "ssh.*-L.*${port}:" | grep -v grep | awk '{print $2}')
    # ... 종료 로직
}
```

---

## 6. 데이터베이스 연결 설정

### 6.1 환경 변수 설정

#### 6.1.1 DATABASE_URL 사용 (권장)
```bash
# 터널을 통한 연결
export DATABASE_URL="postgres://username:password@localhost:5432/pacs_db"

# 애플리케이션 실행
cargo run
```

#### 6.1.2 개별 환경 변수 사용
```bash
# 개별 변수 설정
export DATABASE_USERNAME="pacs_extension_admin"
export DATABASE_PASSWORD="CHANGE_ME_STRONG_PASSWORD"
export DATABASE_HOST="localhost"
export DATABASE_PORT="5432"
export DATABASE_NAME="pacs_db"

# 애플리케이션 실행
cargo run
```

### 6.2 DBeaver 연결 설정

#### 6.2.1 연결 정보
```
Host: localhost
Port: 5432
Database: pacs_db
Username: pacs_extension_admin
Password: CHANGE_ME_STRONG_PASSWORD
```

#### 6.2.2 고급 설정
```
SSL Mode: Disable (터널을 통한 암호화)
Connection Timeout: 30
Socket Timeout: 30
```

---

## 7. 문제 해결

### 7.1 일반적인 문제들

#### 7.1.1 SSH 키 권한 문제
```bash
# 에러: bad permissions
# 해결: chmod 600 ~/.ssh/bastion-keypair.pem
chmod 600 ~/.ssh/bastion-keypair.pem
```

#### 7.1.2 호스트 키 검증 실패
```bash
# 에러: Host key verification failed
# 해결: SSH 옵션 추가
ssh -o StrictHostKeyChecking=no \
    -o UserKnownHostsFile=/dev/null \
    -o LogLevel=ERROR \
    -i ~/.ssh/bastion-keypair.pem \
    ec2-user@13.125.228.206
```

#### 7.1.3 포트 이미 사용 중
```bash
# 에러: Port 5432 already in use
# 해결: 다른 포트 사용 또는 기존 프로세스 종료
./db-tunnel.sh -p 5433
# 또는
./db-tunnel.sh -k  # 모든 터널 종료
```

### 7.2 연결 테스트

#### 7.2.1 터널 연결 테스트
```bash
# 터널 상태 확인
./db-tunnel.sh -s

# 포트 연결 테스트
telnet localhost 5432
```

#### 7.2.2 데이터베이스 연결 테스트
```bash
# psql을 통한 연결 테스트
psql "postgres://username:password@localhost:5432/pacs_db" -c "SELECT version();"

# 연결 정보 확인
psql "postgres://username:password@localhost:5432/pacs_db" -c "\conninfo"
```

---

## 8. 보안 모범 사례

### 8.1 SSH 키 관리

#### 8.1.1 키 파일 보안
```bash
# 적절한 권한 설정
chmod 600 ~/.ssh/bastion-keypair.pem
chmod 700 ~/.ssh/

# 키 파일 백업 (암호화된 저장소에)
gpg --symmetric --cipher-algo AES256 ~/.ssh/bastion-keypair.pem
```

#### 8.1.2 키 로테이션
```bash
# 정기적인 키 로테이션 (월 1회)
# 1. 새 키 생성
# 2. Bastion Host에 새 키 등록
# 3. 기존 키 삭제
# 4. 로컬 키 파일 업데이트
```

### 8.2 네트워크 보안

#### 8.2.1 방화벽 설정
```bash
# 로컬 방화벽에서 불필요한 포트 차단
sudo ufw deny 5432  # 직접 RDS 접근 차단
sudo ufw allow 22   # SSH만 허용
```

#### 8.2.2 VPN 사용
```bash
# 가능한 경우 VPN을 통한 접근
# 1. VPN 연결
# 2. 터널 생성
# 3. 데이터베이스 연결
```

---

## 9. 자동화 및 모니터링

### 9.1 자동 터널 시작

#### 9.1.1 systemd 서비스 생성
```ini
# /etc/systemd/system/db-tunnel.service
[Unit]
Description=PACS Database Tunnel
After=network.target

[Service]
Type=simple
User=developer
ExecStart=/home/developer/workspace/pacs-extension-server/pacs-ext-server/pacs-server/db-tunnel.sh -q
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

#### 9.1.2 서비스 관리
```bash
# 서비스 활성화
sudo systemctl enable db-tunnel.service
sudo systemctl start db-tunnel.service

# 상태 확인
sudo systemctl status db-tunnel.service

# 로그 확인
sudo journalctl -u db-tunnel.service -f
```

### 9.2 모니터링 스크립트

#### 9.2.1 터널 상태 모니터링
```bash
#!/bin/bash
# monitor-tunnel.sh

while true; do
    if ! ./db-tunnel.sh -s > /dev/null 2>&1; then
        echo "$(date): Tunnel is down, restarting..."
        ./db-tunnel.sh -q
    fi
    sleep 30
done
```

#### 9.2.2 알림 설정
```bash
# 터널 실패 시 알림
if ! ./db-tunnel.sh -s > /dev/null 2>&1; then
    # Slack 알림
    curl -X POST -H 'Content-type: application/json' \
        --data '{"text":"Database tunnel is down!"}' \
        $SLACK_WEBHOOK_URL
    
    # 이메일 알림
    echo "Database tunnel is down at $(date)" | mail -s "Tunnel Alert" admin@company.com
fi
```

---

## 10. 성능 최적화

### 10.1 SSH 터널 최적화

#### 10.1.1 SSH 설정 최적화
```bash
# ~/.ssh/config
Host bastion
    HostName 13.125.228.206
    User ec2-user
    IdentityFile ~/.ssh/bastion-keypair.pem
    ServerAliveInterval 60
    ServerAliveCountMax 3
    TCPKeepAlive yes
    Compression yes
    ControlMaster auto
    ControlPath ~/.ssh/control-%r@%h:%p
    ControlPersist 10m
```

#### 10.1.2 연결 풀링
```bash
# SSH 연결 재사용
ssh -M -f -N bastion
ssh -O check bastion  # 연결 상태 확인
ssh -O exit bastion   # 연결 종료
```

### 10.2 데이터베이스 연결 최적화

#### 10.2.1 연결 풀 설정
```rust
// src/infrastructure/config/settings.rs
pub struct DatabaseConfig {
    pub pool_size: u32,
    pub timeout: u64,
    pub max_lifetime: u64,
}

// 권장 설정
DatabaseConfig {
    pool_size: 10,        // 개발 환경
    timeout: 30,          // 30초
    max_lifetime: 3600,   // 1시간
}
```

#### 10.2.2 쿼리 최적화
```sql
-- 인덱스 활용
CREATE INDEX CONCURRENTLY idx_annotation_mask_group_created_at 
ON annotation_mask_group(created_at);

-- 쿼리 최적화
EXPLAIN ANALYZE SELECT * FROM annotation_mask_group 
WHERE created_at > NOW() - INTERVAL '1 day';
```

---

## 11. 백업 및 복구

### 11.1 터널 설정 백업

#### 11.1.1 설정 파일 백업
```bash
# SSH 설정 백업
cp ~/.ssh/config ~/.ssh/config.backup
cp ~/.ssh/bastion-keypair.pem ~/.ssh/bastion-keypair.pem.backup

# 스크립트 백업
cp db-tunnel.sh db-tunnel.sh.backup
```

#### 11.1.2 자동 백업 스크립트
```bash
#!/bin/bash
# backup-tunnel-config.sh

BACKUP_DIR="/home/developer/backups/tunnel-config"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

# 설정 파일 백업
cp ~/.ssh/config "$BACKUP_DIR/config_$DATE"
cp ~/.ssh/bastion-keypair.pem "$BACKUP_DIR/bastion-keypair_$DATE.pem"
cp db-tunnel.sh "$BACKUP_DIR/db-tunnel_$DATE.sh"

# 압축
tar -czf "$BACKUP_DIR/tunnel-config_$DATE.tar.gz" "$BACKUP_DIR"/*_$DATE*

# 오래된 백업 삭제 (30일 이상)
find "$BACKUP_DIR" -name "tunnel-config_*.tar.gz" -mtime +30 -delete
```

### 11.2 재해 복구

#### 11.2.1 빠른 복구 절차
```bash
# 1. 터널 상태 확인
./db-tunnel.sh -s

# 2. 기존 터널 종료
./db-tunnel.sh -k

# 3. 새 터널 시작
./db-tunnel.sh

# 4. 연결 테스트
psql "$DATABASE_URL" -c "SELECT 1;"
```

#### 11.2.2 완전 복구 절차
```bash
# 1. 백업에서 설정 복원
tar -xzf tunnel-config_20241215_143000.tar.gz

# 2. SSH 키 복원
cp bastion-keypair_20241215_143000.pem ~/.ssh/bastion-keypair.pem
chmod 600 ~/.ssh/bastion-keypair.pem

# 3. SSH 설정 복원
cp config_20241215_143000 ~/.ssh/config

# 4. 스크립트 복원
cp db-tunnel_20241215_143000.sh db-tunnel.sh
chmod +x db-tunnel.sh

# 5. 터널 시작
./db-tunnel.sh
```

---

## 12. 결론

### 12.1 주요 성과
1. **보안 강화**: SSH 터널을 통한 안전한 데이터베이스 접근
2. **편의성 향상**: 자동화된 터널 관리 스크립트
3. **모니터링**: 실시간 터널 상태 확인 및 관리
4. **자동화**: systemd 서비스를 통한 자동 시작/재시작

### 12.2 향후 개선사항
1. **고가용성**: 다중 Bastion Host 지원
2. **로드 밸런싱**: 여러 RDS 인스턴스 간 부하 분산
3. **암호화**: 추가적인 암호화 레이어 적용
4. **모니터링**: 더 상세한 메트릭 수집 및 알림

### 12.3 권장사항
1. **정기 점검**: 월 1회 터널 설정 및 보안 검토
2. **백업**: 설정 파일 정기 백업
3. **문서화**: 변경사항 즉시 문서화
4. **팀 교육**: 팀원 대상 터널 사용법 교육

이 문서를 통해 안전하고 효율적인 데이터베이스 터널 환경을 구축하고 유지할 수 있습니다.
