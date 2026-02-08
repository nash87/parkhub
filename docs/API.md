# API Documentation

ParkHub exposes a RESTful JSON API under `/api/v1/`.

## Base URL

```
http://localhost:7878/api/v1
```

## Response Format

Every response wraps data in a standard envelope:

```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "meta": null
}
```

Errors:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable message",
    "details": null
  },
  "meta": null
}
```

## Authentication

Authenticated endpoints require a Bearer token in the `Authorization` header:

```
Authorization: Bearer <access_token>
```

### Login

```bash
curl -X POST http://localhost:7878/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin"}'
```

**Response:**

```json
{
  "success": true,
  "data": {
    "user": {
      "id": "a1b2c3d4-...",
      "username": "admin",
      "email": "admin@parkhub.local",
      "name": "Admin",
      "picture": null,
      "phone": null,
      "role": "admin",
      "created_at": "2026-02-08T00:00:00Z",
      "updated_at": "2026-02-08T00:00:00Z",
      "last_login": "2026-02-08T07:54:25Z",
      "preferences": {
        "default_duration_minutes": null,
        "favorite_slots": [],
        "notifications_enabled": false,
        "email_reminders": false,
        "language": "",
        "theme": ""
      },
      "is_active": true,
      "department": null
    },
    "tokens": {
      "access_token": "37db5bfb-1b74-40c3-86aa-...",
      "refresh_token": "rt_25c04699-adb9-42ba-...",
      "expires_at": "2026-02-09T07:54:19Z",
      "token_type": "Bearer"
    }
  },
  "error": null,
  "meta": null
}
```

**Error (wrong credentials):**

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "INVALID_CREDENTIALS",
    "message": "Invalid username or password",
    "details": null
  },
  "meta": null
}
```

### Register

```bash
curl -X POST http://localhost:7878/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "john", "email": "john@example.com", "password": "SecurePass1", "name": "John Doe"}'
```

The password must be at least 8 characters with uppercase, lowercase, and a digit.

**Response:** Same envelope as login — returns `data.user` and `data.tokens`.

### Refresh Token

```bash
curl -X POST http://localhost:7878/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "rt_25c04699-..."}'
```

### Forgot Password

```bash
curl -X POST http://localhost:7878/api/v1/auth/forgot-password \
  -H "Content-Type: application/json" \
  -d '{"email": "john@example.com"}'
```

Requires SMTP to be configured. See [Configuration](CONFIGURATION.md).

## Users

### Get Current User

```bash
curl http://localhost:7878/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**

```json
{
  "success": true,
  "data": {
    "id": "e487f433-...",
    "username": "testuser",
    "email": "test@test.com",
    "name": "Test User",
    "picture": null,
    "phone": null,
    "role": "user",
    "created_at": "2026-02-08T07:54:19Z",
    "updated_at": "2026-02-08T07:54:19Z",
    "last_login": "2026-02-08T07:54:19Z",
    "preferences": {
      "default_duration_minutes": null,
      "favorite_slots": [],
      "notifications_enabled": false,
      "email_reminders": false,
      "language": "",
      "theme": ""
    },
    "is_active": true,
    "department": null
  },
  "error": null,
  "meta": null
}
```

### Get User by ID

```bash
curl http://localhost:7878/api/v1/users/:id \
  -H "Authorization: Bearer $TOKEN"
```

### Change Password

```bash
curl -X PATCH http://localhost:7878/api/v1/users/me/password \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"current_password": "old", "new_password": "NewSecure1"}'
```

## Parking Lots

### List Lots

```bash
curl http://localhost:7878/api/v1/lots \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**

```json
{
  "success": true,
  "data": [],
  "error": null,
  "meta": null
}
```

When lots exist, `data` is an array of lot objects.

### Get Lot Details

```bash
curl http://localhost:7878/api/v1/lots/:id \
  -H "Authorization: Bearer $TOKEN"
```

### Get Lot Slots

```bash
curl http://localhost:7878/api/v1/lots/:id/slots \
  -H "Authorization: Bearer $TOKEN"
```

### Get / Update Lot Layout

```bash
# Get layout
curl http://localhost:7878/api/v1/lots/:id/layout \
  -H "Authorization: Bearer $TOKEN"

# Update layout (admin)
curl -X PUT http://localhost:7878/api/v1/lots/:id/layout \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"rows": [...], "slots": [...]}'
```

### Get Slot QR Code

```bash
curl http://localhost:7878/api/v1/lots/:lot_id/slots/:slot_id/qr \
  -H "Authorization: Bearer $TOKEN"
```

## Bookings

### List Bookings

```bash
curl http://localhost:7878/api/v1/bookings \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**

```json
{
  "success": true,
  "data": [],
  "error": null,
  "meta": null
}
```

### Create Booking

```bash
curl -X POST http://localhost:7878/api/v1/bookings \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"lot_id": "...", "slot_id": "...", "date": "2026-02-10", "vehicle_id": "..."}'
```

### Update Booking

```bash
curl -X PATCH http://localhost:7878/api/v1/bookings/:id \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"vehicle_id": "new-vehicle-id"}'
```

### Cancel Booking

```bash
curl -X DELETE http://localhost:7878/api/v1/bookings/:id \
  -H "Authorization: Bearer $TOKEN"
```

### Check In

```bash
curl -X POST http://localhost:7878/api/v1/bookings/:id/checkin \
  -H "Authorization: Bearer $TOKEN"
