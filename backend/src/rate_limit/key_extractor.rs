use crate::authorization::dto::AuthenticatedUser;
use crate::rate_limit::constants::{
    FORWARDED_FOR_ENTRY_SEPARATOR, FORWARDED_FOR_HEADER, IPV6_NETWORK_SEGMENT_COUNT,
    MISSING_AUTHENTICATED_USER_LOG_MESSAGE, MISSING_FORWARDED_ADDRESS_LOG_MESSAGE,
    MISSING_PEER_ADDRESS_LOG_MESSAGE, RATE_LIMIT_KEY_UNAVAILABLE_MESSAGE,
};
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, Request, StatusCode};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use tower_governor::GovernorError;
use tower_governor::key_extractor::KeyExtractor;
use uuid::Uuid;

fn client_network(client_address: IpAddr) -> IpAddr {
    let IpAddr::V6(ipv6_address) = client_address else {
        return client_address;
    };
    if let Some(mapped_ipv4_address) = ipv6_address.to_ipv4_mapped() {
        return IpAddr::V4(mapped_ipv4_address);
    }
    let mut network_segments = ipv6_address.segments();
    for host_segment in network_segments.iter_mut().skip(IPV6_NETWORK_SEGMENT_COUNT) {
        *host_segment = 0;
    }
    IpAddr::V6(Ipv6Addr::from(network_segments))
}

fn key_unavailable(log_message: &'static str) -> GovernorError {
    tracing::error!("{log_message}");
    GovernorError::Other {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        msg: Some(RATE_LIMIT_KEY_UNAVAILABLE_MESSAGE.to_string()),
        headers: None,
    }
}

fn forwarded_client_address(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get_all(FORWARDED_FOR_HEADER)
        .iter()
        .next_back()?
        .to_str()
        .ok()?
        .rsplit(FORWARDED_FOR_ENTRY_SEPARATOR)
        .next()?
        .trim()
        .parse::<IpAddr>()
        .ok()
}

#[derive(Clone, Copy, Debug)]
pub struct ClientAddressKeyExtractor {
    pub is_proxy_trusted: bool,
}

impl KeyExtractor for ClientAddressKeyExtractor {
    type Key = IpAddr;

    fn extract<T>(&self, request: &Request<T>) -> Result<Self::Key, GovernorError> {
        if self.is_proxy_trusted {
            return forwarded_client_address(request.headers())
                .map(client_network)
                .ok_or_else(|| key_unavailable(MISSING_FORWARDED_ADDRESS_LOG_MESSAGE));
        }
        request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|connect_info| client_network(connect_info.ip()))
            .ok_or_else(|| key_unavailable(MISSING_PEER_ADDRESS_LOG_MESSAGE))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AuthenticatedUserKeyExtractor;

impl KeyExtractor for AuthenticatedUserKeyExtractor {
    type Key = Uuid;

    fn extract<T>(&self, request: &Request<T>) -> Result<Self::Key, GovernorError> {
        request
            .extensions()
            .get::<AuthenticatedUser>()
            .map(|authenticated_user| authenticated_user.user_id().get_uuid())
            .ok_or_else(|| key_unavailable(MISSING_AUTHENTICATED_USER_LOG_MESSAGE))
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthenticatedUserKeyExtractor, ClientAddressKeyExtractor};
    use crate::authorization::dto::AuthenticatedUser;
    use crate::rate_limit::constants::FORWARDED_FOR_HEADER;
    use axum::extract::ConnectInfo;
    use axum::http::{Request, StatusCode};
    use std::net::{IpAddr, SocketAddr};
    use tower_governor::GovernorError;
    use tower_governor::key_extractor::KeyExtractor;
    use uuid::Uuid;

    const PEER_ADDRESS: &str = "198.51.100.7:40000";

    fn request_with_forwarded_headers(forwarded_values: &[&str]) -> Request<()> {
        let mut request_builder = Request::builder().uri("/api/v1/auth/session");
        for forwarded_value in forwarded_values {
            request_builder = request_builder.header(FORWARDED_FOR_HEADER, *forwarded_value);
        }
        let mut request = request_builder.body(()).unwrap();
        request
            .extensions_mut()
            .insert(ConnectInfo(PEER_ADDRESS.parse::<SocketAddr>().unwrap()));
        request
    }

    fn assert_server_error(result: Result<impl std::fmt::Debug, GovernorError>) {
        assert!(matches!(
            result,
            Err(GovernorError::Other {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                ..
            })
        ));
    }

    #[test]
    fn an_untrusted_proxy_header_is_ignored_and_the_peer_address_is_the_key() {
        let extractor = ClientAddressKeyExtractor {
            is_proxy_trusted: false,
        };
        let key = extractor
            .extract(&request_with_forwarded_headers(&["203.0.113.9"]))
            .unwrap();
        assert_eq!(key, "198.51.100.7".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn a_missing_peer_address_fails_the_request() {
        let extractor = ClientAddressKeyExtractor {
            is_proxy_trusted: false,
        };
        let request = Request::builder().uri("/").body(()).unwrap();
        assert_server_error(extractor.extract(&request));
    }

    #[test]
    fn a_trusted_proxy_header_gives_its_last_entry() {
        let extractor = ClientAddressKeyExtractor {
            is_proxy_trusted: true,
        };
        for (forwarded_values, expected_address) in [
            (vec!["203.0.113.9"], "203.0.113.9"),
            (vec!["10.0.0.1, 203.0.113.9"], "203.0.113.9"),
            (vec!["1.1.1.1,2.2.2.2 ,  203.0.113.9 "], "203.0.113.9"),
            (vec!["1.1.1.1", "2.2.2.2, 203.0.113.9"], "203.0.113.9"),
            (vec!["2001:db8::1"], "2001:db8::"),
            (vec!["2001:db8:1:2:3:4:5:6"], "2001:db8:1:2::"),
            (vec!["2001:db8:1:2:ffff:ffff:ffff:ffff"], "2001:db8:1:2::"),
            (vec!["::ffff:203.0.113.9"], "203.0.113.9"),
        ] {
            let key = extractor
                .extract(&request_with_forwarded_headers(&forwarded_values))
                .unwrap();
            assert_eq!(
                key,
                expected_address.parse::<IpAddr>().unwrap(),
                "{forwarded_values:?}"
            );
        }
    }

    #[test]
    fn a_trusted_proxy_header_that_is_missing_or_not_an_address_fails_the_request() {
        let extractor = ClientAddressKeyExtractor {
            is_proxy_trusted: true,
        };
        for forwarded_values in [
            vec![],
            vec![""],
            vec!["unknown"],
            vec!["203.0.113.9, unknown"],
            vec!["203.0.113.9,"],
            vec!["203.0.113.9:443"],
        ] {
            assert_server_error(
                extractor.extract(&request_with_forwarded_headers(&forwarded_values)),
            );
        }
    }

    #[test]
    fn the_authenticated_user_is_the_key() {
        let user_id = Uuid::new_v4();
        let mut request = Request::builder().uri("/").body(()).unwrap();
        request
            .extensions_mut()
            .insert(AuthenticatedUser::from_validated_session(user_id));
        assert_eq!(
            AuthenticatedUserKeyExtractor.extract(&request).unwrap(),
            user_id
        );
    }

    #[test]
    fn a_request_without_an_authenticated_user_fails() {
        let request = Request::builder().uri("/").body(()).unwrap();
        assert_server_error(AuthenticatedUserKeyExtractor.extract(&request));
    }
}
