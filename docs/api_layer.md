# API Layer Documentation

## 1. Overview
The API Layer serves as the gateway between the ProzChain blockchain and the outside world, providing multiple standardized interfaces for developers and applications to interact with the network. This layer abstracts the internal complexity of the blockchain and presents clean, well-defined interfaces for various client needs.

**Why This Matters**: Without well-designed APIs, even the most advanced blockchain would remain unusable to developers and end-users. Our comprehensive API approach ensures flexibility for different use cases and development paradigms.

## 2. API Framework and Components

```rust
struct JsonRpcServer {
    methods: HashMap<String, Box<dyn RpcMethod>>,
    middleware: Vec<Box<dyn RpcMiddleware>>,
    metrics: RpcMetrics,
    auth_providers: Vec<Box<dyn AuthProvider>>,
    rate_limiter: RateLimiter,
    cors_config: CorsConfig,
}
```

**Core Components**:

1. **Method Registry**: Maps API method names to their implementations, allowing for modular API functionality
2. **Middleware Pipeline**: Processes requests through authentication, logging, metrics collection, and error handling
3. **Metrics Collection**: Tracks API usage, performance, error rates, and other operational data
4. **Authentication Providers**: Verify API client identity and permissions through various methods
5. **Rate Limiter**: Prevents abuse by limiting request frequency based on client identity
6. **CORS Configuration**: Controls cross-origin access for browser-based clients

**Design Rationale**:
- **Modular Architecture**: Components can be updated independently without affecting others
- **Middleware Pattern**: Enables cross-cutting concerns like logging without cluttering core logic
- **Metrics Integration**: Provides visibility into API usage and performance for optimization
- **Pluggable Authentication**: Supports multiple auth methods (API keys, JWT tokens, OAuth)

**For Beginners**: Think of this as a sophisticated reception desk at a building - it verifies visitors (authentication), directs them to the right department (routing), keeps track of who's visiting (metrics), and ensures no one department gets overwhelmed with visitors (rate limiting).

## 3. Core APIs

### 3.1 JSON-RPC API
The primary API interface following industry-standard JSON-RPC 2.0 protocol.

**Available Methods**:
- **Chain Information**: `getBlockHeight`, `getChainId`, `getNodeInfo`
- **Block Operations**: `getBlockByHash`, `getBlockByNumber`, `getBlockTransactions`
- **Transaction Operations**: `getTransaction`, `sendRawTransaction`, `estimateGas`
- **Account Information**: `getBalance`, `getNonce`, `getCode`, `getStorageAt`
- **Contract Interaction**: `call`, `estimateGas`, `getLogs`
- **Network Status**: `getPeerCount`, `getSyncStatus`, `getNetworkStats`

**Example Implementation**:
```rust
#[derive(Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,   // Must be "2.0"
    id: Value,         // Request identifier
    method: String,    // Method name
    params: Value,     // Method parameters
}

#[derive(Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,   // "2.0"
    id: Value,         // Same as request
    result: Option<Value>,  // Result (if successful)
    error: Option<JsonRpcError>,  // Error (if failed)
}

impl JsonRpcServer {
    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Process request through middleware
        // Find method handler
        // Execute method with parameters
        // Format response
        // ...existing code...
    }
    
    fn register_method<M: RpcMethod + 'static>(&mut self, method: M) {
        self.methods.insert(method.name().to_string(), Box::new(method));
    }
}
```

**Design Rationale**:
- **JSON-RPC 2.0**: Industry standard protocol familiar to blockchain developers
- **Comprehensive Method Set**: Covers all common blockchain operations
- **Error Standardization**: Consistent error codes and messages
- **Batch Processing**: Supports multiple operations in a single request

**For Beginners**: JSON-RPC is like ordering at a restaurant - you send a message (request) with what you want, and the kitchen (blockchain) sends back what you ordered (response) or an explanation if they can't make it (error).

### 3.2 GraphQL API
A flexible query language for complex data retrieval with minimal round trips.

**Schema Structure**:
```graphql
type Query {
  block(hash: String, number: Int): Block
  transaction(hash: String!): Transaction
  account(address: String!): Account
  chainStats: ChainStats
}

type Block {
  hash: String!
  number: Int!
  timestamp: Int!
  transactions: [Transaction!]!
  stateRoot: String!
  parentHash: String!
  transactionCount: Int!
}

type Transaction {
  hash: String!
  from: String!
  to: String
  value: String!
  data: String!
  blockNumber: Int
  blockHash: String
  status: TransactionStatus
  logs: [Log!]
}
```

