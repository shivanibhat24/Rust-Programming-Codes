# Production Deployment Guide

## Prerequisites

### System Requirements
- **OS**: Linux (Ubuntu 22.04+ recommended) or Docker
- **RAM**: Minimum 512MB, Recommended 2GB+
- **CPU**: 1+ cores
- **Storage**: 10GB+ for logs and database
- **Network**: Static IP, firewall configured

### Software Requirements
- Rust 1.75+ (if building from source)
- PostgreSQL 14+ or SQLite 3.35+ (production should use PostgreSQL)
- Nginx or Caddy (reverse proxy)
- systemd (for service management)
- certbot (for TLS certificates)

## Production Configuration

### Environment Variables

Create `/etc/secure-messenger/.env`:
```bash
# Database (use PostgreSQL in production)
DATABASE_URL=postgresql://messenger:secure_pass@localhost/messenger_db

# Server
BIND_ADDRESS=127.0.0.1:3000  # Behind reverse proxy
RUST_LOG=secure_messenger=info,tower_http=warn

# Security
SESSION_SECRET=$(openssl rand -hex 32)
MAX_MESSAGE_SIZE=10485760  # 10MB
ENABLE_RATE_LIMITING=true
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW_SECONDS=60

# TLS (if terminating TLS in app)
TLS_CERT_PATH=/etc/letsencrypt/live/yourdomain.com/fullchain.pem
TLS_KEY_PATH=/etc/letsencrypt/live/yourdomain.com/privkey.pem

# Monitoring
ENABLE_METRICS=true
METRICS_PORT=9090
```

### Database Setup (PostgreSQL)

```bash
# Install PostgreSQL
sudo apt-get update
sudo apt-get install postgresql postgresql-contrib

# Create database and user
sudo -u postgres psql <<EOF
CREATE DATABASE messenger_db;
CREATE USER messenger WITH ENCRYPTED PASSWORD 'secure_password_here';
GRANT ALL PRIVILEGES ON DATABASE messenger_db TO messenger;
\q
EOF

# Configure PostgreSQL
sudo nano /etc/postgresql/14/main/postgresql.conf
# Set:
# max_connections = 100
# shared_buffers = 256MB
# effective_cache_size = 1GB
# work_mem = 4MB

# Enable SSL
# ssl = on
# ssl_cert_file = '/path/to/server.crt'
# ssl_key_file = '/path/to/server.key'

sudo systemctl restart postgresql
```

### Reverse Proxy (Nginx)

Create `/etc/nginx/sites-available/secure-messenger`:
```nginx
upstream secure_messenger {
    server 127.0.0.1:3000 fail_timeout=0;
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    listen [::]:80;
    server_name messenger.yourdomain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name messenger.yourdomain.com;

    # TLS Configuration
    ssl_certificate /etc/letsencrypt/live/messenger.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/messenger.yourdomain.com/privkey.pem;
    ssl_protocols TLSv1.3 TLSv1.2;
    ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384';
    ssl_prefer_server_ciphers on;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;
    ssl_stapling on;
    ssl_stapling_verify on;

    # Security Headers
    add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "no-referrer" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self'; object-src 'none';" always;

    # Rate Limiting
    limit_req_zone $binary_remote_addr zone=messenger_limit:10m rate=10r/s;
    limit_req zone=messenger_limit burst=20 nodelay;

    # Logging
    access_log /var/log/nginx/messenger-access.log;
    error_log /var/log/nginx/messenger-error.log;

    # WebSocket support (if needed)
    location /ws {
        proxy_pass http://secure_messenger;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # API endpoints
    location / {
        proxy_pass http://secure_messenger;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
        
        # Buffer size
        client_max_body_size 10M;
    }
}
```

Enable site:
```bash
sudo ln -s /etc/nginx/sites-available/secure-messenger /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### TLS Certificate (Let's Encrypt)

```bash
# Install certbot
sudo apt-get install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d messenger.yourdomain.com

# Auto-renewal (add to crontab)
sudo crontab -e
# Add: 0 3 * * * certbot renew --quiet
```

## Building for Production

### Option 1: Docker (Recommended)

```bash
# Build image
docker build -t secure-messenger:latest .

