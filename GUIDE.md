# Guide

 - The guiding focus to how to write a client to connect to a Tokyo server, receive information events and create actions to control your ship

## 1. Connection information

`ws://${host}/socket?key={key}&name{name}`

| Parameters | Description |
|--|--|
| {host} | Domain or IP of server want to connect |
| {key} | User's identity is unique and used to distinguish bots |
| {name} | The display name will be shown on UI |

When the WebSocket connection is established successfully, your bot is registered, displayed on web UI and ready to use.

## 2. Action commands

Through WebSocket, a client can send action commands to control their ship.

### 2.1. Rotate the ship

```json
{"e": "rotate", "data": 0.5}
```

| Fields | Description |
|--|--|
| e | Event information "rotate" |
| data | Set the radian value that the ship will head. Value's between [0, 2π] |

![Radian example](https://flylib.com/books/3/315/1/html/2/images/figu345_1.jpg)

### 2.2. Throttle

Set the speed of your ship. Your ship starts to move forward with a new speed.
Set 0 to stop your ship.

```json
{"e": "throttle", "data": 0.5}
```

| Fields | Description |
|--|--|
| e | Event information "throttle" |
| data | Speed value. It's between [0, 1] |

### 2.3. Fire a bullet

```json
{"e": "fire"}
```

| Fields | Description |
|--|--|
| e | Event information "fire" |

## 3. Events

From WebSocket, the server consecutively sends events to the client every tick with the following structure.
Event message structure contains all world info of the game, including players, bullets, dead person, etc.

### 3.1. Events structure

#### 3.1.1. State event

State event contains world map states that includes map info, players, bullets, dead and scoreboard

```json
{
  "e": "state",
  "data": {
    "bounds": [1200.0, 800.0],
    "players": [
      {},
      {}
    ],
    "bullets": [
      {},
      {}
    ],
    "dead": [
      "respawn":{
         "secs_since_epoch":1631299999,
         "nanos_since_epoch":940876053
       },
       "player": {}
    ],
    "scoreboard":{"0":100,"1":90,"2":80}
  }
}
```

| Fields | Description |
|--|--|
| e | State event is is always "state" |
| data | Detail data of event "e" |
| bounds | Boundary of the game, players spawn and navigate their ship in boundary from position [0,0] to this max size boundary. It's an array with two values, width and height |
| players | List of players/ships in the game currently. Detail of the player object will be described in the next sections |
| bullets | List of bullets that's fired by ships in the game currently. Detail of bullet object will be described in the next sections |
| dead | List of dead users and the respawn periods. Information of player is a structure with "players" |
| scoreboard | Top user scores with format "player_id: score" |

#### 3.1.2. Current user event

Event contains current user id

```json
{"e":"id","data":247}
```

| Fields | Description |
|--|--|
| e | State event is is always "id" |
| data | id of current user |

#### 3.1.3. User event

Event contains all users and their ids in the game.

```json
{
   "e":"teamnames",
   "data":{
      "60":"thich_khanh_ngoc",
      "178":"z",
      "206":"chicken_killer",
      "222":"tuan",
      "161":"don't kill me",
      "240":"coward_dog",
      "244":"do_anh_bat_duoc_em",
      "134":"thich_khanh_ngoc",
      "154":"tuan",
      "175":"hieuk09",
      "176":"z",
      "173":"don't kill me",
      "229":"tuan",
   }
}
```

| Fields | Description |
|--|--|
| e | State event is is always "teamnames" |
| data | Hash map of id-name of users |

### 3.2. Player structure

```json
{
   "id":0,
   "angle":9.350119,
   "throttle":1.0,
   "x":579.5356,
   "y":118.02286
},
```

| Fields | Description |
|--|--|
| id | Player/ship's identify (ID) |
| angle | Angle of the ship is heading. Radian value's between [0, 2π] |
| throttle | Throttle or speed of the ship. 0 = no speed, 1 = max speed |
| x, y | Ship's position |

### 3.3. Bullet structure

```json
{
   "id":564,
   "player_id":1,
   "angle":8.630102,
   "x":1013.78644,
   "y":312.22202
}
```

| Fields | Description |
|--|--|
| id | Bullet's identify (ID) |
| player_id | Identify the ship that fires this bullet |
| angle | Angle of the bullet is heading. It will move forward |
| x, y | Bullet's position |


## 4. Others

 - Number of ticks per second: 30
 - Dead waiting: 1 second
 - Max concurrent bullet per user: 4
 - Bullet's radius: 2
 - Player's radius: 10

## 5. Real example

### 5.1. Event from server to client

```json
{
  "e":"state",
  "data":{
    "bounds":[2000.0,2000.0],
    "players":[
      {"id":2,"angle":3.7490678,"throttle":0.0,"x":235.03761,"y":146.60205},
      {"id":0,"angle":318.01907,"throttle":1.0,"x":10.0,"y":426.38736},
      {"id":1,"angle":317.3284,"throttle":1.0,"x":10.0,"y":788.89844}
    ],
    "dead":[],
    "bullets":[],
    "scoreboard":{"0":100,"1":90,"2":80}
  }
}
```

```json
{
   "e":"state",
   "data":{
      "bounds":[
         1200.0,
         800.0
      ],
      "players":[
         {
            "id":0,
            "angle":9.350119,
            "throttle":1.0,
            "x":579.5356,
            "y":118.02286
         },
         {
            "id":1,
            "angle":8.690104,
            "throttle":1.0,
            "x":1085.2515,
            "y":235.13806
         }
      ],
      "dead":[
         
      ],
      "bullets":[
         {
            "id":564,
            "player_id":1,
            "angle":8.630102,
            "x":1013.78644,
            "y":312.22202
         },
         {
            "id":565,
            "player_id":0,
            "angle":9.290117,
            "x":475.8172,
            "y":135.10085
         },
         {
            "id":566,
            "player_id":1,
            "angle":8.660103,
            "x":1038.9604,
            "y":280.79645
         },
         {
            "id":567,
            "player_id":0,
            "angle":9.320118,
            "x":514.9718,
            "y":125.70969
         },
         {
            "id":568,
            "player_id":1,
            "angle":8.690104,
            "x":1074.1208,
            "y":245.19324
         },
         {
            "id":569,
            "player_id":0,
            "angle":9.350119,
            "x":564.5774,
            "y":119.1417
         }
      ],
      "scoreboard":{
         "0":11,
         "1":10
      }
   }
}
```

```json
{
   "e":"state",
   "data":{
      "bounds":[
         2000.0,
         2000.0
      ],
      "players":[
         {
            "id":1,
            "angle":1.7171067,
            "throttle":1.0,
            "x":442.45584,
            "y":1257.3445
         },
         {
            "id":0,
            "angle":1.4245026,
            "throttle":1.0,
            "x":482.95755,
            "y":249.16882
         },
         {
            "id":18,
            "angle":6.4407263,
            "throttle":0.0,
            "x":577.8954,
            "y":893.48627
         }
      ],
      "dead":[
         {
            "respawn":{
               "secs_since_epoch":1631300003,
               "nanos_since_epoch":109227250
            },
            "player":{
               "id":15,
               "angle":2.6952791,
               "throttle":0.0,
               "x":562.2551,
               "y":723.3329
            }
         }
      ],
      "bullets":[
         {
            "id":3762,
            "player_id":0,
            "angle":0.47937274,
            "x":840.3276,
            "y":335.62506
         },
         {
            "id":3766,
            "player_id":15,
            "angle":4.12334,
            "x":255.71953,
            "y":46.337547
         },
         {
            "id":3767,
            "player_id":0,
            "angle":1.4245025,
            "x":490.97507,
            "y":303.58133
         },
         {
            "id":3768,
            "player_id":0,
            "angle":1.4245025,
            "x":486.60187,
            "y":273.90176
         }
      ],
      "scoreboard":{
         "3":65,
         "0":327,
         "2":53,
         "15":9,
         "18":6,
         "19":3,
         "1":90,
         "12":3,
         "11":6,
         "13":3,
         "17":7,
         "16":1
      }
   }
}
```
