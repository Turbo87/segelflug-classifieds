ARG BUILD_FROM
FROM $BUILD_FROM

# Install curl for downloading the binary
RUN apk add --no-cache curl

# Copy run script
COPY run.sh /
RUN chmod a+x /run.sh

# Labels
LABEL \
    io.hass.name="segelflug-classifieds" \
    io.hass.description="Segelflug.de Kleinanzeigen Telegram Bot" \
    io.hass.arch="armhf|aarch64|armv7" \
    io.hass.type="addon" \
    io.hass.version="1.0.0"

CMD [ "/run.sh" ]
