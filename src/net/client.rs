use crate::chessmove::*;
use crate::game::Chess;
use crate::net::connection::connection::Connection;
use crate::net::frame::Frame;
use crate::net::Command;
use crate::net::*;
use crate::util::*;
use crate::{pieces::Piece, tile::Tile};

pub struct Client {
    pub chess: Option<Chess>,
    pub name: String,

    conn: Connection,
}

impl Client {
    pub fn new(name: String, conn: Connection) -> Client {
        Client {
            chess: None,
            name,
            conn,
        }
    }

    pub fn new_game(&mut self, new_game: NewGame) {
        info!(
            "New Game! (\"{}\" ({}) hoster side: {:?})",
            new_game.name, new_game.mode, new_game.hoster_side
        );
        self.chess = Some(Chess::new());
    }

    pub fn make_move(&mut self, chessmove: ChessMove) -> Vec<(Tile, Option<Piece>)> {
        if self.chess.is_none() {
            return vec![];
        }

        let changes = self.chess.as_mut().unwrap().make_move(chessmove);

        changes
    }

    pub fn join_game(&self, id: String) {
        info!("Join Game. id: {:?}", id)
    }
    pub async fn run(&mut self) {
        // TODO: Hand shake
        //        self.out_buf.write(&[1, 2, 3, 4]);
        //        self.in_buf.read();
        loop {
            // only reading the message, no further validation.
            // this blocks the task until a full message is available
            let frame: Option<Frame> = self.conn.buf.read_frame().await;
            let frame = if let Some(f) = frame {
                f
            } else {
                continue;
            };

            // now we interpret the message
            let cmd = if let Some(cmd) = frame.parse() {
                cmd
            } else {
                error!("{fg_red}invalid command received: !{fg_reset}");
                continue;
            };

            info!("{fg_green}Received command: {cmd}!{fg_reset}");

            // and execute the command
            let response = self.exec(cmd);
            // finally sent respond to client
            if !self.conn.buf.write(response).await {
                info!("{fg_red}sending command failed!: {fg_reset}");
            }
        }
    }

    pub fn exec(&mut self, cmd: Command) -> Frame {
        let null_frame = self.conn.create_frame();
        match cmd {
            Command::Nickname(name) => {
                self.name = name;
                Frame {
                    len: 0,
                    content: self.conn.buf.buf,
                }
            }
            Command::NewGame(new_game) => {
                self.new_game(new_game);
                Frame {
                    len: 0,
                    content: self.conn.buf.buf,
                }
            }
            Command::JoinGame(id) => {
                self.join_game(id);
                Frame {
                    len: 0,
                    content: self.conn.buf.buf,
                }
            }
            Command::Move(mov) => {
                if self.conn.is_ingame() {
                    let chessmove = if let Some(unpacked_mov) = mov.parse() {
                        unpacked_mov
                    } else {
                        warn!(
                            "cannot parse move: {style_bold}{fg_red}{}{style_reset}{fg_reset}",
                            mov
                        );
                        return null_frame;
                    };
                    let fields = self.make_move(chessmove);

                    self.conn.buf.fields_to_buffer(&fields);
                    self.conn.create_frame()
                } else {
                    warn!("received chess move but not in a game");
                    null_frame
                }
            }
            Command::_Invalid => {
                warn!("Invalid Command received!: {:?}", cmd);
                null_frame
            }
        }
    }
}
