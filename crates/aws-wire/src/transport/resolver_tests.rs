use super::{Addresses, Resolving, resolve};
use std::{
    io,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    task::Poll,
    time::Duration,
};
use tokio::sync::{Semaphore, oneshot};

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
#[derive(Default)]
struct Activity {
    active: AtomicUsize,
    peak: AtomicUsize,
}
struct Job {
    future: Resolving,
    started: oneshot::Receiver<()>,
    release: mpsc::SyncSender<()>,
}

#[tokio::test]
async fn abandoned_dns_waiters_retain_admission_until_the_blocking_jobs_finish() -> TestResult {
    let slots = Arc::new(Semaphore::new(2));
    let activity = Arc::<Activity>::default();
    let mut first = job(&slots, &activity)?;
    let mut second = job(&slots, &activity)?;
    started(&mut first).await?;
    started(&mut second).await?;

    drop(first.future);
    drop(second.future);
    let mut third = job(&slots, &activity)?;
    waiting(&mut third).await?;
    assert_eq!(slots.available_permits(), 0);
    first.release.send(())?;
    started(&mut third).await?;
    second.release.send(())?;
    third.release.send(())?;
    assert_eq!(third.future.await?.count(), 1);
    let _settled = Arc::clone(&slots).acquire_many_owned(2).await?;
    assert_eq!(activity.active.load(Ordering::SeqCst), 0);
    assert_eq!(activity.peak.load(Ordering::SeqCst), 2);
    Ok(())
}

fn job(slots: &Arc<Semaphore>, activity: &Arc<Activity>) -> Result<Job, io::Error> {
    let (release, released) = mpsc::sync_channel(1);
    let (ready, started) = oneshot::channel();
    let activity = Arc::clone(activity);
    let name = "slow.test".parse().map_err(io::Error::other)?;
    let lookup = move |_| slow(ready, released, activity);
    let future = Box::pin(resolve(name, Arc::clone(slots), lookup));
    Ok(Job {
        future,
        started,
        release,
    })
}

fn slow(
    ready: oneshot::Sender<()>,
    released: mpsc::Receiver<()>,
    activity: Arc<Activity>,
) -> io::Result<Addresses> {
    let count = activity
        .active
        .fetch_add(1, Ordering::SeqCst)
        .saturating_add(1);
    activity.peak.fetch_max(count, Ordering::SeqCst);
    ready
        .send(())
        .map_err(|()| io::Error::other("lookup observer dropped"))?;
    let result = released.recv_timeout(Duration::from_secs(10));
    activity.active.fetch_sub(1, Ordering::SeqCst);
    result.map_err(io::Error::other)?;
    Ok(vec![SocketAddr::from(([127, 0, 0, 1], 0))].into_iter())
}

async fn started(job: &mut Job) -> TestResult {
    tokio::select! {
        result = &mut job.future => Err(format!("lookup completed before release: {result:?}").into()),
        signal = &mut job.started => { signal?; Ok(()) }
    }
}

async fn waiting(job: &mut Job) -> TestResult {
    std::future::poll_fn(|cx| match job.future.as_mut().poll(cx) {
        Poll::Pending => Poll::Ready(Ok(())),
        Poll::Ready(_) => Poll::Ready(Err("lookup escaped admission".into())),
    })
    .await
}
