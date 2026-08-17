# Fragrans Storage Security Refactor Implementation Plan

> **For Gemini 3.1 Pro:** Implement this plan task-by-task in order. Use test-driven changes, run every required verification command, and do not claim completion while any required check is failing.

**Goal:** Remove the current data-corruption and authorization risks, make storage encrypted and streamable, make Docker deployment actually usable, and leave one reproducible verification path for the whole project.

**Architecture:** Keep the existing Axum → service → MongoDB/local-storage shape. Use per-user content-addressed files, SHA-256, a versioned chunked AES-256-GCM file format, typed API DTOs, and file-scoped download tokens. Do not introduce repository traits, a dependency-injection framework, microservices, or a second storage backend.

**Tech Stack:** Rust 2024, Axum 0.8, Tokio, MongoDB 3.5, Serde, `sha2`, `aes-gcm`, JWT, Docker Compose.

---

## 1. Execution contract

This document is both the implementation specification and the acceptance checklist.

Gemini must:

1. Work from a clean branch or worktree.
2. Preserve unrelated user changes.
3. Complete tasks in the listed order.
4. Write the named failing test before each non-trivial implementation.
5. Run the narrow test first, then the full verification suite.
6. Use the existing modules and types unless this document explicitly requires a new one.
7. Keep public API compatibility except where this document explicitly removes an unsafe endpoint or field.
8. Never delete an existing database or bucket automatically.
9. Never print JWT secrets, storage master keys, passwords, or signed download URLs in test output or logs.
10. Stop and report the exact blocker if a required migration cannot verify legacy plaintext.

Gemini must not:

- Add generic repository/service traits with one implementation.
- Add a DI container, event bus, background job framework, or microservice.
- Replace MongoDB, Axum, Tokio, JWT, or the local filesystem.
- Add cursor pagination before an explain plan or benchmark proves offset pagination is a problem.
- Silence errors with `let _ =`, `.ok()`, `unwrap_or_default()`, or `if let Ok(...)` on security, storage, migration, database, and token paths.
- weaken validation or skip a required test to make CI green.

## 2. Current baseline

Before editing, run:

```bash
git status --short
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
du -sh target
```

Baseline observed on 2026-07-27:

- Production Rust: approximately 2,847 lines.
- Rust tests: approximately 1,323 lines.
- `cargo test`: 9 integration tests pass when MongoDB is available.
- `cargo fmt --all -- --check`: fails.
- strict Clippy: fails with 7 errors.
- `target/`: approximately 3.7 GB, mostly debug dependencies.
- No CI workflow is present.
- No `.codegraph/` index is present.

Record any different result in the implementation summary. A different baseline is not permission to skip later checks.

## 3. Fixed design decisions

These decisions are requirements, not open design questions.

### 3.1 Logical files and physical objects

- Logical file/folder metadata remains in the `storage` MongoDB collection.
- Physical file deduplication is **per user**, never global.
- New physical paths use:

```text
{storage_root}/{user_id}/{hash[0..2]}/{hash[2..4]}/{hash[4..6]}/{sha256_hash}
```

- `user_id` must be a valid MongoDB ObjectId hex string.
- `sha256_hash` must be exactly 64 lowercase hexadecimal characters.
- Filenames never participate in physical paths.
- Multiple logical documents may reference the same per-user physical object.
- Concurrent writes of identical content are allowed; atomic rename guarantees that readers see one complete valid object.

### 3.2 New storage metadata

Add these backward-compatible fields to `domain::storage::Storage`:

```rust
#[serde(rename = "contentHash", default, skip_serializing_if = "Option::is_none")]
pub content_hash: Option<String>,

#[serde(rename = "hashAlgorithm", default, skip_serializing_if = "Option::is_none")]
pub hash_algorithm: Option<String>,

#[serde(rename = "encryptionFormat", default, skip_serializing_if = "Option::is_none")]
pub encryption_format: Option<u8>,
```

Keep legacy `MD5Hash` and `iv` fields readable until migration is complete. New writes must set:

```text
contentHash     = lowercase SHA-256
hashAlgorithm   = "sha256"
encryptionFormat = 1
MD5Hash         = null/absent
iv              = null/absent
```

Do not silently reinterpret the existing `MD5Hash` field as SHA-256.

### 3.3 Version 1 encrypted file format

Use AES-256-GCM with a 32-byte master key loaded from `STORAGE_MASTER_KEY_HEX`.

The encrypted file starts with this exact big-endian header:

| Field | Bytes | Value |
|---|---:|---|
| magic | 8 | ASCII `FRAGRNS\0` |
| version | 1 | `1` |
| chunk size | 4 | default `1_048_576` |
| plaintext size | 8 | original byte length |
| base nonce | 12 | cryptographically random |

Header size is 33 bytes.

For plaintext chunk index `i`:

1. Copy the 12-byte base nonce.
2. XOR its last four bytes with `u32::to_be_bytes(i)`.
3. Use the result as the AES-GCM nonce.
4. Authenticate this AAD:

