# Wazuh MCP Server Helm Chart

This Helm chart deploys the Wazuh MCP Server to a Kubernetes cluster.

## Prerequisites

- Kubernetes 1.19+
- Helm 3.0+
- Access to a Wazuh Manager and Wazuh Indexer instance

## Installing the Chart

To install the chart with the release name `my-wazuh-mcp`:

```bash
helm install my-wazuh-mcp ./helm/mcp-server-wazuh
```

## Configuration

The following table lists the configurable parameters and their default values.

### Image Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `image.repository` | Container image repository | `ghcr.io/pvrmza/mcp-server-wazuh` |
| `image.tag` | Container image tag | `""` (uses appVersion) |
| `image.pullPolicy` | Image pull policy | `IfNotPresent` |

### Secret Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `existingSecret` | Name of existing Secret to use instead of creating one | `""` (creates new secret) |

**Note**: If `existingSecret` is provided, the Secret must contain these keys:
- `WAZUH_API_USERNAME`
- `WAZUH_API_PASSWORD`
- `WAZUH_INDEXER_USERNAME`
- `WAZUH_INDEXER_PASSWORD`

### Wazuh Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `wazuh.api.host` | Wazuh Manager API hostname | `localhost` |
| `wazuh.api.port` | Wazuh Manager API port | `55000` |
| `wazuh.api.username` | Wazuh Manager API username (only if existingSecret not set) | `wazuh` |
| `wazuh.api.password` | Wazuh Manager API password (only if existingSecret not set) | `wazuh` |
| `wazuh.indexer.host` | Wazuh Indexer hostname | `localhost` |
| `wazuh.indexer.port` | Wazuh Indexer port | `9200` |
| `wazuh.indexer.username` | Wazuh Indexer username (only if existingSecret not set) | `admin` |
| `wazuh.indexer.password` | Wazuh Indexer password (only if existingSecret not set) | `admin` |
| `wazuh.verifySSL` | Enable SSL certificate verification | `false` |
| `wazuh.protocol` | Protocol for connections (http/https) | `https` |

### Server Mode Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `mode` | Server mode: `stdio` or `http` | `stdio` |
| `http.port` | HTTP server port (used when mode=http) | `3000` |
| `http.host` | HTTP server bind address (used when mode=http) | `0.0.0.0` |

**Mode Options:**
- **`stdio`** (default): Standard MCP server using stdin/stdout. Suitable for local clients and job-based workloads.
- **`http`**: HTTP REST API server. Exposes MCP tools via HTTP endpoints. Automatically enables Service.

### Service Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `service.enabled` | Manually enable Service | `false` (auto-enabled when mode=http) |
| `service.type` | Kubernetes Service type | `ClusterIP` |
| `service.port` | Service port | `3000` |
| `service.annotations` | Service annotations | `{}` |

**Note:** When `mode=http`, the Service is automatically created even if `service.enabled=false`.

### Logging Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `logging.level` | Log level (trace, debug, info, warn, error) | `info` |

### Resource Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `resources.limits.cpu` | CPU limit | `500m` |
| `resources.limits.memory` | Memory limit | `512Mi` |
| `resources.requests.cpu` | CPU request | `100m` |
| `resources.requests.memory` | Memory request | `128Mi` |

## Examples

### Custom Values File

Create a `custom-values.yaml`:

```yaml
wazuh:
  api:
    host: "wazuh-manager.example.com"
    port: 55000
    username: "admin"
    password: "SecurePassword123"
  indexer:
    host: "wazuh-indexer.example.com"
    port: 9200
    username: "admin"
    password: "SecurePassword456"
  verifySSL: true
  protocol: "https"

resources:
  limits:
    cpu: 1000m
    memory: 1Gi
  requests:
    cpu: 250m
    memory: 256Mi

logging:
  level: "debug"
```

Install with custom values:

```bash
helm install my-wazuh-mcp ./helm/mcp-server-wazuh -f custom-values.yaml
```

### HTTP Mode Deployment

Deploy the server in HTTP mode with a LoadBalancer for external access:

```yaml
# http-mode-values.yaml
mode: http

wazuh:
  api:
    host: "wazuh-manager.example.com"
    port: 55000
    username: "admin"
    password: "SecurePassword123"
  indexer:
    host: "wazuh-indexer.example.com"
    port: 9200
    username: "admin"
    password: "SecurePassword456"
  verifySSL: true

service:
  type: LoadBalancer
  annotations:
    service.beta.kubernetes.io/aws-load-balancer-type: "nlb"

http:
  port: 3000
  host: "0.0.0.0"

resources:
  limits:
    cpu: 1000m
    memory: 1Gi
  requests:
    cpu: 250m
    memory: 256Mi
```

