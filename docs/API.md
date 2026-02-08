# API Reference

Base URL: `http://localhost:7878/api/v1`

All authenticated endpoints need the `Authorization: Bearer <token>` header. The token is a UUID v4 string you get from the login endpoint. It's not a JWT — it's a session ID stored server-side in redb.

Responses follow this shape:

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

On error:

```json
{
  "success": false,
  "data": null,
  "error": { "code": "INVALID_CREDENTIALS", "message": "Invalid username or password" }
}
```

---

## Authentication

Auth is session-based. The server generates a UUID v4 as the access token and stores a `Session` object in the `sessions` redb table. Passwords are hashed with Argon2id.

### Login

```bash
curl -s -X POST http://localhost:7878/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "changeme"}' | jq .
```

Response:

```json
{
  "success": true,
  "data": {
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "admin",
      "email": "admin@example.com",
      "first_name": "Admin",
      "last_name": "User",
      "role": "admin",
      "department": null,
      "is_active": true,
      "last_login": "2026-02-08T07:00:00Z"
    },
    "tokens": {
      "access_token": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "refresh_token": "rt_98765432-abcd-ef01-2345-678901234567",
      "expires_at": "2026-02-09T07:00:00Z",
      "token_type": "Bearer"
    }
  }
}
```

You can log in with either username or email. The server checks `users_by_username` first, then falls back to `users_by_email`.

Store `access_token` — you'll need it for every other request. Sessions expire after 24 hours.

### Register

```bash
curl -s -X POST http://localhost:7878/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "jdoe",
    "email": "jdoe@example.com",
    "password": "MyP@ssw0rd!",
    "first_name": "Jane",
    "last_name": "Doe"
  }' | jq .
```

The first registered user automatically gets the `admin` role. All subsequent users get `user`.

Returns the same shape as login (user + tokens), so the user is immediately logged in.

### Refresh Token

```bash
curl -s -X POST http://localhost:7878/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "rt_98765432-abcd-ef01-2345-678901234567"}' | jq .
```

### Forgot Password

```bash
curl -s -X POST http://localhost:7878/api/v1/auth/forgot-password \
  -H "Content-Type: application/json" \
  -d '{"email": "jdoe@example.com"}'
```

Triggers a password reset flow (requires SMTP configuration).

### First-Run Setup

```bash
# Check if setup is complete
curl -s http://localhost:7878/api/v1/setup/status | jq .

# Change default admin password during setup
curl -s -X POST http://localhost:7878/api/v1/setup/change-password \
  -H "Content-Type: application/json" \
  -d '{"current_password": "changeme", "new_password": "MyNewSecureP@ss"}'

# Mark setup as complete
curl -s -X POST http://localhost:7878/api/v1/setup/complete \
  -H "Authorization: Bearer $TOKEN"
```

---

## Users

### Get Current User

```bash
TOKEN="a1b2c3d4-e5f6-7890-abcd-ef1234567890"

curl -s http://localhost:7878/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### Get User by ID

```bash
curl -s http://localhost:7878/api/v1/users/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### Change Password

```bash
curl -s -X PATCH http://localhost:7878/api/v1/users/me/password \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"current_password": "OldP@ss", "new_password": "NewP@ss123!"}' | jq .
```

---

## Parking Lots

Lots are the top-level containers. Each lot has a layout (grid of rows and slots) and contains parking slots.

### List All Lots

```bash
curl -s http://localhost:7878/api/v1/lots \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### Get Lot Details

```bash
curl -s http://localhost:7878/api/v1/lots/LOT_ID \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### Get Slots for a Lot

Returns every slot in the lot with current status (free, booked, disabled).

```bash
curl -s http://localhost:7878/api/v1/lots/LOT_ID/slots \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### Layout (Get / Update)

The layout defines the visual grid — rows, columns, slot positions. The admin updates this through the visual lot designer in the UI, but you can also do it via API.

```bash
# Get layout
curl -s http://localhost:7878/api/v1/lots/LOT_ID/layout \
  -H "Authorization: Bearer $TOKEN" | jq .

# Update layout (admin)
curl -s -X PUT http://localhost:7878/api/v1/lots/LOT_ID/layout \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"rows": [...], "slots": [...]}' | jq .
```

When a layout is updated, the server syncs the `parking_slots` table — creating new slots and removing deleted ones.

### Slot QR Code

Generates a QR code image for a specific slot. Used for check-in — user scans the QR at the parking spot.

```bash
curl -s http://localhost:7878/api/v1/lots/LOT_ID/slots/SLOT_ID/qr \
  -H "Authorization: Bearer $TOKEN" -o slot-qr.png
