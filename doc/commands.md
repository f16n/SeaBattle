Commands to be given from the client 

CLI command structure game client
===================================================
Global arguments (can be set using env vars):
--server <fqdn:port>    Server to talk to: default http://localhost:5555)
--user <userID>       Authenticating user 
--password <pwdHash>        Password of authentication user 
---------------------------------------------------
game <gameID>
--new [--opponent <userID>] --default        create new game optionally against an existing user or against 
                                             the server (--default sets gameID env var)
--join                                       joining an existing waiting game
--show                                       show an existing gameboard
--list [--user [<userID>]][--waiting]        list existing games, optionally filtered by user and/or waiting state
--shoot <xy>                                 fire a shot in a game
--ship <ship> --place <xy> --orientation <o> place a ship on a location
--start                                      start game after placement of ships
--status                                     show game status
--default                                    set game as default game (sets gameID env var)

server
--highscores                                 show server highscore
--status                                     show server or game status

----------------- admin role only -----------------
user <userID>
--create --password <pwdHash> --role <role>  create a new user  
--delete                                     delete (deactivates) a user
--update --password <pwdHash> --role <role>  change password or role of a user

game <gameID> 
--delete                                     delete a game
===================================================


Server arguments
===================================================
--access                                     clients allowed from (partial fqdn, network segment)
--admin                                      initial admin user
--password                                   initial admin user password
===================================================