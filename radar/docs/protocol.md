# Protocol

## Game to Server

Endpoint: `/server`

1. S -> G: `{"uuid": "..."}`
2. G -> S: `{"players": {...}}`

## Client to Server

Endpoint: `/client`

1. C -> S: `{"uuid": "..."}`
2. S -> C: `{"players": {...}}`
3.
