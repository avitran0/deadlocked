# Web Radar

## Using

simply enter a radar server url into the given text box, without protocol or port, like in the examples below.
after it connects you can click "Open" to open the page directly in your browser.

available servers:

- `relay.avitrano.com`
- `radar.disphost.cc`

the session uuid identifies your cheat session, so that other people can watch your game.
it can be reset at any time, but you will have to distribute new links with the new uuid if you do.

## Hosting

the server can be built with `cargo build --release --bin server`.
the resulting binary will be located in `target/release/server`.

make sure that port 6346 is open for tcp connections.
the server listens for http connections on port 6347.
make sure that you have a reverse proxy set up, which listens on port 443 and forwards that https traffic to port 6347.
the reverse proxy needs to forward `/client`, and optionally `/stats` (which just displays connection counts, for debugging).

to use it as a systemd service, simply add this as a service description:

```ini
[Unit]
Description=deadlocked relay server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=$YOUR_USER
Group=$YOUR_GROUP
WorkingDirectory=$DOESNT_MATTER
ExecStart=$PATH_TO_SERVER_BIN
Restart=always
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

i don't have experience with docker, but hosting it should also be very straight forward.
