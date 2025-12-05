#!/usr/bin/env bash

# ============================================================================
# AUR PKGBUILD Generator Script for LazyLora
# ============================================================================
#
# This script generates compliant PKGBUILD files for both source and binary
# variants of LazyLora, following AUR submission guidelines.
#
# Author: Altynbek Orumbayev <aorumbayev@pm.me>
# Repository: https://github.com/aorumbayev/lazylora
# License: MIT
#
# Usage:
#   ./generate-aur-pkgbuild.sh [OPTIONS]
#
# Options:
#   -v, --version VERSION    Override version (default: extract from Cargo.toml)
#   -o, --output DIR         Output directory (default: ./aur-pkgbuilds)
#   -s, --source-only        Generate only source PKGBUILD
#   -b, --binary-only        Generate only binary PKGBUILD
#   -f, --force              Force overwrite existing files
#   --validate               Validate generated PKGBUILDs (requires makepkg)
#   --no-checksums           Skip checksum calculation (for testing)
#   -h, --help               Show this help message
#
# Examples:
#   ./generate-aur-pkgbuild.sh                    # Generate both variants
#   ./generate-aur-pkgbuild.sh -v 1.0.0          # Use specific version
#   ./generate-aur-pkgbuild.sh -s -o /tmp/aur    # Source only to /tmp/aur
#   ./generate-aur-pkgbuild.sh --validate        # Generate and validate
#
# Generated Files:
#   - PKGBUILD-source: Source package (lazylora)
#   - PKGBUILD-bin: Binary package (lazylora-bin)
#   - .SRCINFO-source: Source metadata
#   - .SRCINFO-bin: Binary metadata
#
# ============================================================================

set -Eeuo pipefail

# Script directory detection
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DEFAULT_OUTPUT_DIR="$PROJECT_ROOT/aur-pkgbuilds"

# LazyLora specific configuration
readonly REPO_URL="https://github.com/aorumbayev/lazylora"
readonly BINARY_NAME="lazylora"
readonly SOURCE_PKGNAME="lazylora"
readonly BINARY_PKGNAME="lazylora-bin"
readonly MAINTAINER="Altynbek Orumbayev <aorumbayev@pm.me>"
readonly PKG_DESCRIPTION="Unofficial terminal user interface for Algorand Blockchain exploration"
readonly PKG_LICENSE="MIT"

# Runtime configuration
VERSION=""
OUTPUT_DIR="$DEFAULT_OUTPUT_DIR"
GENERATE_SOURCE=true
GENERATE_BINARY=true
FORCE_OVERWRITE=false
VALIDATE_PKGBUILDS=false
CALCULATE_CHECKSUMS=true

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly CYAN='\033[0;36m'
readonly BOLD='\033[1m'
readonly NC='\033[0m'

# ============================================================================
# Logging Functions
# ============================================================================

log_info() {
    printf '%b%s%b %s\n' "${BLUE}" "INFO:" "${NC}" "$*" >&2
}

log_success() {
    printf '%b✓%b %s\n' "${GREEN}" "${NC}" "$*" >&2
}

log_warning() {
    printf '%b%s%b %s\n' "${YELLOW}" "WARNING:" "${NC}" "$*" >&2
}

log_error() {
    printf '%b%s%b %s\n' "${RED}" "ERROR:" "${NC}" "$*" >&2
}

log_step() {
    printf '%b→%b %s...\n' "${CYAN}" "${NC}" "$*" >&2
}

log_header() {
    printf '\n%b%s%b\n' "${BOLD}" "$*" "${NC}" >&2
    printf '%s\n' "$(printf '=%.0s' {1..60})" >&2
}

# ============================================================================
# Help Message
# ============================================================================

