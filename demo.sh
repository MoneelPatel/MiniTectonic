#!/bin/bash

# Build the project
cargo build --release

# Set up storage directory
STORAGE_DIR="storage"
rm -rf "$STORAGE_DIR"
mkdir -p "$STORAGE_DIR"

# Create an alias for the command
MINI_TECTONIC="./target/release/mini-tectonic-rs -s $STORAGE_DIR"

echo "1. Registering tenant 'posts'..."
$MINI_TECTONIC register-tenant -t posts

echo -e "\n2. Creating a test post..."
echo "This is a test post content" > test_post.txt

echo -e "\n3. Storing the test post..."
BLOB_ID=$($MINI_TECTONIC put -t posts -f test_post.txt | grep -o 'ID: .*' | cut -d' ' -f2)

echo -e "\n4. Listing blobs for tenant 'posts'..."
$MINI_TECTONIC list-blobs -t posts

echo -e "\n5. Retrieving the blob..."
$MINI_TECTONIC get -t posts -b "$BLOB_ID" -o retrieved_post.txt

echo -e "\n6. Comparing original and retrieved content:"
echo "Original content:"
cat test_post.txt
echo -e "\nRetrieved content:"
cat retrieved_post.txt

echo -e "\n7. Deleting the blob..."
$MINI_TECTONIC delete -t posts -b "$BLOB_ID"

echo -e "\n8. Verifying deletion..."
$MINI_TECTONIC list-blobs -t posts 