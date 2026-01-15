# services/risk_assessment.rs

## File Metadata

**File Path**: `crates/auth-core/src/services/risk_assessment.rs`  
**Crate**: `auth-core`  
**Module**: `services::risk_assessment`  
**Layer**: Domain (Business Logic)  
**Security-Critical**: ✅ **YES** - Adaptive security and fraud detection

## Purpose

Provides risk assessment engine for evaluating login attempts and user behavior, enabling adaptive security policies based on contextual risk factors.

### Problem It Solves

- Detects suspicious login patterns
- Prevents account takeover attempts
- Enables adaptive MFA requirements
- Identifies anomalous behavior
- Supports fraud prevention

---

## Detailed Code Breakdown

### Struct: `RiskContext`

**Purpose**: Input data for risk assessment

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | `Uuid` | User being assessed |
| `tenant_id` | `Uuid` | Tenant context |
| `ip_address` | `Option<String>` | Client IP |
| `user_agent` | `Option<String>` | Browser/device info |
| `device_fingerprint` | `Option<String>` | Unique device ID |
| `geolocation` | `Option<String>` | Geographic location |
| `previous_logins` | `Vec<LoginHistory>` | Historical login data |

---

### Struct: `LoginHistory`

**Purpose**: Historical login record

**Fields**:
- `timestamp`: When login occurred
- `ip_address`: IP used
- `success`: Whether login succeeded

---

### Struct: `RiskAssessment`

**Purpose**: Risk evaluation result

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `score` | `f32` | Risk score (0.0-1.0) |
| `level` | `RiskLevel` | Categorical risk level |
| `factors` | `Vec<RiskFactor>` | Contributing risk factors |
| `recommendations` | `Vec<String>` | Security recommendations |

---

### Enum: `RiskLevel`

**Purpose**: Categorical risk classification

**Variants**:

| Level | Score Range | Action |
|-------|-------------|--------|
| `Low` | 0.0-0.3 | Allow login |
| `Medium` | 0.3-0.6 | Allow with monitoring |
| `High` | 0.6-0.8 | Require MFA |
| `Critical` | 0.8-1.0 | Block login |

---

### Struct: `RiskFactor`

**Purpose**: Individual risk contributor

**Fields**:
- `name`: Factor identifier (e.g., "new_ip")
- `weight`: Contribution to total score
- `description`: Human-readable explanation

---

### Trait: `RiskAssessor`

**Purpose**: Risk assessment interface

**Methods**:

```rust
async fn assess_risk(&self, context: RiskContext) -> Result<RiskAssessment, AuthError>;
async fn update_user_risk_score(&self, user_id: Uuid, score: f32) -> Result<(), AuthError>;
```

---

### Struct: `RiskEngine`

**Purpose**: Default risk assessment implementation

---

## Risk Assessment Algorithm

### Method: `calculate_ip_risk()`

**Purpose**: Assess IP address risk

**Logic**:
```rust
fn calculate_ip_risk(&self, ip: &Option<String>, history: &[LoginHistory]) -> (f32, Option<RiskFactor>) {
    if let Some(current_ip) = ip {
        let known_ips: HashSet<&String> = history.iter().map(|h| &h.ip_address).collect();
        if !known_ips.contains(current_ip) {
            return (0.3, Some(RiskFactor {
                name: "new_ip".to_string(),
                weight: 0.3,
                description: "Login from new IP address".to_string(),
            }));
        }
    }
    (0.0, None)
}
```

**Risk Factors**:
- **New IP**: +0.3 (30% risk increase)
- **Known IP**: +0.0 (no risk)

---

### Method: `assess_risk()`

**Purpose**: Comprehensive risk evaluation

**Algorithm**:

#### 1. IP Risk Assessment
```rust
let (ip_score, ip_factor) = self.calculate_ip_risk(&context.ip_address, &context.previous_logins);
score += ip_score;
if let Some(f) = ip_factor { factors.push(f); }
```

#### 2. Device Fingerprint Check
```rust
if context.device_fingerprint.is_none() {
    score += 0.2;
    factors.push(RiskFactor {
        name: "missing_fingerprint".to_string(),
        weight: 0.2,
        description: "Device fingerprinting unavailable".to_string(),
    });
}
```

**Risk**: +0.2 if no device fingerprint

#### 3. Failed Login Attempts
```rust
let recent_failures = context.previous_logins.iter().filter(|l| !l.success).count();
if recent_failures > 3 {
    score += 0.4;
    factors.push(RiskFactor {
        name: "multiple_failures".to_string(),
        weight: 0.4,
        description: format!("{} recent failed attempts", recent_failures),
    });
    recommendations.push("Enable MFA".to_string());
}
```

**Risk**: +0.4 if >3 recent failures

#### 4. Risk Level Classification
```rust
let level = match score {
    s if s < 0.3 => RiskLevel::Low,
    s if s < 0.6 => RiskLevel::Medium,
    s if s < 0.8 => RiskLevel::High,
    _ => RiskLevel::Critical,
};
```

---

## Usage Examples

### Example 1: Login Risk Assessment

