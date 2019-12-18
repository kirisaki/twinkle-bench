use std::time::Instant;
use futures::future::join;
use tokio::time::{timeout, Duration};
extern crate redis;
use redis::Commands;

const NUM: usize = 20000;
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
        let (mut client, mut listener) = twinkle::open("127.0.0.1:3000".to_string()).await.unwrap();
        join(
            timeout(WAIT, listener.listen()),
            async move {
                let foo = b"foo".to_vec();
                let bar = b"bar".to_vec();
                measure!(
                    for _ in 0..NUM {
                        client.set(foo.clone(), bar.clone()).await.unwrap();
                    }
                )
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
