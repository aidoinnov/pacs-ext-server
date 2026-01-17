"""
성능 메트릭 수집 및 분석
"""
import time
from dataclasses import dataclass, field
from typing import List, Dict, Any
import statistics
import logging

logger = logging.getLogger(__name__)


@dataclass
class PerformanceMetrics:
    """성능 메트릭"""
    name: str
    response_times: List[float] = field(default_factory=list)
    errors: List[str] = field(default_factory=list)
    status_codes: List[int] = field(default_factory=list)
    
    def add_response(self, response_time: float, status_code: int, error: str = None):
        """응답 기록"""
        self.response_times.append(response_time)
        self.status_codes.append(status_code)
        if error:
            self.errors.append(error)
    
    def get_stats(self) -> Dict[str, Any]:
        """통계 계산"""
        if not self.response_times:
            return {
                "name": self.name,
                "total_requests": 0,
                "error_count": 0,
                "error_rate": 0.0
            }
        
        sorted_times = sorted(self.response_times)
        total = len(self.response_times)
        
        return {
            "name": self.name,
            "total_requests": total,
            "success_count": total - len(self.errors),
            "error_count": len(self.errors),
            "error_rate": len(self.errors) / total * 100,
            "min_time": min(self.response_times),
            "max_time": max(self.response_times),
            "avg_time": statistics.mean(self.response_times),
            "median_time": statistics.median(self.response_times),
            "p95_time": sorted_times[int(total * 0.95)] if total > 0 else 0,
            "p99_time": sorted_times[int(total * 0.99)] if total > 0 else 0,
            "status_codes": dict((code, self.status_codes.count(code)) for code in set(self.status_codes))
        }


class MetricsCollector:
    """메트릭 수집기"""
    
    def __init__(self):
        self.metrics: Dict[str, PerformanceMetrics] = {}
    
    def get_or_create(self, name: str) -> PerformanceMetrics:
        """메트릭 가져오기 또는 생성"""
        if name not in self.metrics:
            self.metrics[name] = PerformanceMetrics(name=name)
        return self.metrics[name]
    
    def record_request(self, name: str, response_time: float, status_code: int, error: str = None):
        """요청 기록"""
        metric = self.get_or_create(name)
        metric.add_response(response_time, status_code, error)
    
    def get_all_stats(self) -> List[Dict[str, Any]]:
        """모든 메트릭 통계"""
        return [metric.get_stats() for metric in self.metrics.values()]
    
    def print_summary(self):
        """요약 출력"""
        print("\n" + "="*80)
        print("Performance Test Summary")
        print("="*80)
        
        for stats in self.get_all_stats():
            print(f"\n{stats['name']}:")
            print(f"  Total Requests: {stats['total_requests']}")
            print(f"  Success: {stats.get('success_count', 0)}")
            print(f"  Errors: {stats['error_count']} ({stats['error_rate']:.2f}%)")
            
            if stats['total_requests'] > 0:
                print(f"  Response Times (ms):")
                print(f"    Min: {stats['min_time']*1000:.2f}")
                print(f"    Avg: {stats['avg_time']*1000:.2f}")
                print(f"    Median: {stats['median_time']*1000:.2f}")
                print(f"    P95: {stats['p95_time']*1000:.2f}")
                print(f"    P99: {stats['p99_time']*1000:.2f}")
                print(f"    Max: {stats['max_time']*1000:.2f}")

