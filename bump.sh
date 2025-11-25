#!/bin/bash
# Version bump script for tvcode

CARGO_TOML="Cargo.toml"

# Get current version
CURRENT=$(grep '^version = ' "$CARGO_TOML" | sed 's/version = "\(.*\)"/\1/')
echo "Current version: $CURRENT"

# Parse version components
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

# Determine bump type
case "${1:-patch}" in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    patch|*)
        PATCH=$((PATCH + 1))
        ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"
echo "New version: $NEW_VERSION"

# Update Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"

echo "✅ Updated $CARGO_TOML"
echo ""
echo "Next steps:"
echo "  cargo build --release"
echo "  sudo cp target/release/tvcode /usr/local/bin/"
echo "  git add -A && git commit -m \"Bump version to $NEW_VERSION\""
