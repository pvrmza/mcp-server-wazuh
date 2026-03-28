# Documentación del Proyecto - MCP Server Wazuh

## 1. Visión General del Proyecto

El **MCP Server Wazuh** es un servidor written en Rust que actúa como puente entre el sistema SIEM Wazuh y aplicaciones que utilizan el **Model Context Protocol (MCP)**. Su función principal es permitir que asistentes de IA como Claude Desktop consulten datos de seguridad de Wazuh de forma natural.

### Propósito

El servidor expone las capacidades del SIEM Wazuh como herramientas MCP, permitiendo:
- Consultar alertas de seguridad desde Wazuh Indexer
- Obtener información de agentes y su estado
- Consultar vulnerabilidades detectadas
- Revisar reglas de detección y compliance
- Monitorear logs y estadísticas del sistema
- Verificar salud del cluster de Wazuh

### Componentes Principales

El proyecto se estructura en los siguientes módulos:

| Módulo | Archivo | Descripción |
|--------|---------|-------------|
| **AlertTools** | `alerts.rs` | Consulta alertas desde Wazuh Indexer |
| **AgentTools** | `agents.rs` | Gestión y monitoreo de agentes |
| **VulnerabilityTools** | `vulnerabilities.rs` | Consulta de vulnerabilidades |
| **RuleTools** | `rules.rs` | Consultas de reglas de detección |
| **StatsTools** | `stats.rs` | Logs, estadísticas y cluster |

---

## 2. Herramientas MCP Disponibles

A continuación se detallan todas las herramientas que expose el servidor:

### 2.1. Herramientas de Alertas

#### `get_wazuh_alert_summary`

**Descripción**: Recupera un resumen de las alertas de seguridad de Wazuh. Devuelve información formateada incluyendo ID, timestamp, descripción, agente, nivel e información adicional como IP de origen/destino y usuarios.

**Origen de datos**: Wazuh Indexer (Elasticsearch)

**Parámetros**:

| Parámetro | Tipo | Requerido | Default | Descripción |
|-----------|------|-----------|---------|-------------|
| `limit` | `u32` | No | 300 | Cantidad máxima de alertas a recuperar |

**Datos que devuelve**:
- Alert ID
- Timestamp (fecha y hora)
- Nombre del agente que generó la alerta
- Nivel de la regla (0-15)
- Descripción de la regla
- IP de origen (si está disponible)
- IP de destino (si está disponible)
- Usuario involucrado (si está disponible)

**Ejemplo de uso**:
```json
{
  "name": "get_wazuh_alert_summary",
  "arguments": { "limit": 50 }
}
```

---

### 2.2. Herramientas de Reglas

#### `get_wazuh_rules_summary`

**Descripción**: Recupera un resumen de las reglas de seguridad de Wazuh. Devuelve información formateada incluyendo ID, nivel, descripción, grupos, archivo, estado y compliance.

**Origen de datos**: Wazuh Manager API

**Parámetros**:

| Parámetro | Tipo | Requerido | Default | Descripción |
|-----------|------|-----------|---------|-------------|
| `limit` | `u32` | No | 300 | Cantidad máxima de reglas a recuperar |
| `level` | `u32` | No | - | Filtrar por nivel de regla (0-15) |
| `group` | `String` | No | - | Filtrar por grupo de reglas |
| `filename` | `String` | No | - | Filtrar por nombre de archivo de reglas |

**Datos que devuelve**:
- Rule ID
- Level (nivel numérico y texto: Low/Medium/High/Critical)
- Description
- Groups (grupos a los que pertenece la regla)
- File (archivo donde está definida la regla)
- Status (enabled/disabled)
- Compliance (GDPR, HIPAA, PCI DSS, NIST 800-53 si aplica)

**Ejemplo de uso**:
```json
{
  "name": "get_wazuh_rules_summary",
  "arguments": { "limit": 50, "level": 12 }
}
```

