import requests

resp = requests.post(
    "http://localhost:8080/api/auth/login",
    json={"username": "iaid-pacs-admin", "password": "Qlalfqjsgh1!"},
)
token = resp.json()["token"]
print(f"Token: {token}")
resp2 = requests.get(
    "http://localhost:8080/api/me/dicom/studies?project_id=2&user_id=56",
    headers={"Authorization": f"Bearer {token}"},
)
print(f"Status: {resp2.status_code}")
data = resp2.json()
print(f"Series count: {len(data) if isinstance(data, list) else 0}")
