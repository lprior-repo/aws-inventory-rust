use super::{Resolver, Resolving};
use hickory_resolver::{
    Hosts, TokioResolver,
    config::{ConnectionConfig, LookupIpStrategy, NameServerConfig, ResolverConfig, ResolverOpts},
    net::runtime::TokioRuntimeProvider,
    proto::{
        op::Message,
        rr::{RData, Record, RecordType},
    },
};
use hyper_util::client::legacy::connect::dns::Name;
use std::{io, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::UdpSocket, sync::Semaphore};
use tower_service::Service;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::test]
async fn cancelled_dns_query_releases_admission_for_a_new_network_lookup() -> TestResult {
    tokio::time::timeout(Duration::from_secs(5), cancellation_recovery()).await??;
    Ok(())
}

async fn cancellation_recovery() -> TestResult {
    let server = UdpSocket::bind("127.0.0.1:0").await?;
    let mut resolver = resolver(server.local_addr()?)?;
    cancel_lookup(&mut resolver, &server).await?;
    assert_eq!(resolver.slots.available_permits(), 1);
    let addresses = complete_lookup(&mut resolver, &server).await?;
    let expected: SocketAddr = "192.0.2.17:0".parse()?;
    assert_eq!(addresses, vec![expected]);
    assert_eq!(resolver.slots.available_permits(), 1);
    Ok(())
}

async fn cancel_lookup(resolver: &mut Resolver, server: &UdpSocket) -> io::Result<()> {
    let mut lookup = resolver.call(name("first.cancel.test")?);
    observe_query(&mut lookup, server, "first.cancel.test.").await?;
    drop(lookup);
    Ok(())
}

async fn complete_lookup(
    resolver: &mut Resolver,
    server: &UdpSocket,
) -> io::Result<Vec<SocketAddr>> {
    let mut lookup = resolver.call(name("second.cancel.test")?);
    answer_query(&mut lookup, server, "second.cancel.test.").await?;
    lookup.await.map(Iterator::collect)
}

fn resolver(address: SocketAddr) -> Result<Resolver, io::Error> {
    let mut connection = ConnectionConfig::udp();
    connection.port = address.port();
    let server = NameServerConfig::new(address.ip(), true, vec![connection]);
    let config = ResolverConfig::from_name_servers(vec![server]);
    let mut options = ResolverOpts::default();
    options.ip_strategy = LookupIpStrategy::Ipv4Only;
    options.timeout = Duration::from_secs(2);
    let builder = TokioResolver::builder_with_config(config, TokioRuntimeProvider::default())
        .with_options(options);
    let mut inner = builder.build().map_err(io::Error::other)?;
    inner.set_hosts(Arc::new(Hosts::default()));
    Ok(Resolver {
        inner,
        slots: Arc::new(Semaphore::new(1)),
    })
}

fn name(value: &str) -> Result<Name, io::Error> {
    value.parse().map_err(io::Error::other)
}

async fn observe_query(
    future: &mut Resolving,
    server: &UdpSocket,
    expected: &str,
) -> io::Result<()> {
    let (packet, _) = next_packet(future, server).await?;
    let request = Message::from_vec(&packet).map_err(io::Error::other)?;
    verify_query(&request, expected)
}

async fn answer_query(
    future: &mut Resolving,
    server: &UdpSocket,
    expected: &str,
) -> io::Result<()> {
    let (packet, peer) = next_packet(future, server).await?;
    let response = Message::from_vec(&packet).map_err(io::Error::other)?;
    verify_query(&response, expected)?;
    let query_name = response
        .queries
        .first()
        .map(|query| query.name().clone())
        .ok_or_else(|| io::Error::other("DNS request has no question"))?;
    let mut response = response.into_response();
    response.add_answer(Record::from_rdata(
        query_name,
        30,
        RData::A(hickory_resolver::proto::rr::rdata::A::new(192, 0, 2, 17)),
    ));
    let bytes = response.to_vec().map_err(io::Error::other)?;
    server.send_to(&bytes, peer).await?;
    Ok(())
}

fn verify_query(request: &Message, expected: &str) -> io::Result<()> {
    let valid = request.queries.first().is_some_and(|query| {
        query.name().to_ascii() == expected && query.query_type() == RecordType::A
    });
    if valid {
        return Ok(());
    }
    Err(io::Error::other("unexpected DNS query"))
}

async fn next_packet(
    future: &mut Resolving,
    server: &UdpSocket,
) -> io::Result<(Vec<u8>, SocketAddr)> {
    let mut packet = vec![0; 512];
    let (length, peer) = tokio::select! {
        result = future => {
            return match result {
                Ok(_) => Err(io::Error::other("DNS lookup completed without a response")),
                Err(error) => Err(error),
            };
        }
        result = server.recv_from(&mut packet) => result?,
    };
    packet.truncate(length);
    Ok((packet, peer))
}