---

### 2.3. Herramientas de Vulnerabilidades

#### `get_wazuh_vulnerability_summary`

**Descripción**: Recupera un resumen de vulnerabilidades para un agente específico. Soporta filtrado por severidad y CVE.

**Origen de datos**: Wazuh Manager API

**Parámetros**:

| Parámetro | Tipo | Requerido | Default | Descripción |
|-----------|------|-----------|---------|-------------|
| `agent_id` | `String` o `Number` | **Sí** | - | ID del agente (acepta "001" o 1) |
| `limit` | `u32` | No | 10000 | Cantidad máxima de vulnerabilidades |
| `severity` | `String` | No | - | Filtrar por severidad (Low/Medium/High/Critical) |
| `cve` | `String` | No | - | Filtrar por ID de CVE específico |

**Nota**: El `agent_id` acepta tanto strings ("001") como números (1) para mayor compatibilidad con clientes MCP.

**Datos que devuelve**:
- CVE ID
- Severity (Critical/High/Medium/Low con indicadores emoji)
- Title
- Description
- Published date
- Updated date
- Detection time
- Agent info (nombre e ID)
- CVSS scores (CVSS2, CVSS3 si están disponibles)
- Reference URL

**Ejemplo de uso**:
```json
{
  "name": "get_wazuh_vulnerability_summary",
  "arguments": { "agent_id": "001", "severity": "Critical", "limit": 50 }
}
```

#### `get_wazuh_critical_vulnerabilities`

**Descripción**: Recupera solo las vulnerabilidades críticas de un agente específico.

**Origen de datos**: Wazuh Manager API

**Parámetros**:

| Parámetro | Tipo | Requerido | Default | Descripción |
|-----------|------|-----------|---------|-------------|
| `agent_id` | `String` o `Number` | **Sí** | - | ID del agente |
| `limit` | `u32` | No | 300 | Cantidad máxima de vulnerabilidades |

**Datos que devuelve**: Similar a `get_wazuh_vulnerability_summary` pero solo incluye vulnerabilidades críticas.

**Ejemplo de uso**:
```json
{
  "name": "get_wazuh_critical_vulnerabilities",
  "arguments": { "agent_id": 1 }
}
```

---

### 2.4. Herramientas de Agentes

#### `get_wazuh_agents`

**Descripción**: Recupera información de los agentes de Wazuh. Soporta múltiples filtros para encontrar agentes específicos.

**Origen de datos**: Wazuh Manager API

**Parámetros**:

| Parámetro | Tipo | Requerido | Default | Descripción |
|-----------|------|-----------|---------|-------------|
| `limit` | `u32` | No | 300 | Cantidad máxima de agentes |
| `status` | `String` o `Number` | **Sí** | - | Estado del agente (active/disconnected/pending/never_connected) |
| `name` | `String` | No | - | Filtrar por nombre de agente |
| `ip` | `String` | No | - | Filtrar por IP del agente |
| `group` | `String` | No | - | Filtrar por grupo |
| `os_platform` | `String` | No | - | Filtrar por sistema operativo |
| `version` | `String` | No | - | Filtrar por versión del agente |

**Estados válidos**:
- `active` (o "0")
- `disconnected` (o "1")
- `pending` (o "2")
- `never_connected` (o "3")

**Datos que devuelve**:
- Agent ID
- Name
- IP
- Status
- Group
- OS Platform
- Version
- Y otros campos del agente

**Ejemplo de uso**:
```json
{
  "name": "get_wazuh_agents",
  "arguments": { "status": "active", "limit": 50 }
}
```

#### `get_wazuh_agent_processes`

**Descripción**: Recupera los procesos que están corriendo en un agente específico.

**Origen de datos**: Wazuh Manager API

**Parámetros**:

