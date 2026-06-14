# Hardonia Compliance Agent — Deployment Guide

## Prerequisites

1. **PostgreSQL 15+** — Primary database
2. **Redis 7+** — Caching and job queues
3. **Docker** — Container deployment
4. **Stripe Account** — For billing
5. **OpenAI API Key** — For LLM analysis

## Quick Start (Docker Compose)

```bash
# Clone and configure
git clone https://github.com/Hardonian/hardonia-compliance-agent.git
cd hardonia-compliance-agent
cp .env.example .env

# Edit .env with your configuration
# Required:
#   DATABASE_URL=postgres://user:pass@localhost:5432/compliance_agent
#   REDIS_URL=redis://localhost:6379
#   OPENAI_API_KEY=sk-...
#   STRIPE_SECRET_KEY=sk_live_...
#   STRIPE_WEBHOOK_SECRET=whsec_...

# Start services
docker compose up -d

# Run migrations
docker compose exec api sqlx migrate run

# Verify health
curl http://localhost:8080/health
```

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | ✅ |
| `REDIS_URL` | Redis connection string | ✅ |
| `OPENAI_API_KEY` | OpenAI API key for LLM analysis | ✅ |
| `STRIPE_SECRET_KEY` | Stripe secret key for billing | ✅ |
| `STRIPE_WEBHOOK_SECRET` | Stripe webhook signing secret | ✅ |
| `API_PORT` | HTTP server port (default: 8080) | ❌ |
| `LOG_LEVEL` | Logging level (default: info) | ❌ |
| `ENVIRONMENT` | Environment name (default: production) | ❌ |

## Database Setup

```bash
# Create database
createdb compliance_agent

# Run migrations
sqlx migrate run

# Seed default regulatory sources (already in migration)
# SEC EDGAR, FINRA, CFPB, OFAC, BSA/AML, GDPR, FCA
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| POST | `/api/v1/agent/run` | Trigger agent check |
| GET | `/api/v1/agent/runs` | List agent runs |
| GET | `/api/v1/agent/runs/{id}` | Get agent run details |
| GET | `/api/v1/sources` | List regulatory sources |
| POST | `/api/v1/sources` | Create regulatory source |
| GET | `/api/v1/changes` | List regulatory changes |
| GET | `/api/v1/tasks` | List compliance tasks |
| GET | `/api/v1/tasks/{id}` | Get task details |
| POST | `/api/v1/tasks/{id}/status` | Update task status |
| GET | `/api/v1/dashboard/summary` | Dashboard summary |
| GET | `/api/v1/dashboard/compliance-score` | Compliance score |

## Stripe Configuration

1. **Create Products:**
   - Starter: $499/mo (100 tasks, 5 users)
   - Growth: $1,499/mo (500 tasks, 20 users)
   - Enterprise: $4,999/mo (unlimited)

2. **Create Prices:**
   ```bash
   # Use the setup script
   python3 scripts/setup_stripe_prices.py
   ```

3. **Configure Webhook:**
   - URL: `https://api.compliance.hardonia.com/api/v1/stripe/webhook`
   - Events: `checkout.session.completed`, `customer.subscription.*`, `invoice.*`

## Production Deployment

### Fly.io
```bash
fly launch
fly secrets set DATABASE_URL=... REDIS_URL=... OPENAI_API_KEY=...
fly deploy
```

### Railway
```bash
railway up
railway variables set DATABASE_URL=... REDIS_URL=...
```

### AWS ECS
```bash
# Build and push to ECR
docker build -t compliance-agent .
docker tag compliance-agent:latest <account>.dkr.ecr.<region>.amazonaws.com/compliance-agent:latest
docker push <account>.dkr.ecr.<region>.amazonaws.com/compliance-agent:latest

# Update ECS service
aws ecs update-service --cluster compliance --service api --force-new-deployment
```

## Monitoring

- **Metrics:** Prometheus at `/metrics`
- **Traces:** OpenTelemetry to Jaeger/Tempo
- **Logs:** JSON structured logs to stdout
- **Health:** `/health` endpoint for load balancer

## Security Checklist

- [ ] HTTPS enforced (TLS termination at load balancer)
- [ ] API key authentication on all endpoints
- [ ] Rate limiting enabled (100 req/min per tenant)
- [ ] CORS locked to allowed origins
- [ ] SQL injection prevention (sqlx parameterized queries)
- [ ] Secrets in environment variables (not code)
- [ ] Audit logging for all mutations
- [ ] Stripe webhook signature verification