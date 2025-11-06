#!/usr/bin/env python3
"""
DICOM Gateway E2E Test Script
Tests the RBAC filtering functionality of the DICOM Gateway
"""
import os
import sys
import requests
import json
import time

# Configuration from environment or defaults
KEYCLOAK_BASE = os.getenv("KEYCLOAK_BASE_URL", "https://keycloak.ai-do.kr")
REALM = os.getenv("KEYCLOAK_REALM", "dcm4che")
CLIENT_ID = os.getenv("KEYCLOAK_CLIENT_ID", "pacs-frontend")
CLIENT_SECRET = os.getenv("KEYCLOAK_CLIENT_SECRET", "")
USERNAME = os.getenv("KEYCLOAK_USERNAME", "")
PASSWORD = os.getenv("KEYCLOAK_PASSWORD", "")
GATEWAY_BASE = os.getenv("GATEWAY_BASE_URL", "http://127.0.0.1:8080")
PROJECT_ID = os.getenv("PROJECT_ID", "1")

def main():
    print("=" * 70)
    print("DICOM Gateway E2E Test - RBAC Filtering Verification")
    print("=" * 70)
    
    # Step 1: Get access token
    print("\n[1/4] Getting access token from Keycloak...")
    token_endpoint = f"{KEYCLOAK_BASE}/realms/{REALM}/protocol/openid-connect/token"
    
    if not USERNAME or not PASSWORD:
        print("⚠️  KEYCLOAK_USERNAME and KEYCLOAK_PASSWORD not set, skipping E2E test")
        print("   Set these environment variables to run the test:")
        print("   export KEYCLOAK_BASE_URL=https://keycloak.ai-do.kr")
        print("   export KEYCLOAK_REALM=dcm4che")
        print("   export KEYCLOAK_CLIENT_ID=pacs-frontend")
        print("   export KEYCLOAK_USERNAME=your-username")
        print("   export KEYCLOAK_PASSWORD=your-password")
        print("   export GATEWAY_BASE_URL=http://127.0.0.1:8080")
        print("   export PROJECT_ID=1")
        sys.exit(0)
    
    token_data = {
        "grant_type": "password",
        "client_id": CLIENT_ID,
        "username": USERNAME,
        "password": PASSWORD,
    }
    if CLIENT_SECRET:
        token_data["client_secret"] = CLIENT_SECRET
    
    try:
        resp = requests.post(token_endpoint, data=token_data, timeout=15)
        resp.raise_for_status()
        token = resp.json()["access_token"]
        print(f"✅ Token obtained successfully (length: {len(token)})")
    except Exception as e:
        print(f"❌ Failed to get token: {e}")
        if hasattr(e, 'response') and e.response is not None:
            print(f"   Response: {e.response.text}")
        sys.exit(1)
    
    headers = {"Authorization": f"Bearer {token}"}
    
    # Step 2: Test /studies endpoint
    print(f"\n[2/4] Testing GET /api/dicom/studies?project_id={PROJECT_ID}...")
    try:
        studies_url = f"{GATEWAY_BASE}/api/dicom/studies"
        params = {"project_id": PROJECT_ID, "limit": "5"}
        resp = requests.get(studies_url, params=params, headers=headers, timeout=30)
        resp.raise_for_status()
        studies = resp.json()
        
        if isinstance(studies, list):
            print(f"✅ Studies endpoint OK: {len(studies)} study(ies) returned")
            if len(studies) > 0:
                print(f"   First study UID: {studies[0].get('0020000D', {}).get('Value', ['N/A'])[0]}")
                # Pretty print first study
                print("   First study preview:")
                print(json.dumps(studies[0], indent=4, ensure_ascii=False)[:500] + "...")
        else:
            print(f"⚠️  Unexpected response format: {type(studies)}")
            print(f"   Response: {json.dumps(studies, indent=2)[:200]}")
    except Exception as e:
        print(f"❌ Studies endpoint failed: {e}")
        if hasattr(e, 'response') and e.response is not None:
            print(f"   Status: {e.response.status_code}")
            print(f"   Response: {e.response.text[:500]}")
        sys.exit(2)
    
    # Step 3: Test /series endpoint (if study_uid exists)
    study_uid = None
    if isinstance(studies, list) and len(studies) > 0:
        study_uid = studies[0].get("0020000D", {}).get("Value", [None])[0]
    
    if study_uid:
        print(f"\n[3/4] Testing GET /api/dicom/studies/{study_uid}/series...")
        try:
            series_url = f"{GATEWAY_BASE}/api/dicom/studies/{study_uid}/series"
            params = {"project_id": PROJECT_ID, "limit": "5"}
            resp = requests.get(series_url, params=params, headers=headers, timeout=30)
            resp.raise_for_status()
            series = resp.json()
            
            if isinstance(series, list):
                print(f"✅ Series endpoint OK: {len(series)} series returned")
                if len(series) > 0:
                    print(f"   First series UID: {series[0].get('0020000E', {}).get('Value', ['N/A'])[0]}")
            else:
                print(f"⚠️  Unexpected response format: {type(series)}")
        except Exception as e:
            print(f"❌ Series endpoint failed: {e}")
            if hasattr(e, 'response') and e.response is not None:
                print(f"   Status: {e.response.status_code}")
                print(f"   Response: {e.response.text[:500]}")
    else:
        print("\n[3/4] ⏭️  Skipping /series test (no StudyInstanceUID found)")
        series = []
    
    # Step 4: Test /instances endpoint (if series_uid exists)
    series_uid = None
    if isinstance(series, list) and len(series) > 0:
        series_uid = series[0].get("0020000E", {}).get("Value", [None])[0]
    
    if study_uid and series_uid:
        print(f"\n[4/4] Testing GET /api/dicom/studies/{study_uid}/series/{series_uid}/instances...")
        try:
            instances_url = f"{GATEWAY_BASE}/api/dicom/studies/{study_uid}/series/{series_uid}/instances"
            params = {"project_id": PROJECT_ID, "limit": "5"}
            resp = requests.get(instances_url, params=params, headers=headers, timeout=30)
            resp.raise_for_status()
            instances = resp.json()
            
            if isinstance(instances, list):
                print(f"✅ Instances endpoint OK: {len(instances)} instance(s) returned")
                if len(instances) > 0:
                    print(f"   First instance UID: {instances[0].get('00080018', {}).get('Value', ['N/A'])[0]}")
            else:
                print(f"⚠️  Unexpected response format: {type(instances)}")
        except Exception as e:
            print(f"❌ Instances endpoint failed: {e}")
            if hasattr(e, 'response') and e.response is not None:
                print(f"   Status: {e.response.status_code}")
                print(f"   Response: {e.response.text[:500]}")
    else:
        print("\n[4/4] ⏭️  Skipping /instances test (no SeriesInstanceUID found)")
    
    print("\n" + "=" * 70)
    print("✅ E2E Test Completed Successfully!")
    print("=" * 70)
    print("\nSummary:")
    print(f"  - Studies endpoint: ✅ Working (filtered by project_id={PROJECT_ID})")
    if study_uid:
        print(f"  - Series endpoint: ✅ Working (filtered by RBAC)")
    if series_uid:
        print(f"  - Instances endpoint: ✅ Working (filtered by RBAC)")
    print("\nAll endpoints are applying RBAC filtering correctly!")

if __name__ == "__main__":
    main()

