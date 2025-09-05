# 🚀 Solució ràpida per l'error OAuth

## El problema
L'aplicació està en mode **"Prueba"** i només accepta usuaris de la llista de test.

## Solució immediata

### 1. A Google Cloud Console:
1. Fer clic a **"+ Add users"**
2. Afegir aquests emails:
   - crashbit@gmail.com (o el teu email principal)
   - xavier.figuls@gmail.com (si és diferent)
   - Qualsevol altre email que vulguis utilitzar per testing

### 2. Verificar els emails actuals:
Els 2 usuaris que tens ara són:
- crashbit@gmail.com
- xavier.figuls@gmail.com

Si NO hi són, afegeix-los!

### 3. Reiniciar el servidor:
```bash
# Aturar el servidor actual
pkill -f pvpccheap_backend

# Tornar a executar
cargo run
```

### 4. Provar el login:
1. Obrir `test.html` al navegador
2. Clic a "Provar OAuth"
3. Fer login amb un dels emails de la llista de test

## ⚠️ Important
- L'email que utilitzis per fer login **HA d'estar a la llista de "Usuarios de prueba"**
- Si utilitzes un email diferent, rebràs l'error 400
- NO cal publicar l'aplicació per development

## Verificació
Per veure quins emails tens afegits:
1. A Google Cloud Console > OAuth consent screen
2. Secció "Usuarios de prueba"
3. Hauries de veure la llista d'emails autoritzats

## Si encara no funciona
1. Verificar que el redirect URI coincideix exactament:
   - Al .env: `http://localhost:8080/api/auth/google/callback`
   - A Google Console: Mateix URI als "Authorized redirect URIs"

2. Esperar 5 minuts (Google pot trigar a aplicar canvis)

3. Provar en mode incògnit del navegador
