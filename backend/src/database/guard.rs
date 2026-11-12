use crate::common::Configurations;
use crate::common::errors::ConnectionGuardError;
use crate::connection::constants::SourceType;
use crate::database::constants::{
    BENCHMARKING_FIRST_OCTET, BENCHMARKING_SECOND_OCTET_MASK, BENCHMARKING_SECOND_OCTET_PREFIX,
    CARRIER_GRADE_NAT_FIRST_OCTET, CARRIER_GRADE_NAT_SECOND_OCTET_MASK,
    CARRIER_GRADE_NAT_SECOND_OCTET_PREFIX, IPV6_LINK_LOCAL_MASK, IPV6_LINK_LOCAL_PREFIX,
    IPV6_LITERAL_CLOSING_BRACKET, IPV6_LITERAL_OPENING_BRACKET, IPV6_NAT64_PREFIX,
    IPV6_SITE_LOCAL_PREFIX, IPV6_UNIQUE_LOCAL_MASK, IPV6_UNIQUE_LOCAL_PREFIX,
    PROTOCOL_ASSIGNMENTS_OCTETS, RESERVED_FIRST_OCTET_MINIMUM, THIS_NETWORK_FIRST_OCTET,
    UNIX_SOCKET_HOST_PREFIX,
};
use crate::feature_flag::constants::FeatureFlag;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub fn ensure_source_type_is_enabled(
    configurations: &Configurations,
    source_type: SourceType,
) -> Result<(), ConnectionGuardError> {
    if configurations
        .disabled_connection_types
        .contains(&source_type)
    {
        return Err(ConnectionGuardError::TypeDisabled { source_type });
    }
    Ok(())
}

pub async fn ensure_host_is_permitted(
    configurations: &Configurations,
    host: &str,
    port: u16,
) -> Result<(), ConnectionGuardError> {
    if configurations
        .feature_flags
        .is_enabled(FeatureFlag::InternalAddressConnections)
    {
        return Ok(());
    }
    if host.starts_with(UNIX_SOCKET_HOST_PREFIX) {
        return Err(ConnectionGuardError::HostNotAllowed);
    }
    let host_without_brackets = host
        .trim_start_matches(IPV6_LITERAL_OPENING_BRACKET)
        .trim_end_matches(IPV6_LITERAL_CLOSING_BRACKET);
    let resolved_addresses: Vec<IpAddr> = match host_without_brackets.parse::<IpAddr>() {
        Ok(literal_address) => vec![literal_address],
        Err(_) => tokio::net::lookup_host((host_without_brackets, port))
            .await
            .map_err(|lookup_error| {
                tracing::warn!(
                    "refusing a connection whose host could not be resolved: {lookup_error}"
                );
                ConnectionGuardError::HostNotAllowed
            })?
            .map(|socket_address| socket_address.ip())
            .collect(),
    };
    if resolved_addresses.is_empty() || resolved_addresses.iter().copied().any(is_internal_address)
    {
        return Err(ConnectionGuardError::HostNotAllowed);
    }
    Ok(())
}

pub fn is_internal_address(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(ipv4_address) => is_internal_ipv4_address(ipv4_address),
        IpAddr::V6(ipv6_address) => is_internal_ipv6_address(ipv6_address),
    }
}

fn is_internal_ipv4_address(address: Ipv4Addr) -> bool {
    let octets = address.octets();
    let is_carrier_grade_nat = octets[0] == CARRIER_GRADE_NAT_FIRST_OCTET
        && octets[1] & CARRIER_GRADE_NAT_SECOND_OCTET_MASK == CARRIER_GRADE_NAT_SECOND_OCTET_PREFIX;
    let is_benchmarking = octets[0] == BENCHMARKING_FIRST_OCTET
        && octets[1] & BENCHMARKING_SECOND_OCTET_MASK == BENCHMARKING_SECOND_OCTET_PREFIX;
    address.is_loopback()
        || octets[..PROTOCOL_ASSIGNMENTS_OCTETS.len()] == PROTOCOL_ASSIGNMENTS_OCTETS
        || octets[0] >= RESERVED_FIRST_OCTET_MINIMUM
        || is_benchmarking
        || address.is_private()
        || address.is_link_local()
        || address.is_unspecified()
        || address.is_broadcast()
        || address.is_multicast()
        || octets[0] == THIS_NETWORK_FIRST_OCTET
        || is_carrier_grade_nat
}

