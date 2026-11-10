#!/usr/bin/with-contenv bashio

export RUST_LOG="segelflug_classifieds=info"
export TELEGRAM_TOKEN=$(bashio::config 'telegram_token')
export SENTRY_DSN=$(bashio::config 'sentry_dsn')

# Create data directory and change to it for working directory
mkdir -p "/data"
cd /data

# Binary download with fallback logic
BINARY_PATH="/data/segelflug-classifieds"

case "$(uname -m)" in
    x86_64)
        TARGET="x86_64-unknown-linux-musl"
        ;;
    aarch64 | armv6l | armv7l | armv8l)
        TARGET="armv7-unknown-linux-musleabihf"
        ;;
    *)
        bashio::log.error "Unsupported architecture: $(uname -m)"
        exit 1
        ;;
esac

DOWNLOAD_URL="https://turbo87.github.io/segelflug-classifieds/${TARGET}/segelflug-classifieds"

bashio::log.info "Attempting to download latest binary..."

if curl --fail --silent --show-error --location --output "${BINARY_PATH}.tmp" "${DOWNLOAD_URL}"; then
    bashio::log.info "Successfully downloaded latest binary"
    mv "${BINARY_PATH}.tmp" "${BINARY_PATH}"
    chmod +x "${BINARY_PATH}"
elif [ -f "${BINARY_PATH}" ]; then
    bashio::log.warning "Download failed, using existing binary"
else
    bashio::log.error "Download failed and no existing binary found"
    exit 1
fi

# Run the application
bashio::log.info "Starting segelflug-classifieds bot..."
"${BINARY_PATH}" --watch