```

---

## Bookings

### List My Bookings

```bash
curl -s http://localhost:7878/api/v1/bookings \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### Create a Booking

```bash
curl -s -X POST http://localhost:7878/api/v1/bookings \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "lot_id": "LOT_ID",
    "slot_id": "SLOT_ID",
    "date": "2026-02-10",
    "vehicle_id": "VEHICLE_ID"
  }' | jq .
```

The server checks:
- Is the slot free on that date?
- Does the user already have a booking on that date?
- Is the slot disabled?

If any check fails, you get a `409 Conflict`.

### Update a Booking

```bash
curl -s -X PATCH http://localhost:7878/api/v1/bookings/BOOKING_ID \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"vehicle_id": "NEW_VEHICLE_ID", "notes": "Running late"}' | jq .
```

### Cancel a Booking

```bash
curl -s -X DELETE http://localhost:7878/api/v1/bookings/BOOKING_ID \
  -H "Authorization: Bearer $TOKEN" | jq .
```

Cancelling checks if the booking belongs to the current user (unless admin). If there's a waitlist entry for this lot+date, the next person gets notified.

### Check In

```bash
curl -s -X POST http://localhost:7878/api/v1/bookings/BOOKING_ID/checkin \
  -H "Authorization: Bearer $TOKEN" | jq .
```

Marks the booking as checked-in. Typically triggered by scanning the slot's QR code.

### Export as iCal

```bash
curl -s http://localhost:7878/api/v1/bookings/ical \
  -H "Authorization: Bearer $TOKEN" -o my-bookings.ics
```

Returns a standard `.ics` file. Subscribe in any calendar app for live updates.

---

## Vehicles

Users can register vehicles with license plates and optional photos.

### List Vehicles

```bash
curl -s http://localhost:7878/api/v1/vehicles \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### Add a Vehicle

```bash
curl -s -X POST http://localhost:7878/api/v1/vehicles \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "license_plate": "M-AB 1234",
    "make": "BMW",
    "model": "320d",
    "color": "black"
  }' | jq .
```

Only `license_plate` is required. The rest is optional.

### Upload Vehicle Photo

```bash
curl -s -X POST http://localhost:7878/api/v1/vehicles/VEHICLE_ID/photo \
  -H "Authorization: Bearer $TOKEN" \
  -F "photo=@car.jpg" | jq .
```

### Delete a Vehicle

```bash
curl -s -X DELETE http://localhost:7878/api/v1/vehicles/VEHICLE_ID \
  -H "Authorization: Bearer $TOKEN" | jq .
```

---

## Homeoffice

Users set their work-from-home schedule. On WFH days, their parking slot (if permanently assigned) is released for others.

### Get Settings

```bash
curl -s http://localhost:7878/api/v1/homeoffice \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### Update Settings

```bash
curl -s -X PUT http://localhost:7878/api/v1/homeoffice \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}' | jq .
```

### Set Recurring Pattern

```bash
curl -s -X PUT http://localhost:7878/api/v1/homeoffice/pattern \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"days": ["monday", "wednesday", "friday"]}' | jq .
```

Days are stored as weekday numbers (0=Sunday, 6=Saturday) internally.

### Add a One-Off WFH Day

```bash
curl -s -X POST http://localhost:7878/api/v1/homeoffice/days \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"date": "2026-02-14"}' | jq .
```

### Remove a WFH Day

```bash
curl -s -X DELETE http://localhost:7878/api/v1/homeoffice/days/DAY_ID \
  -H "Authorization: Bearer $TOKEN" | jq .
```

---

## Waitlist

When all slots in a lot are booked for a given date, users can join a waitlist.

### View Waitlist

```bash
curl -s http://localhost:7878/api/v1/lots/LOT_ID/waitlist \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### Join Waitlist

```bash
curl -s -X POST http://localhost:7878/api/v1/lots/LOT_ID/waitlist \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"date": "2026-02-10"}' | jq .
```

When someone cancels a booking, the next person on the waitlist gets a push notification (if subscribed).

---

## Push Notifications

### Subscribe

```bash
curl -s -X POST http://localhost:7878/api/v1/push/subscribe \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "endpoint": "https://fcm.googleapis.com/fcm/send/...",
    "p256dh": "BNcRd...",
    "auth": "tBHI..."
  }' | jq .
```

Standard Web Push API subscription. The frontend handles this via the service worker.

---

## Admin Endpoints

All require `admin` role.

### List All Users

```bash
curl -s http://localhost:7878/api/v1/admin/users \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### Create a User

