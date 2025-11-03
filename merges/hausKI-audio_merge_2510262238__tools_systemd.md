### 📄 tools/systemd/hauski-backend.service

**Größe:** 407 B | **md5:** `73c0533d49ac86ed402c50f0f7831ce1`

```plaintext
[Unit]
Description=Hauski Audio Backend (axum)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
# Passe das Repo/Wurzelverzeichnis an (WorkingDirectory sollte die Skripte finden).
WorkingDirectory=%h/repos/hauski-audio
EnvironmentFile=%h/.config/hauski-audio/backend.env
ExecStart=%h/.local/bin/hauski-backend
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
```

