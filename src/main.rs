use std::time::Instant;
use futures::future::{join3, join_all};
use tokio::time::{timeout, Duration};
use darkredis::ConnectionPool;

const NUM: usize = 3000;
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
    println!("twinkle:");
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
                        c.set(b"foo".to_vec(), b"bar".to_vec()).await.unwrap();
                        let res = c.get(b"foo".to_vec()).await.unwrap();
                        assert_eq!(res, b"bar".to_vec());
                    })
                };
                measure!({
                    join_all(cs).await;
                })
            },
        ).await;
    }

    println!("redis:");
    {
        let pool = ConnectionPool::create("127.0.0.1:6379".into(), None, num_cpus::get()).await.unwrap();
        let mut cs = vec![];
        for _ in 0..NUM {
            cs.push(async {
                let mut con = pool.get().await;
                con.set("foo", "bar").await.unwrap();
                let res = con.get("foo").await.unwrap();
                assert_eq!(res, Some(b"bar".to_vec()));
            });
        };
        measure!({
            join_all(cs).await;
        });
    }
}

