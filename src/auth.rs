use openssl::{asn1::Asn1Time, x509::X509};
use tonic::Status;

use crate::grpc_client::{CertificateAuthorityClient, ClientError};

pub async fn get_ca_certificate(
    client: &mut CertificateAuthorityClient,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let ca_certificate = match std::fs::read("ca_cert.pem") {
        Ok(ca_certificate) => Some(ca_certificate),
        Err(_) => None,
    };

    match client.get_ca_certificate(ca_certificate.clone()).await? {
        Some(res) => {
            std::fs::write("ca_cert.pem", &res)?;
            Ok(res)
        }
        None => match ca_certificate {
            Some(ca_certificate) => Ok(ca_certificate),
            None => panic!("failed to get ca certificate"),
        },
    }
}

pub async fn get_certificate(
    client: &mut CertificateAuthorityClient,
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    match (std::fs::read("cert.pem"), std::fs::read("key.pem")) {
        (Ok(certificate), Ok(key)) => {
            let now = Asn1Time::days_from_now(0).map_err(|e| {
                ClientError::Request(Status::internal(format!(
                    "failed to generate expiration time for CSR: {:?}",
                    e
                )))
            })?;
            let cert = X509::from_pem(&certificate).map_err(|e| {
                ClientError::Request(Status::internal(format!(
                    "failed to read certificate from pem: {:?}",
                    e
                )))
            })?;
            let expiration = cert.not_after();

            Ok(match now.compare(expiration)? {
                std::cmp::Ordering::Less => (certificate, key),
                _ => client.sign().await?,
            })
        }
        _ => Ok(client.sign().await?),
    }
}