| Parámetro | Tipo | Requerido | Default | Descripción |
|-----------|------|-----------|---------|-------------|
| `agent_id` | `String` o `Number` | **Sí** | - | ID del agente |
| `limit` | `u32` | No | 300 | Cantidad máxima de procesos |
| `search` | `String` | No | - | Filtrar por nombre de proceso o comando |

**Datos que devuelve**:
- PID
- Name (nombre del proceso)
- Command (línea de comando completa)
- Usuario
- Estado
- Memoria CPU y otros recursos

**Ejemplo de uso**:
```json
{
  "name": "get_wazuh_agent_processes",
  "arguments": { "agent_id": "001", "limit": 50 }
}
```

#### `get_wazuh_agent_ports`

**Descripción**: Recupera los puertos de red abiertos en un agente específico.

**Origen de datos**: Wazuh Manager API

**Parámetros**:

| Parámetro | Tipo | Requerido | Default | Descripción |
|-----------|------|-----------|---------|-------------|
| `agent_id` | `String` o `Number` | **Sí** | - | ID del agente |
| `limit` | `u32` | No | 300 | Cantidad máxima de puertos |
| `protocol` | `String` o `Number` | No | - | Filtrar por protocolo (tcp/udp) |
| `state` | `String` o `Number` | No | - | Filtrar por estado (LISTENING/ESTABLISHED/etc) |

**Estados de puerto**: LISTENING, ESTABLISHED, TIME_WAIT, etc.

**Datos que devuelve**:
- Protocol (TCP/UDP)
- Local IP
- Local Port
- Remote IP
- Remote Port
- State
- PID

**Ejemplo de uso**:
```json
{
  "name": "get_wazuh_agent_ports",
  "arguments": { "agent_id": "001", "protocol": "tcp" }
}
```

---

### 2.5. Herramientas de Estadísticas y Logs

#### `search_wazuh_manager_logs`

**Descripción**: Busca en los logs del Wazuh Manager con filtros avanzados.

**Origen de datos**: Wazuh Manager API (Logs)

**Parámetros**:

| Parámetro | Tipo | Requerido | Default | Descripción |
|-----------|------|-----------|---------|-------------|
| `limit` | `u32` | No | 300 | Cantidad máxima de entradas |
| `offset` | `u32` | No | 0 | Cantidad de entradas a saltar |
| `level` | `String` o `Number` | **Sí** | - | Nivel de log (error/warning/info) |
| `tag` | `String` | No | - | Filtrar por tag (ej: wazuh-modulesd) |
| `search_term` | `String` | No | - | Término de búsqueda en descripción |

**Niveles válidos**: error, warning, info, debug, trace

**Datos que devuelve**:
- Timestamp
- Level
- Tag
- Description

**Ejemplo de uso**:
```json
{
  "name": "search_wazuh_manager_logs",
  "arguments": { "level": "error", "limit": 50 }
}
```

#### `get_wazuh_manager_error_logs`

**Descripción**: Recupera únicamente los logs de error del Wazuh Manager.

**Origen de datos**: Wazuh Manager API (Logs)

**Parámetros**:

| Parámetro | Tipo | Requerido | Default | Descripción |
|-----------|------|-----------|---------|-------------|
| `limit` | `u32` | No | 300 | Cantidad máxima de entradas |

**Datos que devuelve**: Similar a `search_wazuh_manager_logs` pero solo errores.

**Ejemplo de uso**:
```json
{
  "name": "get_wazuh_manager_error_logs",
  "arguments": { "limit": 50 }
}
```

#### `get_wazuh_log_collector_stats`

**Descripción**: Obtiene estadísticas del colector de logs de un agente específico.

**Origen de datos**: Wazuh Manager API

**Parámetros**:

| Parámetro | Tipo | Requerido | Default | Descripción |
|-----------|------|-----------|---------|-------------|
| `agent_id` | `String` o `Number` | **Sí** | - | ID del agente |