Install:
```bash
helm install wazuh-mcp-http ./helm/mcp-server-wazuh -f http-mode-values.yaml
```

Test the HTTP endpoint:
```bash
# Get the service external IP/hostname
kubectl get svc -l app.kubernetes.io/name=mcp-server-wazuh

# Health check
curl http://<EXTERNAL-IP>:3000/health

# Call MCP tool
curl -X POST http://<EXTERNAL-IP>:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }'
```

### Using with External Secrets

For production deployments, use an existing Secret managed by External Secrets Operator, Sealed Secrets, or Vault:

#### Option 1: Using External Secrets Operator

```yaml
# external-secret.yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: wazuh-credentials
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: vault-backend
    kind: SecretStore
  target:
    name: wazuh-credentials
    creationPolicy: Owner
  data:
    - secretKey: WAZUH_API_USERNAME
      remoteRef:
        key: wazuh/api
        property: username
    - secretKey: WAZUH_API_PASSWORD
      remoteRef:
        key: wazuh/api
        property: password
    - secretKey: WAZUH_INDEXER_USERNAME
      remoteRef:
        key: wazuh/indexer
        property: username
    - secretKey: WAZUH_INDEXER_PASSWORD
      remoteRef:
        key: wazuh/indexer
        property: password
```

Then reference it in values.yaml:

```yaml
existingSecret: "wazuh-credentials"

wazuh:
  api:
    host: "wazuh-manager.example.com"
    port: 55000
    # Credentials managed by external secret
  indexer:
    host: "wazuh-indexer.example.com"
    port: 9200
    # Credentials managed by external secret
```

#### Option 2: Using Sealed Secrets

```bash
# Create a regular secret
kubectl create secret generic wazuh-credentials \
  --from-literal=WAZUH_API_USERNAME=admin \
  --from-literal=WAZUH_API_PASSWORD=SecurePass123 \
  --from-literal=WAZUH_INDEXER_USERNAME=admin \
  --from-literal=WAZUH_INDEXER_PASSWORD=SecurePass456 \
  --dry-run=client -o yaml > wazuh-secret.yaml

# Seal it
kubeseal -f wazuh-secret.yaml -w wazuh-sealed-secret.yaml

# Apply the sealed secret
kubectl apply -f wazuh-sealed-secret.yaml
```

Then in values.yaml:

```yaml
existingSecret: "wazuh-credentials"
# Rest of configuration...
```

#### Option 3: Manual Secret Creation

```bash
# Create secret manually
kubectl create secret generic wazuh-credentials \
  --from-literal=WAZUH_API_USERNAME=admin \
  --from-literal=WAZUH_API_PASSWORD=SecurePass123 \
  --from-literal=WAZUH_INDEXER_USERNAME=admin \
  --from-literal=WAZUH_INDEXER_PASSWORD=SecurePass456
```

Then install the chart:

```bash
helm install wazuh-mcp ./helm/mcp-server-wazuh \
  --set existingSecret=wazuh-credentials \
  --set wazuh.api.host=wazuh-manager.example.com \
  --set wazuh.indexer.host=wazuh-indexer.example.com
```

## Uninstalling

To uninstall/delete the `my-wazuh-mcp` deployment:

```bash
helm uninstall my-wazuh-mcp
```

## Security Considerations

⚠️ **Important Security Notes:**

1. **Never commit credentials to version control** - Use Helm secrets, Sealed Secrets, or External Secrets Operator
2. **Enable SSL verification in production** - Set `wazuh.verifySSL: true` with proper certificates
3. **Use strong passwords** - Change default credentials before deployment
4. **Network policies** - Consider implementing Kubernetes Network Policies to restrict traffic
5. **RBAC** - The chart creates a ServiceAccount with minimal permissions

## Troubleshooting

View logs:
```bash
kubectl logs -l app.kubernetes.io/name=mcp-server-wazuh
```

Check pod status:
```bash
kubectl get pods -l app.kubernetes.io/name=mcp-server-wazuh
```

Describe pod:
```bash
kubectl describe pod -l app.kubernetes.io/name=mcp-server-wazuh
```
