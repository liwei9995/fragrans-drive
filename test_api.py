import requests

url = "http://localhost:3821"

print("Registering...")
res = requests.post(f"{url}/v1/users", json={
    "email": "test@test.com", 
    "password": "password",
    "firstName": "Alex",
    "lastName": "T"
})
print("Register:", res.status_code, res.text)

print("Logging in...")
res = requests.post(f"{url}/v1/auth/login", json={"email": "test@test.com", "password": "password"})
print("Login:", res.status_code)
try:
    token = res.json().get("access_token", "")
    print("Token obtained")
except Exception as e:
    print("Failed to parse token:", res.text)
    exit(1)

headers = {"Authorization": f"Bearer {token}"}

print("Creating Folder...")
res = requests.post(f"{url}/v1/storage/folder", json={"name": "test_folder", "parentId": "root", "type": "folder"}, headers=headers)
print("Create Folder:", res.status_code, res.text)
folder_id = res.json().get("id")

print("Uploading File...")
files = {'file': ('test.txt', b'Hello, world! This is a test file to verify multipart.', 'text/plain')}
data = {'parentId': folder_id}
res = requests.post(f"{url}/v1/storage/upload", files=files, data=data, headers=headers)
print("Upload File:", res.status_code, res.text)
