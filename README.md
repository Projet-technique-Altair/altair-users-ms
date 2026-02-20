# Altaïr Users Microservice

> **Internal identity service bridging Keycloak authentication with Altaïr's user system**
> 

---

## Description

The **Altaïr Users Microservice** converts external Keycloak identities into internal user records stored in PostgreSQL. It provides controlled access to user profiles through a REST API and serves as the **application identity layer** for the entire Altaïr platform.

This service does **not** perform authentication itself—it operates on a **trust model** where the upstream API Gateway validates JWT tokens and injects verified identity headers.

**Key capabilities:**

- Create internal user records from Keycloak identities
- Map Keycloak IDs to internal UUIDs
- Provide role-based access to user profiles (`learner`, `creator`, `admin`)
- Auto-generate user pseudos and metadata
- Health checks for orchestration

---

## Security Notice

**This service must NEVER be publicly accessible.**

It trusts identity headers injected by the API Gateway and does not perform JWT validation itself. Direct access would allow header forgery and identity spoofing.

**Deployment requirement:** Must run in a private VPC or behind an authenticated API Gateway.

---

## Architecture

```
Frontend → API Gateway (validates Keycloak JWT) → User Microservice → PostgreSQL
                ↓                                          ↓
           Injects headers                          Stores user records
       (x-altair-keycloak-id, etc.)
```

**Trust model:** The service assumes the upstream gateway has already validated authentication and injects trusted identity headers.

---

## Tech Stack

| Component | Technology |
| --- | --- |
| **Language** | Rust (nightly) |
| **HTTP Framework** | Axum (async) |
| **Async Runtime** | Tokio |
| **Database** | PostgreSQL |
| **DB Client** | SQLx (compile-time checked queries) |
| **Logging** | tracing + EnvFilter |
| **Containerization** | Docker (multi-stage build) |
| **Deployment** | Google Cloud Run (serverless auto-scaling) |
| **CI/CD** | GitHub Actions (fmt, clippy, tests, Gitleaks) |

---

## Requirements

### Development

- **Rust** nightly toolchain
- **Docker** & Docker Compose
- **PostgreSQL** 14+ (via `docker compose up postgres`)

### Runtime (Production)

- **DATABASE_URL** environment variable (PostgreSQL connection string)
- **PORT** environment variable (default: `3001`)
- **ALLOWED_ORIGINS** CORS allowlist (CSV)
- **ALLOWED_METHODS** CORS methods allowlist (CSV)
- **ALLOWED_HEADERS** CORS headers allowlist (CSV)
- Private network access (no public internet exposure)

### Environment Variables

```bash
DATABASE_URL=postgresql://user:password@postgres:5432/altair_users
PORT=3001  # Optional, defaults to 3001
ALLOWED_ORIGINS=http://localhost:5173,http://localhost:3000
ALLOWED_METHODS=GET,OPTIONS
ALLOWED_HEADERS=authorization,content-type,x-altair-keycloak-id,x-altair-name,x-altair-email,x-altair-roles,x-altair-user-id
RUST_LOG=info  # Optional, for logging level
```

**CORS env format rules:**

- CSV values
- No spaces between values
- Header names in lowercase
- HTTP methods in uppercase

---

## Installation

### 0. Start infrastructure (database required)

```bash
docker compose up postgres
```

### 1. Build the Docker image

**Build only if:**

- You modified the users code
- You modified the Dockerfile
- First run on a new machine

```bash
cd altair-users-ms
docker build -t altair-users-ms .
```

### 2. Run the service

```bash
docker run --rm -it \
  --network altair-infra_default \
  -p 3001:3001 \
  --env-file .env \
  --name altair-users-ms \
  altair-users-ms
```

**Note:** The service is designed to be destroyed when the terminal closes. Rebuild is necessary for code changes.

---

## Usage

### API Endpoints

#### **GET /health**

Health check for liveness/readiness probes.

**Response:**

```json
{
  "status": "ok"
}
```

---

#### **GET /me/**

Resolve the current user from Keycloak identity, creating the user record if it doesn't exist.

**Headers (injected by Gateway):**

- `x-altair-keycloak-id` (required) – Keycloak user ID
- `x-altair-name` (optional)
- `x-altair-email` (optional)
- `x-altair-roles` (optional, CSV) – Role list

**Role selection logic:**

- Role values are normalized before matching:
    - trim spaces
    - lowercase
    - strip common prefixes (`role_`, `realm:`, `client:<name>:`)
- Contains `admin` → role = `admin`
- Else contains `creator` → role = `creator`
- Else contains `learner` → role = `learner`
- If no recognized role is found, request is rejected with `403 Forbidden`

**Response:**

```json
{
  "success": true,
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "role": "learner",
    "name": "John Doe",
    "pseudo": "johndoe",
    "email": "john@example.com",
    "avatar": null,
    "last_login": null,
    "created_at": "2026-02-08T15:30:00Z"
  },
  "meta": {
    "request_id": "...",
    "timestamp": "2026-02-08T15:30:00Z"
  }
}
```

 *This is the primary endpoint the gateway calls to map Keycloak tokens to internal Altaïr user IDs.*

---

#### **GET /users/:id**

Retrieve a user by `user_id` with access control.

**Headers (injected by Gateway):**

- `x-altair-user-id` (required, UUID) – Caller's internal user ID
- `x-altair-roles` (optional, CSV)

**Authorization rules:**

- Allow if caller has `admin` role
- Allow if `caller.user_id == target_user_id`
- Otherwise return `403 Forbidden`

**Response:**