```bash
curl -s -X POST http://localhost:7878/api/v1/admin/users \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "newuser",
    "email": "new@example.com",
    "password": "TempP@ss123",
    "role": "user"
  }' | jq .
```

### Update a User

```bash
curl -s -X PATCH http://localhost:7878/api/v1/admin/users/USER_ID \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"role": "admin", "is_active": true, "department": "Engineering"}' | jq .
```

### List All Bookings

```bash
curl -s http://localhost:7878/api/v1/admin/bookings \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### Statistics

```bash
curl -s http://localhost:7878/api/v1/admin/stats \
  -H "Authorization: Bearer $TOKEN" | jq .
```

Returns: total users, total bookings, active bookings, occupancy rates.

### Reports

```bash
curl -s http://localhost:7878/api/v1/admin/reports \
  -H "Authorization: Bearer $TOKEN" | jq .

# With date filter
curl -s "http://localhost:7878/api/v1/admin/reports?from=2026-01-01&to=2026-01-31" \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### Update a Slot (Admin)

```bash
curl -s -X PATCH http://localhost:7878/api/v1/admin/slots/SLOT_ID \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"status": "disabled"}' | jq .
```

### Update Slot Properties (Admin)

```bash
curl -s -X PUT http://localhost:7878/api/v1/lots/LOT_ID/slots/SLOT_ID \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"label": "A-15", "type": "ev_charging"}' | jq .
```

### Delete a Lot (Admin)

```bash
curl -s -X DELETE http://localhost:7878/api/v1/admin/lots/LOT_ID \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### Branding

```bash
# Get (admin view, includes all config)
curl -s http://localhost:7878/api/v1/admin/branding \
  -H "Authorization: Bearer $TOKEN" | jq .

# Update
curl -s -X PUT http://localhost:7878/api/v1/admin/branding \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"organization_name": "My Organization"}' | jq .

# Upload logo
curl -s -X POST http://localhost:7878/api/v1/admin/branding/logo \
  -H "Authorization: Bearer $TOKEN" \
  -F "logo=@logo.png" | jq .
```

### Reset Database

Wipes everything and starts fresh. Dangerous. Requires confirmation.

```bash
curl -s -X POST http://localhost:7878/api/v1/admin/reset \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"confirm": "RESET"}' | jq .
```

---

## GDPR Endpoints

### Export Your Data

Downloads everything the server knows about you: profile, bookings, vehicles, homeoffice settings.

```bash
curl -s http://localhost:7878/api/v1/users/me/export \
  -H "Authorization: Bearer $TOKEN" -o my-data.json
```

### Delete Your Account

Permanently removes your account and all associated data.

```bash
curl -s -X DELETE http://localhost:7878/api/v1/users/me/delete \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"confirm": "RESET"}' | jq .
```

---

## Public Endpoints (No Auth)

| Method | Endpoint | Returns |
|--------|----------|---------|
| `GET` | `/health` | `"ok"` — general health |
| `GET` | `/health/live` | `200` — Kubernetes liveness probe |
| `GET` | `/health/ready` | `200`/`503` — readiness probe (checks DB) |
| `GET` | `/status` | Server version, uptime, user count |
| `POST` | `/handshake` | Protocol negotiation for the desktop client |
| `GET` | `/metrics` | Prometheus-format metrics |
| `GET` | `/api/v1/privacy` | Privacy policy text |
| `GET` | `/api/v1/about` | About/version info |
| `GET` | `/api/v1/help` | FAQ / help content |
| `GET` | `/api/v1/branding` | Organization name + logo URL |
| `GET` | `/api/v1/branding/logo` | Logo image file |
| `GET` | `/api/v1/setup/status` | Whether first-run setup is complete |

---

## Error Codes

| HTTP | Code | Meaning |
|------|------|---------|
| `400` | `BAD_REQUEST` | Missing/invalid fields |
| `401` | `UNAUTHORIZED` / `INVALID_CREDENTIALS` | No token, expired token, or wrong password |
| `403` | `FORBIDDEN` / `ACCOUNT_DISABLED` | Wrong role or deactivated account |
| `404` | `NOT_FOUND` | Resource doesn't exist |
| `409` | `CONFLICT` | Slot already booked, duplicate username/email |
| `429` | `RATE_LIMITED` | Too many requests |
| `500` | `SERVER_ERROR` | Something broke. Check logs. |

---

Back to [README](../README.md) · Previous: [Configuration](CONFIGURATION.md) · Next: [Deployment](DEPLOYMENT.md)
