# MySQL Migration Runner Script
# Runs all migrations on the MySQL database

$host = "srv1873.hstgr.io"
$database = "u413456342_sias"
$user = "u413456342_sias"
$password = "V&zTudOgd9v1"

Write-Host "==================================" -ForegroundColor Cyan
Write-Host "MySQL Migration Runner" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host "Host: $host" -ForegroundColor Yellow
Write-Host "Database: $database" -ForegroundColor Yellow
Write-Host ""

# Check if mysql client is available
$mysqlPath = (Get-Command mysql -ErrorAction SilentlyContinue)
if (-not $mysqlPath) {
    Write-Host "ERROR: MySQL client not found!" -ForegroundColor Red
    Write-Host "Please install MySQL client or use an alternative method." -ForegroundColor Yellow
    exit 1
}

# Migration files in order
$migrations = @(
    "migrations/001_initial_schema.sql",
    "migrations/002_create_refresh_tokens_table.sql",
    "migrations/003_create_revoked_tokens_table.sql"
)

$successCount = 0
$failCount = 0

foreach ($migration in $migrations) {
    Write-Host "Running migration: $migration" -ForegroundColor Cyan
    
    if (-not (Test-Path $migration)) {
        Write-Host "  ERROR: Migration file not found!" -ForegroundColor Red
        $failCount++
        continue
    }
    
    try {
        # Run migration
        $result = & mysql -h $host -u $user -p"$password" $database < $migration 2>&1
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  ✓ SUCCESS" -ForegroundColor Green
            $successCount++
        } else {
            Write-Host "  ✗ FAILED: $result" -ForegroundColor Red
            $failCount++
        }
    }
    catch {
        Write-Host "  ✗ FAILED: $_" -ForegroundColor Red
        $failCount++
    }
    
    Write-Host ""
}

Write-Host "==================================" -ForegroundColor Cyan
Write-Host "Migration Summary" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host "Successful: $successCount" -ForegroundColor Green
Write-Host "Failed: $failCount" -ForegroundColor $(if ($failCount -gt 0) { "Red" } else { "Green" })
Write-Host ""

if ($failCount -eq 0) {
    Write-Host "All migrations completed successfully! ✓" -ForegroundColor Green
    exit 0
} else {
    Write-Host "Some migrations failed. Please check the errors above." -ForegroundColor Red
    exit 1
}
