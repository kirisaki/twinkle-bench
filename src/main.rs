use std::time::Instant;
use futures::future::{join, join3, join_all};
use tokio::time::{timeout, Duration};
extern crate redis;
use redis::Commands;

const NUM: usize = 700;
const WAIT: Duration = Duration::from_secs(4);

macro_rules! measure {
    ( $x:expr) => {
        {
            let start = Instant::now();
            let result = $x;
            let end = start.elapsed();
            println!("{}.{:03}secs elapsed for {} requests", end.as_secs(), end.subsec_nanos() / 1_000_000, NUM);
            result
        }
    };
}

#[tokio::main]
async fn main() {
    println!("set:");
    {
        let (client, mut dispatcher, mut listener) = twinkle::client::Client::open("127.0.0.1:3000".to_string()).await.unwrap();
        join3(
            timeout(WAIT, listener.listen()),
            timeout(WAIT, dispatcher.run()),
            async move {
                let mut cs = vec![];
                for _ in 0..NUM {
                    cs.push(async {
                        let mut c = client.clone();
                        c.set(b"foo".to_vec(), b"bar".to_vec()).await;
                    })
                };
                measure!({
                    join_all(cs).await;
                })
            },
        ).await;
    }
    {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();
        measure!(
            for _ in 0..NUM {
                redis_test(&mut con);
            }
        )
    }
}

fn redis_test(con: &mut redis::Connection) -> redis::RedisResult<()> {
    let _ : () = con.set("my_key", 42)?;
    Ok(())
}
