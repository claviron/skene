use skene::{Options, Runner};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> skene::Result<()> {
    Runner::new().with_options(Options::from_args()?).run().await
}