```rust
let risk_engine = RiskEngine::new();

let context = RiskContext {
    user_id,
    tenant_id,
    ip_address: Some("192.168.1.100".to_string()),
    user_agent: Some("Mozilla/5.0...".to_string()),
    device_fingerprint: Some("abc123".to_string()),
    geolocation: Some("US-CA".to_string()),
    previous_logins: vec![
        LoginHistory {
            timestamp: Utc::now() - Duration::hours(1),
            ip_address: "192.168.1.1".to_string(),
            success: true,
        },
    ],
};

let assessment = risk_engine.assess_risk(context).await?;

match assessment.level {
    RiskLevel::Low => {
        // Allow login
        create_session(user).await?;
    }
    RiskLevel::Medium => {
        // Allow with monitoring
        create_session_with_monitoring(user).await?;
    }
    RiskLevel::High => {
        // Require MFA
        require_mfa_challenge(user).await?;
    }
    RiskLevel::Critical => {
        // Block login
        return Err(AuthError::AuthorizationDenied {
            permission: "login".to_string(),
            resource: "session".to_string(),
        });
    }
}
```

---

### Example 2: Adaptive MFA

```rust
pub async fn adaptive_mfa_check(
    user: &User,
    risk_assessment: &RiskAssessment,
) -> bool {
    // Always require MFA for high-risk scenarios
    if risk_assessment.score >= 0.7 {
        return true;
    }
    
    // Require MFA for sensitive roles
    if user.roles.contains(&"admin".to_string()) {
        return true;
    }
    
    // Check if user has MFA enabled
    user.mfa_enabled
}
```

---

## Advanced Risk Factors

### Geo-Velocity Detection

```rust
fn calculate_geo_velocity_risk(
    current_location: &str,
    previous_logins: &[LoginHistory],
) -> f32 {
    if let Some(last_login) = previous_logins.first() {
        let time_diff = Utc::now() - last_login.timestamp;
        let distance = calculate_distance(current_location, &last_login.location);
        
        // Impossible travel: 1000km in 1 hour
        let velocity = distance / time_diff.num_hours() as f64;
        if velocity > 1000.0 {
            return 0.8; // Critical risk
        }
    }
    0.0
}
```

### Time-Based Anomalies

```rust
fn calculate_time_anomaly_risk(
    timestamp: DateTime<Utc>,
    user_timezone: &str,
) -> f32 {
    let local_hour = timestamp.with_timezone(&user_timezone).hour();
    
    // Login at unusual hours (2 AM - 5 AM)
    if (2..=5).contains(&local_hour) {
        return 0.2;
    }
    0.0
}
```

### Device Reputation

```rust
fn calculate_device_risk(
    device_fingerprint: &str,
    known_devices: &[String],
) -> f32 {
    if !known_devices.contains(&device_fingerprint.to_string()) {
        return 0.3; // New device
    }
    0.0
}
```

---

## Integration with Session Service

```rust
impl SessionService {
    pub async fn create_session(
        &self,
        user: User,
        risk_context: RiskContext,
    ) -> Result<Session, AuthError> {
        // Assess risk
        let risk_assessment = self.risk_engine.assess_risk(risk_context.clone()).await?;
        
        // Block critical risk
        if risk_assessment.score >= 0.9 {
            return Err(AuthError::AuthorizationDenied {
                permission: "login".to_string(),
                resource: "session".to_string(),
            });
        }
        
        // Create session with risk score
        let session = Session {
            id: Uuid::new_v4(),
            user_id: user.id,
            risk_score: risk_assessment.score,
            // ... other fields
        };
        
        self.store.create(session).await
    }
}
```

---

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_low_risk_known_ip() {
    let engine = RiskEngine::new();
    
    let context = RiskContext {
        user_id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        ip_address: Some("192.168.1.1".to_string()),
        previous_logins: vec![
            LoginHistory {
                timestamp: Utc::now() - Duration::hours(1),
                ip_address: "192.168.1.1".to_string(),
                success: true,
            },
        ],
        ..Default::default()
    };
    
    let assessment = engine.assess_risk(context).await.unwrap();
    assert!(matches!(assessment.level, RiskLevel::Low));
}

#[tokio::test]
async fn test_high_risk_multiple_failures() {
    let engine = RiskEngine::new();
    
    let context = RiskContext {
        previous_logins: vec![
            LoginHistory { success: false, ..Default::default() },
            LoginHistory { success: false, ..Default::default() },
            LoginHistory { success: false, ..Default::default() },
            LoginHistory { success: false, ..Default::default() },
        ],
        ..Default::default()
    };
    
    let assessment = engine.assess_risk(context).await.unwrap();
    assert!(assessment.score >= 0.4);
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `uuid` | Identifiers |
| `chrono` | Timestamps |
| `async-trait` | Async trait support |

### Internal Dependencies

- [error.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/error.md) - AuthError
- [services/session_service.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/services/session_service.md) - Session integration

---

## Related Files

- [services/session_service.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/services/session_service.md) - Session service
- [services/identity.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/services/identity.md) - Identity service

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 135  
**Security Level**: CRITICAL
