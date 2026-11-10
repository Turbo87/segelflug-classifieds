FROM ghcr.io/home-assistant/base:latest

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
