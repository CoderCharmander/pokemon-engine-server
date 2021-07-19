Protocol description
====================

This Pokémon-like dragon battle simulator uses a stateful protocol both
over HTTP and WebSocket.

Basic concept
-------------
The idea of this system is originated around _rooms_, virtual places where
chatting and battles can take place. When a client first joins, it is placed
into the _main room_, where battles can **NOT** take place. Other rooms may be
created, and the first two users who join can have a battle.

Most of the time, messages are sent to exactly one room (even chat in the main
room can not be seen in other ones), but there are exceptions.

Secondary (not main) rooms are identified by a five-character, all-uppercase,
alphanumeric string called the Room ID. This ID may be used to connect to the
room.

Secondary rooms are destroyed if there are no people in the room, but are kept
intact for at least 1 minute after creation, to give the client that created the
room time to join.

Message structure
-----------------
All messages, including HTTP responses and WebSocket messages both from the
client and the server are in the same, JSON-based format.

```json
{"action":"ACTION","data":{DATA}}
```

The `action` field is a string that specifies an action a client has performed
or an event that has happened on the server, or triggered by an action of
another client.

The `data` field may be null, and it specifies additional data that is
neccessary to correctly react to the action.

HTTP endpoints
--------------

### `/health` - Check server health
`/health` is exceptional: it does not provide a message in a specific format,
but if the server is up, it should always return an HTTP 200 OK result code.

WebSocket endpoint
------------------

### `/echo/<username>` - main room

This is the main WebSocket endpoint. All real-time communication is done here.
The `<username>` parameter specifies the username the client will join with.

Connection flow
---------------

1. The client connects to the WebSocket endpoint at `/echo/<username>`, where
`<username>` is replaced with the username of the client.
2. The server either sends an `user_exists` or a `welcome` message to indicate
whether the client was successfully added to the main room.
3. The client may send any message that is allowed in the main room, and the
server will then respond accordingly.

Full message documentation
--------------------------

### Request errors

Request errors are a special kind of message, since they may indicate lots of
events. They are always sent from the server to the client making the erroneus
request.

**Data:**

```json
{
    "reason": "<short error description>"
}
```

### `welcome`

**Sent:** by the server, to all users in the affected room

**Data:**

```json
{
    "name": "<username>"
}
```

Sent after a client joins any room (including the main room, via connecting)
to the WebSocket endpoint, to all users in the room, also the one who just
joined.

### `user_exists`

**Sent:** by the server, to a connecting user

**Data:**

```json
{}
```

Sent if a client is already connected with the same username. The connection
is terminated after sending this message.

### `chat`

**Sent:** by the client

**Data:**

```json
{
    "msg": "<chat message>"
}
```

The `msg` attribute will be copied and sent to all clients in the same room
with a `chat_notify` message.

### `chat_notify`

**Sent:** by the server, to all users in the affected room

**Data:**

```json
{
    "msg": "<chat message>",
    "source_name": "<sender username>",
}
```

Sent after a client sends a `chat` message. Contains the `msg` attribute from
`chat` and also the username of the sender client.

### `create_room`

**Sent:** by the client

**Data:**

```json
{}
```

Used to create a room, which the client will automatically join. The server
shall indicate the newly created room ID with a `room_created` message.

### `room_created`

**Sent:** by the server, to the client requesting room creation

**Data:**

```json
{
    "room_id": "<room id>"
}
```

Sent after a room was created as per a `create_room` request. Contains a room
ID that can be sent to other clients and used to join.

### `join_room`

**Sent:** by the client

**Data:**

```json
{
    "room_id": "<room id>"
}
```

Requests to join a room that has the ID of `room_id`. The status will be indicated
in a `room_join_status` message.

### `room_join_status`

**Sent:** by the server, to the client requesting to join a room

**Data:**

```json
{
    "room_id": "<room id>",
    "succeeded": true
}
```

Sent as a response to a `join_room` request. If the room with the ID exists,
`succeeded` will be true, otherwise, it will be false. If `succeeded` is true,
the client immediately leaves the main room and is placed into the specified
room. If it is false, the client stays in the main room.

### `leave_room`

**Sent:** by the client

**Data:**

```json
{}
```

Requests to exit a room and be placed back into the main room. Any battles
the requesting client is in will be immediately terminated, and if the client
is the last one in the room, the room will be deleted.

If the client is in the main room, the `already_in_main_room` request error
is received.