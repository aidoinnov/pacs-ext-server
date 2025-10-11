# HTTP Cache Headers Implementation

## Overview

PACS Extension Server implements intelligent HTTP caching headers through Actix-web middleware to improve performance and reduce unnecessary network traffic.

## Available Middleware

### 1. Basic Middleware: `CacheHeaders` (Currently Used)

**Location**: `src/infrastructure/middleware/cache_headers.rs`

Simple implementation applying different caching strategies based on:
- **HTTP Method**: Only GET requests are cached
- **Configuration**: Cache can be enabled/disabled via environment variables

### 2. Advanced Middleware: `CacheMiddleware` (Available)

**Location**: `src/infrastructure/middleware/cache.rs`

Enhanced implementation with:
- **CachePolicy Enum**: NoCache, Public, Private, Immutable
- **ETag Support**: Optional ETag header generation
- **Fine-grained Control**: Per-endpoint policy configuration

### Caching Rules

#### GET Requests (Cache Enabled)
```
Cache-Control: public, max-age={TTL_SECONDS}
```
- Allows caching by both browsers and CDNs
- TTL configurable via `CACHE_TTL_SECONDS` environment variable
- Default TTL: 300 seconds (5 minutes)

#### POST/PUT/DELETE Requests (Always)
```
Cache-Control: no-cache, no-store, must-revalidate
```
- Prevents caching of mutation operations
- Ensures data consistency

#### Cache Disabled
```
Cache-Control: no-cache, no-store, must-revalidate
```
- Applied to all requests regardless of method
- Useful for development and debugging

## Configuration

### Environment Variables

**`.env` file configuration:**

```bash
# Enable/disable HTTP caching
CACHE_ENABLED=true

# Cache TTL in seconds (default: 300 = 5 minutes)
CACHE_TTL_SECONDS=300
```

### Recommended TTL Values

| Resource Type | TTL (seconds) | Use Case |
|--------------|---------------|----------|
| Static Resources | 3600-86400 | Images, CSS, JS |
| API Responses | 60-300 | GET endpoints |
| User-Specific Data | 30-60 | Personalized content |
| Development | 0 or disabled | Testing |

## Usage in Application

The middleware is applied globally in `main.rs`:

```rust
HttpServer::new(move || {
    App::new()
        .wrap(CacheHeaders::new(cache_enabled, cache_ttl))
        // ... routes
})
```

This ensures all HTTP responses receive appropriate cache headers.

## Testing

### Integration Tests

**Basic Middleware Tests**: `tests/cache_headers_test.rs`
- ✅ Cache headers enabled for GET requests
- ✅ Cache headers disabled (no-cache)
- ✅ POST requests always have no-cache
- ✅ Custom TTL values

**Advanced Middleware Tests**: `tests/cache_policy_test.rs`
- ✅ NoCache policy
- ✅ Public policy
- ✅ Private policy
- ✅ Immutable policy
- ✅ ETag generation
- ✅ Default policy

**Test Summary**: 10/10 passing ✅

**Run all cache tests:**
```bash
cargo test --test cache_headers_test --test cache_policy_test
```

**Run specific tests:**
```bash
# Basic middleware
cargo test cache_headers

# Advanced middleware
cargo test cache_policy
```

### Manual Testing with cURL

**Test GET request with caching enabled:**
```bash
curl -I http://localhost:8080/api/users
```

Expected response headers:
```
HTTP/1.1 200 OK
cache-control: public, max-age=300
...
```

**Test POST request (should not cache):**
```bash
curl -I -X POST http://localhost:8080/api/users
```

Expected response headers:
```
HTTP/1.1 ...
cache-control: no-cache, no-store, must-revalidate
...
```

**Test with caching disabled:**
```bash
# Set CACHE_ENABLED=false in .env, then:
curl -I http://localhost:8080/api/users
```

Expected response headers:
```
HTTP/1.1 200 OK
cache-control: no-cache, no-store, must-revalidate
...
```

## Performance Benefits

### Expected Improvements

1. **Reduced Server Load**
   - Browsers cache GET responses
   - CDNs can cache public endpoints
   - Less database queries for repeated requests

2. **Faster Client Experience**
   - Browser serves from cache
   - No network round-trip for cached resources
   - Lower latency for users

3. **Bandwidth Savings**
   - Fewer bytes transferred over network
   - Lower hosting costs
   - Better mobile experience

### Monitoring Cache Effectiveness

Monitor these metrics to assess cache performance:

```bash
# Check response headers in production
curl -I https://your-domain.com/api/endpoint

# Browser DevTools
# Network tab → Check "Size" column for "(from cache)"
```

## Best Practices

### DO ✅