**Implementation Overview**:
```rust
struct GraphQLServer {
    schema: GraphQLSchema,
    context_factory: Box<dyn Fn(Request) -> GraphQLContext>,
    execution_options: GraphQLExecutionOptions,
}

impl GraphQLServer {
    async fn handle_query(&self, query: GraphQLQuery, request: &Request) -> GraphQLResponse {
        let context = (self.context_factory)(request.clone());
        
        // Execute query against schema with context
        // Handle errors and format response
        // ...existing code...
    }
}
```

**Design Rationale**:
- **Efficient Data Retrieval**: Clients can request exactly the data they need
- **Reduced Round Trips**: Complex queries can be resolved in a single request
- **Type Safety**: Strong typing prevents many common API errors
- **Introspection**: Self-documenting API with built-in schema exploration

**For Beginners**: GraphQL is like going to a customizable sandwich shop where you specify exactly what ingredients you want (fields), rather than ordering a pre-made sandwich (REST) that might have things you don't need.

### 3.3 REST API
A resource-oriented API design following HTTP standards for simpler integration.

**Key Endpoints**:
- **GET /api/v1/blocks/:id** - Retrieve block by number or hash
- **GET /api/v1/transactions/:hash** - Get transaction details
- **GET /api/v1/accounts/:address** - Query account information
- **POST /api/v1/transactions** - Submit new transaction

**Implementation Example**:
```rust
async fn get_block_handler(path: web::Path<String>, blockchain: web::Data<Arc<Blockchain>>) -> impl Responder {
    let block_id = path.into_inner();
    
    // Try to parse as block number or hash
    // Retrieve block from blockchain
    // Format response with appropriate HTTP status
    // ...existing code...
}

fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(web::resource("/blocks/{id}").route(web::get().to(get_block_handler)))
            .service(web::resource("/transactions/{hash}").route(web::get().to(get_transaction_handler)))
            .service(web::resource("/accounts/{address}").route(web::get().to(get_account_handler)))
            .service(web::resource("/transactions").route(web::post().to(post_transaction_handler)))
    );
}
```

**Design Rationale**:
- **Familiar Pattern**: REST is widely understood by developers
- **HTTP Cache Compatibility**: Can leverage standard HTTP caching
- **Simple Integration**: Works with basic HTTP clients
- **Stateless**: Each request contains all information needed

**For Beginners**: REST API is like a file cabinet with clear labels - GET retrieves information, POST submits new information, etc. Each resource (block, transaction) has its own "drawer" with a clear address.

### 3.4 WebSocket Subscriptions
Real-time event notification system for dynamic applications.

**Subscription Types**:
- **newHeads**: Notifies when new blocks are added
- **logs**: Filtered event logs matching specified criteria
- **newPendingTransactions**: Notifies of new transactions entering mempool
- **syncing**: Status updates during chain synchronization

**Implementation Overview**:
```rust
struct WebSocketSubscription {
    id: SubscriptionId,
    type_: SubscriptionType,
    filters: Option<SubscriptionFilters>,
    client: WebSocketClient,
}

struct WebSocketServer {
    subscriptions: HashMap<SubscriptionId, WebSocketSubscription>,
    blockchain_listener: BlockchainEventListener,
}

impl WebSocketServer {
    async fn handle_subscription_request(&mut self, request: SubscriptionRequest, client: WebSocketClient) -> Result<SubscriptionId> {
        // Validate request
        // Create subscription with filters
        // Register with blockchain event system
        // Return subscription ID
        // ...existing code...
    }
    
    async fn distribute_event(&self, event: BlockchainEvent) {
        // Find matching subscriptions
        // Format event for each subscription
        // Send to subscribers
        // ...existing code...
    }
}
```

**Design Rationale**:
- **Push vs. Poll**: Efficient real-time updates without polling
- **Connection Reuse**: Maintains a single connection for multiple subscriptions
- **Filtered Events**: Clients specify exactly what events they care about
- **Reduced Server Load**: Eliminates unnecessary API calls for checking state

**For Beginners**: WebSocket subscriptions are like signing up for text alerts - instead of repeatedly checking for updates, you get automatically notified when something you care about happens.

## 4. Authentication and Security

### 4.1 Authentication Methods
Multiple authentication options for different security requirements.

**Supported Authentication**:
- **API Keys**: Simple string-based authentication
- **JWT Tokens**: Time-limited tokens with claims verification
- **TLS Client Certificates**: High-security mutual TLS authentication
- **OAuth 2.0**: Integration with existing identity providers