```

### Export iCal

```bash
curl http://localhost:7878/api/v1/bookings/ical \
  -H "Authorization: Bearer $TOKEN" \
  -o bookings.ics
```

## Vehicles

### List Vehicles

```bash
curl http://localhost:7878/api/v1/vehicles \
  -H "Authorization: Bearer $TOKEN"
```

### Add Vehicle

```bash
curl -X POST http://localhost:7878/api/v1/vehicles \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"license_plate": "M-AB 1234", "make": "BMW", "model": "320d", "color": "black"}'
```

### Delete Vehicle

```bash
curl -X DELETE http://localhost:7878/api/v1/vehicles/:id \
  -H "Authorization: Bearer $TOKEN"
```

### Upload Vehicle Photo

```bash
curl -X POST http://localhost:7878/api/v1/vehicles/:id/photo \
  -H "Authorization: Bearer $TOKEN" \
  -F "photo=@car.jpg"
```

## Homeoffice

### Get Settings

```bash
curl http://localhost:7878/api/v1/homeoffice \
  -H "Authorization: Bearer $TOKEN"
```

### Update Settings

```bash
curl -X PUT http://localhost:7878/api/v1/homeoffice \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'
```

### Update Pattern

```bash
curl -X PUT http://localhost:7878/api/v1/homeoffice/pattern \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"days": ["monday", "wednesday", "friday"]}'
```

### Add Homeoffice Day

```bash
curl -X POST http://localhost:7878/api/v1/homeoffice/days \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"date": "2026-02-10"}'
```

### Remove Homeoffice Day

```bash
curl -X DELETE http://localhost:7878/api/v1/homeoffice/days/:id \
  -H "Authorization: Bearer $TOKEN"
```

## Waitlist

### Get Waitlist for Lot

```bash
curl http://localhost:7878/api/v1/lots/:id/waitlist \
  -H "Authorization: Bearer $TOKEN"
```

### Join Waitlist

```bash
curl -X POST http://localhost:7878/api/v1/lots/:id/waitlist \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"date": "2026-02-10"}'
```

## Admin Endpoints

All admin endpoints require the `admin` role.

### List Users

```bash
curl http://localhost:7878/api/v1/admin/users \
  -H "Authorization: Bearer $TOKEN"
```

### Create User

```bash
curl -X POST http://localhost:7878/api/v1/admin/users \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username": "jane", "email": "jane@example.com", "password": "TempPass1", "role": "user"}'
```

### Update User

```bash
curl -X PATCH http://localhost:7878/api/v1/admin/users/:id \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"role": "admin", "is_active": true, "department": "Engineering"}'
```

### List All Bookings

```bash
curl http://localhost:7878/api/v1/admin/bookings \
  -H "Authorization: Bearer $TOKEN"
```

### Statistics

```bash
curl http://localhost:7878/api/v1/admin/stats \
  -H "Authorization: Bearer $TOKEN"
```

### Reports

```bash
curl http://localhost:7878/api/v1/admin/reports \
  -H "Authorization: Bearer $TOKEN"
```

### Update Slot

```bash
curl -X PATCH http://localhost:7878/api/v1/admin/slots/:id \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"status": "disabled"}'
```

### Delete Lot

```bash
curl -X DELETE http://localhost:7878/api/v1/admin/lots/:id \
  -H "Authorization: Bearer $TOKEN"
```

### Branding

```bash
# Get branding
curl http://localhost:7878/api/v1/admin/branding \
  -H "Authorization: Bearer $TOKEN"

# Update branding
curl -X PUT http://localhost:7878/api/v1/admin/branding \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"company_name": "Acme Corp", "primary_color": "#3B82F6"}'

# Upload logo
curl -X POST http://localhost:7878/api/v1/admin/branding/logo \
  -H "Authorization: Bearer $TOKEN" \
  -F "logo=@logo.png"
```

### Reset Database

```bash
curl -X POST http://localhost:7878/api/v1/admin/reset \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"confirm": true}'
```

## GDPR Endpoints

### Export Personal Data

```bash
curl http://localhost:7878/api/v1/users/me/export \
  -H "Authorization: Bearer $TOKEN" \
  -o my-data.json
```

### Delete Account

```bash
curl -X DELETE http://localhost:7878/api/v1/users/me/delete \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"confirm": true}'
```

## Public Endpoints

These do not require authentication:

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/health/live` | Liveness probe |
| `GET` | `/health/ready` | Readiness probe |
| `GET` | `/status` | Server status |
| `GET` | `/metrics` | Prometheus metrics |
| `POST` | `/handshake` | Client handshake |
| `GET` | `/api/v1/privacy` | Privacy policy |
| `GET` | `/api/v1/about` | About information |
| `GET` | `/api/v1/help` | Help / FAQ |
| `GET` | `/api/v1/branding` | Public branding info |
| `GET` | `/api/v1/branding/logo` | Company logo |
| `GET` | `/api/v1/setup/status` | First-run setup status |

## Error Codes

| HTTP Code | Meaning |
|-----------|---------|
| `400` | Bad request / validation error |
| `401` | Unauthorized / invalid token |
| `403` | Forbidden / insufficient role |
| `404` | Resource not found |
| `409` | Conflict (e.g., slot already booked) |
| `429` | Rate limit exceeded |
| `500` | Internal server error |

---

Back to [README](../README.md) · Previous: [Configuration](CONFIGURATION.md) · Next: [Deployment](DEPLOYMENT.md)