fn is_internal_ipv6_address(address: Ipv6Addr) -> bool {
    let segments = address.segments();
    if let Some(embedded_ipv4_address) = address.to_ipv4() {
        return is_internal_ipv4_address(embedded_ipv4_address);
    }
    if segments[..IPV6_NAT64_PREFIX.len()] == IPV6_NAT64_PREFIX {
        let translated_ipv4_address = Ipv4Addr::new(
            (segments[6] >> 8) as u8,
            segments[6] as u8,
            (segments[7] >> 8) as u8,
            segments[7] as u8,
        );
        return is_internal_ipv4_address(translated_ipv4_address);
    }
    address.is_loopback()
        || address.is_unspecified()
        || address.is_multicast()
        || segments[0] & IPV6_UNIQUE_LOCAL_MASK == IPV6_UNIQUE_LOCAL_PREFIX
        || segments[0] & IPV6_LINK_LOCAL_MASK == IPV6_LINK_LOCAL_PREFIX
        || segments[0] & IPV6_LINK_LOCAL_MASK == IPV6_SITE_LOCAL_PREFIX
}

#[cfg(test)]
mod tests {
    use super::{ensure_host_is_permitted, is_internal_address};
    use crate::common::Configurations;
    use crate::common::errors::ConnectionGuardError;
    use crate::feature_flag::constants::FeatureFlag;
    use std::net::IpAddr;

    const POSTGRES_PORT: u16 = 5432;

    fn configurations(are_internal_address_connections_on: bool) -> Configurations {
        Configurations::builder()
            .frontend_dir("frontend")
            .encryption_key("encryption-key")
            .admin_username("admin")
            .admin_password("password")
            .admin_email("admin@example.test")
            .admin_full_name("The Administrator")
            .feature_flag(
                FeatureFlag::InternalAddressConnections,
                are_internal_address_connections_on,
            )
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn public_literal_hosts_are_permitted_when_internal_addresses_are_off() {
        let configurations = configurations(false);
        for public_host in ["8.8.8.8", "2606:4700:4700::1111", "[2606:4700:4700::1111]"] {
            assert!(
                ensure_host_is_permitted(&configurations, public_host, POSTGRES_PORT)
                    .await
                    .is_ok(),
                "{public_host}"
            );
        }
    }

    #[tokio::test]
    async fn internal_hosts_are_refused_when_internal_addresses_are_off() {
        let configurations = configurations(false);
        for internal_host in [
            "127.0.0.1",
            "::1",
            "[::1]",
            "localhost",
            "/var/run/postgresql",
            "",
        ] {
            assert!(
                matches!(
                    ensure_host_is_permitted(&configurations, internal_host, POSTGRES_PORT).await,
                    Err(ConnectionGuardError::HostNotAllowed)
                ),
                "{internal_host:?}"
            );
        }
    }

    #[tokio::test]
    async fn every_host_is_permitted_when_internal_addresses_are_on() {
        let configurations = configurations(true);
        for host in ["127.0.0.1", "localhost", "/var/run/postgresql", "8.8.8.8"] {
            assert!(
                ensure_host_is_permitted(&configurations, host, POSTGRES_PORT)
                    .await
                    .is_ok(),
                "{host}"
            );
        }
    }

    fn is_internal(address: &str) -> bool {
        is_internal_address(address.parse::<IpAddr>().unwrap())
    }

    #[test]
    fn internal_addresses_are_recognised() {
        for internal_address in [
            "127.0.0.1",
            "127.8.9.10",
            "10.0.0.5",
            "172.16.0.1",
            "172.31.255.254",
            "192.168.1.1",
            "169.254.169.254",
            "0.0.0.0",
            "0.1.2.3",
            "100.64.0.1",
            "100.127.255.254",
            "255.255.255.255",
            "224.0.0.1",
            "::1",
            "::",
            "fc00::1",
            "fd12:3456:789a::1",
            "fe80::1",
            "ff02::1",
            "::ffff:127.0.0.1",
            "::ffff:10.0.0.5",
            "::ffff:169.254.169.254",
            "::10.0.0.5",
            "64:ff9b::a00:5",
            "64:ff9b::7f00:1",
            "198.18.0.1",
            "198.19.255.254",
            "192.0.0.8",
            "240.0.0.1",
            "fec0::1",
        ] {
            assert!(is_internal(internal_address), "{internal_address}");
        }
    }

    #[test]
    fn public_addresses_are_not_internal() {
        for public_address in [
            "8.8.8.8",
            "1.1.1.1",
            "100.63.255.254",
            "100.128.0.1",
            "172.32.0.1",
            "192.169.0.1",
            "169.253.0.1",
            "198.17.0.1",
            "198.20.0.1",
            "192.0.1.1",
            "203.0.113.9",
            "2606:4700:4700::1111",
            "::ffff:8.8.8.8",
            "64:ff9b::808:808",
        ] {
            assert!(!is_internal(public_address), "{public_address}");
        }
    }
}
