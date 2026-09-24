pub mod accounts;
pub mod auth;
pub mod campaigns;
pub mod conversations;
pub mod leads;
pub mod lists;
pub mod login;
pub mod senders;
pub mod stats;
pub mod tags;
pub mod webhooks;

use serde_json::Value;

use crate::account;
use crate::cli::{Auth, Campaigns, Cli, Command, Conversations, Leads, Lists, Senders, Stats, Tags, Webhooks};
use crate::error::Result;
use crate::output;
use crate::readme;

pub fn print(v: Value) {
    output::write(&v);
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::AgentReadme => {
            readme::print();
            Ok(())
        }
        Command::Login(args) => login::run(args),
        Command::Accounts(args) => accounts::run(args),
        cmd => {
            let client = account::resolve(cli.account.as_deref(), cli.api_key.as_deref())?.client(cli.verbose);
            let res = match cmd {
                Command::Auth(Auth::Check) => auth::check(&client)?,
                Command::Campaigns(c) => match c {
                    Campaigns::List { limit, offset, status } => campaigns::list(&client, limit, offset, status)?,
                    Campaigns::Get { id } => campaigns::get(&client, id)?,
                    Campaigns::Pause { id } => campaigns::pause(&client, id)?,
                    Campaigns::Resume { id } => campaigns::resume(&client, id)?,
                    Campaigns::AddLeads { id, file } => campaigns::add_leads(&client, id, file)?,
                    Campaigns::StopLead { campaign_id, lead } => campaigns::stop_lead(&client, campaign_id, lead)?,
                },
                Command::Lists(l) => match l {
                    Lists::List { limit, offset } => lists::list(&client, limit, offset)?,
                    Lists::Get { id } => lists::get(&client, id)?,
                    Lists::Create { name, r#type } => lists::create(&client, name, r#type)?,
                    Lists::Leads { id, limit, offset, keyword } => lists::leads(&client, id, limit, offset, keyword)?,
                    Lists::AddLead { id, linkedin_url } => lists::add_lead(&client, id, linkedin_url)?,
                    Lists::RemoveLead { id, lead } => lists::remove_lead(&client, id, lead)?,
                    Lists::Companies { id, limit, offset } => lists::companies(&client, id, limit, offset)?,
                },
                Command::Leads(ld) => match ld {
                    Leads::Get { linkedin_url } => leads::get(&client, linkedin_url)?,
                    Leads::Lists { linkedin_url } => leads::lists(&client, linkedin_url)?,
                    Leads::UpdateStatus { lead, status } => leads::update_status(&client, lead, status)?,
                },
                Command::Senders(s) => match s {
                    Senders::List { limit, offset } => senders::list(&client, limit, offset)?,
                    Senders::Get { id } => senders::get(&client, id)?,
                    Senders::Network { id, limit, offset } => senders::network(&client, id, limit, offset)?,
                },
                Command::Conversations(conv) => match conv {
                    Conversations::List { limit, offset, campaign_id, sender_id } => {
                        conversations::list(&client, limit, offset, campaign_id, sender_id)?
                    }
                    Conversations::Send { lead, message, sender_id } => {
                        conversations::send(&client, lead, message, sender_id)?
                    }
                },
                Command::Stats(st) => match st {
                    Stats::Get { from, to, campaign_id, sender_id } => {
                        stats::get(&client, from, to, campaign_id, sender_id)?
                    }
                },
                Command::Webhooks(w) => match w {
                    Webhooks::List { limit, offset } => webhooks::list(&client, limit, offset)?,
                    Webhooks::Get { id } => webhooks::get(&client, id)?,
                    Webhooks::Create { name, url, event, campaign_id } => {
                        webhooks::create(&client, name, url, event, campaign_id)?
                    }
                    Webhooks::Update { id, name, url, event, active } => {
                        webhooks::update(&client, id, name, url, event, active)?
                    }
                    Webhooks::Delete { id } => webhooks::delete(&client, id)?,
                },
                Command::Tags(t) => match t {
                    Tags::Create { name, color } => tags::create(&client, name, color)?,
                },
                Command::AgentReadme | Command::Login(_) | Command::Accounts(_) => unreachable!(),
            };
            print(res);
            Ok(())
        }
    }
}
