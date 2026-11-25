FROM ghcr.io/home-assistant/base:latest@sha256:5021aac8a4b7f2a1397e23a9749dfe9f736463ffa471df6b200216c68d530fe6

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
