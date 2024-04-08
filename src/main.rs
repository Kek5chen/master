use std::{env, str::FromStr};
use lettre::{message::{header::ContentType, Mailbox}, Message, SmtpTransport, Transport};
use reqwest::blocking;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Value};


// const GUFF_ID_SEARCH: &str = "CID_752_Athena_Commando_M_Comet";
const SHOP_API_URL: &str = "https://fortnite-api.com/v2/shop/br";

#[derive(Serialize, Deserialize, Debug)]
struct FortniteItem {
    id: String,
    name: String,
    description: String,
}

fn fetch_store_data() -> Result<String, String> {
    let response = match blocking::get(SHOP_API_URL) {
        Ok(res) => res,
        Err(_) => return Err("Fortnite Item Shop Notifier could not reach the API at fortnite-api.com".to_string())
    };

    match response.text() {
        Ok(data) => Ok(data),
        Err(_) => Err("Fortnite Item Shop Notifier could not fetch the shop data from fortnite-api.com".to_string())
    }
}

fn find_item(store_data: &str, cid: &str) -> Option<FortniteItem> {
    let store_data: Value = match serde_json::from_str(store_data) {
        Ok(data) => data,
        Err(_) => {
            eprintln!("Got a non-json response from fortnite-api.com. Could not parse.");
            return None;
        }
    };

    if store_data["status"].as_u64().unwrap() != 200u64 {
        eprintln!("Got a non 200 response from fortnite-api.com.");
        return None;
    }

    if !store_data["data"]["featured"]["entries"].is_array() {
        eprintln!("The data from fortnite-api.com was not in the expected format.");
        return None;
    }

    store_data["data"]["featured"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|arr| {
            arr["items"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|item| {
                    let item = from_value::<FortniteItem>(item.clone()).ok();

                    if item.is_some() && item.as_ref().unwrap().id == cid {
                        return item;
                    }
                    None
                })
                .next()
        }).collect::<Vec<FortniteItem>>().pop()
}

fn send_info(item: &FortniteItem, mail_to: &str) -> Result<(), String> {
    let mail_recv: Mailbox = match Mailbox::from_str(&mail_to) {
        Ok(mbox) => mbox,
        Err(_) => return Err(format!("The Email Address <{}> was invalid", &item.name))
    };
    let mail = Message::builder()
        .from("Fortnite Item Shop Notifier <notifier@imkx.dev>".parse().unwrap())
        .to(mail_recv)
        .subject(format!("{} is now available in Fornite!", item.name))
        .header(ContentType::TEXT_PLAIN)
        .body(format!(r#"
                We think we located {} in the item store, this is your notification!
                
                Some extra info for you to validate that it is truly the item we detected:
                CID: {}
                Name: {}
                Description: {}
                "#, &item.name, &item.id, &item.name, &item.description))
        .unwrap();

    let mailer = SmtpTransport::unencrypted_localhost();

    match mailer.send(&mail) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string())
    }
}

fn main() -> Result<(), String> {
    let args = env::args();
    if args.len() != 3 {
        return Err("Syntax: <COSMETIC_ID> <EMAIL>".to_string());
    }
    let args: Vec<String> = args.into_iter().collect();

    let store_data = fetch_store_data()?;

    let item = match find_item(&store_data, &args[1]) {
        Some(item) => item,
        None => {
            println!("The item {} was not found in the item shop", &args[1]);
            return Ok(());
        }
    };

    match send_info(&item, &args[2]) {
        Ok(_) => {
            println!("The item {} was found and an email has successfully been sent to the recipient", &item.name);
            Ok(())
        }
        Err(_) => Err(format!("The item {} was found on the item store but an email could not be sent", &item.name))
    }
}