```text
header_bytes || user_id || 0x00 || content_hash || u32_be(chunk_index) || u32_be(plaintext_chunk_length)
```

5. Append ciphertext followed by the 16-byte GCM tag.

Rules:

- Chunk indexes start at zero.
- Reject objects requiring more than `u32::MAX` chunks.
- A zero-length file still contains one authenticated empty chunk.
- Decryption fails on a bad magic, unsupported version, invalid chunk size, truncated header, truncated ciphertext, extra ciphertext, wrong key, wrong user, wrong hash, reordered chunks, or bad tag.
- Writes go to a random temporary file in the final directory, call `sync_all`, then atomically rename to the final path.
- A failed write must remove its temporary file.
- Do not overwrite the final path with a partially written file.

### 3.4 Token purposes

JWT claims become:

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Claims {
    pub user_id: String,
    pub exp: usize,
    pub purpose: TokenPurpose,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenPurpose {
    Access,
    Download,
}
```

- Login issues `purpose=access`, `file_id=None`.
- `auth_guard` accepts only access tokens.
- Download URLs use `purpose=download`, contain exactly one `file_id`, and expire after 15 minutes.
- The download handler accepts only a download token whose `file_id` equals the route ID, or a valid access token in the Authorization header.
- A download token must never authorize another API route.

### 3.5 Existing-data policy

Default to preserving data.

- Never run `cargo clean`, delete MongoDB collections, delete Docker volumes, or remove legacy bucket files as part of migration.
- If the operator explicitly confirms there is no valuable data, they may reset the development database/bucket manually.
- Otherwise implement and run the migration in Task 6.
- A legacy object whose decrypted bytes do not match its stored MD5 is reported as corrupted and left unchanged.

## 4. Task 1: Establish a truthful test and CI baseline

**Files:**

- Modify: `tests/common/mod.rs`
- Create: `.github/workflows/ci.yml`
- Create: `scripts/verify.sh`
- Modify: `Cargo.toml`

### Step 1: Make integration setup fail instead of silently skipping

Change:

```rust
pub async fn setup() -> Option<TestContext>
```

to:

```rust
pub async fn setup() -> TestContext
```

Replace `.ok()?` and `return None` with `expect` messages that identify the failed setup operation. Mongo unavailability must fail the test.

Update all integration tests from:

```rust
let Some(ctx) = setup().await else {
    return;
};
```

to:

```rust
let ctx = setup().await;
```

### Step 2: Run the tests without MongoDB

Run:

```bash
TEST_MONGO_URI=mongodb://127.0.0.1:1 cargo test --test users_auth
```

Expected: non-zero exit and an explicit Mongo connection/ping failure. A passing or skipped test is incorrect.

### Step 3: Add the CI workflow

The workflow must:

- run on pushes and pull requests;
- use a MongoDB 8 service with authentication;
- set test-only `TEST_MONGO_URI`, `JWT_SECRET_KEY`, and `STORAGE_MASTER_KEY_HEX`;
- run format, strict Clippy, tests, and Docker build;
- cache Cargo registry/git/target data using the standard GitHub Actions cache mechanism;
- never use production secrets.

Required commands:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
docker build -t fragrans:verify .
```

### Step 4: Add one local verification entry point

`scripts/verify.sh` must use `set -euo pipefail` and run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
docker compose config -q
docker build -t fragrans:verify .
```

It may assume MongoDB is already available through `TEST_MONGO_URI`; it must not start, stop, or delete containers or volumes.

Make it executable.

### Step 5: Verify and commit

Run:

```bash
TEST_MONGO_URI='mongodb://test:nest@127.0.0.1:25018/?authSource=admin' ./scripts/verify.sh
```

Expected at this stage: tests may pass, but format/Clippy may still fail until Task 11. Record the failures; do not weaken the script.

Suggested commit:

```bash
git add tests .github/workflows/ci.yml scripts/verify.sh Cargo.toml
git commit -m "test: make verification failures visible"
```

## 5. Task 2: Close user authorization holes

**Files:**

- Create: `tests/users_authorization.rs`
- Modify: `src/api/mod.rs`
- Modify: `src/api/users.rs`
- Modify: `src/infrastructure/db/user_repo.rs`

### Step 1: Write failing authorization tests

Create two users and tokens. Add these exact behaviors:

```text
ordinary_user_cannot_list_all_users
ordinary_user_cannot_read_another_user
ordinary_user_cannot_update_another_user
ordinary_user_cannot_delete_another_user
current_user_can_read_own_profile
current_user_can_update_own_profile
```

Expected status for removed/forbidden management routes: `404 NOT_FOUND`.

### Step 2: Verify that the tests expose the current bug

Run:

```bash
cargo test --test users_authorization
```

Expected before implementation: at least the cross-user read/update/delete assertions fail.

### Step 3: Remove unused administrator endpoints

This is a personal storage service and has no administrator authorization model. Remove these routes:

```text
GET    /v1/users
GET    /v1/users/{id}
POST   /v1/users/profile/{id}
DELETE /v1/users/{id}
```

Delete their handlers, OpenAPI registrations, and repository methods that become unused.

Keep:

```text
POST  /v1/users          public registration
POST  /v1/auth/login     public login
GET   /v1/profile        authenticated current profile
PATCH /v1/profile        authenticated current-profile update
POST  /v1/users/password authenticated current-user password update
```

The PATCH handler gets the user ID only from `UserContext`. It never accepts a path/user ID from the client.

### Step 4: Validate profile input

Apply these bounds:

```text
firstName: trimmed, 1..=100 characters when supplied
lastName:  trimmed, 1..=100 characters when supplied
age:       0..=150 when supplied
avatar:    <= 2048 characters when supplied
gender:    preserve existing integer compatibility
```

If no field is supplied, return `400 BAD_REQUEST`.

### Step 5: Run tests and commit

Run:

```bash
cargo test --test users_authorization
cargo test --test users_auth
```

Expected: all pass.

Suggested commit:

```bash
git add src/api src/infrastructure/db/user_repo.rs tests/users_authorization.rs tests/users_auth.rs
git commit -m "fix: restrict user operations to current profile"
```

## 6. Task 3: Replace raw MongoDB input with typed storage commands

**Files:**

- Create: `tests/storage_authorization.rs`
- Create: `tests/storage_hierarchy.rs`
- Modify: `src/api/storage.rs`
- Modify: `src/service/storage.rs`
- Modify: `src/infrastructure/db/storage_repo.rs`

### Step 1: Write failing mass-assignment tests

Test that `PUT /v1/storage/{id}` cannot change:

```text
userId
MD5Hash
contentHash
hashAlgorithm
iv
encryptionFormat
type
thumbnail
trashed
createdAt
updatedAt
```

Unknown fields must cause `400 BAD_REQUEST`, not be ignored.

Test that one user cannot update, move, trash, restore, list, or download another user's item.

### Step 2: Add typed DTOs

Replace `Json<Document>` request bodies with:

```rust
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateStorageDto {
    pub name: String,
}

#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct StorageQueryDto {
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct GetDownloadUrlDto {
    #[serde(rename = "fileId")]
    pub file_id: String,
}
```

Change `GetFilesDto.query` from `Document` to `StorageQueryDto`. Keep the existing JSON shape:

```json
{
  "query": {
    "parentId": "root"
  }
}
```

Service/repository code may construct BSON documents internally. BSON operators never cross the HTTP trust boundary.

### Step 3: Validate names and parents

Use one shared validation function for file/folder names:

```text
trimmed
1..=255 Unicode scalar values
not "." or ".."
contains no NUL
```

For create and move:

- `"root"` is valid.
- Any other parent ID must parse as ObjectId.
- The parent must exist, belong to the current user, be a non-trashed folder, and not be the item itself.
- Moving a folder under one of its descendants returns `400 BAD_REQUEST`.

Use the existing subtree traversal rather than adding MongoDB `$graphLookup`.

### Step 4: Add cycle protection

Even after validating writes, `get_path` must maintain a `HashSet<ObjectId>`. If an existing cycle is found, return an internal consistency error and log the item ID; never loop forever or return a silently truncated path.

Database errors while walking parents must propagate.

### Step 5: Verify

Run:

```bash
cargo test --test storage_authorization
cargo test --test storage_hierarchy
cargo test --test storage_folder
cargo test --test storage_trash
```

Expected: all pass, and no test can mutate another user's metadata.

Suggested commit:

```bash
git add src/api/storage.rs src/service/storage.rs src/infrastructure/db/storage_repo.rs tests
git commit -m "fix: type storage commands and validate hierarchy"
```

## 7. Task 4: Make configuration and startup fail safely

**Files:**

- Modify: `src/config/mod.rs`
- Modify: `src/infrastructure/db/mod.rs`
- Modify: `src/main.rs`
- Modify: `src/api/mod.rs`
- Modify: `.env.example`
- Modify: `docker-compose.yaml`
- Modify: `Dockerfile`
- Create or modify tests inside: `src/config/mod.rs`

### Step 1: Write failing config tests

Serialize environment-mutating tests with `serial_test`.

Required cases:

```text
missing_jwt_secret_fails
short_jwt_secret_fails
missing_storage_master_key_fails
invalid_storage_master_key_hex_fails
missing_mongo_uri_fails
valid_config_loads
```

### Step 2: Replace infallible defaults

Implement:

```rust
impl Config {
    pub fn from_env() -> Result<Self, ConfigError>;
}
```

Required fields:

```text
MONGO_URI
JWT_SECRET_KEY              at least 32 bytes
STORAGE_MASTER_KEY_HEX      exactly 64 hexadecimal characters
```

Optional fields:

```text
PORT                        default 3821
DRIVE_DOMAIN                default http://localhost:{PORT}
STORAGE_DESTINATION         default bucket/storage
MAX_UPLOAD_BYTES            default 104857600
```

Add `storage_destination: PathBuf`, `storage_master_key: [u8; 32]`, and `max_upload_bytes: usize` to `Config`.

Do not keep `default_secret_please_change`.

### Step 3: Make database startup observable

Change:

```rust
pub async fn init_db(config: &Config) -> Database
```

to return a `Result<Database, ...>`.

Before returning:

1. Parse client options.
2. create the client.
3. run `{ ping: 1 }`.
4. create every required index.
5. propagate any failure.

Never ignore index creation errors.

### Step 4: Fix Docker runtime settings

`docker-compose.yaml` service must set:

```yaml
environment:
  MONGO_URI: mongodb://test:nest@mongo:27017/fragrans?authSource=admin
  JWT_SECRET_KEY: ${JWT_SECRET_KEY}
  STORAGE_MASTER_KEY_HEX: ${STORAGE_MASTER_KEY_HEX}
  STORAGE_DESTINATION: /app/bucket/storage
  DRIVE_DOMAIN: ${DRIVE_DOMAIN:-http://localhost:8085}
```

Set runtime `WORKDIR /app` in `Dockerfile`. Keep the volume at `/app/bucket`.

Run the service as a non-root user that can write `/app/bucket`.

Add:

```text
GET /health/live   returns 200 if the process is running
GET /health/ready  returns 200 only after Mongo ping and writable storage-root check
```

Compose `depends_on` must use Mongo health status, not startup order alone.

### Step 5: Verify

Run:

```bash
cargo test config::tests
docker compose config -q
JWT_SECRET_KEY='test-secret-key-that-is-at-least-32-bytes' \
STORAGE_MASTER_KEY_HEX='0000000000000000000000000000000000000000000000000000000000000000' \
docker compose up --build -d
curl --fail http://localhost:8085/health/live
curl --fail http://localhost:8085/health/ready
docker compose ps
```

Expected: both health endpoints return 200 and the service is healthy.

Do not run `docker compose down -v`.

Suggested commit:

```bash
git add src/config src/infrastructure/db src/main.rs src/api/mod.rs .env.example Dockerfile docker-compose.yaml
git commit -m "fix: fail startup on invalid production configuration"
```

## 8. Task 5: Implement authenticated per-user physical storage

**Files:**

- Create: `tests/storage_crypto.rs`
- Modify: `Cargo.toml`
- Modify: `src/domain/storage.rs`
- Modify: `src/infrastructure/storage/local.rs`
- Modify: `src/utils/encryption.rs`
- Modify: `src/utils/md5.rs`
- Modify: `src/utils/mod.rs`
- Modify: `src/api/mod.rs`
- Modify: `src/service/storage.rs`

### Step 1: Write pure filesystem/crypto tests

These tests must not require MongoDB.

Use `TempDir` and a fixed test master key. Required test names:

```text
encrypted_roundtrip_for_boundary_sizes
tampered_header_is_rejected
tampered_ciphertext_is_rejected
wrong_key_is_rejected
wrong_user_is_rejected
wrong_hash_is_rejected
same_content_for_different_users_uses_different_paths
failed_write_leaves_no_temp_or_partial_file
concurrent_identical_writes_produce_one_readable_object
two_users_upload_same_content_and_both_download_original
```

Boundary sizes:

```text
0
1
chunk_size - 1
chunk_size
chunk_size + 1
2 * chunk_size + 17
```

### Step 2: Change direct dependencies

Add direct dependencies:

```toml
sha2 = "0.10"
aes-gcm = "0.10"
```

After implementation remove direct dependencies that are no longer used:

```text
cipher
```

Keep `md-5`, `aes`, and `ctr` temporarily for the Task 6 legacy migration reader. Keep `hex` and `rand`. Remove the legacy crypto dependencies only in a later release after every valuable object is migrated and the migration binary is retired.

### Step 3: Make `LocalStorage` explicit and reusable

Construct it once:

```rust
#[derive(Clone)]
pub struct LocalStorage {
    root_path: Arc<PathBuf>,
    master_key: Arc<[u8; 32]>,
}

impl LocalStorage {
    pub fn new(root_path: PathBuf, master_key: [u8; 32]) -> Result<Self, std::io::Error>;
}
```

Do not read environment variables or create directories on every request.

Store one `LocalStorage` in `AppState` and pass a clone into `StorageService`.

### Step 4: Implement the version 1 format

Implement the exact format in section 3.3.

Expose only the smallest necessary API:

```rust
pub async fn store_from_file(
    &self,
    user_id: &str,
    content_hash: &str,
    source: &Path,
) -> Result<(), StorageIoError>;

pub async fn read_all(
    &self,
    user_id: &str,
    content_hash: &str,
) -> Result<Vec<u8>, StorageIoError>;

pub async fn remove(
    &self,
    user_id: &str,
    content_hash: &str,
) -> Result<(), StorageIoError>;

pub async fn exists(
    &self,
    user_id: &str,
    content_hash: &str,
) -> Result<bool, StorageIoError>;
```

`read_all` exists for tests, thumbnails, and migration. Task 8 adds the streaming reader used by downloads.

`store_from_file` recomputes SHA-256 while reading and rejects a mismatch with `content_hash`.

### Step 5: Change new upload metadata

Upload hashing remains streaming, but uses SHA-256.

New objects use the new metadata fields. Dedup/reuse queries use:

```text
userId + contentHash + hashAlgorithm=sha256 + encryptionFormat=1 + type
```

Physical access always includes `user_id`.

`empty_trash` checks remaining references by both `userId` and `contentHash` before removing the per-user physical object.

### Step 6: Verify

Run:

```bash
cargo test --test storage_crypto
cargo test --test storage_upload
cargo test --test storage_trash
```

Expected: all pass, including cross-user path isolation and tamper detection.

Suggested commit:

```bash
git add Cargo.toml Cargo.lock src tests/storage_crypto.rs tests/storage_upload.rs tests/storage_trash.rs
git commit -m "feat: add authenticated per-user content storage"
```

## 9. Task 6: Preserve and migrate legacy objects

**Files:**

- Create: `src/bin/migrate_storage_v1.rs`
- Create: `tests/storage_migration.rs`
- Modify: `src/infrastructure/storage/local.rs`
- Modify: `src/infrastructure/db/storage_repo.rs`
- Modify: `src/domain/storage.rs`

### Step 1: Write migration tests

Required behaviors:

```text
dry_run_changes_nothing
valid_legacy_object_migrates_and_roundtrips
corrupted_legacy_object_is_reported_and_not_updated
missing_legacy_object_is_reported_and_not_updated
already_migrated_object_is_ignored
shared_legacy_hash_is_not_deleted
migration_is_idempotent
```

### Step 2: Keep a legacy-only reader

Legacy AES-CTR/MD5 code may remain only in a private migration/read-compatibility module. New upload and normal write paths must not call it.

For a legacy record:

1. Read the old global MD5 path.
2. Decrypt using the legacy MD5-derived key and stored IV.
3. Recompute MD5 and compare it with `MD5Hash`.
4. If mismatched, report corruption and do not update MongoDB.
5. Compute SHA-256.
6. Store and verify the v1 per-user object.
7. Update every matching logical document for that user.
8. Leave the old object in place.

### Step 3: Implement a safe CLI

Default invocation is dry-run:

```bash
cargo run --bin migrate_storage_v1
```

Applying changes requires an explicit flag:

```bash
cargo run --bin migrate_storage_v1 -- --apply
```

Use standard-library argument parsing; do not add Clap for one flag.

Output only counts and object/document IDs:

```text
scanned
already_v1
migratable
migrated
corrupted
missing
failed
```

Exit non-zero if `corrupted`, `missing`, or `failed` is non-zero during `--apply`.

### Step 4: Verify

Run:

```bash
cargo test --test storage_migration
cargo run --bin migrate_storage_v1
```

Expected: tests pass and dry-run performs no MongoDB or bucket mutation.

Before running `--apply` against valuable data:

1. Back up MongoDB.
2. Back up the bucket.
3. Run dry-run and save its counts.
4. Require operator confirmation outside this agent.
5. Run apply.
6. Run dry-run again; `migratable` must be zero.

Suggested commit:

```bash
git add src/bin/migrate_storage_v1.rs src/infrastructure src/domain tests/storage_migration.rs
git commit -m "feat: add verified legacy storage migration"
```

## 10. Task 7: Make uploads bounded, deterministic, and honest

**Files:**

- Modify: `Cargo.toml`
- Modify: `src/api/storage.rs`
- Modify: `src/service/storage.rs`
- Modify: `src/infrastructure/image/thumbnail.rs`
- Modify: `tests/storage_upload.rs`

### Step 1: Add failing upload tests

Required behaviors:

```text
parent_id_order_does_not_change_destination
malformed_multipart_returns_400
file_processing_failure_is_not_reported_as_success
request_over_limit_returns_413
blank_filename_returns_400
invalid_parent_returns_400
temporary_files_are_removed_after_success
temporary_files_are_removed_after_failure
oversized_image_dimensions_are_rejected
```

### Step 2: Parse before persisting

Stream every file field to an RAII temporary file and record its metadata. Parse the entire multipart body before calling `StorageService`.

This makes `parentId` field ordering irrelevant.

Move `tempfile` from dev-dependencies to dependencies and use it for cleanup. Do not invent a custom temp-file guard.

### Step 3: Enforce limits

Apply `DefaultBodyLimit::max(config.max_upload_bytes)` to upload routes and keep an explicit byte counter while streaming.

Use:

```text
HTTP 413 for total/request size overflow
HTTP 400 for malformed multipart or invalid metadata
HTTP 500 for storage/database failures
```

Propagate the first file-processing failure. Do not log-and-continue and then return 200.

Earlier files in the same request may already be committed; retries remain safe because content writes are idempotent and existing metadata is returned. Document this non-transactional multipart behavior in OpenAPI.

### Step 4: Bound image work

Before decoding the full image:

- reject an encoded image larger than the upload limit;
- inspect dimensions;
- reject width or height over 20,000 pixels;
- reject total pixels over 100,000,000.

Continue using `spawn_blocking` for decode/resize/encode.

### Step 5: Verify

Run:

```bash
cargo test --test storage_upload
```

Expected: every case passes and the test temp directory contains no upload temp files.

Suggested commit:

```bash
git add Cargo.toml Cargo.lock src/api/storage.rs src/service/storage.rs src/infrastructure/image/thumbnail.rs tests/storage_upload.rs
git commit -m "fix: bound uploads and report failures"
```

## 11. Task 8: Scope download tokens and stream decrypted files

**Files:**

- Create: `tests/storage_download.rs`
- Modify: `src/api/middleware.rs`
- Modify: `src/api/storage.rs`
- Modify: `src/api/mod.rs`
- Modify: `src/domain/storage.rs`
- Modify: `src/infrastructure/storage/local.rs`
- Modify: `src/service/storage.rs`

### Step 1: Write failing token tests

Required behaviors:

```text
access_token_can_call_authenticated_api
download_token_cannot_call_authenticated_api
download_token_can_fetch_only_its_file
download_token_cannot_fetch_sibling_file
download_url_rejects_unowned_file
expired_download_token_is_rejected
download_response_does_not_log_query_token
```

### Step 2: Issue scoped tokens

Implement one token helper used by login, list responses, and `POST /v1/storage/download/url`.

Token encoding errors propagate as `AppError`; never use `unwrap_or_default`.

Before issuing a URL, load the file and verify:

- it exists;
- it belongs to the current user;
- it is not trashed;
- it is a file or thumbnail.

List response URLs require separate file-scoped tokens for the file and thumbnail IDs.

### Step 3: Remove query-token authentication from general middleware

`auth_guard` reads only `Authorization: Bearer ...` and accepts only `purpose=access`.

The public GET download route performs its own download/access token validation.

### Step 4: Add a streaming v1 reader

Add a method that returns a fallible byte stream or an Axum `Body`. It must:

- parse and validate the header before returning success;
- read one encrypted chunk at a time;
- authenticate and yield one plaintext chunk at a time;
- keep memory bounded to roughly one encrypted and one plaintext chunk;
- stop immediately on authentication/truncation errors.

The normal v1 download path must no longer call `read_to_end`.

Legacy data may continue using the bounded migration path until migrated; if legacy download compatibility is retained, enforce `MAX_UPLOAD_BYTES` as its maximum allocation and mark the path deprecated in code.

### Step 5: Set safe response headers

Download responses include:

```text
Content-Type
Content-Length
Content-Disposition
Cache-Control: private, no-store
Referrer-Policy: no-referrer
X-Content-Type-Options: nosniff
```

Sanitize the filename used in `Content-Disposition`.

Configure `TraceLayer` to record only `uri.path()`, never the query string.

### Step 6: Verify memory behavior

Add a test that downloads a file larger than three encryption chunks and verifies body chunks are produced before the entire response is collected.

Run:

```bash
cargo test --test storage_download
cargo test --test storage_upload
```

Expected: all pass; a download token for file A receives 401/404 for file B.

Suggested commit:

```bash
git add src tests/storage_download.rs
git commit -m "fix: scope download tokens and stream file responses"
```

## 12. Task 9: Remove redundant queries and create indexes that match real access

**Files:**

- Modify: `src/api/storage.rs`
- Modify: `src/service/storage.rs`
- Modify: `src/infrastructure/db/storage_repo.rs`
- Modify: `src/infrastructure/db/mod.rs`
- Modify: `tests/storage_folder.rs`
- Modify: `tests/storage_trash.rs`

### Step 1: Add list-filter tests

Required behaviors:

```text
types_file_returns_only_files
types_folder_returns_only_folders
keyword_is_literal_not_regex
unknown_type_returns_empty_or_400_consistently
sort_uses_id_as_stable_tiebreaker
raw_mongo_operators_are_rejected_by_deserialization
restore_folder_name_conflict_returns_409_and_leaves_item_trashed
```

Choose `400 BAD_REQUEST` for unknown types.

### Step 2: Delete thumbnail exclusion queries

The list query already restricts type to file/folder. Delete:

- `thumbnail_object_ids`
- its calls from active and trash lists
- associated error swallowing

Build the trusted type filter once from the DTO. Do not overwrite it later in the service.

Treat keyword as a literal substring. Add `regex = "1"` as a direct dependency, use `regex::escape`, and limit the keyword to 100 characters. Do not maintain a hand-written escaping table.

### Step 3: Propagate errors

Replace all silent fallbacks on required database/storage operations. In particular:

- trashed folder lookup errors;
- parent traversal errors;
- storage fetch/decrypt errors;
- index creation errors;
- JWT encoding errors;
- file cleanup failures.

A missing object may return 404. An I/O permission error, malformed encrypted file, or Mongo failure must return/log an internal error, not 404.

### Step 4: Create minimum required indexes

Create named indexes:

```text
users_email_unique
  keys: { email: 1 }
  unique: true

storage_active_list
  keys: { userId: 1, trashed: 1, parentId: 1, type: 1, updatedAt: -1, _id: 1 }

storage_parent_walk
  keys: { userId: 1, parentId: 1 }

storage_content_reference
  keys: { userId: 1, contentHash: 1 }

storage_active_folder_name_unique
  keys: { userId: 1, parentId: 1, name: 1 }
  unique: true
  partial filter: { type: "folder", trashed: false }
```

Map duplicate folder/user insert errors to `409 CONFLICT`.

Do not add one index per sort option yet.

### Step 5: Verify

Run:

```bash
cargo test --test storage_folder
cargo test --test storage_trash
cargo test --test storage_authorization
```

Use MongoDB `explain("executionStats")` manually for the default active list and parent-walk queries. Record winning index names in the implementation summary.

Suggested commit:

```bash
git add src tests
git commit -m "perf: remove redundant storage queries and align indexes"
```

## 13. Task 10: Keep password work off Tokio workers

**Files:**

- Modify: `src/utils/crypto.rs`
- Modify: `src/api/users.rs`
- Modify: `src/api/error.rs`
- Modify: `src/infrastructure/db/user_repo.rs`
- Modify: `tests/common/mod.rs`
- Modify: `tests/users_auth.rs`

### Step 1: Make crypto helpers async

Wrap bcrypt hash and verify in `tokio::task::spawn_blocking`. Return errors; do not panic with `unwrap`.

Required signatures:

```rust
pub async fn hash_password(password: String) -> Result<String, CryptoError>;
pub async fn verify_password(password: String, hashed: String) -> Result<bool, CryptoError>;
```

### Step 2: Validate registration input

Registration requires:

```text
email: trimmed, non-empty, <= 320 characters, contains one @ with non-empty sides
password: 8..=1024 bytes
firstName/lastName: same profile bounds
```

Normalize email to lowercase before lookup/insert.

Do not add a full email-validation dependency.

### Step 3: Verify

Run:

```bash
cargo test --test users_auth
cargo test --test users_authorization
```

Expected: all pass. Duplicate registration returns `409 CONFLICT`.

Suggested commit:

```bash
git add src/utils/crypto.rs src/api/users.rs tests
git commit -m "perf: move bcrypt work off async workers"
```

## 14. Task 11: Remove dead weight and make documentation truthful

**Files:**

- Modify: `Cargo.toml`
- Modify: `.gitignore`
- Modify: `.dockerignore`
- Modify: `README.md`
- Modify: `.env.example`
- Delete: `rewrite.py`
- Delete: `test_api.py`
- Modify: any Rust file reported by formatter/Clippy

### Step 1: Remove unused dependencies

Confirm each removal with literal source search and `cargo check`.

Expected candidates:

```text
config
futures
cipher
```

`md-5`, `aes`, and `ctr` remain while the checked-in legacy migration reader exists. Remove them only when migration support is deliberately retired in a later change.

Move test-only crates such as `tower` and `bytes` to dev-dependencies.

Change:

- `tower-http` to only required features, initially `trace`;
- `tokio` from `full` to the exact used features;
- `image` to `default-features = false` and only formats actually accepted/generated.

Do not optimize transitive duplicate versions manually unless Cargo can unify them through a direct dependency update.

### Step 2: Delete obsolete files

Delete:

- `rewrite.py`, which performs no useful operation;
- `test_api.py`, which duplicates Rust integration coverage and depends on undeclared `requests`.

Replace the Node-oriented `.gitignore` with Rust/project entries only:

```text
/target
/.env
/.env.test
/bucket
*.log
.DS_Store
```

Keep Docker build context exclusions relevant to this Rust project.

### Step 3: Update README

Document:

- actual minimum Rust version through `rust-version` in Cargo.toml;
- required environment variables;
- local Mongo startup;
- `scripts/verify.sh`;
- v1 encrypted storage and user-local deduplication;
- migration dry-run/apply procedure;
- health endpoints;
- Docker Compose commands;
- upload size default;
- signed download-link lifetime.

Do not claim a fixed Docker image size without measuring it in CI.

### Step 4: Format and lint

Run:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged
cargo fmt --all
```

Review every automatic change. Then run strict Clippy without `--fix`.

Suggested commit:

```bash
git add -A
git commit -m "chore: remove dead dependencies and document operations"
```

## 15. Task 12: Complete verification

No implementation is complete until every item in this section passes.

### 15.1 Repository state

```bash
git status --short
git diff --check
```

Expected:

- no unintended files;
- no conflict markers or whitespace errors;
- generated secrets, `.env`, bucket contents, Mongo dumps, and migration backups are untracked.

### 15.2 Rust quality gates

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --doc
cargo build --release
```

Expected: every command exits zero with no warnings.

### 15.3 Required named regression tests

Confirm the test list contains and passes at least:

```text
ordinary_user_cannot_read_another_user
ordinary_user_cannot_update_another_user
ordinary_user_cannot_delete_another_user
same_content_for_different_users_uses_different_paths
tampered_ciphertext_is_rejected
concurrent_identical_writes_produce_one_readable_object
raw_mongo_operators_are_rejected_by_deserialization
file_processing_failure_is_not_reported_as_success
temporary_files_are_removed_after_failure
download_token_cannot_fetch_sibling_file
download_token_cannot_call_authenticated_api
valid_legacy_object_migrates_and_roundtrips
corrupted_legacy_object_is_reported_and_not_updated
```

Run:

```bash
cargo test -- --list
```

### 15.4 Docker verification

Use test-only secrets:

```bash
export JWT_SECRET_KEY='test-secret-key-that-is-at-least-32-bytes'
export STORAGE_MASTER_KEY_HEX='0000000000000000000000000000000000000000000000000000000000000000'
export DRIVE_DOMAIN='http://localhost:8085'

docker compose config -q
docker compose up --build -d
docker compose ps
curl --fail http://localhost:8085/health/live
curl --fail http://localhost:8085/health/ready
```

Inspect logs without printing query strings or secrets:

```bash
docker compose logs --no-color yi-svc-storage
```

Expected:

- service and Mongo are healthy;
- no fallback/default-secret warning;
- no Mongo connection error;
- no signed query token appears in request spans;
- files are created beneath the mounted `/app/bucket/storage`.

Do not use `docker compose down -v`.

### 15.5 API smoke verification

Using a unique test email:

1. Register user A.
2. Register user B.
3. Login both.
4. A uploads content X.
5. B uploads the same content X.
6. Download both files and compare with X.
7. A attempts to fetch/update/delete B's resources and receives 404.
8. Obtain A's scoped URL and verify it cannot download another A file.
9. Tamper with a copied encrypted object in a temporary test bucket and verify download fails.
10. Create, move, trash, restore, and empty a folder tree.

Prefer Rust integration tests for these operations. Shell smoke commands are supplemental and must not replace assertions.

### 15.6 Migration verification

For a disposable fixture database/bucket:

```bash
cargo run --bin migrate_storage_v1
cargo run --bin migrate_storage_v1 -- --apply
cargo run --bin migrate_storage_v1
```

Expected:

- first dry-run reports migratable legacy objects;
- apply exits zero and reports matching migrated counts;
- second dry-run reports zero migratable objects;
- every migrated file downloads byte-for-byte;
- legacy files still exist.

Do not run `--apply` against valuable data without external operator confirmation and backups.

### 15.7 Final single-command verification

With test Mongo available:

```bash
TEST_MONGO_URI='mongodb://test:nest@127.0.0.1:25018/?authSource=admin' \
JWT_SECRET_KEY='test-secret-key-that-is-at-least-32-bytes' \
STORAGE_MASTER_KEY_HEX='0000000000000000000000000000000000000000000000000000000000000000' \
./scripts/verify.sh
```

Expected: exit code zero.

## 16. Definition of done

The refactor is complete only when:

- no ordinary user-management endpoint permits cross-user access;
- storage update bodies cannot write arbitrary BSON;
- identical content uploaded by different users cannot overwrite or corrupt either user's data;
- new physical files use SHA-256 and authenticated encryption;
- tampering and wrong-key use fail closed;
- uploads and downloads have bounded memory/disk behavior;
- file download tokens are file-scoped and unusable as API access tokens;
- production configuration has no default JWT or encryption secret;
- Mongo/index/storage readiness failures prevent readiness;
- Docker Compose connects to Mongo and writes into the mounted volume;
- legacy migration is dry-run by default, verified, idempotent, and non-destructive;
- format, strict Clippy, all tests, release build, Docker build, health checks, and the verification script pass;
- README and `.env.example` describe the actual behavior;
- `git diff --check` passes and no secrets/data artifacts are included.

## 17. Final implementation report template

Gemini must return:

```markdown
## Completed

- [task/commit summary]

## Verification

- `cargo fmt --all -- --check`: PASS
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS
- `cargo test`: PASS, N tests
- `cargo build --release`: PASS
- `docker build -t fragrans:verify .`: PASS
- Docker health checks: PASS
- migration fixture dry-run/apply/dry-run: PASS

## Data migration

- mode: reset | preserve
- scanned:
- migrated:
- corrupted:
- missing:
- legacy objects deleted: 0

## Remaining limitations

- [Only measured or explicitly deferred limitations; do not list speculative architecture work.]
```
