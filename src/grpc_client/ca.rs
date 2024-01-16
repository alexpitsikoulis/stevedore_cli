pub mod certificate_authority {
    tonic::include_proto!("certificate_authority");
}

use super::ClientError;
use certificate_authority::{GetRootCertificateRequest, SignCertificateRequest};
use openssl::{
    hash::MessageDigest,
    nid::Nid,
    pkey::PKey,
    rsa::Rsa,
    x509::{X509NameBuilder, X509ReqBuilder},
};
use tonic::transport::Channel;
use uuid::Uuid;

pub struct CertificateAuthorityClient(
    certificate_authority::certificate_authority_client::CertificateAuthorityClient<Channel>,
);

impl CertificateAuthorityClient {
    pub async fn new(addr: &'static str) -> Result<Self, ClientError> {
        match certificate_authority::certificate_authority_client::CertificateAuthorityClient::connect(addr).await {
            Ok(client) => Ok(CertificateAuthorityClient(client)),
            Err(e) => Err(ClientError::Transport(e)),
        }
    }

    pub async fn get_ca_certificate(
        &mut self,
        certificate: Option<Vec<u8>>,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        match self
            .0
            .get_root_certificate(GetRootCertificateRequest { certificate })
            .await
        {
            Ok(res) => Ok(res.into_inner().certificate),
            Err(e) => Err(Box::new(ClientError::Request(e))),
        }
    }

    pub async fn sign(&mut self) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
        let mut name_builder = X509NameBuilder::new()?;
        name_builder.append_entry_by_nid(Nid::COUNTRYNAME, "US")?;
        name_builder.append_entry_by_nid(Nid::STATEORPROVINCENAME, "California")?;
        name_builder.append_entry_by_nid(Nid::LOCALITYNAME, "Los Angeles")?;
        name_builder.append_entry_by_nid(Nid::ORG, "Stevedore")?;
        name_builder.append_entry_by_nid(Nid::COMMONNAME, "Stevedore")?;
        name_builder.append_entry_by_nid(Nid::ACCOUNT, &Uuid::new_v4().to_string())?;
        let name = name_builder.build();

        let rsa = Rsa::generate(4096)?;
        let private_key = PKey::from_rsa(rsa.clone())?;
        let public_key = PKey::from_rsa(Rsa::from_public_components(
            rsa.n().to_owned()?,
            rsa.e().to_owned()?,
        )?)?;

        let mut csr_builder = X509ReqBuilder::new()?;
        csr_builder.set_subject_name(&name)?;
        csr_builder.set_version(2)?;
        csr_builder.set_pubkey(&public_key)?;

        csr_builder.sign(&private_key, MessageDigest::sha256())?;
        let csr = csr_builder.build().to_pem()?;

        match self
            .0
            .sign_certificate(SignCertificateRequest { csr })
            .await
        {
            Ok(res) => {
                let res = res.into_inner();
                std::fs::write("cert.pem", &res.certificate).map_err(ClientError::FileSystem)?;
                let private_key = private_key.private_key_to_pem_pkcs8()?;
                std::fs::write("key.pem", private_key.clone()).map_err(ClientError::FileSystem)?;
                Ok((res.certificate, private_key))
            }
            Err(e) => Err(Box::new(ClientError::Request(e))),
        }
    }
}
