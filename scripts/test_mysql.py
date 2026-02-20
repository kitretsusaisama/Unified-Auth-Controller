"""
Simple MySQL Connection Test & Data Seeder
Runs migrations and inserts sample data
"""

import mysql.connector
from mysql.connector import Error
import uuid
import os
import re

def main():
    print("=" * 50)
    print("MySQL Connection Test")
    print("=" * 50)
    print()

    # Connection details
    config = {
        'host': os.getenv('DB_HOST', 'localhost'),
        'database': os.getenv('DB_NAME', 'auth_platform'),
        'user': os.getenv('DB_USER', 'root'),
        'password': os.getenv('DB_PASSWORD')
    }

    # Try to parse from AUTH__DATABASE__MYSQL_URL if provided
    mysql_url = os.getenv('AUTH__DATABASE__MYSQL_URL')
    if mysql_url:
        match = re.match(r"mysql://(.*?):(.*?)@(.*?)/(.*)", mysql_url)
        if match:
            config['user'] = match.group(1)
            config['password'] = match.group(2)
            config['host'] = match.group(3)
            config['database'] = match.group(4)

    if not config['password']:
        print("❌ Error: DB_PASSWORD environment variable is not set")
        return

    try:
        print("📡 Connecting to MySQL...")
        print(f"   Host: {config['host']}")
        print(f"   Database: {config['database']}\n")

        connection = mysql.connector.connect(**config)

        if connection.is_connected():
            print("✅ Connection successful!\n")

            cursor = connection.cursor()

            # Test database access
            print("🔍 Testing database access...")
            cursor.execute("SELECT DATABASE()")
            result = cursor.fetchone()
            print(f"   Current database: {result[0]}\n")

            # Run migrations
            print("📋 Running migrations...")
            run_migrations(connection, cursor)

            # Insert sample data
            print("\n📝 Inserting sample data...")
            insert_sample_data(connection, cursor)

            # Verify
            print("\n🔍 Verifying data...")
            verify_data(cursor)

            print("\n" + "=" * 50)
            print("✅ All tests passed!")
            print("=" * 50)

    except Error as e:
        print(f"❌ Error: {e}")

    finally:
        if connection and connection.is_connected():
            cursor.close()
            connection.close()
            print("\n📡 Connection closed")


def run_migrations(connection, cursor):
    """Run database migrations from file"""
    try:
        with open('../migrations/complete_migration.sql', 'r', encoding='utf-8') as f:
            sql = f.read()

        # Split and execute statements
        for statement in sql.split(';'):
            statement = statement.strip()
            if statement and not statement.startswith('--') and not statement.startswith('/*'):
                if "SELECT 'Migrations" not in statement:
                    try:
                        cursor.execute(statement)
                    except Error as e:
                        if "already exists" not in str(e):
                            print(f"   Warning: {e}")

        connection.commit()
        print("   ✅ Migrations completed")

    except FileNotFoundError:
        print("   ⚠️  Migration file not found, skipping")


def insert_sample_data(connection, cursor):
    """Insert sample data into tables"""
    # Check if data exists
    cursor.execute("SELECT COUNT(*) FROM organizations")
    count = cursor.fetchone()[0]

    if count > 0:
        print("   Data already exists, skipping")
        return

    # Generate UUIDs
    org_id = str(uuid.uuid4()).replace('-', '')
    tenant_id = str(uuid.uuid4()).replace('-', '')
    user1_id = str(uuid.uuid4()).replace('-', '')
    user2_id = str(uuid.uuid4()).replace('-', '')

    # Insert organization
    print("   Creating organization...")
    cursor.execute(
        "INSERT INTO organizations (id, name, domain, status) VALUES (%s, %s, %s, 'active')",
        (org_id, "Acme Corporation", "acme.com")
    )

    # Insert tenant
    print("   Creating tenant...")
    cursor.execute(
        "INSERT INTO tenants (id, organization_id, name, slug, status) VALUES (%s, %s, %s, %s, 'active')",
        (tenant_id, org_id, "Acme Production", "acme-prod")
    )

    # Insert users
    print("   Creating users...")
    password_hash = "$argon2id$v=19$m=19456,t=2,p=1$test$hash"

    for user_id, email in [(user1_id, "admin@acme.com"), (user2_id, "user@acme.com")]:
        cursor.execute(
            "INSERT INTO users (id, email, password_hash, status) VALUES (%s, %s, %s, 'active')",
            (user_id, email, password_hash)
        )

        cursor.execute(
            "INSERT INTO user_tenants (user_id, tenant_id, status) VALUES (%s, %s, 'active')",
            (user_id, tenant_id)
        )

    connection.commit()
    print("   ✅ Sample data inserted")


def verify_data(cursor):
    """Verify data in tables"""
    tables = ["organizations", "tenants", "users", "user_tenants",
              "roles", "permissions", "refresh_tokens", "revoked_tokens"]

    for table in tables:
        cursor.execute(f"SELECT COUNT(*) FROM {table}")
        count = cursor.fetchone()[0]
        print(f"   {table}: {count} rows")

    # Show sample users
    cursor.execute("SELECT email, status FROM users LIMIT  3")
    users = cursor.fetchall()

    if users:
        print("\n   Sample Users:")
        for email, status in users:
            print(f"   - {email} ({status})")


if __name__ == "__main__":
    main()