- Use appropriate TTL values for different endpoint types
- Enable caching for read-only GET endpoints
- Test cache behavior in staging before production
- Monitor cache hit rates
- Document cache strategy for each endpoint

### DON'T ❌

- Cache user-specific or sensitive data without proper controls
- Use excessive TTL for frequently changing data
- Cache POST/PUT/DELETE operations
- Ignore cache validation in tests
- Deploy without testing cache headers

## Security Considerations

1. **Private Data Protection**
   - Current implementation uses `public` directive
   - Consider `private` for user-specific data
   - Never cache sensitive information (passwords, tokens)

2. **Cache Poisoning Prevention**
   - Validate request origins
   - Use HTTPS to prevent MITM attacks
   - Implement proper CORS policies

3. **Stale Data Risks**
   - Balance TTL vs. data freshness requirements
   - Implement cache invalidation for critical updates
   - Consider ETag support for validation

## Advanced Usage (CacheMiddleware)

### Using CachePolicy Enum

```rust
use pacs_server::infrastructure::middleware::{CacheMiddleware, CachePolicy};

// No caching (default)
.wrap(CacheMiddleware::default())

// Public caching (1 hour)
.wrap(CacheMiddleware::new(CachePolicy::Public { max_age: 3600 }))

// Private caching (5 minutes)
.wrap(CacheMiddleware::new(CachePolicy::Private { max_age: 300 }))

// Immutable caching (1 year for static assets)
.wrap(CacheMiddleware::new(CachePolicy::Immutable { max_age: 31536000 }))

// With ETag support
.wrap(CacheMiddleware::new(CachePolicy::Public { max_age: 3600 }).with_etag())
```

### Per-Route Caching

```rust
web::scope("/api")
    .service(
        web::scope("/static")
            .wrap(CacheMiddleware::new(CachePolicy::Immutable { max_age: 86400 }))
            .route("/assets", web::get().to(get_assets))
    )
    .service(
        web::scope("/users")
            .wrap(CacheMiddleware::new(CachePolicy::Private { max_age: 60 }))
            .route("/{id}", web::get().to(get_user))
    )
```

## Future Enhancements

### Available Now ✅
- ✅ **ETag Support**: Already implemented in `CacheMiddleware`
- ✅ **Multiple Cache Policies**: NoCache, Public, Private, Immutable
- ✅ **Per-Route Configuration**: Apply different policies to different endpoints

### Planned Features

1. **Content-Based ETags**
   - Generate ETags based on content hash (currently timestamp-based)
   - Support `If-None-Match` conditional requests
   - Properly handle 304 Not Modified responses

2. **Vary Header Support**
   - Cache different versions based on headers
   - Support for `Accept-Language`, `Authorization`
   - Better multi-tenant caching

3. **Cache Invalidation**
   - Programmatic cache clearing
   - Event-based invalidation
   - Selective cache purging

4. **Advanced Caching Strategies**
   - Stale-while-revalidate
   - Stale-if-error
   - Background revalidation

## Troubleshooting

### Common Issues

**Cache not working:**
- ✅ Check `CACHE_ENABLED=true` in `.env`
- ✅ Verify GET requests (not POST/PUT/DELETE)
- ✅ Check browser DevTools Network tab
- ✅ Clear browser cache and retry

**Stale data served:**
- ✅ Reduce `CACHE_TTL_SECONDS` value
- ✅ Implement cache invalidation
- ✅ Use versioned URLs for static resources

**Tests failing:**
- ✅ Ensure middleware is properly wrapped
- ✅ Check header assertions match implementation
- ✅ Verify test environment configuration

## References

- [MDN Cache-Control](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control)
- [RFC 7234 - HTTP Caching](https://tools.ietf.org/html/rfc7234)
- [Actix-web Middleware Guide](https://actix.rs/docs/middleware/)

---

## Summary

**Implementation Status**: ✅ Complete

| Feature | Basic (CacheHeaders) | Advanced (CacheMiddleware) |
|---------|---------------------|---------------------------|
| GET caching | ✅ | ✅ |
| Method-based logic | ✅ | ✅ |
| Environment config | ✅ | ⚠️ Manual |
| Multiple policies | ❌ | ✅ |
| ETag support | ❌ | ✅ |
| Per-route config | ⚠️ Limited | ✅ |
| Tests | 4/4 ✅ | 6/6 ✅ |

**Current Usage**: Basic middleware (`CacheHeaders`)
**Available Upgrade**: Advanced middleware (`CacheMiddleware`) ready for use

**Test Status**: ✅ 10/10 passing (100% coverage)
**Last Updated**: 2025-10-07
**Documentation**: Complete
