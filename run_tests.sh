#!/bin/bash

# UBI Chain Test Runner
# This script runs all tests for the UBI blockchain project

set -e  # Exit on any error

# Print header
echo "====================================="
echo "UBI Chain Test Suite"
echo "====================================="
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Function to run tests with proper formatting
run_test() {
    local package=$1
    local description=$2
    
    echo -e "${YELLOW}Running $description tests...${NC}"
    
    if cargo test -p $package; then
        echo -e "${GREEN}✓ $description tests passed${NC}"
        echo
        return 0
    else
        echo -e "${RED}✗ $description tests failed${NC}"
        echo
        return 1
    fi
}

# Function to run integration tests
run_integration_tests() {
    echo -e "${YELLOW}Running integration tests...${NC}"
    
    if cargo test --test '*'; then
        echo -e "${GREEN}✓ Integration tests passed${NC}"
        echo
        return 0
    else
        echo -e "${RED}✗ Integration tests failed${NC}"
        echo
        return 1
    fi
}

# Function to run shell-based integration test
run_shell_integration_test() {
    echo -e "${YELLOW}Running shell-based integration test...${NC}"
    
    if [ -f "tests/integration_test.sh" ]; then
        if tests/integration_test.sh; then
            echo -e "${GREEN}✓ Shell integration test passed${NC}"
            echo
            return 0
        else
            echo -e "${RED}✗ Shell integration test failed${NC}"
            echo
            return 1
        fi
    else
        echo -e "${YELLOW}Shell integration test script not found, skipping${NC}"
        echo
        return 0
    fi
}

# Check code formatting
echo -e "${YELLOW}Checking code formatting...${NC}"
if cargo fmt --all -- --check; then
    echo -e "${GREEN}✓ Code formatting is correct${NC}"
else
    echo -e "${RED}✗ Code formatting issues detected${NC}"
    echo -e "Run 'cargo fmt --all' to fix formatting issues"
fi
echo

# Run clippy for code quality
echo -e "${YELLOW}Running clippy for code quality...${NC}"
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo -e "${GREEN}✓ No clippy warnings${NC}"
else
    echo -e "${RED}✗ Clippy warnings detected${NC}"
fi
echo

# Initialize counters
PASSED=0
FAILED=0

# Run tests for each package
if run_test ubi-chain-runtime "Runtime"; then
    ((PASSED++))
else
    ((FAILED++))
fi

if run_test ubi-chain-rpc "RPC"; then
    ((PASSED++))
else
    ((FAILED++))
fi

if run_test ubi-chain-node "Node"; then
    ((PASSED++))
else
    ((FAILED++))
fi

# Run integration tests if they exist
if [ -d "tests" ]; then
    if run_integration_tests; then
        ((PASSED++))
    else
        ((FAILED++))
    fi
    
    # Run shell-based integration test
    if run_shell_integration_test; then
        ((PASSED++))
    else
        ((FAILED++))
    fi
fi

# Run doc tests
echo -e "${YELLOW}Running documentation tests...${NC}"
if cargo test --doc; then
    echo -e "${GREEN}✓ Documentation tests passed${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ Documentation tests failed${NC}"
    ((FAILED++))
fi
echo

# Summary
echo "====================================="
echo -e "${YELLOW}Test Summary:${NC}"
echo "====================================="
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo

# Exit with appropriate code
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed successfully!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed. Please check the output above.${NC}"
    exit 1
fi 