#!/bin/bash

# Set up colors for better readability
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Build the project
echo -e "${BLUE}Building the project...${NC}"
cargo build --release

# Set up storage directory
STORAGE_DIR="storage"
rm -rf "$STORAGE_DIR"
mkdir -p "$STORAGE_DIR"

# Create an alias for the command
MINI_TECTONIC="./target/release/mini-tectonic-rs -s $STORAGE_DIR"

# Create test files with different content
echo -e "${BLUE}\nCreating test files...${NC}"
echo "This is file 1 - will remain untampered" > test_file1.txt
echo "This is file 2 - will be tampered with" > test_file2.txt

# Register a tenant for testing
echo -e "${BLUE}\n1. Registering tenant 'checksum_test'...${NC}"
$MINI_TECTONIC register-tenant -t checksum_test

# Store both files
echo -e "${BLUE}\n2. Storing both test files...${NC}"

echo -e "\n${BLUE}Storing untampered file (test_file1.txt):${NC}"
echo "Original checksum of test_file1.txt:"
ORIGINAL_CHECKSUM1=$(shasum -a 256 test_file1.txt)
echo "$ORIGINAL_CHECKSUM1"
BLOB_ID1=$($MINI_TECTONIC put -t checksum_test -f test_file1.txt | grep -o 'ID: .*' | cut -d' ' -f2)
echo "Blob ID 1: $BLOB_ID1"

echo -e "\n${BLUE}Storing file that will be tampered (test_file2.txt):${NC}"
echo "Original checksum of test_file2.txt:"
ORIGINAL_CHECKSUM2=$(shasum -a 256 test_file2.txt)
echo "$ORIGINAL_CHECKSUM2"
BLOB_ID2=$($MINI_TECTONIC put -t checksum_test -f test_file2.txt | grep -o 'ID: .*' | cut -d' ' -f2)
echo "Blob ID 2: $BLOB_ID2"

# Show initial metadata for both files
echo -e "${BLUE}\n3. Viewing stored metadata for both files:${NC}"
$MINI_TECTONIC list-blobs -t checksum_test

# Demonstrate successful verification (untampered file)
echo -e "${BLUE}\n4. Testing untampered file retrieval and verification...${NC}"
$MINI_TECTONIC get -t checksum_test -b "$BLOB_ID1" -o retrieved_file1.txt

echo "Checksum of retrieved untampered file:"
RETRIEVED_CHECKSUM1=$(shasum -a 256 retrieved_file1.txt)
echo "$RETRIEVED_CHECKSUM1"

if [ "$ORIGINAL_CHECKSUM1" = "$RETRIEVED_CHECKSUM1" ]; then
    echo -e "${GREEN}✓ Untampered file verification successful - checksums match!${NC}"
else
    echo -e "${RED}✗ Unexpected: checksums don't match for untampered file!${NC}"
fi

# Demonstrate failed verification (tampered file)
echo -e "${BLUE}\n5. Testing tampered file detection...${NC}"
echo -e "${RED}Tampering with second file...${NC}"
echo "Modified malicious content" > "$STORAGE_DIR/chunks/$BLOB_ID2.blob"

echo "Attempting to retrieve tampered file..."
if $MINI_TECTONIC get -t checksum_test -b "$BLOB_ID2" -o retrieved_file2.txt; then
    echo -e "${RED}✗ Warning: Retrieved tampered file without detection${NC}"
else
    echo -e "${GREEN}✓ Success: Checksum verification caught the tampering!${NC}"
fi

# Compare original vs tampered checksums
echo -e "\n${BLUE}6. Comparing checksums:${NC}"
echo -e "Original File (untampered):"
echo "  Original:  $ORIGINAL_CHECKSUM1"
echo "  Retrieved: $RETRIEVED_CHECKSUM1"

echo -e "\nTampered File:"
echo "  Original:  $ORIGINAL_CHECKSUM2"
if [ -f retrieved_file2.txt ]; then
    TAMPERED_CHECKSUM=$(shasum -a 256 retrieved_file2.txt)
    echo "  Tampered:  $TAMPERED_CHECKSUM"
else
    echo "  Tampered:  [File retrieval blocked due to checksum mismatch]"
fi

# Show final state of blobs
echo -e "${BLUE}\n7. Final state of all stored blobs:${NC}"
$MINI_TECTONIC list-blobs -t checksum_test

# Cleanup
echo -e "${BLUE}\n8. Cleaning up...${NC}"
rm -f test_file1.txt test_file2.txt retrieved_file1.txt retrieved_file2.txt

echo -e "${GREEN}\nDemo completed successfully!${NC}" 