show_help() {
    cat << 'EOF'
AUR PKGBUILD Generator for LazyLora

This script generates compliant PKGBUILD files for both source and binary
variants of LazyLora, following AUR submission guidelines.

USAGE:
    generate-aur-pkgbuild.sh [OPTIONS]

OPTIONS:
    -v, --version VERSION    Override version (default: extract from Cargo.toml)
    -o, --output DIR         Output directory (default: ./aur-pkgbuilds)
    -s, --source-only        Generate only source PKGBUILD
    -b, --binary-only        Generate only binary PKGBUILD
    -f, --force              Force overwrite existing files
    --validate               Validate generated PKGBUILDs (requires makepkg)
    --no-checksums           Skip checksum calculation (for testing)
    -h, --help               Show this help message

EXAMPLES:
    generate-aur-pkgbuild.sh                    Generate both variants
    generate-aur-pkgbuild.sh -v 1.0.0          Use specific version
    generate-aur-pkgbuild.sh -s -o /tmp/aur    Source only to /tmp/aur
    generate-aur-pkgbuild.sh --validate        Generate and validate

GENERATED FILES:
    PKGBUILD-source         Source package (lazylora)
    PKGBUILD-bin           Binary package (lazylora-bin)
    .SRCINFO-source        Source package metadata
    .SRCINFO-bin           Binary package metadata

AUR COMPLIANCE FEATURES:
    ✓ Follows AUR package guidelines and naming conventions
    ✓ Proper metadata including pkgname, pkgver, pkgrel, pkgdesc, arch, license
    ✓ Automatic version extraction from Cargo.toml
    ✓ Dynamic checksum calculation for source and binary packages
    ✓ Support for x86_64 architecture
    ✓ Proper maintainer information and comments
    ✓ Generated .SRCINFO files for AUR submission

ARCHITECTURE SUPPORT:
    x86_64                  Full support for both source and binary packages

For more information, see: https://wiki.archlinux.org/title/AUR_submission_guidelines
EOF
}

# ============================================================================
# Argument Parsing
# ============================================================================

parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--version)
                if [[ -z "${2:-}" ]]; then
                    log_error "Option $1 requires an argument"
                    exit 1
                fi
                VERSION="$2"
                shift 2
                ;;
            -o|--output)
                if [[ -z "${2:-}" ]]; then
                    log_error "Option $1 requires an argument"
                    exit 1
                fi
                OUTPUT_DIR="$2"
                shift 2
                ;;
            -s|--source-only)
                GENERATE_SOURCE=true
                GENERATE_BINARY=false
                shift
                ;;
            -b|--binary-only)
                GENERATE_SOURCE=false
                GENERATE_BINARY=true
                shift
                ;;
            -f|--force)
                FORCE_OVERWRITE=true
                shift
                ;;
            --validate)
                VALIDATE_PKGBUILDS=true
                shift
                ;;
            --no-checksums)
                CALCULATE_CHECKSUMS=false
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            --)
                shift
                break
                ;;
            -*)
                log_error "Unknown option: $1"
                log_info "Use --help to see available options"
                exit 1
                ;;
            *)
                log_error "Unexpected argument: $1"
                log_info "Use --help to see available options"
                exit 1
                ;;
        esac
    done
}

# ============================================================================
# Dependency Checking
# ============================================================================

check_dependencies() {
    log_step "Checking dependencies"

    local missing_deps=()

    for cmd in curl sha256sum; do
        if ! command -v "$cmd" &>/dev/null; then
            missing_deps+=("$cmd")
        fi
    done

    # Check for optional validation tools
    if [[ "$VALIDATE_PKGBUILDS" == true ]] && ! command -v makepkg &>/dev/null; then
        log_error "makepkg is required for validation but not found"
        log_info "Install base-devel group: pacman -S base-devel"
        exit 1
    fi

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        exit 1
    fi

    log_success "All dependencies found"
}

# ============================================================================
# Version Extraction
# ============================================================================

extract_version() {
    log_step "Extracting version information"

    if [[ -n "$VERSION" ]]; then
        log_info "Using provided version: $VERSION"
        return 0
    fi

    local cargo_toml="$PROJECT_ROOT/Cargo.toml"
    if [[ ! -f "$cargo_toml" ]]; then
        log_error "Cargo.toml not found at: $cargo_toml"
        exit 1
    fi

    VERSION=$(grep '^version = ' "$cargo_toml" | head -n1 | sed 's/version = "\(.*\)"/\1/')

    if [[ -z "$VERSION" ]]; then
        log_error "Could not extract version from Cargo.toml"
        exit 1
    fi

    # Validate version format (semantic versioning)
    if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([+-][a-zA-Z0-9.-]+)?$ ]]; then
        log_error "Invalid version format: $VERSION"
        log_info "Expected semantic versioning format (e.g., 1.0.0, 1.0.0-beta.1)"
        exit 1
    fi

    log_success "Version: $VERSION"
}

# ============================================================================
# Checksum Calculation
# ============================================================================

