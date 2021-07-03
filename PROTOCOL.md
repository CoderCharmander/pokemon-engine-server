Protocol description
====================

This Pokémon-like dragon battle simulator uses a stateful protocol both
over HTTP and WebSocket.

Basic concept
-------------
The idea of this system is originated around _rooms_, virtual places where
chatting and battle can take place. When a client first joins, it is placed
into the _main room_, where battles can **NOT** take place. Other rooms may
be created, and the first two users who join can have a battle.

Most of the time, messages are sent to exactly one room (even chat in the
main room can not be seen in other ones), but there are exceptions.

Secondary (not main) rooms are identified by a five-character, all-uppercase,
alphanumeric string called the Room ID. This ID may be used to connect to
the room.

Secondary rooms are destroyed if there are no people in the room, but are kept intact
for at least 1 minute after creation, to give the client that created the room
time to join.

Message structure
-----------------
All messages, including HTTP responses and WebSocket messages both from the
client and the server are in the same, JSON-based format.
```json
{"action":"ACTION","data":{DATA}}
```
<!-- Or as a JSON schema:
```json
{
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "type": "object",
    "properties": {
        "action": {
            "description": "Specifies a performed action or an event",
            "type": "string"
        },
        "data": {
            "description": "Extra data connected to the action or event.",
            "type": "object"
        }
    },
    "required": [ "action", "data" ]
}
``` -->
The `action` field is a string that specifies an action a client has performed
or an event that has happened on the server, or triggered by an action of another
client.

The `data` field may be null, and it specifies additional data that is neccessary
to correctly react to the action.

## HTTP endpoints

### `/health` - Check server health
`/health` is exceptional: it does not provide a message in a specific format,
but if the server is up, it should always return an HTTP 200 OK result code.

### `/register-room` - Create a new room
`/register-room` returns a message in this format:
```json
{
    "action": "room_created",
    "data": {
        "room_id": "ABC12"
    }
}
```
`room_id` is a randomly generated, but unique Room ID. It can be used to join
the newly created room, but it is deleted after one minute - so clients should
immediately join the new room.

## WebSocket endpoints

### `/echo/<username>` - main room

This is the WebSocket endpoint to the main room. No battles can be started here.