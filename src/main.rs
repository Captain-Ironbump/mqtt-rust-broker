use lunatic::net::TcpListener;


fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1883")?;
    while let Ok((stream, _)) = listener.accept() {
        println!("New client connected: {:?}", stream.peer_addr());
    }
    drop(listener);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
