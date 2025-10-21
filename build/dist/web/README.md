# StarRocks Admin Frontend

This directory contains the built Angular frontend application.

## Serving the Frontend

### Option 1: Using Node.js http-server

```bash
npm install -g http-server
http-server -p 4200
```

### Option 2: Using Python

```bash
# Python 3
python3 -m http.server 4200

# Python 2
python -m SimpleHTTPServer 4200
```

### Option 3: Using Nginx

Create an nginx configuration:

```nginx
server {
    listen 4200;
    server_name localhost;
    
    root /path/to/starrocks-admin/build/dist/web;
    index index.html;
    
    location / {
        try_files $uri $uri/ /index.html;
    }
    
    # Proxy API requests to backend
    location /api/ {
        proxy_pass http://10.119.43.216:8081/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## Development

For development mode, use the npm dev server from the frontend directory:

```bash
cd frontend
npm start
```