```json
{
  "success": true,
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "role": "learner",
    "name": "John Doe",
    "pseudo": "johndoe",
    "email": "john@example.com",
    "avatar": null,
    "last_login": null,
    "created_at": "2026-02-08T15:30:00Z"
  },
  "meta": { /* ... */ }
}
```

**Error (403):**

```json
{
  "success": false,
  "error": {
    "code": "FORBIDDEN",
    "message": "No recognized role in x-altair-roles",
    "details": null
  },
  "meta": { /* ... */ }
}
```

---

### Standard Response Format

**Success:**

```json
{
  "success": true,
  "data": { /* payload */ },
  "meta": {
    "request_id": "...",
    "timestamp": "2026-02-08T15:30:00Z"
  }
}
```

**Error:**

```json
{
  "success": false,
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "User not found",
    "details": null
  },
  "meta": { /* ... */ }
}
```

---

## Database Schema

**Table: `users`**

| Column | Type | Constraints | Description |
| --- | --- | --- | --- |
| `user_id` | UUID | PRIMARY KEY | Internal user identifier (generated by DB) |
| `keycloak_id` | TEXT | UNIQUE, NOT NULL | External Keycloak identity |
| `role` | TEXT | NOT NULL | User role: `learner`, `creator`, or `admin` |
| `name` | TEXT | NOT NULL | User display name |
| `pseudo` | TEXT | NOT NULL | Auto-generated as `lowercase(name)` |
| `email` | TEXT |  | User email address |
| `avatar` | TEXT | NULLABLE | Avatar URL |
| `last_login` | TIMESTAMP | NULLABLE | Last login timestamp (not yet implemented) |
| `created_at` | TIMESTAMP | NOT NULL | Account creation timestamp |

 **Note:** The repository does not include SQL migrations. The database schema must be provisioned externally.

---

## Project Structure

```
altair-users-ms/
├── Cargo.toml                    # Rust dependencies
├── Dockerfile                    # Multi-stage build
├── README.md                     # This file
├── .github/
│   └── workflows/
│       └── ci.yml               # CI pipeline
└── src/
    ├── main.rs                  # Server bootstrap, CORS, routes
    ├── state.rs                 # App state (DB pool + services)
    ├── error.rs                 # Error handling
    ├── routes/
    │   ├── health.rs           # Health check endpoint
    │   ├── me.rs               # Current user resolver
    │   ├── users.rs            # Get user by ID
    │   └── metrics.rs          # Metrics endpoint (not mounted)
    ├── models/
    │   ├── user.rs             # User data models
    │   └── api.rs              # API response wrappers
    ├── services/
    │   ├── users_service.rs    # User data access layer
    │   └── extractor.rs        # Identity header extraction
    ├── extractors/
    │   └── auth_user.rs        # Axum auth extractor
    └── tests/
        └── users_smoke_test.rs # E2E tests (WIP)
```

---

## Deployment (Google Cloud Run)

The service is containerized and deployed to **Google Cloud Run** with the following characteristics:

**Container Configuration:**

- Listens on port `3001` (configurable via `PORT` env variable)
- Multi-stage Docker build optimizes image size
- Rust nightly toolchain for compilation

**Runtime Requirements:**

- `DATABASE_URL` environment variable (Cloud SQL or external PostgreSQL)
- Must be deployed in a **private VPC** or behind an API Gateway
- Should not be publicly accessible (no direct internet traffic)

**Scaling:**

- Auto-scales based on request load
- Cold start optimized with Rust's fast startup time
- Stateless design enables horizontal scaling

---

## Known Issues & Limitations

### 🔴 Compilation Blockers
- [ ]  Unused imports trigger failures with `clippy -D warnings`

### 🟡 Security Concerns

- [ ]  Keep CORS allowlists (`ALLOWED_*`) synchronized across envs (dev/staging/prod)

### 🟡 Operational Gaps

- [ ]  No database migration files included
- [ ]  `last_login` tracking not implemented
- [ ]  No metrics endpoint exposed in router
- [ ]  Test suite cannot currently pass

### 🟡 Business Logic Limitations

- **No profile updates:** Existing users are never updated (no UPDATE queries for role/name/email)
- **`last_login` not maintained:** Never updated on access
- **Pseudo collisions:** `pseudo = lowercase(name)` can create duplicates
- **No unique slug generation**

---

## CI/CD Pipeline

GitHub Actions workflow (`.github/workflows/ci.yml`):

1. **Secret Scanning** – Gitleaks detects leaked credentials
2. **Format Check** – `cargo fmt --check`
3. **Linting** – `cargo clippy -D warnings`
4. **Tests** – `cargo test`
5. **Release Build** – `cargo build --release`

---

## Project Status

** Current Status: Pre-Alpha / Development**

This microservice is under active development and has several **operational gaps** that must be resolved before production use.

**Recent updates (implemented):**

- [x] Removed `COPY .env` from `Dockerfile` (avoid embedding secrets in image)
- [x] Simplified `GET /users/:id` and kept header convention via `extract_caller`
- [x] Replaced hardcoded `0.0.0.0:3001` bind with `PORT` env (fallback to `3001`)
- [x] Replaced permissive CORS (`Any`) with strict allowlists
- [x] Externalized CORS config to `.env` with safe code defaults

**Immediate priorities:**

1. Add database migration scripts
2. Implement `last_login` tracking
3. Add comprehensive integration tests

**Maintainers:** This is an internal Altaïr platform service. For questions or contributions, contact the platform team.

---

## Notes

- **Authentication** is handled by Keycloak, not by this service
- This service only manages user-related data
- The **frontend must never access this service directly**
- Always access through the authenticated API Gateway
- The service uses **fail-fast semantics** (panics if DATABASE_URL is missing or unreachable)

---

## License

Internal Altaïr Platform Service – Not licensed for external use.