# Run with docker-compose
docker-compose -f docker-compose.prod.yml up -d
```

Create `docker-compose.prod.yml`:
```yaml
version: '3.8'

services:
  db:
    image: postgres:14-alpine
    environment:
      POSTGRES_DB: messenger_db
      POSTGRES_USER: messenger
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    volumes:
      - postgres-data:/var/lib/postgresql/data
    secrets:
      - db_password
    restart: always

  app:
    image: secure-messenger:latest
    depends_on:
      - db
    environment:
      DATABASE_URL: postgresql://messenger:password@db/messenger_db
      BIND_ADDRESS: 0.0.0.0:3000
      RUST_LOG: secure_messenger=info
    volumes:
      - app-logs:/var/log/secure-messenger
    restart: always
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '0.5'
          memory: 512M

  nginx:
    image: nginx:alpine
    depends_on:
      - app
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - /etc/letsencrypt:/etc/letsencrypt:ro
      - nginx-logs:/var/log/nginx
    restart: always

volumes:
  postgres-data:
  app-logs:
  nginx-logs:

secrets:
  db_password:
    file: ./secrets/db_password.txt
```

### Option 2: Systemd Service

Build the binary:
```bash
# On build server
cargo build --release --target x86_64-unknown-linux-gnu

# Copy to production
scp target/release/secure-messenger user@server:/usr/local/bin/
```

Create systemd service `/etc/systemd/system/secure-messenger.service`:
```ini
[Unit]
Description=Secure Messenger Service
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=messenger
Group=messenger
WorkingDirectory=/opt/secure-messenger
EnvironmentFile=/etc/secure-messenger/.env
ExecStart=/usr/local/bin/secure-messenger
Restart=always
RestartSec=10
StandardOutput=append:/var/log/secure-messenger/app.log
StandardError=append:/var/log/secure-messenger/error.log

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/secure-messenger /var/lib/secure-messenger

# Resource limits
LimitNOFILE=65536
LimitNPROC=512

[Install]
WantedBy=multi-user.target
```

Setup:
```bash
# Create user
sudo useradd -r -s /bin/false messenger

# Create directories
sudo mkdir -p /opt/secure-messenger
sudo mkdir -p /var/log/secure-messenger
sudo mkdir -p /var/lib/secure-messenger
sudo chown messenger:messenger /opt/secure-messenger
sudo chown messenger:messenger /var/log/secure-messenger
sudo chown messenger:messenger /var/lib/secure-messenger

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable secure-messenger
sudo systemctl start secure-messenger

