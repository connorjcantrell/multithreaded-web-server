use hello::thread_pool::ThreadPool;
use hello::server::handle_connection;
use reqwest;


#[test]
fn main_test() {
    // use reqwest crate to simulate a client
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(4) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}