#!/bin/sh

set -eu

REPO_API_URL_DEFAULT="https://api.github.com/repos/AI-Eden/eden-skills/releases/latest"
REPO_RELEASE_BASE_URL_DEFAULT="https://github.com/AI-Eden/eden-skills/releases/download"

info() {
    printf '%s\n' "$*"
}

warn() {
    printf 'warning: %s\n' "$*" >&2
}

die() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        die "Required command not found: $1"
    fi
}

uname_s() {
    if [ -n "${EDEN_SKILLS_TEST_UNAME_S:-}" ]; then
        printf '%s\n' "$EDEN_SKILLS_TEST_UNAME_S"
        return
    fi
    uname -s
}

uname_m() {
    if [ -n "${EDEN_SKILLS_TEST_UNAME_M:-}" ]; then
        printf '%s\n' "$EDEN_SKILLS_TEST_UNAME_M"
        return
    fi
    uname -m
}

resolve_target() {
    os_name=$(uname_s)
    arch_name=$(uname_m)

    case "${os_name}:${arch_name}" in
        Linux:x86_64)
            printf '%s\n' "x86_64-unknown-linux-gnu"
            ;;
        Linux:aarch64 | Linux:arm64)
            printf '%s\n' "aarch64-unknown-linux-gnu"
            ;;
        Darwin:x86_64)
            printf '%s\n' "x86_64-apple-darwin"
            ;;
        Darwin:aarch64 | Darwin:arm64)
            printf '%s\n' "aarch64-apple-darwin"
            ;;
        *)
            die "Unsupported platform: ${os_name} ${arch_name}"
            ;;
    esac
}

normalize_version() {
    case "$1" in
        v*)
            printf '%s\n' "${1#v}"
            ;;
        *)
            printf '%s\n' "$1"
            ;;
    esac
}

download_text() {
    curl -fsSL "$1"
}

download_file() {
    url="$1"
    output_path="$2"
    if ! curl -fsSL "$url" -o "$output_path"; then
        die "Failed to download ${url}"
    fi
}

resolve_version() {
    if [ -n "${EDEN_SKILLS_VERSION:-}" ]; then
        normalize_version "$EDEN_SKILLS_VERSION"
        return
    fi

    api_url="${EDEN_SKILLS_RELEASE_API_URL:-$REPO_API_URL_DEFAULT}"
    latest_json=$(download_text "$api_url") || die "Failed to query ${api_url}"
    tag_name=$(printf '%s' "$latest_json" | tr -d '\n' | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')

    if [ -z "$tag_name" ]; then
        die "Failed to resolve the latest release version from ${api_url}"
    fi

    normalize_version "$tag_name"
}

sha256_file() {
    file_path="$1"

    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file_path" | awk '{print $1}'
        return
    fi

    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$file_path" | awk '{print $1}'
        return
    fi

    if command -v openssl >/dev/null 2>&1; then
        openssl dgst -sha256 "$file_path" | awk '{print $NF}'
        return
    fi

    die "No SHA-256 tool available. Install sha256sum, shasum, or openssl."
}

verify_sha256() {
    archive_path="$1"
    checksums_path="$2"
    archive_name="$3"

    expected_hash=$(awk -v archive="$archive_name" '$2 == archive { print $1; exit }' "$checksums_path")
    if [ -z "$expected_hash" ]; then
        die "Checksum entry not found for ${archive_name}"
    fi

    actual_hash=$(sha256_file "$archive_path")
    if [ "$actual_hash" != "$expected_hash" ]; then
        die "SHA-256 mismatch for ${archive_name}"
    fi
}

shell_rc_file() {
    case "${SHELL:-}" in
        */zsh)
            printf '%s\n' "~/.zshrc"
            ;;
        */bash)
            printf '%s\n' "~/.bashrc"
            ;;
        *)
            printf '%s\n' "~/.profile"
            ;;
    esac
}

path_contains() {
    case ":${PATH:-}:" in
        *:"$1":*)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

print_path_hint() {
    install_dir="$1"
    rc_file=$(shell_rc_file)

    info ""
    info "Add ${install_dir} to your PATH by appending this line to ${rc_file}:"
    if [ "$install_dir" = "${HOME}/.eden-skills/bin" ]; then
        info 'export PATH="$HOME/.eden-skills/bin:$PATH"'
    else
        info "export PATH=\"${install_dir}:\$PATH\""
    fi
}

main() {
    if [ -z "${HOME:-}" ]; then
        die "HOME must be set before running this installer."
    fi

    require_command curl
    require_command tar

    target=$(resolve_target)
    version=$(resolve_version)
    release_base_url="${EDEN_SKILLS_RELEASE_BASE_URL:-$REPO_RELEASE_BASE_URL_DEFAULT}"
    install_dir="${EDEN_SKILLS_INSTALL_DIR:-${HOME}/.eden-skills/bin}"
    archive_name="eden-skills-${version}-${target}.tar.gz"
    checksums_name="eden-skills-${version}-checksums.txt"
    archive_url="${release_base_url}/v${version}/${archive_name}"
    checksums_url="${release_base_url}/v${version}/${checksums_name}"
    temp_dir=$(mktemp -d 2>/dev/null || mktemp -d -t eden-skills-install)
    extract_dir="${temp_dir}/extract"
    archive_path="${temp_dir}/${archive_name}"
    checksums_path="${temp_dir}/${checksums_name}"
    installed_binary="${install_dir}/eden-skills"

    cleanup() {
        rm -rf "$temp_dir"
    }
    trap cleanup EXIT HUP INT TERM

    info "Detected target: ${target}"
    info "Resolved version: ${version}"
    info "Downloading ${archive_name}"
    download_file "$archive_url" "$archive_path"
    info "Downloading ${checksums_name}"
    download_file "$checksums_url" "$checksums_path"
    verify_sha256 "$archive_path" "$checksums_path" "$archive_name"

    mkdir -p "$extract_dir" || die "Failed to create extraction directory"
    if ! tar -xzf "$archive_path" -C "$extract_dir"; then
        die "Failed to extract ${archive_name}"
    fi

    if [ ! -f "${extract_dir}/eden-skills" ]; then
        die "Archive did not contain the eden-skills binary"
    fi

    mkdir -p "$install_dir" || die "Failed to create install directory ${install_dir}"
    cp "${extract_dir}/eden-skills" "$installed_binary" || die "Failed to install eden-skills into ${install_dir}"
    chmod +x "$installed_binary" 2>/dev/null || true

    info "Installed eden-skills ${version} to ${install_dir}"

    if ! command -v git >/dev/null 2>&1; then
        warn "Git was not found in PATH. Source sync commands require git."
    fi

    if ! "$installed_binary" --version; then
        die "Installed binary failed verification."
    fi

    if ! path_contains "$install_dir"; then
        print_path_hint "$install_dir"
    fi
}

main "$@"
