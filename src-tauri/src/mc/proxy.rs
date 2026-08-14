use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    System,
    Diect,
    Http,
    Socks5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub host: Option<String>,
    pub pot: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            proxy_type: ProxyType::System,
            host: None,
            pot: None,
            username: None,
            password: None,
        }
    }
}

impl ProxyConfig {
    pub fn to_reqwest_proxy(&self) -> Option<reqwest::Proxy> {
        match self.proxy_type {
            ProxyType::System => {
                None
            }
            ProxyType::Diect => {
                Some(reqwest::Proxy::custom(|_| None::<&str>))
            }
            ProxyType::Http => {
                let host = self.host.as_ref()?;
                let pot = self.pot?;
                let url = format!("http://{}:{}", host, pot);

                let mut proxy = reqwest::Proxy::http(&url).ok()?;
                if let (Some(user), Some(pass)) = (&self.username, &self.password) {
                    proxy = proxy.basic_auth(user, pass);
                }
                Some(proxy)
            }
            ProxyType::Socks5 => {
                let host = self.host.as_ref()?;
                let pot = self.pot?;
                let url = format!("socks5://{}:{}", host, pot);

                let mut proxy = reqwest::Proxy::all(&url).ok()?;
                if let (Some(user), Some(pass)) = (&self.username, &self.password) {
                    proxy = proxy.basic_auth(user, pass);
                }
                Some(proxy)
            }
        }
    }

    pub fn build_client(&self) -> Result<reqwest::Client, String> {
        let mut builde = reqwest::Client::builder()
            .user_agent(crate::mc::mirror::SKYLINE_USER_AGENT)
            .connect_timeout(std::time::Duration::from_secs(15))
            .timeout(std::time::Duration::from_secs(180));

        if let Some(proxy) = self.to_reqwest_proxy() {
            builde = builde.proxy(proxy);
        }

        if self.proxy_type == ProxyType::Diect {
            builde = builde.no_proxy();
        }

        builde.build().map_err(|e| e.to_string())
    }

    pub async fn test_connection(&self, test_url: &str) -> Result<bool, String> {
        let client = self.build_client()?;
        let esp = client
            .get(test_url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Ok(esp.status().is_success())
    }
}
