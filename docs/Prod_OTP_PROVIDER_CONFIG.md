# OTP Provider Configuration

## Environment Variables

```bash
# Firebase OTP Provider
FIREBASE_PROJECT_ID=your-project-id
FIREBASE_API_KEY=your-api-key

# Alternative: Generic SMS Provider (MSG91, Kaleyra, etc.)
SMS_API_URL=https://api.sms-provider.com/send
SMS_API_KEY=your-sms-api-key
SMS_SENDER_ID=YOURAPP

# SMTP Email Configuration
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=noreply@yourapp.com
SMTP_FROM_NAME="Your App Name"
```

## Supported Providers

### OTP/SMS Providers

1. **Firebase Auth** (Recommended)
   - Built-in OTP generation and delivery
   - Global SMS coverage
   - Automatic rate limiting
   - [Setup Guide](https://firebase.google.com/docs/auth/phone-auth)

2. **MSG91**
   - Indian SMS provider
   - Cost-effective for India
   - Good delivery rates

3. **Kaleyra**
   - Enterprise-grade
   - Global coverage
   - Advanced analytics

4. **Twilio** (Alternative)
   - Premium pricing
   - Excellent reliability
   - Global coverage

### Email Providers

1. **SMTP (Any provider)**
   - Gmail, Outlook, SendGrid, AWS SES, etc.
   - Standard protocol
   - Easy configuration

2. **Recommended SMTP Providers**:
   - **Gmail**: Easy setup, free tier
   - **AWS SES**: Production-grade, cheap
   - **SendGrid**: Dedicated email service
   - **Mailgun**: Developer-friendly

## Setup Instructions

### Firebase Setup

```bash
# 1. Create Firebase project at console.firebase.google.com
# 2. Enable Phone Authentication
# 3. Get API key from project settings
# 4. Add to environment variables

export FIREBASE_PROJECT_ID=your-project-id
export FIREBASE_API_KEY=your-api-key
```

### Gmail SMTP Setup

```bash
# 1. Enable 2FA on Google account
# 2. Generate App Password at https://myaccount.google.com/apppasswords
# 3. Use App Password as SMTP_PASSWORD

export SMTP_HOST=smtp.gmail.com
export SMTP_PORT=587
export SMTP_USERNAME=your-email@gmail.com
export SMTP_PASSWORD=your-16-char-app-password
```

### AWS SES Setup

```bash
# 1. Verify domain in AWS SES
# 2. Create SMTP credentials
# 3. Configure

export SMTP_HOST=email-smtp.us-east-1.amazonaws.com
export SMTP_PORT=587
export SMTP_USERNAME=your-ses-username
export SMTP_PASSWORD=your-ses-password
```

## Testing

```bash
# Test OTP sending
cargo test --package auth-core send_phone_otp

# Test email sending
cargo test --package auth-core send_email_otp

# Test circuit breakers
cargo test --package auth-core circuit_breaker
```
