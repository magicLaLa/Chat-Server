use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::TcpListener, sync::broadcast};

#[tokio::main]
async fn main() {
    // 创建一个 tcp 链接
    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    // 创建一个广播
    let (tx, _rx) = broadcast::channel(10);
    // 创建一个循环可以接收多个客户端连接
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let tx= tx.clone();
        let mut rx = tx.subscribe();
        // 创建一个单独的任务（异步块）来单独运行，这样就不会造成多个任务用同一个线程来处理
        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            // 创建一个循环来接收多条信息
            loop {
                // 可以运行多个异步任务（中间有一些共享状态来使用），其中一个任务运行其他的任务就不会执行
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }
                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();
                        if addr != other_addr{
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
