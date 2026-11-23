FROM postgres:16-alpine3.24 AS patched
RUN apk upgrade --no-cache \
    && apk add --no-cache su-exec \
    && ln -sf /sbin/su-exec /usr/local/bin/gosu

# Drop the replaced binary from the exported image layers.
FROM scratch
COPY --from=patched / /
COPY --chmod=0755 entrypoint.sh /usr/local/bin/postal-postgres-entrypoint.sh
COPY --chmod=0755 mark-volume.sh /docker-entrypoint-initdb.d/zz-postal-volume.sh
ENV PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
    LANG=en_US.utf8 \
    PGDATA=/var/lib/postgresql/data
VOLUME ["/var/lib/postgresql/data"]
EXPOSE 5432
STOPSIGNAL SIGINT
ENTRYPOINT ["/usr/local/bin/postal-postgres-entrypoint.sh"]
CMD ["postgres"]
