use hello::thread_pool::ThreadPool;
use hello::server;
use std::net::TcpListener;
use std::thread;
use reqwest;



#[test]
fn simulate() {
    thread::spawn(move || {
        graceful_shutdown()
    });

    thread::spawn(move || {
        let client = reqwest::Client::new();
        client.get("127.0.0.1:7878");
        client.get("127.0.0.1:7878");
    });
}

fn graceful_shutdown() {
    // Simulate client with reqwest
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        pool.execute(|| {
            server::handle_connection(stream);
        });
    }
    println!("Shutting down.");
}