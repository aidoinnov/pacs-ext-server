#!/bin/bash

TOKEN="eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJ5X1EwUDd0MDhjcEZZeFNEUFdseGdKcGFUcWtsOFd0eUJYRGRGaVVUQXBJIn0.eyJleHAiOjE3NjQ4NDEwNTgsImlhdCI6MTc2NDgzOTI1OCwianRpIjoiNzdkNWRlNzktYjZkNS00ODFjLTgyNWEtOTM2NGNmMzg3NzAxIiwiaXNzIjoiaHR0cHM6Ly9rZXljbG9hay5wYWNzLmFpLWRvLmtyL3JlYWxtcy9kY200Y2hlIiwiYXVkIjpbImlhaWQtcGFjcy1jbGllbnQiLCJhY2NvdW50Il0sInN1YiI6ImY0ZTJlMzU1LTIxMDItNGZiNi04YzZmLTg4YzI3NDQzZjVkOCIsInR5cCI6IkJlYXJlciIsImF6cCI6ImlhaWQtcGFjcy1jbGllbnQiLCJzZXNzaW9uX3N0YXRlIjoiYWQ3NjE3ZDAtN2Q3NS00M2RmLTk3NjYtZWM5ZjgyYmU0YjQ3IiwiYWNyIjoiMSIsImFsbG93ZWQtb3JpZ2lucyI6WyIqIl0sInJlYWxtX2FjY2VzcyI6eyJyb2xlcyI6WyJvZmZsaW5lX2FjY2VzcyIsImRlZmF1bHQtcm9sZXMtZGNtNGNoZSIsInVtYV9hdXRob3JpemF0aW9uIiwidXNlciJdfSwicmVzb3VyY2VfYWNjZXNzIjp7ImFjY291bnQiOnsicm9sZXMiOlsibWFuYWdlLWFjY291bnQiLCJtYW5hZ2UtYWNjb3VudC1saW5rcyIsInZpZXctcHJvZmlsZSJdfX0sInNjb3BlIjoiUEFDUy1hdWRpZW5jZS1zZXJ2aWNlIHByb2ZpbGUgZW1haWwiLCJzaWQiOiJhZDc2MTdkMC03ZDc1LTQzZGYtOTc2Ni1lYzlmODJiZTRiNDciLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwicHJlZmVycmVkX3VzZXJuYW1lIjoiaWFpZC1wYWNzLWFkbWluIn0.lwTScxhBmoClsUjrPK6w6fPh1w_Co8kBDR3lh1HQNleM-kDh5xkHT2qqKpYijzZJR-CEAL8OCgNtMY90ce1Dfxv0Iu4xG84e6csbOB_-MOiW_ABBKTF-Z2JHmQxlIokyFs-scGB82N2aeWNy3RUpu6AyCqg9W3b4H6sauW8hYvR56FMpGyvOFEnI_uYoz7MtRhjjyE1qMQtpCZ_w8iq5plXQXszq015Kz9v7YCv0MqjkeVIqxDrbGC2AyTE3N3A88PZSZlusI3JO-BcbXCU4raSFH1NTtui3NkvPliZDUv1Ze9rW2kzlXUpU7DCeaCFnZPdrVIyZM6GkxMw4eA1Fyw"

echo "=== Test 1: user_id 파라미터로 Series 조회 (Bearer 토큰 포함) ==="
curl -s "http://localhost:8080/api/dicom/series?project_id=2&user_id=56" \
  -H "Authorization: Bearer $TOKEN" \
  | jq 'if type == "array" then "✅ Success! Got \(length) series" else . end'

echo ""
echo "=== Test 2: PatientID 필터 추가 ==="
curl -s "http://localhost:8080/api/dicom/series?project_id=2&user_id=56&PatientID=SarcopeniaCase1" \
  -H "Authorization: Bearer $TOKEN" \
  | jq 'if type == "array" then "✅ Success! Got \(length) series for SarcopeniaCase1" else . end'

echo ""
echo "=== Test 3: 첫 번째 Series의 주요 필드 확인 ==="
curl -s "http://localhost:8080/api/dicom/series?project_id=2&user_id=56&PatientID=SarcopeniaCase1" \
  -H "Authorization: Bearer $TOKEN" \
  | jq '.[0] | {
    PatientID: .["00100020"].Value[0],
    StudyInstanceUID: .["0020000D"].Value[0],
    SeriesInstanceUID: .["0020000E"].Value[0],
    Modality: .["00080060"].Value[0],
    thumbnail_url: .thumbnail_url
  }'

