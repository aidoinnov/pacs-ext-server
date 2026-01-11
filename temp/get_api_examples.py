#!/usr/bin/env python3
"""Patient 및 Series API 응답 예시 수집"""
import requests
import json

API_URL = "http://localhost:8080"

# 1. Login
print("=== 1. Login ===")
response = requests.post(
    f"{API_URL}/api/test/login",
    json={"username": "test_super_admin", "password": "TestAdmin123!"}
)
token = response.json()["access_token"]
print(f"Token: {token[:50]}...")
print()

headers = {"Authorization": f"Bearer {token}"}

# 2. Patient 목록 조회
print("=== 2. Patient 목록 조회 API ===")
print("GET /api/dicom/patients?project_id=2")
print()
response = requests.get(f"{API_URL}/api/dicom/patients?project_id=2", headers=headers)
patients = response.json()
print(json.dumps(patients, indent=2, ensure_ascii=False))
print()
print(f"총 {len(patients)}명의 환자")
print()

# 3. 첫 번째 Patient의 Series 목록
if len(patients) > 0:
    patient_id = patients[0]["00100020"]["Value"][0]
    print(f"=== 3. Patient '{patient_id}'의 Series 목록 조회 API ===")
    print(f"GET /api/dicom/series?project_id=2&PatientID={patient_id}")
    print()
    response = requests.get(
        f"{API_URL}/api/dicom/series",
        headers=headers,
        params={"project_id": 2, "PatientID": patient_id}
    )
    series = response.json()
    print(json.dumps(series, indent=2, ensure_ascii=False))
    print()
    print(f"총 {len(series)}개의 Series")
    print()

# 4. 두 번째 Patient의 Series 목록
if len(patients) > 1:
    patient_id = patients[1]["00100020"]["Value"][0]
    print(f"=== 4. Patient '{patient_id}'의 Series 목록 조회 API ===")
    print(f"GET /api/dicom/series?project_id=2&PatientID={patient_id}")
    print()
    response = requests.get(
        f"{API_URL}/api/dicom/series",
        headers=headers,
        params={"project_id": 2, "PatientID": patient_id}
    )
    series = response.json()
    print(json.dumps(series, indent=2, ensure_ascii=False))
    print()
    print(f"총 {len(series)}개의 Series")
    print()

# 5. API 요약
print("=" * 80)
print("📋 API 요약")
print("=" * 80)
print()
print("1️⃣  Patient 목록 조회:")
print("   GET /api/dicom/patients?project_id={project_id}")
print("   - 응답: DICOM JSON 배열 (Patient 정보)")
print("   - 주요 태그: 00100020 (PatientID), 00100010 (PatientName)")
print()
print("2️⃣  특정 Patient의 Series 목록 조회:")
print("   GET /api/dicom/series?project_id={project_id}&PatientID={patient_id}")
print("   - 응답: DICOM JSON 배열 (Series 정보)")
print("   - 주요 태그: 0020000E (SeriesInstanceUID), 00080060 (Modality)")
print("   - 추가 필드: thumbnail_url (WADO-RS 썸네일 URL)")
print()

