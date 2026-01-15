#!/bin/bash
# Development script for the SSO platform

set -e

echo "Starting development environment..."

# Set development environment
export AUTH__ENVIRONMENT=development

# Check if .env file exists
if [ ! -f .env ]; then
    echo "Creating .env file from template..."
    cp .env.example .env
    echo "Please update .env file with your configuration"
fi

# Start the application in development mode
echo "Starting SSO Platform in development mode..."
cargo run