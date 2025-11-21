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

### Wazuh Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `wazuh.api.host` | Wazuh Manager API hostname | `localhost` |
| `wazuh.api.port` | Wazuh Manager API port | `55000` |
| `wazuh.api.username` | Wazuh Manager API username | `wazuh` |
| `wazuh.api.password` | Wazuh Manager API password | `wazuh` |
| `wazuh.indexer.host` | Wazuh Indexer hostname | `localhost` |
| `wazuh.indexer.port` | Wazuh Indexer port | `9200` |
| `wazuh.indexer.username` | Wazuh Indexer username | `admin` |
| `wazuh.indexer.password` | Wazuh Indexer password | `admin` |
| `wazuh.verifySSL` | Enable SSL certificate verification | `false` |
| `wazuh.protocol` | Protocol for connections (http/https) | `https` |

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

### Using with External Secrets

For production deployments, consider using Kubernetes External Secrets:

```yaml
# Don't store credentials in values.yaml
# Instead, reference an external secret
wazuh:
  api:
    host: "wazuh-manager.example.com"
    port: 55000
    # Username and password managed externally
  indexer:
    host: "wazuh-indexer.example.com"
    port: 9200
    # Username and password managed externally
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
