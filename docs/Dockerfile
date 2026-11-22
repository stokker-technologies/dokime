# Dokime — SPDX-License-Identifier: Apache-2.0
# Serve the Assembled Current Entry and Published Release Lines.
FROM nginx:alpine@sha256:fb4658584bd9a916ffb6f6e365f97c25cfdb2923801f551ea08356e695f14ff7
ARG REVISION
LABEL org.opencontainers.image.source="https://github.com/stokker-technologies/dokime" \
      org.opencontainers.image.revision="${REVISION}"
COPY . /usr/share/nginx/html/
