root: <server>/api/v1/

>login                   GET <root>/login   (curl -X GET --user '<user>:<pwd}' https://<root>/login)              

Retrieve highscore      GET <root>/server/highscore             (table per unique combi of boardsize and player count)
                        returns:
                        Status: 200
                        Total-Count: x (max 10)
                        Next-Page: false
                        Content-Type: application/json
                        [
                            {
                                "playerId": "{userID}",
                                "highScore": "{score}",
                                "datetime": "{datetimeFinihed}"
                            },
                            {
                                "playerId": "{userID}",
                                "highScore": "{score}",
                                "datetime": "{datetimeFinihed}"
                            }
                        ]
Retrieve server status  GET <root>/server/status
                        return: {
                                    "uptime":"{datetime}",
                                    "totalGames":"{#}",
                                    "totalPlayers":"{#}",
                                    "activeGames":"{#}"
                                    "motd":"{message}"
                                }
>Set motd                POST <root>/server/motd                            (admin role only)
#Clean shutdown          POST <root>/server/shutdown                        (after all games finished, admin role only)
#Dirty shutdown          POST <root>/server/kill                            (admin role only)

/user                   collection
signup                   POST    <root>/signup                              (possible without login)
                            {
                                "user_name":"{username}",
                                "password":"{password}"
                                "display_name":"{display name}",
                                "email_address":"{email address}",
                                "notify":{boolean}                          (true/false)
                            }
>New user                POST    <root>/user                                (admin role only)
                            {
                                "user_name":"{username}",
                                "password":"{password}",
                                "display_name":{display name},
                                "email_address":"{email address}",
                                "admin_role":"{boolean}",                   (true/flase)
                                "user_active":"{boolean},                   (true/false)
                                "notify":{boolean}
                            }
>Change password         PUT  <root>/user/:{username}/password
                        {
                            "new":"{pwdhash}"
                        }
>Update user             POST  <root>/user/:{username}                         (admin role only)
>Show user               GET   <root>/user/:{username}                         (admin role only)

/game                   collection
Create game             POST    <root>/game
Add Server              POST    <root>/game/server                              (server is one of the players)
Join game               POST    <root>/game/:<gameID>
Game status             GET     <root>/game/:<gameID>/status
List games              GET     <root>/game/[?player={UserID}][?status={Status}]
Start game              POST    <root>/game/:<gameID>/?status=start
Delete game             DELETE  <root>/game/:<gameID>                           (admin role only -> deactivate user)

/players                subcollection
New player              POST    <root>/game/:<gameID>/players
Retrieve players        GET     <root>/game/:<gameID>/players
Retrieve player         GET     <root>/game/:<gameID>/players/<id>
Retrieve board          GET     <root>/game/:<gameId>/players/<id>/board
Get players score       GET     <root>/game/:<gameID>/players/<id>/score
Set players score       POST    <root>/game/:<gameID>/players/<id>/score

/ships                  subsubcollection
Place a ship            POST    <root>/game/:<id>/player/:<id>/ship
                        {
                            coordinates:{xy},
                            orientation:{o},
                            type:{t},
                        }
Retrieve ships          GET     <root>/game/:<id>/player/:<id>/ship
Retrieve ship           GET     <root>/game/:<id>/player/:<id>/ship/:<id>

/shots                  collection
Fire a shot             POST    <root>/game/:<id>/player/:<id>/shot
                        {
                            coordinates={xy}
                        }
Fired shots             -> Retrieve board