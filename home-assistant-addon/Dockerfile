FROM ghcr.io/home-assistant/base:latest@sha256:58422480a299b16b6ea376c2167463206d430619ed02b40cd687c8bd6875519e

# Install curl for downloading the binary
RUN apk add --no-cache curl

# Copy run script
COPY run.sh /
RUN chmod a+x /run.sh

# Labels
LABEL \
    io.hass.name="segelflug-classifieds" \
    io.hass.description="Segelflug.de Kleinanzeigen Telegram Bot" \
    io.hass.arch="aarch64|amd64" \
    io.hass.type="addon" \
    io.hass.version="1.1.0"

CMD [ "/run.sh" ]
