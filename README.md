# PVPCCheap Backend

Backend per a l'aplicació PVPCCheap - control intel·ligent de dispositius domòtics amb optimització de preus elèctrics.

## Arquitectura

- **Framework web**: Actix-web
- **Base de dades**: PostgreSQL amb Diesel ORM
- **Autenticació**: OAuth 2.0 amb Google + JWT
- **Notificacions**: Firebase Cloud Messaging (FCM)
- **WebSockets**: Per actualitzacions en temps real

## Prerequisits

- Rust 1.75+
- PostgreSQL 14+
- Compte de Google Cloud amb OAuth configurat
- Compte de Firebase per FCM (opcional)

## Instal·lació

1. **Clonar el repositori i instal·lar dependències:**
```bash
cd backend
cargo build
```

2. **Configurar variables d'entorn:**
```bash
cp env.example .env
# Editar .env amb les teves credencials
```

3. **Configurar la base de dades:**
```bash
# Crear base de dades
createdb pvpccheap

# Instal·lar Diesel CLI si no el tens
cargo install diesel_cli --no-default-features --features postgres

# Executar migracions
diesel migration run
```

4. **Executar el servidor:**
```bash
cargo run
# O en mode desenvolupament amb auto-reload:
cargo install cargo-watch
cargo watch -x run
```

## Estructura del projecte

```
backend/
├── src/
│   ├── main.rs              # Entry point i configuració
│   ├── schema.rs            # Esquema de base de dades (generat per Diesel)
│   ├── models/              # Models de dades
│   │   ├── user.rs
│   │   ├── device.rs
│   │   ├── rule.rs
│   │   ├── schedule.rs
│   │   └── command.rs
│   ├── handlers/            # Controladors HTTP
│   │   ├── auth.rs          # OAuth i autenticació
│   │   ├── mobile.rs        # Sincronització amb app mòbil
│   │   ├── device.rs        # Gestió de dispositius
│   │   ├── rule.rs          # Gestió de regles
│   │   ├── schedule.rs      # Horaris optimitzats
│   │   └── websocket.rs     # WebSocket per temps real
│   ├── middleware/          # Middleware
│   │   └── auth.rs          # Middleware d'autenticació
│   ├── services/            # Serveis de negoci
│   └── utils/               # Utilitats
├── migrations/              # Migracions de DB
└── Cargo.toml              # Dependències
```

## API Endpoints

### Autenticació
- `GET /api/auth/google` - Iniciar login amb Google
- `GET /api/auth/google/callback` - Callback OAuth
- `POST /api/auth/logout` - Tancar sessió
- `GET /api/auth/me` - Obtenir usuari actual

### Sincronització mòbil
- `POST /api/mobile/sync` - Sincronitzar dispositius des de l'app
- `POST /api/mobile/heartbeat` - Heartbeat i obtenir comandes pendents
- `POST /api/mobile/command_result` - Reportar resultat de comanda

### Dispositius
- `GET /api/devices` - Llistar dispositius
- `GET /api/devices/:id` - Obtenir dispositiu
- `GET /api/devices/:id/state` - Obtenir estat actual
- `POST /api/devices/:id/command` - Enviar comanda

### Regles
- `GET /api/rules` - Llistar regles
- `POST /api/rules` - Crear regla
- `GET /api/rules/:id` - Obtenir regla
- `PUT /api/rules/:id` - Actualitzar regla
- `DELETE /api/rules/:id` - Eliminar regla
- `POST /api/rules/preview` - Previsualitzar horari

### Horaris
- `GET /api/schedules` - Llistar horaris
- `GET /api/schedules/today` - Horaris d'avui
- `POST /api/schedules/rebuild` - Recalcular horaris

### WebSocket
- `WS /api/ws` - Connexió WebSocket per actualitzacions en temps real

## Flux de funcionament

1. **Registre/Login:**
   - L'usuari fa login amb Google OAuth
   - Es crea/actualitza el perfil d'usuari

2. **Sincronització de dispositius:**
   - L'app Android llegeix dispositius de Google Home APIs
   - Envia la llista al backend via `/api/mobile/sync`
   - El backend guarda els dispositius i capacitats

3. **Creació de regles:**
   - L'usuari crea regles des de l'app (ex: "6 hores més barates")
   - Les regles es guarden al backend

4. **Optimització d'horaris:**
   - Cada dia es descarreguen els preus elèctrics
   - El backend calcula els horaris òptims segons les regles
   - Es generen els schedules per cada dispositiu

5. **Execució de comandes:**
   - El backend encua comandes segons els horaris
   - L'app fa heartbeat i rep comandes pendents
   - L'app executa via Google Home APIs i reporta resultats

## Configuració de Google OAuth

1. Anar a [Google Cloud Console](https://console.cloud.google.com/)
2. Crear un nou projecte o seleccionar-ne un existent
3. Habilitar Google+ API
4. Crear credencials OAuth 2.0
5. Afegir URL de callback: `http://localhost:8080/api/auth/google/callback`
6. Copiar Client ID i Client Secret al `.env`

## Configuració de Firebase (FCM)

1. Crear projecte a [Firebase Console](https://console.firebase.google.com/)
2. Afegir app Android
3. Descarregar `google-services.json` per l'app
4. Obtenir Server Key des de Project Settings > Cloud Messaging
5. Afegir Server Key al `.env`

## Desenvolupament

### Executar tests
```bash
cargo test
```

### Generar schema després de canvis a les migracions
```bash
diesel migration run
diesel print-schema > src/schema.rs
```

### Logs
Configurar el nivell de logs amb la variable `RUST_LOG`:
```bash
RUST_LOG=debug cargo run
```

## TODO

- [ ] Implementar motor d'optimització de regles
- [ ] Integració amb API de preus elèctrics (PVPC)
- [ ] Sistema de notificacions FCM
- [ ] WebSocket per actualitzacions en temps real
- [ ] Tests unitaris i d'integració
- [ ] Docker i docker-compose
- [ ] CI/CD pipeline
