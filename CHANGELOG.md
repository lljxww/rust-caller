# Changelog

All notable user-visible changes are recorded here. This project follows
Semantic Versioning; before 1.0, minor versions may contain API migrations.

## 0.4.0 - 2026-09-07

### Added

- Instance and global typed/separated request APIs: `call_params`, `call_args`,
  retry variants, and download variants.
- Real `CallerBuilder` middleware integration.
- Fallible dynamic authentication providers through `try_new`.
- Loopback HTTP protocol tests for request bodies, headers, URL encoding,
  authentication, timeouts, and middleware.
- A single public `ConfigFormat`; the two previous root names remain aliases.
- Queryable asynchronous config-watch errors.
- Server safety controls for remote binding, proxy routes, and permissive CORS.
- Bounded response buffering, response headers/duration, and
  `ApiResult::error_for_status`.
- Repeated query/form keys through `RequestArgs` pair methods.
- GET/POST/PUT/DELETE/PATCH/HEAD/OPTIONS support.

### Changed

- Typed JSON parameters preserve booleans, signed/unsigned integers, arrays,
  objects, and null values on the wire.
- `CallParams::to_json` and `OpenApiGenerator::new` now return errors instead of
  silently accepting invalid state.
- `ApiResult::build` now returns `ApiResult` directly because text responses are
  valid and construction is infallible; the unused `ResponseBody::Bytes`
  variant was removed in favor of `DownloadResult` for binary responses.
- `DownloadResult::from_response` now returns `DownloadResult` directly because
  it cannot fail.
- `ConfigLoader::init_with_config`, `ServiceBuilder::api`, request-context
  headers, and header middleware now report invalid input instead of panicking
  or silently ignoring it.
- Path parameters are percent-encoded as individual path segments.
- Service-level timeout and endpoint content type are applied to requests.
- OAuth2 uses the standard `Authorization` header.
- Dynamic environment credentials fail explicitly when missing.
- Sensitive authentication `Debug` output is redacted.
- Default User-Agent includes the package version.
- The development server refuses non-loopback binds unless explicitly allowed,
  no longer enables wildcard CORS by default, and accepts configured HTTP
  methods plus JSON/form bodies.
- Minimum supported Rust version is 1.88.
- Configuration rejects unknown fields, embedded URL credentials, base URL
  queries/fragments, invalid names, zero timeouts, and malformed path templates.
- Retry only repeats transport errors, honors integer `Retry-After`, and no
  longer repeats authentication/configuration/middleware failures.
- Exponential retry delays include configurable positive jitter (20% default).
- Response and error middleware unwind in reverse registration order.
- Legacy serialized authorization metadata is accepted on input but omitted on
  output to avoid copying stored credentials.
- Public rustdoc coverage and unreachable-public-item checks are enforced at
  the crate root and by the strict documentation/Clippy CI jobs.

### Fixed

- Prevented unsigned integer wraparound and non-finite float coercion.
- Prevented remote download filenames from escaping the selected directory.
- Fixed retry-delay integer overflow.
- Removed unused crypto, URL, and futures dependencies.
- Replaced the deprecated `serde_yaml` dependency with `serde_yaml_ng`.
- Removed the unused internal HTTP client and the no-op `RetryMiddleware`.
- Removed legacy catch-all error variants in favor of structured transport,
  status, protocol, and I/O errors with source chains.

### Security

- Remote-provided `Content-Disposition` filenames are rejected when unsafe.
- Tokens, passwords, API keys, and configured authorization values are no
  longer exposed through derived debug output.
- The built-in server is explicitly constrained as a local development tool by
  default.
- Ordinary responses and downloads are bounded by default to prevent unbounded
  memory growth.
