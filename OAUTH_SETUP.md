# 🔐 Configuració OAuth de Google

## ⚠️ Problema actual
L'error 400 que veus indica que l'aplicació OAuth no està ben configurada a Google Cloud Console.

## ✅ Passos per solucionar-ho:

### 1. Configurar OAuth Consent Screen
1. Anar a [Google Cloud Console](https://console.cloud.google.com/)
2. Seleccionar el teu projecte
3. Anar a **APIs & Services > OAuth consent screen**
4. Configurar:
   - **User Type**: External (per permetre qualsevol compte de Google)
   - **App name**: PVPCCheap
   - **User support email**: el teu email
   - **Developer contact**: el teu email
   - **Scopes**: Afegir aquests scopes:
     - `email`
     - `profile`
     - `openid`

### 2. Configurar OAuth 2.0 Client
1. Anar a **APIs & Services > Credentials**
2. Buscar el client amb ID `36945434996-ggk9a18caipjmm073c4j73igsnmhp98e.apps.googleusercontent.com`
3. Fer clic per editar-lo
4. **IMPORTANT**: Afegir aquests Authorized redirect URIs:
   ```
   http://localhost:8080/api/auth/google/callback
   http://127.0.0.1:8080/api/auth/google/callback
   ```

### 3. Verificar APIs habilitades
1. Anar a **APIs & Services > Enabled APIs**
2. Assegurar que aquestes APIs estan habilitades:
   - Google+ API (si encara existeix)
   - Google Identity Toolkit API
   - People API

### 4. Mode de prova vs Producció
Si l'app està en **mode de prova (Testing)**:
- Només els usuaris afegits com a "Test users" poden fer login
- Afegir el teu email a la llista de test users

Per passar a **producció**:
- Has de verificar l'aplicació amb Google (pot trigar dies/setmanes)
- Per development, és millor mantenir-la en mode de prova

## 🧪 Com provar-ho

1. Reiniciar el servidor:
```bash
# Aturar el servidor actual (si està funcionant)
pkill -f pvpccheap_backend

# Tornar a executar
cargo run
```

2. Obrir `test.html` al navegador

3. Fer clic a "Provar OAuth"

## 🔍 Debug addicional

Si encara no funciona, comprova:

1. **Que el servidor estigui accessible**:
```bash
curl http://localhost:8080/health
```

2. **Que les credencials estiguin carregades**:
```bash
grep GOOGLE .env
```

3. **Logs del servidor** per veure l'error exacte

## 📝 Notes importants

- El redirect URI ha de coincidir EXACTAMENT (inclòs http vs https, port, path)
- Si canvies el redirect URI al .env, també l'has de canviar a Google Cloud Console
- Google pot trigar uns minuts a aplicar els canvis

## 🚀 Credencials actuals

- **Client ID**: `36945434996-ggk9a18caipjmm073c4j73igsnmhp98e.apps.googleusercontent.com`
- **Redirect URI**: `http://localhost:8080/api/auth/google/callback`
