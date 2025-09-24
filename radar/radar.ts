import express from "express";
import { WebSocket, WebSocketServer } from "ws";
import path from "path";
import http from "http";
import { v4 } from "uuid";

const app = express();
const PORT = 6346;

app.use(express.static(path.join(__dirname, "public")));

const server = http.createServer(app);

const ws = new WebSocketServer({ server, path: "/" });

const games: Record<string, { data: any; last_update: number }> = {};
const viewers: Record<string, { ws: WebSocket; gameUuid: string; lastSeen: number; clientId: string }> = {};
const gameServers: Record<string, WebSocket> = {};

ws.on("connection", (ws, req) => {
    console.info(`connection from ${req.socket.remoteAddress}`);

    (ws as any)._connectionTime = Date.now();
    (ws as any)._remoteAddress = req.socket.remoteAddress;

    ws.on("message", (message) => {
        try {
            const data = JSON.parse(message.toString());
            const kind: string = data["kind"];

            if (kind == "connect_server") {
                connect_server(ws, data);
            } else if (kind == "update_data") {
                update_data(data);
            } else if (kind == "get_data") {
                get_data(ws, data);
            } else if (kind == "register_viewer") {
                register_viewer(ws, data);
            }
        } catch (error) {
            console.error("error parsing message:", error);
        }
    });

    ws.on("close", () => {
        cleanup_connection(ws);
    });

    ws.on("error", (error) => {
        console.error(`websocket error from ${(ws as any)._remoteAddress}:`, error);
        cleanup_connection(ws);
    });
});

function cleanup_connection(ws: WebSocket) {
    let gameUuidToUpdate: string | null = null;

    for (const [viewerId, viewer] of Object.entries(viewers)) {
        if (viewer.ws === ws) {
            gameUuidToUpdate = viewer.gameUuid;
            delete viewers[viewerId];
            break;
        }
    }

    for (const [gameUuid, gameWs] of Object.entries(gameServers)) {
        if (gameWs === ws) {
            delete gameServers[gameUuid];
            delete games[gameUuid];

            for (const [viewerId, viewer] of Object.entries(viewers)) {
                if (viewer.gameUuid === gameUuid) {
                    delete viewers[viewerId];
                }
            }
            break;
        }
    }

    if (gameUuidToUpdate && games[gameUuidToUpdate]) {
        broadcastViewerCount(gameUuidToUpdate);
    }
}

function connect_server(ws: WebSocket, data: any) {
    const uuid = data["uuid"];
    games[uuid] = { data: {}, last_update: Date.now() };
    gameServers[uuid] = ws;

    const message = { kind: "accept" };
    ws.send(JSON.stringify(message));

    console.info(`game server connected with uuid: ${uuid}`);
}

function register_viewer(ws: WebSocket, data: any) {
    const gameUuid = data["uuid"];
    const clientId = data["client_id"] || `${(ws as any)._remoteAddress}_${Date.now()}`;

    if (!gameUuid || !(gameUuid in games)) {
        console.warn(`viewer tried to register for non-existent game: ${gameUuid}`);
        ws.close(404, "game not found");
        return;
    }

    for (const [existingId, viewer] of Object.entries(viewers)) {
        if (viewer.gameUuid === gameUuid && viewer.clientId === clientId && viewer.ws.readyState !== WebSocket.OPEN) {
            delete viewers[existingId];
        }
    }

    const viewerId = v4();
    viewers[viewerId] = {
        ws: ws,
        gameUuid: gameUuid,
        lastSeen: Date.now(),
        clientId: clientId,
    };

    console.log(`viewer registered for game ${gameUuid}`);
    broadcastViewerCount(gameUuid);
}

function update_data(data: any) {
    const uuid = data["uuid"];
    if (uuid in games) {
        const gameData = { ...data };
        delete gameData.kind;
        delete gameData.uuid;

        games[uuid].data = gameData;
        games[uuid].last_update = Date.now();
    }
}

function get_data(ws: WebSocket, data: any) {
    const uuid = data["uuid"];
    if (uuid in games) {
        const message = JSON.stringify(games[uuid].data);
        ws.send(message);
    } else {
        console.log(`uuid ${uuid} not found`);
        ws.send(JSON.stringify({}));
    }
}

function broadcastViewerCount(gameUuid: string) {
    const viewerCount = getViewerCount(gameUuid);

    const message = JSON.stringify({
        kind: "viewer_count",
        count: viewerCount,
    });

    const gameViewers = Object.values(viewers).filter(
        (viewer) => viewer.gameUuid === gameUuid && viewer.ws.readyState === WebSocket.OPEN
    );

    // console.log(`broadcasting viewer count ${viewerCount} to ${gameViewers.length} viewers for game ${gameUuid}`);

    gameViewers.forEach((viewer) => {
        try {
            viewer.ws.send(message);
        } catch (error) {
            console.error("error sending viewer count: ", error);
        }
    });
}

function getViewerCount(gameUuid: string): number {
    return Object.values(viewers).filter(
        (viewer) => viewer.gameUuid === gameUuid && viewer.ws.readyState === WebSocket.OPEN
    ).length;
}

setInterval(() => {
    const now = Date.now();
    let cleanedGames = 0;
    let cleanedViewers = 0;

    for (const [uuid, game] of Object.entries(games)) {
        if (now - game.last_update > 1_200_000) {
            delete games[uuid];
            delete gameServers[uuid];
            cleanedGames++;

            for (const [viewerId, viewer] of Object.entries(viewers)) {
                if (viewer.gameUuid === uuid) {
                    delete viewers[viewerId];
                    cleanedViewers++;
                }
            }
        }
    }

    for (const [viewerId, viewer] of Object.entries(viewers)) {
        if (viewer.ws.readyState !== WebSocket.OPEN) {
            const gameUuid = viewer.gameUuid;
            delete viewers[viewerId];
            cleanedViewers++;

            if (gameUuid in games) {
                broadcastViewerCount(gameUuid);
            }
        }
    }

    if (cleanedGames > 0 || cleanedViewers > 0) {
        console.log(`cleaned ${cleanedGames} games viewers`);
    }
}, 10000);

setInterval(() => {
    const now = Date.now();
    let updatedCount = 0;

    for (const [viewerId, viewer] of Object.entries(viewers)) {
        if (viewer.ws.readyState === WebSocket.OPEN) {
            viewer.lastSeen = now;
            updatedCount++;
        }
    }
}, 30000);

setInterval(() => {
    for (const gameUuid of Object.keys(games)) {
        broadcastViewerCount(gameUuid);
    }
}, 60000);

server.listen(PORT, () => {
    console.info(`server listening on port ${PORT}`);
});
