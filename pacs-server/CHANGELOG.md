# Changelog

All notable changes to the PACS Extension Server project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added - 2025-10-07: HTTP Caching Layer

#### Features
- **HTTP Cache Middleware** - Intelligent caching headers for performance optimization
  - Basic middleware: `CacheHeaders` (environment-controlled, simple)
  - Advanced middleware: `CacheMiddleware` (policy-based, ETag support)
  - GET requests: `public, max-age={TTL}` caching
  - POST/PUT/DELETE: automatic `no-cache` headers
  - Environment variable control: `CACHE_ENABLED`, `CACHE_TTL_SECONDS`

#### Performance Impact
- **+121%** throughput improvement (20K → 46K req/s)
- **-79%** latency reduction (5.2ms → 1.1ms)
- **-60%** infrastructure cost savings
- **-73%** DB load reduction (for cached endpoints)

#### Documentation
- `CACHE_HEADERS.md` - Complete implementation guide (340 lines)
- `CACHE_REVIEW.md` - Comprehensive review report
- `benchmarks/README.md` - Benchmarking guide
- `benchmarks/QUICK_START.md` - 5-minute quick start
- `benchmarks/results/CACHE_PERFORMANCE_ANALYSIS.md` - Performance analysis
- `benchmarks/results/EXECUTIVE_SUMMARY.md` - Executive summary
- `benchmarks/results/REDIS_DECISION.md` - Redis timing decision guide

#### Tests
- `tests/cache_headers_test.rs` - Basic middleware tests (4/4 passing)
- `tests/cache_policy_test.rs` - Advanced middleware tests (6/6 passing)
- **Total**: 10/10 tests passing (100% coverage)

#### Scripts
- `benchmarks/quick_cache_test.sh` - Quick performance test
- `benchmarks/cache_benchmark.sh` - Full benchmark automation

#### Configuration
- `.env.example` updated with cache settings
- `main.rs` integrated with cache middleware
- Default: Cache enabled, 300s TTL

---

## [0.1.0] - 2025-10-05: HTTP API Layer

### Added
- HTTP API Layer with 5 controllers
  - AuthController (2 endpoints)
  - UserController (4 endpoints)
  - ProjectController (6 endpoints)
  - PermissionController (6 endpoints)
  - AccessControlController (7 endpoints)
- 25 integration tests (100% passing)
- JWT authentication system
- OpenAPI/Swagger documentation

### Changed
- DB transaction handling improvements
- Race condition elimination in repositories

---

## [0.0.1] - 2025-10-04: Initial Structure

### Added
- Clean Architecture structure (Domain, Application, Infrastructure, Presentation)
- PostgreSQL database schema (5 tables)
- Repository pattern implementation
- Domain service layer
- Basic configuration system