**Implementation Example**:
```rust
trait AuthProvider {
    fn authenticate(&self, request: &Request) -> Result<AuthContext>;
    fn name(&self) -> &'static str;
}

struct ApiKeyAuthProvider {
    keys: HashMap<String, ApiKeyPermissions>,
}

impl AuthProvider for ApiKeyAuthProvider {
    fn authenticate(&self, request: &Request) -> Result<AuthContext> {
        // Extract API key from header or query parameter
        // Look up permissions
        // Return auth context or error
        // ...existing code...
    }
}
```

**Design Rationale**:
- **Multiple Methods**: Different security needs for different clients
- **Pluggable Architecture**: Easy to add new authentication methods
- **Least Privilege**: Granular permissions based on authentication
- **Auditability**: All authenticated requests are logged

**For Beginners**: Think of authentication like showing ID at a secure building - there might be key cards, biometric scanners, or security guards checking credentials, all to make sure only authorized people get in.

### 4.2 Rate Limiting
Protects against abuse and ensures fair resource distribution.

**Rate Limiting Features**:
- **Tiered Limits**: Different limits based on client identity/tier
- **Multiple Scopes**: Global, per-method, and per-resource limits
- **Dynamic Adjustment**: Limits can adjust based on system load
- **Graceful Degradation**: Prioritizes critical operations under heavy load

**Implementation Example**:
```rust
struct RateLimiter {
    rules: Vec<RateLimitRule>,
    counters: HashMap<CounterKey, RateCounter>,
}

impl RateLimiter {
    fn check_rate_limit(&mut self, request: &Request, auth_context: &AuthContext) -> Result<()> {
        // Identify applicable rules
        // Check counters against limits
        // Update counters
        // Return error if limit exceeded
        // ...existing code...
    }
}
```

**Design Rationale**:
- **Fair Resource Allocation**: Prevents monopolization by single client
- **DoS Protection**: Mitigates resource exhaustion attacks
- **Service Quality**: Maintains responsiveness under heavy load
- **Business Rules**: Enables SLA enforcement for different client tiers

**For Beginners**: Rate limiting is like a traffic control system that ensures no single road (client) gets congested by too many cars (requests), maintaining smooth traffic flow for everyone.

## 5. Performance Optimization

### 5.1 Caching Strategy
Multi-level caching for improved response times and reduced load.

**Cache Layers**:
- **In-Memory Response Cache**: Frequently requested data
- **Materialized Views**: Pre-computed aggregates and derived data
- **Block and Transaction Cache**: Recently accessed blockchain data
- **State Cache**: Frequently accessed account state

**Implementation Overview**:
```rust
struct ApiCache {
    response_cache: TTLCache<CacheKey, CachedResponse>,
    materialized_views: HashMap<ViewType, MaterializedView>,
    eviction_policy: EvictionPolicy,
}

impl ApiCache {
    fn get_cached_response(&self, request: &Request) -> Option<CachedResponse> {
        // Generate cache key from request
        // Look up in cache
        // Check freshness
        // ...existing code...
    }
    
    fn cache_response(&mut self, request: &Request, response: Response) {
        // Generate cache key
        // Determine TTL based on response type
        // Store in cache
        // ...existing code...
    }
}
```

**Design Rationale**:
- **Reduced Latency**: Faster responses for common queries
- **Lower Resource Usage**: Decreased database and CPU load
- **Scalability**: Handles more clients with same resources
- **Intelligent Invalidation**: Cache entries are properly invalidated when data changes

**For Beginners**: Caching is like a restaurant keeping popular dishes pre-prepared - it's much faster to serve what's already made, as long as it's still fresh.

### 5.2 Connection Pooling
Efficient management of database and internal service connections.

**Features**:
- **Pre-warmed Connections**: Maintains ready-to-use connections
- **Connection Reuse**: Avoids overhead of establishing new connections
- **Health Checking**: Monitors and replaces unhealthy connections
- **Dynamic Sizing**: Adjusts pool size based on load patterns

**Implementation Example**:
```rust
struct ConnectionPool<T> {
    connections: Vec<PooledConnection<T>>,
    config: PoolConfig,
    connection_factory: Box<dyn Fn() -> Result<T>>,
}

impl<T> ConnectionPool<T> {
    async fn get_connection(&self) -> Result<PooledConnection<T>> {
        // Get connection from pool or create new
        // Track usage and return
        // ...existing code...
    }
}
```

**Design Rationale**:
- **Reduced Overhead**: Eliminates connection establishment costs
- **Connection Limits**: Enforces maximum connection count
- **Failure Resilience**: Handles connection errors gracefully
- **Resource Management**: Ensures proper connection cleanup

