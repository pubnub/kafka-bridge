mod publish;
mod subscribe;

pub use self::publish::Client as PublishClient;
pub use self::subscribe::Client as SubscribeClient;

pub struct Message {
    pub topic: String,
    pub group: String,
    pub data: String,
}

#[derive(Debug)]
pub enum Error {
    KafkaInitialize,
    Publish,
    PublishWrite,
    PublishResponse,
    Subscribe,
    SubscribeWrite,
    SubscribeRead,
    MissingTopic,
    HTTPResponse,
}

pub enum ClientConfig {
    Plain,
    SaslPlain {
        username: String,
        password: String,
    },
    SaslSsl {
        username: String,
        password: String,
        ca_location: String,
        certificate_location: String,
        key_location: String,
        key_password: String,
    },
    SaslGssapi {
        kerberos_service_name: String,
        kerberos_keytab: String,
        kerberos_principal: String,
        ca_location: String,
        certificate_location: String,
        key_location: String,
        key_password: String,
    },
}

impl From<ClientConfig> for rdkafka::ClientConfig {
    fn from(config: ClientConfig) -> Self {
        let mut rdkafka_config = rdkafka::ClientConfig::new();
        match config {
            ClientConfig::Plain => {}
            ClientConfig::SaslPlain { username, password } => {
                rdkafka_config
                    .set("security.protocol", "SASL_PLAINTEXT")
                    .set("sasl.mechanism", "PLAIN")
                    .set("sasl.username", &username)
                    .set("sasl.password", &password);
            }
            ClientConfig::SaslSsl {
                username,
                password,
                ca_location,
                certificate_location,
                key_location,
                key_password,
            } => {
                rdkafka_config
                    .set("security.protocol", "SASL_SSL")
                    .set("sasl.mechanism", "PLAIN")
                    .set("sasl.username", &username)
                    .set("sasl.password", &password)
                    .set("ssl.ca.location", &ca_location)
                    .set("ssl.certificate.location", &certificate_location)
                    .set("ssl.key.location", &key_location)
                    .set("ssl.key.password", &key_password);
            }
            ClientConfig::SaslGssapi {
                kerberos_service_name,
                kerberos_keytab,
                kerberos_principal,
                ca_location,
                certificate_location,
                key_location,
                key_password,
            } => {
                rdkafka_config
                    .set("security.protocol", "SASL_SSL")
                    .set("sasl.kerberos.service.name", &kerberos_service_name)
                    .set("sasl.kerberos.keytab", &kerberos_keytab)
                    .set("sasl.kerberos.principal", &kerberos_principal)
                    .set("ssl.ca.location", &ca_location)
                    .set("ssl.certificate.location", &certificate_location)
                    .set("ssl.key.location", &key_location)
                    .set("ssl.key.password", &key_password);
            }
        }
        rdkafka_config
    }
}
