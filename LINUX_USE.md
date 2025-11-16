# Linux Setup (Headless Mode)

Minimal steps to run the crawler on Linux.

---

## 1. Create service user

```bash
sudo useradd -r -m -d /home/crawler -s /usr/sbin/nologin crawler
```

---

## 2. Remove snap Chromium

```bash
sudo snap remove chromium || true
```

---

## 3. Install Google Chrome

```bash
wget https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb
sudo apt install ./google-chrome-stable_current_amd64.deb
```

---

## 4. Deploy project

```bash
sudo mkdir -p /var/www/crawler
sudo cp -r <repo-path>/* /var/www/crawler/
sudo chown -R crawler:crawler /var/www/crawler
```

---

## 5. Create systemd service

Create file:

```
/etc/systemd/system/crawler.service
```

Add:

```ini
[Unit]
Description=Crawler Service
After=network.target

[Service]
User=crawler
Group=crawler
WorkingDirectory=/var/www/crawler
Environment=HOME=/home/crawler
ExecStart=/var/www/crawler/rust-crawler
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable:

```bash
sudo systemctl daemon-reload
sudo systemctl enable crawler.service
sudo systemctl restart crawler.service
```

---

## 6. Verify Chrome

```bash
sudo -u crawler HOME=/home/crawler   /usr/bin/google-chrome   --headless --disable-gpu   --remote-debugging-port=9222   --user-data-dir=/home/crawler/chrome-data   https://example.com
```

Expected output:

```
DevTools listening on ws://127.0.0.1:9222/...
```