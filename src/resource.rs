use std::collections::HashMap;

pub struct Account<'a> {
    pub total: u32,
    pub in_stock: u32,
    pub data: HashMap<String, String>,
    pub scope: ResourceScope<'a>
}

pub struct Secret<'a> {
    pub user_id: &'a str,
    pub data: HashMap<String, String>,
    pub scope: ResourceScope<'a>
}

pub enum ResourceScope<'a> {
    Global,
    User(&'a str),
    Art(&'a str),
}

pub fn generate_secret<'a>(_user_id: &'a str, _secret_name: &'a str, _art_id: &'a str) -> HashMap<String, String>{
    HashMap::from([("service_account_key".to_owned(), "mock key".to_owned()), ("pivnet_token".to_owned(), "token123456".to_owned())])
}
