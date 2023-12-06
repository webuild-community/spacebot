var chart = document.getElementById("scoreboard");

// fetch rooms list
function fetchRooms() {
  const isLocalServer = window.location.host.indexOf('localhost') !== -1;
  const protocol = isLocalServer ? 'http://' : 'https://';
  fetch(`${protocol}${window.location.host}/rooms`)
    .then(response => response.json())
    .then(data => drawRooms(data));
}
fetchRooms();

function sanitizeHTML(text) {
  var element = document.createElement('div');
  element.innerText = text;
  return element.innerHTML;
}

function drawRooms(rooms) {
  var tableHtml = "<tbody>";
  for (const room of rooms) {
    const room_id = room.id;
    const room_token = room.token;
    const room_name = sanitizeHTML(room.name);
    const max_players = room.max_players;

    tableHtml += `
            <tr>
              <td class="room-id">${room_id}</td>
              <td class="room-name"><a href="/room?room_token=${room_token}">${room_name}</a></td>
              <td class="room-players">Max players: ${max_players}</td>
            </tr>`;
  }
  chart.innerHTML = tableHtml + "</tbody>";
}