# Check status
sudo systemctl status secure-messenger
```

## Monitoring

### Logging

Configure log rotation `/etc/logrotate.d/secure-messenger`:
```
/var/log/secure-messenger/*.log {
    daily
    rotate 30
    compress
    delaycompress
    notifempty
    create 0640 messenger messenger
    sharedscripts
    postrotate
        systemctl reload secure-messenger > /dev/null 2>&1 || true
    endscript
}
```

### Metrics (Prometheus)

Add Prometheus endpoint to your app, then configure Prometheus:
```yaml
# /etc/prometheus/prometheus.yml
scrape_configs:
  - job_name: 'secure-messenger'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 15s
```

### Alerting

Configure alerts in `/etc/prometheus/alert.rules.yml`:
```yaml
groups:
  - name: secure_messenger
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
      
      - alert: ServiceDown
        expr: up{job="secure-messenger"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Service is down"
```

## Backup Strategy

### Database Backups

```bash
#!/bin/bash
# /opt/scripts/backup-database.sh

BACKUP_DIR="/var/backups/messenger"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="messenger_db_${TIMESTAMP}.sql.gz"

mkdir -p $BACKUP_DIR

# PostgreSQL backup
PGPASSWORD=$DB_PASSWORD pg_dump \
    -h localhost \
    -U messenger \
    -d messenger_db \
    | gzip > "${BACKUP_DIR}/${BACKUP_FILE}"

# Encrypt backup
gpg --encrypt --recipient backup@yourdomain.com \
    "${BACKUP_DIR}/${BACKUP_FILE}"

# Upload to S3 (optional)
aws s3 cp "${BACKUP_DIR}/${BACKUP_FILE}.gpg" \
    s3://your-backup-bucket/messenger/

# Clean old backups (keep 30 days)
find $BACKUP_DIR -name "*.sql.gz*" -mtime +30 -delete

echo "Backup completed: ${BACKUP_FILE}"
```

Add to crontab:
```bash
# Daily at 2 AM
0 2 * * * /opt/scripts/backup-database.sh
```

## Security Hardening

### Firewall (UFW)

```bash
# Default deny
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH
sudo ufw allow 22/tcp

# Allow HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Enable firewall
sudo ufw enable
```

### Fail2ban

```bash
# Install
sudo apt-get install fail2ban

# Configure
sudo nano /etc/fail2ban/jail.local
```

```ini
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 5

[nginx-limit-req]
enabled = true
filter = nginx-limit-req
logpath = /var/log/nginx/error.log
```

### Regular Updates

```bash
# Create update script
cat > /opt/scripts/update-system.sh <<'EOF'
#!/bin/bash
apt-get update
apt-get upgrade -y
apt-get autoremove -y
systemctl restart secure-messenger
EOF

chmod +x /opt/scripts/update-system.sh

# Weekly updates
sudo crontab -e
# Add: 0 3 * * 0 /opt/scripts/update-system.sh
```

## Health Checks

### Monitoring Script

```bash
#!/bin/bash
# /opt/scripts/health-check.sh

ENDPOINT="https://messenger.yourdomain.com/health"
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" $ENDPOINT)

if [ "$RESPONSE" != "200" ]; then
    echo "Health check failed: HTTP $RESPONSE"
    # Send alert
    echo "Service down" | mail -s "Messenger Alert" admin@yourdomain.com
    # Restart service
    systemctl restart secure-messenger
fi
```

Add to crontab:
```bash
*/5 * * * * /opt/scripts/health-check.sh
```

## Performance Tuning

### System Limits

```bash
# /etc/security/limits.conf
messenger soft nofile 65536
messenger hard nofile 65536
messenger soft nproc 4096
messenger hard nproc 4096
```

### Kernel Parameters

```bash
# /etc/sysctl.d/99-messenger.conf
net.core.somaxconn = 4096
net.ipv4.tcp_max_syn_backlog = 4096
net.ipv4.ip_local_port_range = 1024 65535
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 15
```

Apply:
```bash
sudo sysctl -p /etc/sysctl.d/99-messenger.conf
```

## Disaster Recovery

### Recovery Plan

1. **Database Corruption**
   ```bash
   # Restore from latest backup
   gunzip < backup.sql.gz | psql -U messenger messenger_db
   ```

2. **Complete System Failure**
   ```bash
   # Provision new server
   # Restore from backup
   # Update DNS
   ```

3. **Key Compromise**
   ```bash
   # Rotate all keys
   # Notify users
   # Force re-authentication
   ```

## Deployment Checklist

- [ ] Database configured and secured
- [ ] TLS certificates installed
- [ ] Reverse proxy configured
- [ ] Firewall rules set
- [ ] Systemd service created
- [ ] Logging configured
- [ ] Monitoring set up
- [ ] Backups automated
- [ ] Health checks running
- [ ] Security hardening applied
- [ ] Documentation updated
- [ ] Team trained on operations
- [ ] Incident response plan ready
- [ ] Load testing completed
- [ ] Disaster recovery tested

## Troubleshooting

### Service Won't Start
```bash
# Check logs
sudo journalctl -u secure-messenger -n 100 --no-pager

# Check configuration
sudo systemctl cat secure-messenger

# Test binary manually
sudo -u messenger /usr/local/bin/secure-messenger
```

### High Memory Usage
```bash
# Check memory
free -h
ps aux | grep secure-messenger

# Restart service
sudo systemctl restart secure-messenger
```

### Database Connection Issues
```bash
# Test connection
psql -h localhost -U messenger -d messenger_db

# Check PostgreSQL status
sudo systemctl status postgresql

# Check logs
sudo tail -f /var/log/postgresql/postgresql-14-main.log
```
