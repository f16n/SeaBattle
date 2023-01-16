Game is designed with the option to change the rules of the game in various ways:
- board-size
- number of players
- number of ships (and the size of each ship)
the schema is build to make these changes possible without schema updates
============================================
How to encode ships in the database?
nr of ships: x (usually 5, but we could extend...)
per ship:
- pos (x,y)
- length (x) -> max 8
- direction (H/V) (or N/S/E/W for future game rules. It might be interesting to know in which direction a ship is moving)
- damage (bitmap) -> stored in a u8, UNSIGNED TINYINT

vec of ships

ship struct {
    pos
    len
    dir
    damage
}


=========================================================================================
there is always a special user who's name is server.name

when a game is created by a user, the game status is 'active' and the user_status = 'placing'

when a user is joining, the amount_of_players is increased by one and the user_status = 'placing' and the player_id = amount_of_players.

when a user is done placing, the user_status = 'waiting'
when all users are done placing, the game status is 'active' and the user_status of a random user is set to 'shooting'
when a user shoots, the following things happen:
- Shots placed by changing the damage on each ship
- New score calculated and changed
- If all ships of a player have sunk, the players state transitions to "lost"
- the player's state transistions to "waiting"
- If the state of all other players is "lost", the state transistions to "won" and the game transistions to "finished"
- if the state of the game is "active" the next player is transistioned to "shooting".

selection of next player:
- if current player_id is amount_of_players -> next player_id = 1 else next player_id = current player_id + 1
- if state of next_player_id <> "waiting" select next player

when a user aborts
- the game status is 'aborted'

when the server is added as a player
- user.name = server.name
- user added to game
======================================================================================
Sign-up

Anyone can sign-up if the server admin chooses to enable this feature using the environment variable "SIGNUP" has the value "YES".

Unauthenticated signups are then possible providing username, password, displayname, email addres and notification preference.
The user will be a normal user (not admin) and inactive by default
The password needs to comply with rules: min 8, max 16, etc.
A verification number is generated and stored in the user record and send to the given email address.

Verification
the sent verification number must match the stored verification number
user is made active and verification number is cleared and new_password_hash is moved to password_hash and cleared
======================================================================================
Change Password (any user)

Need to be logged in and the old and new password need to be provided in the json body
old password is checked
The password needs to comply with rules: min 8, max 16, etc.
A verification number is generated and stored in the user record and send to the given email address.
the new password is stored in new_password_hash
Note that in this case the user is already active as he can already be playing games and some imposter tries to change his password to take over the account.

===============================================================================================
add user (ADMIN)

Create full user with all fields from input.
Password will be hashed but does not need to comply to rules.

================================================================================================
When deploying a server

when the DB is created it is prepopulated with 2 users:
- self: used for the player that is the server itself
- admin: The initial admin account
Both accounts have an initial password of "$u64R:$4creT" (stored as a hash in the DB)
The admin account has the admin role and can be used for admin tasks (and playing)