**For Beginners**: Connection pooling is like having a fleet of cars ready to go, rather than having to buy a new car each time you need to make a trip.

## 6. Client SDK Support

### 6.1 Official SDK Libraries
Pre-built client libraries for common programming languages.

**Supported Languages**:
- **JavaScript/TypeScript**: For web applications
- **Python**: For data science and scripting
- **Rust**: For high-performance applications
- **Java**: For enterprise systems
- **Go**: For backend services

**Common Features**:
- **Type-Safe Interfaces**: Language-specific types for API structures
- **Authentication Handling**: Simplified credential management
- **Retry Logic**: Automatic handling of transient failures
- **Event Subscriptions**: WebSocket wrapper for real-time updates

**Design Rationale**:
- **Lower Barrier to Entry**: Makes integration with ProzChain easier
- **Consistent Usage Patterns**: Common patterns across languages
- **Reduced Client Errors**: Type checking catches issues early
- **Best Practices**: Implements optimal patterns for each language

**For Beginners**: SDKs are like having pre-assembled furniture instead of individual parts - they save time and ensure everything fits together correctly.

### 6.2 OpenAPI Specification
Standardized API description for automatic client generation.

**Key Benefits**:
- **Machine-Readable Spec**: Enables automatic client generation
- **Interactive Documentation**: Powers Swagger UI for exploration
- **Contract First Development**: Clear interface definition for multiple implementations
- **Validation**: Request and response validation against schema

**Example Specification**:
```yaml
openapi: 3.0.0
info:
  title: ProzChain API
  version: 1.0.0
paths:
  /api/v1/blocks/{id}:
    get:
      summary: Get block by number or hash
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Block details
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Block'
```

**Design Rationale**:
- **Industry Standard**: Widely supported format for API documentation
- **Tool Ecosystem**: Many existing tools for working with OpenAPI
- **Client Generation**: Automatic client library generation
- **Testing Integration**: Can generate mock servers and test cases

**For Beginners**: OpenAPI is like a detailed blueprint that anyone can use to build tools that work with our API, ensuring consistency and compatibility.

## 7. Testing and Documentation

### 7.1 API Testing Framework
Comprehensive testing ensures API reliability.

**Testing Levels**:
- **Unit Tests**: Individual method handler testing
- **Integration Tests**: End-to-end API call testing
- **Load Tests**: Performance under high request volume
- **Contract Tests**: Ensures specification compliance
- **Chaos Tests**: Reliability under adverse conditions

**Implementation Example**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_get_block_response_format() {
        let server = test_server();
        let response = server.get("/api/v1/blocks/1").send();
        
        assert_eq!(response.status(), 200);
        assert!(response.json::<Value>().as_object().unwrap().contains_key("hash"));
        // ...existing code...
    }
}
```

**Design Rationale**:
- **Regression Prevention**: Catches breaking changes early
- **Specification Adherence**: Ensures API behaves as documented
- **Performance Baselines**: Establishes and monitors performance metrics
- **Error Handling Verification**: Tests graceful handling of error cases

**For Beginners**: API testing is like quality control in a factory - checking every aspect of the product before it reaches customers to ensure it works perfectly.

### 7.2 Interactive Documentation
Self-service documentation for developers.

**Documentation Features**:
- **Swagger UI**: Interactive API explorer
- **Request Examples**: Sample requests for common operations
- **Response Schemas**: Detailed response format documentation
- **Authentication Guide**: Step-by-step authentication instructions
- **Error Code Reference**: Comprehensive error documentation

**Implementation Example**:
```rust
fn configure_documentation(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/docs")
            .service(web::resource("").route(web::get().to(swagger_ui_handler)))
            .service(web::resource("/openapi.json").route(web::get().to(openapi_json_handler)))
    );
}
```

**Design Rationale**:
- **Self-Service**: Reduces support burden through clear documentation
- **Try-It-Now**: Interactive testing encourages experimentation
- **Always Current**: Generated from actual API code
- **Complete Coverage**: Documenting all methods, parameters, and responses

**For Beginners**: Interactive documentation is like a showroom where you can test drive the API before integrating it into your application.

## 8. References

- **JSON-RPC 2.0 Specification**: https://www.jsonrpc.org/specification
- **GraphQL Specification**: https://spec.graphql.org/
- **RESTful Web APIs** by Richardson & Ruby
- **OpenAPI Specification**: https://spec.openapis.org/oas/latest.html
- **WebSocket Protocol**: RFC 6455