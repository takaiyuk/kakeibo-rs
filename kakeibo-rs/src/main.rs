use anyhow::Result;

use kakeibo_rs::handler::run_kakeibo;

#[cfg(not(tarpaulin_include))]
fn main() -> Result<()> {
    run_kakeibo()
}
