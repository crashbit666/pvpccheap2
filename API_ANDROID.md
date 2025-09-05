# 📱 API Endpoints per l'App Android

## Flux de l'aplicació:
1. **Login**: L'app fa OAuth amb Google (utilitzant el backend)
2. **Sincronització**: L'app llegeix dispositius de Google Home i els envia al backend
3. **Regles**: L'usuari configura regles des de l'app
4. **Comandes**: El backend calcula i envia comandes a l'app per executar

## 🔐 Autenticació
Tots els endpoints (excepte auth) requereixen header:
```
Authorization: Bearer <JWT_TOKEN>
```

## 📡 Endpoints necessaris:

### 1. **Autenticació Mobile**
```
POST /api/mobile/register
{
  "device_token": "fcm_token_xyz",
  "platform": "android",
  "app_version": "1.0.0"
}
```

### 2. **Sincronització de Dispositius**
```
POST /api/mobile/sync
{
  "structures": [
    {
      "google_id": "structure_123",
      "name": "Casa",
      "timezone": "Europe/Madrid"
    }
  ],
  "devices": [
    {
      "google_id": "device_abc",
      "structure_id": "structure_123",
      "name": "Endoll Sala",
      "type": "outlet",
      "traits": ["on_off"],
      "room": "Sala d'estar"
    }
  ]
}
```

### 3. **Obtenir Dispositius amb Regles**
```
GET /api/devices
Response:
{
  "devices": [
    {
      "id": "uuid",
      "name": "Endoll Sala",
      "type": "outlet",
      "current_state": "off",
      "rules": [
        {
          "id": "rule_uuid",
          "type": "cheapest_hours",
          "params": {
            "min_hours": 6,
            "time_window": null
          }
        }
      ]
    }
  ]
}
```

### 4. **Crear/Actualitzar Regla**
```
POST /api/rules
{
  "device_id": "device_uuid",
  "type": "cheapest_hours",
  "params": {
    "min_hours": 6,
    "time_window": {
      "start": "06:00",
      "end": "09:00"
    }
  }
}
```

### 5. **Obtenir Comandes Pendents**
```
GET /api/mobile/commands/pending
Response:
{
  "commands": [
    {
      "id": "cmd_uuid",
      "device_id": "device_uuid",
      "google_device_id": "device_abc",
      "action": "turn_on",
      "scheduled_for": "2025-09-05T15:00:00Z"
    }
  ]
}
```

### 6. **Confirmar Execució de Comanda**
```
POST /api/mobile/commands/{id}/confirm
{
  "status": "executed",
  "executed_at": "2025-09-05T15:00:05Z",
  "error": null
}
```

### 7. **Obtenir Horari Optimitzat**
```
GET /api/schedules?date=2025-09-05
Response:
{
  "schedules": [
    {
      "device_id": "device_uuid",
      "device_name": "Endoll Sala",
      "hours": [
        {
          "hour": 3,
          "price": 0.08,
          "action": "on"
        },
        {
          "hour": 4,
          "price": 0.07,
          "action": "on"
        }
      ]
    }
  ]
}
```

### 8. **Heartbeat (Keep-alive)**
```
POST /api/mobile/heartbeat
{
  "device_token": "fcm_token_xyz"
}
```

## 🔄 WebSocket per temps real
```
WS /ws
Messages:
- command: { type: "execute", device_id: "...", action: "turn_on" }
- status_update: { device_id: "...", state: "on" }
```

## 📊 Preus PVPC
```
GET /api/prices?date=2025-09-05
Response:
{
  "date": "2025-09-05",
  "prices": [
    { "hour": 0, "price": 0.12 },
    { "hour": 1, "price": 0.10 },
    ...
  ]
}
```
