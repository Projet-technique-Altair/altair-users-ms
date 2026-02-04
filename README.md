# Altair Users Microservice

## Users

### 0. Start infrastructure (database required)

```bash
docker compose up postgres
```

### 1. Build the Docker image (ONLY if code changed)

Use this step if:

* you modified the users code
* you modified the Dockerfile
* first run on a new machine

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

### Notes

* Authentication is handled by Keycloak, not by this service
* This service only manages user-related data
* The frontend must never access this service directly
* The service is meant to be destroyed when the terminal closes : rebuild necessary



