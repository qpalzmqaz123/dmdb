use dmdb::Connection;

fn main() {
    let conn = Connection::connect("127.0.0.1:5236", "SYSDBA", "SYSDBA").unwrap();
}