**Datos que devuelve**:
- Estado del log collector
- Archivos siendo monitoreados
- Estadísticas de procesamiento
- Bytes/leídas

**Ejemplo de uso**:
```json
{
  "name": "get_wazuh_log_collector_stats",
  "arguments": { "agent_id": "001" }
}
```

#### `get_wazuh_remoted_stats`

**Descripción**: Obtiene estadísticas del daemon remoted de Wazuh (comunicación entre Manager y agentes).

**Origen de datos**: Wazuh Manager API

**Parámetros**: No requiere parámetros

**Datos que devuelve**:
- Conexiones totales
- Eventos recibidos/enviados
- Bytes transferidos
-Cola de mensajes

**Ejemplo de uso**:
```json
{
  "name": "get_wazuh_remoted_stats",
  "arguments": {}
}
```

#### `get_wazuh_weekly_stats`

**Descripción**: Obtiene estadísticas acumuladas de la última semana.

**Origen de datos**: Wazuh Manager API

**Parámetros**: No requiere parámetros

**Datos que devuelve**:
-Alertas por día
- Eventostotal
- Datos procesados
- Tendencias semanales

**Ejemplo de uso**:
```json
{
  "name": "get_wazuh_weekly_stats",
  "arguments": {}
}
```

#### `get_wazuh_cluster_health`

**Descripción**: Verifica el estado de salud del cluster de Wazuh.

**Origen de datos**: Wazuh Manager API (Cluster)

**Parámetros**: No requiere parámetros

**Datos que devuelve**:
- Estado general del cluster
- Nodos activos
- Estado de cada nodo
- Información de sincronización

**Ejemplo de uso**:
```json
{
  "name": "get_wazuh_cluster_health",
  "arguments": {}
}
```

#### `get_wazuh_cluster_nodes`

**Descripción**: Lista los nodos del cluster de Wazuh.

**Origen de datos**: Wazuh Manager API (Cluster)

**Parámetros**:

| Parámetro | Tipo | Requerido | Default | Descripción |
|-----------|------|-----------|---------|-------------|
| `limit` | `u32` | No | 500 | Cantidad máxima de nodos |
| `offset` | `u32` | No | 0 | Cantidad de nodos a saltar |
| `node_type` | `String` | No | - | Filtrar por tipo (master/worker) |

**Datos que devuelve**:
- Node name
- IP
- Type (master/worker)
- Status
- Version

**Ejemplo de uso**:
```json
{
  "name": "get_wazuh_cluster_nodes",
  "arguments": { "node_type": "master" }
}
```

---

## 3. Arquitectura del Sistema

### 3.1. Diagrama de Componentes

```
┌─────────────────────────────────────────────────────────────┐
│                     Cliente MCP                             │
│              (Claude Desktop, etc.)                        │
└─────────────────────────┬───────────────────────────────────┘
                          │ JSON-RPC 2.0
                          │ (stdio o HTTP)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    MCP Server Wazuh                         │
│                       (Rust)                                │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │AlertTools   │  │AgentTools   │  │RuleTools    │        │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘        │
│         │                 │                 │               │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐        │
│  │Vulnerability│  │  StatsTools │  │   ...       │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└───────────┬───────────────────────────────────┬────────────┘
            │                                   │
            ▼                                   ▼
┌─────────────────────┐           ┌─────────────────────┐
│   Wazuh Indexer     │           │   Wazuh Manager API  │
│   (Elasticsearch)   │           │     (puerto 55000)   │
│   (puerto 9200)     │           │                      │
└─────────────────────┘           └─────────────────────┘
```

### 3.2. Flujo de Datos

1. **Recibir solicitud**: El cliente MCP envía una llamada de herramienta vía JSON-RPC 2.0
2. **Deserializar parámetros**: Los parámetros se parsean usando serde con el deserializador personalizado `deserialize_string_or_number`
3. **Validar entrada**: Se validan campos como agent_id usando `ToolUtils::format_agent_id`
4. **Llamar API de Wazuh**: Se hace la llamada al cliente de wazuh-client correspondiente
5. **Procesar respuesta**: Los datos se transforman en objetos `Content` formateados
6. **Devolver resultado**: Se retorna un `CallToolResult` con el contenido formateado

