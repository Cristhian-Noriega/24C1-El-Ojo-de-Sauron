use sauron::connect;

fn main() {
    let client_id: String = "drone1".to_string();
    let connect_pkg = connect(client_id, false, 0, None, None);
    let bytes_vec = connect_pkg.to_bytes();
    println!("{:?}", bytes_vec)
}
