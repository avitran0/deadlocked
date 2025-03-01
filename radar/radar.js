import express from "express";
import expressWs from "express-ws";
import path from "path";
import { fileURLToPath } from "url";
import { v4 as uuidV4 } from "uuid";

const PORT = 6460;

const app = express();
expressWs(app);

const filename = fileURLToPath(import.meta.url);
const dirname = path.dirname(filename);

/** @type {Record<string, Object>} */
const games = {};

app.get("/", (req, res) => {
    res.sendFile(path.join(dirname, "dist/index.html"));
});

app.ws("/", (ws, req) => {
    console.info("new websocket connection established");

    ws.on("message", (message) => {
        // console.info(`received websocket message: ${message}`);
        const content = JSON.parse(message);

        if (content["type"] === "server") {
            // server setup
            const id = uuidV4();
            games[id] = {};
            ws.send(id);
        } else if (content["type"] === "data") {
            // server data
            const id = content["uuid"];
            if (id in games) {
                games[id] = content["data"] ?? {};
            }
        } else if (content["type"] === "client") {
            // client data request
            const id = content["uuid"];
            if (id && id in games) {
                const out = JSON.stringify(games[id]);
                ws.send(out);
            } else {
                ws.send("{}");
            }
        }
    });

    ws.on("close", () => {
        console.info("closing connection");
    });
});

app.use(express.static("dist"));

app.listen(PORT, () => {
    console.info(`server started on port ${PORT}`);
});
