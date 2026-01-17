"""
성능 테스트 리포트 생성
"""
import json
import os
from datetime import datetime
from typing import List, Dict, Any
import matplotlib.pyplot as plt
import pandas as pd
from tabulate import tabulate


class PerformanceReportGenerator:
    """성능 테스트 리포트 생성기"""
    
    def __init__(self, output_dir: str = "reports"):
        self.output_dir = output_dir
        os.makedirs(output_dir, exist_ok=True)
        self.timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    
    def generate_report(self, metrics_data: List[Dict[str, Any]]):
        """리포트 생성"""
        print(f"\nGenerating performance report...")
        
        # 1. 텍스트 리포트
        self._generate_text_report(metrics_data)
        
        # 2. CSV 리포트
        self._generate_csv_report(metrics_data)
        
        # 3. 그래프 생성
        self._generate_charts(metrics_data)
        
        # 4. HTML 리포트
        self._generate_html_report(metrics_data)
        
        print(f"✓ Report generated in {self.output_dir}/")
    
    def _generate_text_report(self, metrics_data: List[Dict[str, Any]]):
        """텍스트 리포트 생성"""
        report_path = os.path.join(self.output_dir, f"performance_report_{self.timestamp}.txt")
        
        with open(report_path, 'w', encoding='utf-8') as f:
            f.write("="*80 + "\n")
            f.write("PACS Server Performance Test Report\n")
            f.write(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
            f.write("="*80 + "\n\n")
            
            for metric in metrics_data:
                f.write(f"\n{metric['name']}\n")
                f.write("-"*80 + "\n")
                f.write(f"Total Requests: {metric['total_requests']}\n")
                f.write(f"Success: {metric.get('success_count', 0)}\n")
                f.write(f"Errors: {metric['error_count']} ({metric['error_rate']:.2f}%)\n")
                
                if metric['total_requests'] > 0:
                    f.write(f"\nResponse Times (ms):\n")
                    f.write(f"  Min:    {metric['min_time']*1000:8.2f}\n")
                    f.write(f"  Avg:    {metric['avg_time']*1000:8.2f}\n")
                    f.write(f"  Median: {metric['median_time']*1000:8.2f}\n")
                    f.write(f"  P95:    {metric['p95_time']*1000:8.2f}\n")
                    f.write(f"  P99:    {metric['p99_time']*1000:8.2f}\n")
                    f.write(f"  Max:    {metric['max_time']*1000:8.2f}\n")
                
                if 'status_codes' in metric:
                    f.write(f"\nStatus Codes:\n")
                    for code, count in metric['status_codes'].items():
                        f.write(f"  {code}: {count}\n")
        
        print(f"  - Text report: {report_path}")
    
    def _generate_csv_report(self, metrics_data: List[Dict[str, Any]]):
        """CSV 리포트 생성"""
        csv_path = os.path.join(self.output_dir, f"performance_metrics_{self.timestamp}.csv")
        
        df_data = []
        for metric in metrics_data:
            if metric['total_requests'] > 0:
                df_data.append({
                    'Test Name': metric['name'],
                    'Total Requests': metric['total_requests'],
                    'Success Count': metric.get('success_count', 0),
                    'Error Count': metric['error_count'],
                    'Error Rate (%)': metric['error_rate'],
                    'Min Time (ms)': metric['min_time'] * 1000,
                    'Avg Time (ms)': metric['avg_time'] * 1000,
                    'Median Time (ms)': metric['median_time'] * 1000,
                    'P95 Time (ms)': metric['p95_time'] * 1000,
                    'P99 Time (ms)': metric['p99_time'] * 1000,
                    'Max Time (ms)': metric['max_time'] * 1000,
                })
        
        df = pd.DataFrame(df_data)
        df.to_csv(csv_path, index=False)
        
        print(f"  - CSV report: {csv_path}")
    
    def _generate_charts(self, metrics_data: List[Dict[str, Any]]):
        """그래프 생성"""
        # 응답 시간 비교 차트
        fig, axes = plt.subplots(2, 2, figsize=(15, 10))
        fig.suptitle('Performance Test Results', fontsize=16)
        
        # 1. 평균 응답 시간
        names = [m['name'] for m in metrics_data if m['total_requests'] > 0]
        avg_times = [m['avg_time'] * 1000 for m in metrics_data if m['total_requests'] > 0]
        
        axes[0, 0].bar(range(len(names)), avg_times, color='skyblue')
        axes[0, 0].set_xticks(range(len(names)))
        axes[0, 0].set_xticklabels(names, rotation=45, ha='right')
        axes[0, 0].set_ylabel('Time (ms)')
        axes[0, 0].set_title('Average Response Time')
        axes[0, 0].grid(axis='y', alpha=0.3)
        
        # 2. P95/P99 응답 시간
        p95_times = [m['p95_time'] * 1000 for m in metrics_data if m['total_requests'] > 0]
        p99_times = [m['p99_time'] * 1000 for m in metrics_data if m['total_requests'] > 0]
        
        x = range(len(names))
        width = 0.35
        axes[0, 1].bar([i - width/2 for i in x], p95_times, width, label='P95', color='orange')
        axes[0, 1].bar([i + width/2 for i in x], p99_times, width, label='P99', color='red')
        axes[0, 1].set_xticks(x)
        axes[0, 1].set_xticklabels(names, rotation=45, ha='right')
        axes[0, 1].set_ylabel('Time (ms)')
        axes[0, 1].set_title('P95 and P99 Response Times')
        axes[0, 1].legend()
        axes[0, 1].grid(axis='y', alpha=0.3)
        
        # 3. 에러율
        error_rates = [m['error_rate'] for m in metrics_data if m['total_requests'] > 0]
        
        axes[1, 0].bar(range(len(names)), error_rates, color='salmon')
        axes[1, 0].set_xticks(range(len(names)))
        axes[1, 0].set_xticklabels(names, rotation=45, ha='right')
        axes[1, 0].set_ylabel('Error Rate (%)')
        axes[1, 0].set_title('Error Rate')
        axes[1, 0].grid(axis='y', alpha=0.3')
        
        # 4. 요청 수
        total_requests = [m['total_requests'] for m in metrics_data if m['total_requests'] > 0]
        
        axes[1, 1].bar(range(len(names)), total_requests, color='lightgreen')
        axes[1, 1].set_xticks(range(len(names)))
        axes[1, 1].set_xticklabels(names, rotation=45, ha='right')
        axes[1, 1].set_ylabel('Count')
        axes[1, 1].set_title('Total Requests')
        axes[1, 1].grid(axis='y', alpha=0.3)
        
        plt.tight_layout()
        
        chart_path = os.path.join(self.output_dir, f"performance_charts_{self.timestamp}.png")
        plt.savefig(chart_path, dpi=300, bbox_inches='tight')
        plt.close()
        
        print(f"  - Charts: {chart_path}")

    def _generate_html_report(self, metrics_data: List[Dict[str, Any]]):
        """HTML 리포트 생성"""
        html_path = os.path.join(self.output_dir, f"performance_report_{self.timestamp}.html")

        # 테이블 데이터 준비
        table_data = []
        for metric in metrics_data:
            if metric['total_requests'] > 0:
                table_data.append([
                    metric['name'],
                    metric['total_requests'],
                    f"{metric.get('success_count', 0)}",
                    f"{metric['error_count']} ({metric['error_rate']:.2f}%)",
                    f"{metric['avg_time']*1000:.2f}",
                    f"{metric['median_time']*1000:.2f}",
                    f"{metric['p95_time']*1000:.2f}",
                    f"{metric['p99_time']*1000:.2f}",
                ])

        html_content = f"""
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>PACS Server Performance Test Report</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            border-bottom: 3px solid #4CAF50;
            padding-bottom: 10px;
        }}
        .timestamp {{
            color: #666;
            font-size: 14px;
            margin-bottom: 20px;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }}
        th, td {{
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }}
        th {{
            background-color: #4CAF50;
            color: white;
            font-weight: bold;
        }}
        tr:hover {{
            background-color: #f5f5f5;
        }}
        .metric-good {{
            color: #4CAF50;
            font-weight: bold;
        }}
        .metric-warning {{
            color: #FF9800;
            font-weight: bold;
        }}
        .metric-bad {{
            color: #F44336;
            font-weight: bold;
        }}
        .summary {{
            background-color: #e8f5e9;
            padding: 15px;
            border-radius: 5px;
            margin: 20px 0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>PACS Server Performance Test Report</h1>
        <div class="timestamp">Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</div>

        <div class="summary">
            <h2>Summary</h2>
            <p>Total test scenarios: {len([m for m in metrics_data if m['total_requests'] > 0])}</p>
            <p>Total requests: {sum(m['total_requests'] for m in metrics_data)}</p>
            <p>Total errors: {sum(m['error_count'] for m in metrics_data)}</p>
        </div>

        <h2>Detailed Results</h2>
        <table>
            <thead>
                <tr>
                    <th>Test Name</th>
                    <th>Requests</th>
                    <th>Success</th>
                    <th>Errors</th>
                    <th>Avg (ms)</th>
                    <th>Median (ms)</th>
                    <th>P95 (ms)</th>
                    <th>P99 (ms)</th>
                </tr>
            </thead>
            <tbody>
"""

        for row in table_data:
            html_content += "                <tr>\n"
            for i, cell in enumerate(row):
                if i == 4:  # Avg time
                    avg_time = float(cell)
                    css_class = "metric-good" if avg_time < 100 else ("metric-warning" if avg_time < 500 else "metric-bad")
                    html_content += f"                    <td class='{css_class}'>{cell}</td>\n"
                else:
                    html_content += f"                    <td>{cell}</td>\n"
            html_content += "                </tr>\n"

        html_content += """
            </tbody>
        </table>
    </div>
</body>
</html>
"""

        with open(html_path, 'w', encoding='utf-8') as f:
            f.write(html_content)

        print(f"  - HTML report: {html_path}")


def load_metrics_from_file(filepath: str) -> List[Dict[str, Any]]:
    """파일에서 메트릭 로드"""
    with open(filepath, 'r') as f:
        return json.load(f)


if __name__ == "__main__":
    import sys

    if len(sys.argv) > 1:
        # 파일에서 메트릭 로드
        metrics_file = sys.argv[1]
        metrics = load_metrics_from_file(metrics_file)
    else:
        # 샘플 데이터
        metrics = [
            {
                "name": "Sample Test",
                "total_requests": 100,
                "success_count": 95,
                "error_count": 5,
                "error_rate": 5.0,
                "min_time": 0.05,
                "avg_time": 0.15,
                "median_time": 0.12,
                "p95_time": 0.25,
                "p99_time": 0.35,
                "max_time": 0.5,
                "status_codes": {"200": 95, "500": 5}
            }
        ]

    generator = PerformanceReportGenerator()
    generator.generate_report(metrics)