### 3.3. Clientes Wazuh

El servidor utiliza los siguientes clientes de `wazuh-client`:

| Cliente | Propósito | Puerto |
|---------|-----------|--------|
| `WazuhIndexerClient` | Consultas de alertas | 9200 (Indexer) |
| `AgentsClient` | Gestión de agentes | 55000 (Manager) |
| `VulnerabilityClient` | Vulnerabilidades | 55000 (Manager) |
| `RulesClient` | Reglas de detección | 55000 (Manager) |
| `LogsClient` | Logs del Manager | 55000 (Manager) |
| `ClusterClient` | Cluster de Wazuh | 55000 (Manager) |

### 3.4. Patrones de Diseño Utilizados

**Facade Pattern**: `WazuhToolsServer` en `main.rs` actúa como facade unificado quedelega a los módulos específicos.

**ToolModule Trait**: Proporciona métodos helpers para formatear resultados:
- `format_error()`: Crea mensajes de error consistentes
- `success_result()`: Retorna éxito con contenido
- `error_result()`: Retorna error con mensaje
- `not_found_result()`: Retorna mensaje de "no encontrado"

---

## 4. Configuración del Servidor

### 4.1. Variables de Entorno Requeridas

| Variable | Descripción |
|----------|-------------|
| `WAZUH_API_HOST` | Hostname del Wazuh Manager |
| `WAZUH_API_PORT` | Puerto del API del Manager (default: 55000) |
| `WAZUH_API_USERNAME` | Usuario para autenticación |
| `WAZUH_API_PASSWORD` | Contraseña para autenticación |
| `WAZUH_INDEXER_HOST` | Hostname del Wazuh Indexer |
| `WAZUH_INDEXER_PORT` | Puerto del Indexer (default: 9200) |
| `WAZUH_INDEXER_USERNAME` | Usuario para Indexer |
| `WAZUH_INDEXER_PASSWORD` | Contraseña para Indexer |

### 4.2. Variables Opcionales

| Variable | Default | Descripción |
|----------|---------|-------------|
| `WAZUH_VERIFY_SSL` | `true` | Verificar certificados SSL |
| `WAZUH_TEST_PROTOCOL` | `https` | Protocolo (http/https) |
| `RUST_LOG` | `info` | Nivel de logging |

### 4.3. Modo de Transporte

El servidor soporta dos modos de transporte:

1. **stdio** (default): Comunicaicón vía entrada/salida estándar
2. **HTTP**: Servidor HTTP en el puerto configurado

**Uso**:
```bash
# Modo stdio (default)
./mcp-server-wazuh

# Modo HTTP
./mcp-server-wazuh --transport http --host 0.0.0.0 --port 8080
```

---

## 5. Casos de Uso y Flujos de Trabajo

### 5.1. Análisis de Alertas de Seguridad

**Objetivo**: Ver las alertas más recientes del SIEM

```
1. Llamar get_wazuh_alert_summary con limit=50
2. Revisar cada alerta:
   - Nivel > 10 = prioridad alta
   - IP origen conocida = investigar
   - Mismo agente múltiples alertas = posible amenaza activa
```

### 5.2. Investigación de Vulnerabilidades

**Objetivo**: Encontrar vulnerabilidades críticas en un servidor

```
1. get_wazuh_agents con status=active y name="web-server-01"
2. get_wazuh_critical_vulnerabilities con agent_id del resultado anterior
3. Priorizar remediación:
   - CVSS > 9 = crítico
   - CVE publicado recientemente = revisar exploit disponible
```

### 5.3. Monitoreo de Agentes