calculate_source_checksum() {
    if [[ "$CALCULATE_CHECKSUMS" != true ]]; then
        printf '%s' "SKIP"
        return 0
    fi

    log_step "Calculating source checksum"

    local tarball_url="${REPO_URL}/archive/refs/tags/v${VERSION}.tar.gz"

    local checksum
    if ! checksum=$(curl -fsSL "$tarball_url" 2>/dev/null | sha256sum | cut -d' ' -f1); then
        log_warning "Failed to download source tarball (release may not exist yet)"
        printf '%s' "PLACEHOLDER_SOURCE_CHECKSUM"
        return 0
    fi

    if [[ -z "$checksum" ]] || [[ ${#checksum} -ne 64 ]]; then
        log_warning "Invalid SHA256 checksum calculated"
        printf '%s' "PLACEHOLDER_SOURCE_CHECKSUM"
        return 0
    fi

    printf '%s' "$checksum"
}

calculate_binary_checksum() {
    local arch="$1"

    if [[ "$CALCULATE_CHECKSUMS" != true ]]; then
        printf '%s' "SKIP"
        return 0
    fi

    log_step "Calculating binary checksum ($arch)"

    local binary_url="${REPO_URL}/releases/download/v${VERSION}/${BINARY_NAME}-${arch}-unknown-linux-gnu.tar.gz"

    local checksum
    if ! checksum=$(curl -fsSL "$binary_url" 2>/dev/null | sha256sum | cut -d' ' -f1); then
        log_warning "Failed to download binary package: $arch"
        log_warning "This might be expected if the release doesn't exist yet"
        printf '%s' "PLACEHOLDER_$(printf '%s' "$arch" | tr '[:lower:]' '[:upper:]')_CHECKSUM"
        return 0
    fi

    if [[ -z "$checksum" ]] || [[ ${#checksum} -ne 64 ]]; then
        log_warning "Invalid SHA256 checksum calculated for $arch"
        printf '%s' "PLACEHOLDER_$(printf '%s' "$arch" | tr '[:lower:]' '[:upper:]')_CHECKSUM"
        return 0
    fi

    printf '%s' "$checksum"
}

# ============================================================================
# Output Directory Setup
# ============================================================================

setup_output_directory() {
    log_step "Setting up output directory"

    if [[ -d "$OUTPUT_DIR" ]]; then
        if [[ "$FORCE_OVERWRITE" != true ]]; then
            log_error "Output directory already exists: $OUTPUT_DIR"
            log_info "Use --force to overwrite existing files"
            exit 1
        fi
        rm -rf -- "$OUTPUT_DIR"
    fi

    mkdir -p -- "$OUTPUT_DIR"
    log_success "Output directory: $OUTPUT_DIR"
}

# ============================================================================
# Source PKGBUILD Generation
# ============================================================================

generate_source_pkgbuild() {
    log_step "Generating source PKGBUILD"

    local output_file="$OUTPUT_DIR/PKGBUILD-source"
    local source_checksum

    source_checksum=$(calculate_source_checksum)

    cat > "$output_file" << EOF
# Maintainer: $MAINTAINER
#
# This is the source package for LazyLora, which builds the application
# from source using the Rust toolchain. For a binary package (pre-compiled),
# see lazylora-bin.

pkgname=$SOURCE_PKGNAME
pkgver=$VERSION
pkgrel=1
pkgdesc="$PKG_DESCRIPTION"
arch=('x86_64')
url="$REPO_URL"
license=('$PKG_LICENSE')
makedepends=('rust' 'cargo')
source=("\${pkgname}-\${pkgver}.tar.gz::${REPO_URL}/archive/refs/tags/v\${pkgver}.tar.gz")
sha256sums=('$source_checksum')

prepare() {
    cd "\${srcdir}/\${pkgname}-\${pkgver}"
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "\$CARCH-unknown-linux-gnu"
}

build() {
    cd "\${srcdir}/\${pkgname}-\${pkgver}"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release
}

check() {
    cd "\${srcdir}/\${pkgname}-\${pkgver}"
    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen --release
}

package() {
    cd "\${srcdir}/\${pkgname}-\${pkgver}"
    install -Dm0755 -t "\$pkgdir/usr/bin/" "target/release/$BINARY_NAME"
    install -Dm0644 LICENSE "\$pkgdir/usr/share/licenses/\$pkgname/LICENSE"
    install -Dm0644 README.md "\$pkgdir/usr/share/doc/\$pkgname/README.md"
}
EOF

    log_success "Source PKGBUILD: $output_file"
}

# ============================================================================
# Binary PKGBUILD Generation
# ============================================================================

generate_binary_pkgbuild() {
    log_step "Generating binary PKGBUILD"

    local output_file="$OUTPUT_DIR/PKGBUILD-bin"
    local x86_64_checksum

    x86_64_checksum=$(calculate_binary_checksum "x86_64")

    cat > "$output_file" << EOF
# Maintainer: $MAINTAINER
#
# This is the binary package for LazyLora, which provides pre-compiled
# binaries for faster installation. For building from source, see lazylora.

pkgname=$BINARY_PKGNAME
pkgver=$VERSION
pkgrel=1
pkgdesc="$PKG_DESCRIPTION (binary package)"
arch=('x86_64')
url="$REPO_URL"
license=('$PKG_LICENSE')
provides=('$SOURCE_PKGNAME')
conflicts=('$SOURCE_PKGNAME')
source_x86_64=("\${pkgname%-bin}-\${pkgver}-x86_64.tar.gz::${REPO_URL}/releases/download/v\${pkgver}/${BINARY_NAME}-x86_64-unknown-linux-gnu.tar.gz")
sha256sums_x86_64=('$x86_64_checksum')

package() {
    # Install binary
    install -Dm0755 "\$srcdir/$BINARY_NAME" "\$pkgdir/usr/bin/$BINARY_NAME"

    # Install documentation and license files required by AUR guidelines
    install -Dm0644 "\$srcdir/LICENSE" "\$pkgdir/usr/share/licenses/\$pkgname/LICENSE"
    install -Dm0644 "\$srcdir/README.md" "\$pkgdir/usr/share/doc/\$pkgname/README.md"
}
EOF

    log_success "Binary PKGBUILD: $output_file"
}

# ============================================================================
# .SRCINFO Generation
# ============================================================================

generate_srcinfo() {
    local pkgbuild_type="$1" # "source" or "bin"
    local pkgbuild_file="$OUTPUT_DIR/PKGBUILD-$pkgbuild_type"
    local srcinfo_file="$OUTPUT_DIR/.SRCINFO-$pkgbuild_type"

    log_step "Generating .SRCINFO ($pkgbuild_type)"

    if [[ ! -f "$pkgbuild_file" ]]; then
        log_error "PKGBUILD file not found: $pkgbuild_file"
        return 1
    fi

    # Check if makepkg is available for proper .SRCINFO generation
    if command -v makepkg &>/dev/null; then
        local temp_dir
        temp_dir=$(mktemp -d)

        cp -- "$pkgbuild_file" "$temp_dir/PKGBUILD"

        local makepkg_success=false
        if (cd "$temp_dir" && makepkg --printsrcinfo) > "$srcinfo_file" 2>/dev/null; then
            makepkg_success=true
        fi

        rm -rf -- "$temp_dir"

        if [[ "$makepkg_success" == true ]]; then
            log_success ".SRCINFO ($pkgbuild_type): $srcinfo_file"
            return 0
        fi

        log_info "makepkg failed, falling back to manual generation"
    fi

    # Fallback: Generate .SRCINFO manually
    log_info "Generating .SRCINFO manually"

    if [[ "$pkgbuild_type" == "source" ]]; then
        generate_source_srcinfo_manual "$srcinfo_file"
    else
        generate_binary_srcinfo_manual "$srcinfo_file"
    fi

    log_success ".SRCINFO ($pkgbuild_type): $srcinfo_file"
}

generate_source_srcinfo_manual() {
    local output_file="$1"
    local source_checksum

    source_checksum=$(calculate_source_checksum)

    cat > "$output_file" << EOF
pkgbase = $SOURCE_PKGNAME
	pkgdesc = $PKG_DESCRIPTION
	pkgver = $VERSION
	pkgrel = 1
	url = $REPO_URL
	arch = x86_64
	license = $PKG_LICENSE
	makedepends = rust
	makedepends = cargo
	source = $SOURCE_PKGNAME-$VERSION.tar.gz::${REPO_URL}/archive/refs/tags/v$VERSION.tar.gz
	sha256sums = $source_checksum

pkgname = $SOURCE_PKGNAME
EOF
}

generate_binary_srcinfo_manual() {
    local output_file="$1"
    local x86_64_checksum

    x86_64_checksum=$(calculate_binary_checksum "x86_64")

    cat > "$output_file" << EOF
pkgbase = $BINARY_PKGNAME
	pkgdesc = $PKG_DESCRIPTION (binary package)
	pkgver = $VERSION
	pkgrel = 1
	url = $REPO_URL
	arch = x86_64
	license = $PKG_LICENSE
	provides = $SOURCE_PKGNAME
	conflicts = $SOURCE_PKGNAME
	source_x86_64 = $BINARY_PKGNAME-$VERSION-x86_64.tar.gz::${REPO_URL}/releases/download/v$VERSION/${BINARY_NAME}-x86_64-unknown-linux-gnu.tar.gz
	sha256sums_x86_64 = $x86_64_checksum

pkgname = $BINARY_PKGNAME
EOF
}

# ============================================================================
# PKGBUILD Validation
# ============================================================================

validate_pkgbuilds() {
    log_step "Validating PKGBUILDs"

    if ! command -v makepkg &>/dev/null; then
        log_warning "makepkg not available, skipping validation"
        return 0
    fi

    local validation_failed=false

    if [[ "$GENERATE_SOURCE" == true ]]; then
        if validate_single_pkgbuild "$OUTPUT_DIR/PKGBUILD-source"; then
            log_success "Source PKGBUILD validation passed"
        else
            log_error "Source PKGBUILD validation failed"
            validation_failed=true
        fi
    fi

    if [[ "$GENERATE_BINARY" == true ]]; then
        if validate_single_pkgbuild "$OUTPUT_DIR/PKGBUILD-bin"; then
            log_success "Binary PKGBUILD validation passed"
        else
            log_error "Binary PKGBUILD validation failed"
            validation_failed=true
        fi
    fi

    if [[ "$validation_failed" == true ]]; then
        log_error "PKGBUILD validation failed"
        exit 1
    fi
}

validate_single_pkgbuild() {
    local pkgbuild_file="$1"
    local temp_dir
    local result

    temp_dir=$(mktemp -d)

    cp -- "$pkgbuild_file" "$temp_dir/PKGBUILD"

    if (cd "$temp_dir" && makepkg --printsrcinfo >/dev/null 2>&1); then
        result=0
    else
        result=1
    fi

    rm -rf -- "$temp_dir"

    return $result
}

# ============================================================================
# Summary Generation
# ============================================================================

generate_summary() {
    log_header "AUR PKGBUILD Generation Complete"

    cat << EOF

Package Information:
  • Source Package: $SOURCE_PKGNAME
  • Binary Package: $BINARY_PKGNAME
  • Version: $VERSION
  • License: $PKG_LICENSE
  • Architecture: x86_64

Generated Files:
EOF

    if [[ "$GENERATE_SOURCE" == true ]]; then
        printf '  • %-20s %s\n' "PKGBUILD-source" "(Source package PKGBUILD)"
        printf '  • %-20s %s\n' ".SRCINFO-source" "(Source package metadata)"
    fi

    if [[ "$GENERATE_BINARY" == true ]]; then
        printf '  • %-20s %s\n' "PKGBUILD-bin" "(Binary package PKGBUILD)"
        printf '  • %-20s %s\n' ".SRCINFO-bin" "(Binary package metadata)"
    fi

    cat << EOF

Output Directory: $OUTPUT_DIR

Next Steps:
  1. Review generated PKGBUILD files for correctness
  2. Test build locally:
     cd $OUTPUT_DIR
     cp PKGBUILD-source PKGBUILD && makepkg -si --nocheck
  3. For AUR submission:
     - Create AUR account at https://aur.archlinux.org
     - Clone the AUR repository: git clone ssh://aur@aur.archlinux.org/$SOURCE_PKGNAME.git
     - Copy PKGBUILD and .SRCINFO to the repository
     - Commit and push changes

Documentation:
  • AUR Submission Guidelines: https://wiki.archlinux.org/title/AUR_submission_guidelines
  • PKGBUILD Reference: https://wiki.archlinux.org/title/PKGBUILD

EOF
}

# ============================================================================
# Main Function
# ============================================================================

main() {
    log_header "AUR PKGBUILD Generator for LazyLora"

    parse_arguments "$@"
    check_dependencies
    extract_version
    setup_output_directory

    if [[ "$GENERATE_SOURCE" == true ]]; then
        generate_source_pkgbuild
        generate_srcinfo "source"
    fi

    if [[ "$GENERATE_BINARY" == true ]]; then
        generate_binary_pkgbuild
        generate_srcinfo "bin"
    fi

    if [[ "$VALIDATE_PKGBUILDS" == true ]]; then
        validate_pkgbuilds
    fi

    generate_summary

    log_success "PKGBUILD generation completed successfully"
}

main "$@"
