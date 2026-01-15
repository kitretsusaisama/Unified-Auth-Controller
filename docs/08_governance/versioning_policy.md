---
title: Versioning Policy
version: 1.0.0
status: Active
last_updated: 2026-01-12
owner: Engineering Team
category: Governance
---

# Versioning Policy

> [!NOTE]
> **Purpose**: Define versioning strategy and change management process.

---

## 1. Semantic Versioning

### 1.1 Format

**Version Format**: `MAJOR.MINOR.PATCH`

**Example**: `1.2.3`

- **MAJOR**: Breaking changes (incompatible API changes)
- **MINOR**: New features (backward-compatible)
- **PATCH**: Bug fixes (backward-compatible)

---

### 1.2 Version Increment Rules

| Change Type | Version Increment | Example |
|-------------|-------------------|---------|
| Breaking API change | MAJOR | 1.0.0 → 2.0.0 |
| New feature (backward-compatible) | MINOR | 1.0.0 → 1.1.0 |
| Bug fix | PATCH | 1.0.0 → 1.0.1 |
| Security fix | PATCH | 1.0.0 → 1.0.1 |

---

## 2. Breaking Change Policy

### 2.1 Definition

A **breaking change** is any change that requires API consumers to modify their code.

**Examples**:
- Removing an API endpoint
- Changing request/response schema
- Renaming fields
- Changing authentication method

---

### 2.2 Breaking Change Process

1. **Announce** breaking change (6 months notice)
2. **Deprecate** old API (mark as deprecated)
3. **Provide migration guide**
4. **Support both** old and new for 6 months
5. **Remove** old API in next major version

---

## 3. API Deprecation

### 3.1 Deprecation Timeline

- **T+0**: Announce deprecation
- **T+6 months**: Stop accepting new usage
- **T+12 months**: Remove deprecated API

### 3.2 Deprecation Notice

**HTTP Header**:
```http
Deprecation: true
Sunset: Sat, 01 Jan 2027 00:00:00 GMT
Link: <https://docs.upflame.com/migration>; rel="deprecation"
```

**Response Body**:
```json
{
  "warning": "This endpoint is deprecated and will be removed on 2027-01-01",
  "migration_guide": "https://docs.upflame.com/migration"
}
```

---

## 4. Changelog Format

### 4.1 Keep a Changelog

**Format**: [Keep a Changelog](https://keepachangelog.com/)

**Categories**:
- **Added**: New features
- **Changed**: Changes to existing features
- **Deprecated**: Soon-to-be-removed features
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Security fixes

### 4.2 Example

```markdown
# Changelog

## [1.2.0] - 2026-01-12

### Added
- WebAuthn/Passkey support
- GraphQL API endpoint

### Changed
- Improved token refresh performance

### Fixed
- Session fingerprinting bug

### Security
- Updated dependencies with security patches
```

---

## 5. Release Cadence

### 5.1 Schedule

- **Major releases**: Annually (January)
- **Minor releases**: Monthly
- **Patch releases**: As needed (bug fixes, security)

### 5.2 Release Process

1. **Code freeze** (1 week before release)
2. **Testing** (staging environment)
3. **Release notes** prepared
4. **Deployment** to production
5. **Post-release monitoring** (24 hours)

---

**Document Status**: Active  
**Owner**: Engineering Team
