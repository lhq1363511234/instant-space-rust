# Instant Space WebSocket nginx 配置

Phase 6 实时房间由 Axum 暴露在 `/ws/spaces/:space_id`。生产站点挂载于
`/inspace`，因此 nginx 必须在通用 `/inspace/` 规则之前保留下面的升级代理：

```nginx
location ^~ /inspace/ws/ {
    rewrite ^/inspace(/ws/.*)$ $1 break;
    proxy_pass http://127.0.0.1:3001;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_read_timeout 3600s;
    proxy_send_timeout 3600s;
    proxy_buffering off;
}
```

修改后执行：

```bash
nginx -t && nginx -s reload
```

浏览器使用稳定公网路径：

```text
wss://opctoai.com/inspace/ws/spaces/{space_id}
```

服务端会校验浏览器 `Origin` 与 `Host` 同源。私密空间还必须携带通过密码验证后生成的 HttpOnly access cookie。