**Objetivo**: Verificar estado de todos los agentes

```
1. get_wazuh_agents con status=disconnected
2. Para cada agente desconectado:
   - get_wazuh_agent_processes para ver qué procesos tenía
   - get_wazuh_agent_ports para ver conexiones activas
3. Identificar patrones: ¿misma red? ¿misma hora?
```

### 5.4. Investigación de Incidentes

**Objetivo**: Investigar un incidente de seguridad

```
1. get_wazuh_alert_summary para ver alertas recientes
2. Identificar agente implicado
3. get_wazuh_agent_processes para ver procesos sospechos
4. get_wazuh_agent_ports para ver comunicaciones de red
5. get_wazuh_vulnerability_summary para ver vulnerabilidades del agente
6. search_wazuh_manager_logs para ver logs del Manager
```

### 5.5. Revisión de Compliance

**Objetivo**: Verificar que las reglas cubran requisitos de compliance

```
1. get_wazuh_rules_summary con level=10 (alta prioridad)
2. Filtrar por PCI DSS, HIPAA, GDPR
3. Identificar gaps: reglas sin compliance tag
4. Revisar reglas deshabilitadas que podrían ser necesarias
```

### 5.6. Salud del Cluster

**Objetivo**: Verificar que el cluster esté operativo

```
1. get_wazuh_cluster_health - estado general
2. get_wazuh_cluster_nodes - verificar todos nodos
3. get_wazuh_remoted_stats - verificar comunicación con agentes
4. get_wazuh_weekly_stats - tendencia de eventos
```

---

## 6. Manejo de Errores

### 6.1. Tipos de Errores

El servidor maneja los siguientes tipos de errores:

| Tipo | Causa | Respuesta |
|------|-------|-----------|
| Error de API de Wazuh | API retorna error HTTP | Mensaje formateado con details |
| Formato inválido | agent_id no válido | Error con ejemplo de formato válido |
| Sin resultados | Query retorna vacío | Mensaje "No se encontraron..." |
| Timeout | Wazuh no responde | Error de timeout |

### 6.2. Formato de agent_id

El servidor aceptaagent_id en estos formatos:

- **Número**: `1`, `12`, `100`
- **String**: `"001"`, `"012"`, `"100"`

El servidor convierte automáticamente al formato de 3 dígitos que espera Wazuh (`"001"`).

**Validación**:
- Debe ser número entre 0-999
- O string de exactamente 3 dígitos

---

## 7. Consideraciones de Seguridad

### 7.1. Autenticación

- Todas las credenciales son **requeridas** (sin defaults)
- El servidor falla si no se proporcionan las variables de autenticación
- Se recomienda usar variables de entorno, no hardcodear en config

### 7.2. SSL/TLS

- `WAZUH_VERIFY_SSL` default es `true`
- Solo setear en `false` para desarrollo/testing
- En producción, siempre usar SSL verificado

### 7.3. Rate Limiting

- No hay rate limiting implementado a nivel de servidor
- Depende del cliente MCP implementar limits
- Recomendación: no hacer más de 10 llamadas/segundo

---

## 8. Dependencias del Proyecto

### 8.1. Dependencias Principales

| Crate | Versión | Propósito |
|-------|---------|-----------|
| `rmcp` | 0.10 | Framework MCP server |
| `wazuh-client` | 0.1.8 | Cliente para APIs de Wazuh |
| `tokio` | - | Async runtime |
| `reqwest` | - | HTTP client |
| `serde` | - | Serialización |
| `schemars` | - | JSON Schema |
| `clap` | - | CLI arguments |
| `dotenv` | - | Variables de entorno |

### 8.2. Builds Disponibles

```bash
# Development
cargo build

# Release (más performant)
cargo build --release
```

El binary queda en `target/release/mcp-server-wazuh`.

---

*Documentación generada en base al código fuente del proyecto.*
*Versión del código: commit actual del repositorio*