# tranco-rs

Rust library for the [Tranco](https://tranco-list.eu) domain ranking.

See [the example](https://github.com/witchof0x20/tranco-rs/blob/main/examples/example.rs) for how to use it. 

```rust
use tranco::Client;

#[tokio::main]
async fn main() {
    let client = Client::new();
    let ranks = client.ranks("google.com").await;
    dbg!(&ranks);
    let list = client.list("LJL44").await;
    dbg!(&list);
    let list2 = client.list_date(2025, 04, 07, Some(false)).await;
    dbg!(&list2);
    let downloaded_list = client.download_list(&list2.unwrap()).await;
    dbg!(&downloaded_list);
}
```
