const SHIP_SIZE = 18;
const BULLET_SIZE = 3;
var websocket_status = document.getElementById("websocket-status");
var chart = document.getElementById("scoreboard");
var c = document.getElementById("canvas");
window.onload = window.onresize = function () {
        c.width = document.body.clientWidth - 100; //document.width is obsolete
        c.height = document.body.clientHeight; //document.height is obsolete
}
var team_names = {};

var ctx = c.getContext("2d");

function connect(handler) {
        websocket_status.innerText = "connecting...";
        websocket_status.style.borderColor = "gray";
        const isLocalServer = window.location.host.indexOf('localhost') !== -1;
        const protocol = isLocalServer === -1 ? 'ws://' : 'ws://';
        const socket = new WebSocket(`${protocol}${window.location.host}/spectate`);
        socket.addEventListener('open', function (event) {
                websocket_status.innerText = "connected";
                websocket_status.style.borderColor = "white";
        });

        socket.addEventListener('close', function (event) {
                websocket_status.innerText = "disconnected";
                websocket_status.style.borderColor = "orange";
                setTimeout(function () {
                        connect(handler);
                }, 1000);
        });

        socket.addEventListener('error', function (event) {
                websocket_status.innerText = "error!";
                websocket_status.style.borderColor = "red";
                socket.close();
        });

        socket.addEventListener('message', function (event) {
                let json = JSON.parse(event.data);
                handler(json);
        });
}

class Ship {
        constructor(obj) {
                this.id = obj.id;
                this.x = Math.floor(obj.x);
                this.y = Math.floor(obj.y);
                this.angle = obj.angle;
        }

        move(x, y) {
                this.x = x;
                this.y = y;
        }

        rotate(theta) {
                this.angle = theta;
        }

        draw(ctx) {
                ctx.save()
                // orient the ship
                ctx.translate(this.x, this.y);
                ctx.rotate(this.angle - Math.PI / 2.0);


                // draw the ship triangle
                ctx.beginPath();
                ctx.moveTo(-SHIP_SIZE * 0.8, -SHIP_SIZE);
                ctx.lineTo(0, SHIP_SIZE);
                ctx.lineTo(SHIP_SIZE * 0.8, -SHIP_SIZE);
                ctx.lineTo(-SHIP_SIZE * 0.8, -SHIP_SIZE);
                ctx.fill();
                ctx.stroke();

                let oldFill = ctx.fillStyle;
                ctx.beginPath();
                ctx.arc(0, 0, 10, 0, 2 * Math.PI);
                ctx.fillStyle = "#e05d5d";
                ctx.fill();
                ctx.fillStyle = oldFill;

                // draw team name
                ctx.rotate(-this.angle + Math.PI / 2.0); // please don't ask me about this math
                oldFill = ctx.fillStyle;
                ctx.font = '32px monospace';
                ctx.textAlign = 'left';
                ctx.textBaseline = 'top';
                let textMeasurements = ctx.measureText(team_names[this.id]);
                ctx.fillStyle = "#000000";
                ctx.fillRect(17, -3, textMeasurements.width + 6, 15);
                ctx.fillStyle = "#ffffff";
                ctx.fillText(team_names[this.id], 20, 0);
                ctx.fillStyle = oldFill;

                // reset transformation
                ctx.restore();
        }
}

class Bullet {
        constructor(obj) {
                this.id = obj.id;
                this.player_id = obj.player_id;
                this.x = obj.x;
                this.y = obj.y;
                this.angle = obj.angle;
        }

        move(x, y) {
                this.x = x;
                this.y = y;
        }

        rotate(theta) {
                this.theta = theta;
        }

        draw(ctx) {
                ctx.save()
                ctx.translate(this.x, this.y);

                let oldFill = ctx.fillStyle;
                ctx.beginPath();
                ctx.arc(0, 0, BULLET_SIZE, 0, 2 * Math.PI);
                ctx.fillStyle = "#f9ca24";
                ctx.fill();
                ctx.fillStyle = oldFill;

                ctx.restore();
        }
}

var last_drawn_scoreboard = {};
var initCanvas = false;
connect(function (json) {
        if (json.e === "teamnames") {
                team_names = json.data;
        } else if (json.e === "state") {
                const data = json.data;
                ctx.save()
                ctx.clearRect(0, 0, c.width, c.height);
                ctx.strokeStyle = "#ffffff";
                ctx.lineWidth = 1;
                ctx.lineCap = "square";
                ctx.lineJoin = "bevel";

                scaleXRatio = c.width / data.bounds[0];
                scaleYRatio = c.height / data.bounds[1];
                scaleRatio = Math.min(scaleXRatio, scaleYRatio);
                ctx.transform(scaleRatio, 0, 0, scaleRatio, 0, 0);

                ctx.beginPath();
                ctx.moveTo(0, 0);
                ctx.lineTo(data.bounds[0], 0);
                ctx.lineTo(data.bounds[0], data.bounds[1]);
                ctx.lineTo(0, data.bounds[1]);
                ctx.lineTo(0, 0);
                ctx.stroke();

                for (const player of data.players) {
                        new Ship(player).draw(ctx);
                }

                for (const bullet of data.bullets) {
                        new Bullet(bullet).draw(ctx);
                }
                ctx.restore()

                if (JSON.stringify(data.scoreboard) !== JSON.stringify(last_drawn_scoreboard)) {
                        draw_scoreboard(data.scoreboard);
                        last_drawn_scoreboard = data.scoreboard;
                }
        }
});

function sanitizeHTML(text) {
  var element = document.createElement('div');
  element.innerText = text;
  return element.innerHTML;
}

function draw_scoreboard(scoreboard) {
        var sorted_players = Object.keys(scoreboard).sort(function (a, b) { return scoreboard[b] - scoreboard[a] });
        var tableHtml = "<tbody>";

        for (let i = 0; i < sorted_players.length; i++) {
                const player_id = sorted_players[i];
                const player_score = String(scoreboard[player_id]).padEnd(3);
                const team_name = sanitizeHTML(team_names[player_id]);
                tableHtml += `
            <tr class="rank-${i + 1}">
              <td class="rank">${i + 1}</td>
              <td class="name">${team_name}</td>
              <td class="score">${player_score}</td>
            </tr>`;
        }
        chart.innerHTML = tableHtml + "</tbody>";
}
