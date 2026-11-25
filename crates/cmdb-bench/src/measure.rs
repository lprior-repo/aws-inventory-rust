use cmdb_view::{Database, PageRequest, ResourceFilter};
use std::error::Error;
use std::path::PathBuf;
use std::time::Instant;
pub(crate) async fn measure(path: PathBuf) -> Result<(), Box<dyn Error>> {
    let target = path.to_str().ok_or("non-UTF-8 database target")?;
    let database = Database::open(target).await?;
    let started = Instant::now();
    let counts = database.counts().await?;
    let counts_time = started.elapsed();
    let started = Instant::now();
    let request = PageRequest {
        limit: 500,
        after: None,
    };
    let page = database
        .page_rows(&ResourceFilter::default(), request)
        .await?;
    let page_time = started.elapsed();
    println!(
        "accounts={} regions={} services={} resources={} counts_us={} page_rows={} page_us={}",
        counts.accounts,
        counts.regions,
        counts.services,
        counts.resources,
        counts_time.as_micros(),
        page.rows.len(),
        page_time.as_micros()
    );
    Ok(())
}
