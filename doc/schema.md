Table: games
- gameId
- datetimeCreated
- datetimeStarted
- datetimeFinished
- status: (waiting / placing / ready / player1 / player2 / finished / aborted)

Table: boards
- gameId
- playerId
- grid
- shots ??? needed ???
- score

Table: accounts
- userId
- pwdHash
- role  (admin / player)
- state (active / inactive)

Table: server
- motd
- key