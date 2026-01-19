#!/usr/bin/env python3
"""
Redis 연결 확인 및 테스트 스크립트

서버가 Redis에 연결할 수 있는지 확인합니다.
"""

import socket
import sys

def test_redis_raw():
    """Raw socket으로 Redis 연결 테스트"""
    print("="*60)
    print("🔍 Redis Raw Socket 연결 테스트")
    print("="*60)
    
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)
        print("1. 소켓 생성 완료")
        
        print("2. localhost:6379 연결 시도...")
        result = sock.connect_ex(('localhost', 6379))
        
        if result == 0:
            print("   ✅ 연결 성공")
            
            # PING 명령 전송
            print("3. PING 명령 전송...")
            ping_cmd = b'*1\r\n$4\r\nPING\r\n'
            sock.send(ping_cmd)
            print(f"   전송한 명령: {ping_cmd}")
            
            # 응답 수신
            print("4. 응답 대기...")
            response = sock.recv(1024)
            print(f"   수신한 응답: {response}")
            
            sock.close()
            
            if b'PONG' in response or b'+PONG' in response:
                print("   ✅ PING/PONG 성공 - Redis 정상 작동")
                return True
            else:
                print("   ⚠️  PONG 응답 없음")
                return False
        else:
            print(f"   ❌ 연결 실패 (코드: {result})")
            sock.close()
            return False
            
    except socket.timeout:
        print("   ❌ 연결 타임아웃")
        return False
    except ConnectionRefusedError:
        print("   ❌ 연결 거부됨")
        return False
    except Exception as e:
        print(f"   ❌ 에러: {e}")
        return False

def test_redis_with_redis_lib():
    """redis 라이브러리를 사용한 테스트 (설치되어 있는 경우)"""
    print("\n" + "="*60)
    print("🔍 Redis 라이브러리 연결 테스트")
    print("="*60)
    
    try:
        import redis
        print("✅ redis 라이브러리 설치됨")
        
        try:
            client = redis.Redis(host='localhost', port=6379, db=0, socket_timeout=5)
            result = client.ping()
            if result:
                print("✅ Redis PING 성공")
                return True
            else:
                print("❌ Redis PING 실패")
                return False
        except redis.ConnectionError as e:
            print(f"❌ Redis 연결 에러: {e}")
            return False
        except Exception as e:
            print(f"❌ 에러: {e}")
            return False
    except ImportError:
        print("⚠️  redis 라이브러리가 설치되지 않음 (선택사항)")
        print("   설치: pip install redis")
        return None

def main():
    print("\n" + "="*60)
    print("🚀 Redis 연결 진단")
    print("="*60)
    
    # Raw socket 테스트
    raw_result = test_redis_raw()
    
    # Redis 라이브러리 테스트
    lib_result = test_redis_with_redis_lib()
    
    # 결과 요약
    print("\n" + "="*60)
    print("📊 결과 요약")
    print("="*60)
    print(f"Raw Socket 테스트: {'✅ 성공' if raw_result else '❌ 실패'}")
    if lib_result is not None:
        print(f"Redis 라이브러리 테스트: {'✅ 성공' if lib_result else '❌ 실패'}")
    
    if raw_result:
        print("\n✅ Redis는 정상 작동 중입니다")
        print("   → 서버가 Redis에 연결하지 못하는 이유:")
        print("     1. 서버 시작 시 Redis 연결 실패")
        print("     2. .env 파일의 APP_REDIS__URL 설정 확인 필요")
        print("     3. 서버 재시작 필요")
        return 0
    else:
        print("\n❌ Redis 연결 실패")
        print("   → SSH 터널 확인:")
        print("     lsof -i :6379")
        print("   → 터널 재시작:")
        print("     ./scripts/start-db-tunnels.sh")
        return 1

if __name__ == "__main__":
    sys.exit(main())


