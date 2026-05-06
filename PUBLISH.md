# Self-Evolving Brain: Production Deployment

Everything is ready for your Hostinger VPS. Since I cannot push code directly, follow these steps to go live at **brain.kluth.cloud**.

## 1. Prepare VPS Infrastructure
Log into your VPS and ensure the `ai-network` exists (it should, based on your current setup):
```bash
docker network create ai-network || true
```

## 2. Docker Compose Configuration
The system is now configured to use **internal Docker networking** via the `ai-network`. This allows containers to communicate using hostnames like `ollama` and `weaviate` without going through the public internet.

```yaml
services:
  brain-ui:
    image: nginx:alpine
    restart: always
    labels:
      - traefik.enable=true
      - traefik.http.routers.brain-ui.rule=Host(`brain.kluth.cloud`)
      - traefik.http.routers.brain-ui.entrypoints=websecure
      - traefik.http.routers.brain-ui.tls.certresolver=letsencrypt
      - traefik.docker.network=ai-network
    volumes:
      - ./brain-ui/dist:/usr/share/nginx/html
    networks:
      - ai-network

  brain-api:
    build: 
      context: .
      dockerfile: brain-api/Dockerfile
    restart: always
    environment:
      - WEAVIATE_URL=http://weaviate:8080
    labels:
      - traefik.enable=true
      - traefik.http.routers.brain-api.rule=Host(`api.brain.kluth.cloud`)
      - traefik.http.routers.brain-api.entrypoints=websecure
      - traefik.http.routers.brain-api.tls.certresolver=letsencrypt
      - traefik.docker.network=ai-network
    networks:
      - ai-network

  brain-ingestion:
    build:
      context: .
      dockerfile: brain-ingestion/Dockerfile
    restart: always
    environment:
      - WEAVIATE_URL=http://weaviate:8080
      - REDIS_URL=redis://redis:6379/
      - TRANSLATE_URL=http://translate:5000/translate
      - OLLAMA_URL=http://ollama:11434/api/generate
      - OLLAMA_MODEL=llama3
    networks:
      - ai-network

  redis:
    image: redis:7-alpine
    restart: always
    networks:
      - ai-network

  translate:
    image: libretranslate/libretranslate:latest
    restart: always
    environment:
      - LT_HOST=0.0.0.0
      - LT_PORT=5000
    command: ["--load-only", "en"]
    networks:
      - ai-network

networks:
  ai-network:
    external: true
```

## 3. Final Deployment
1. **Push your code** to a private GitHub repository.
2. Use the **"Create New Project from URL"** feature in Hostinger's VPS panel.
3. Select your repository.
4. Hostinger will build the images and Traefik will automatically provision the SSL certificates.

---
**Status Update:**
* **Internal Networking:** Successfully migrated from external URLs (kluth.cloud) to internal hostnames (ollama, weaviate, redis).
* **Ollama Integration:** Recreated Ollama as a project named `ollama-internal` on the VPS, reachable via `ai-network`.
* **Rust Code Defaults:** Updated `brain-api` and `brain-ingestion` to use internal hostnames as fallback defaults.
