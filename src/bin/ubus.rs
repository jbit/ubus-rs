use std::path::Path;

fn main() {
    let socket = Path::new("/var/run/ubus.sock");

    let mut connection = match ubus::Connection::connect(&socket) {
        Ok(connection) => connection,
        Err(err) => {
            eprintln!("{}: Failed to open ubus socket. {}", socket.display(), err);
            return;
        }
    };
    connection
        .lookup(
            |obj| {
                println!("\n{:?}", obj);
            },
            |sig| {
                print!("  {}(", sig.name);
                for (name, ty) in sig.args {
                    print!("{}: {:?}, ", name, ty);
                }
                std::println!(")");
            },
        )
        .unwrap();
}
