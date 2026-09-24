//! The command tree for HeyReach CLI.

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "heyreach",
    version,
    about = "CLI for the HeyReach LinkedIn outreach API - manage campaigns, leads, senders, lists, webhooks and more",
    after_help = "An LLM agent should start with: heyreach agent-readme",
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Print raw JSON instead of YAML, for scripting
    #[arg(long, global = true)]
    pub json: bool,

    /// Output format: yaml or json (default: yaml)
    #[arg(long, global = true, value_name = "FORMAT")]
    pub format: Option<String>,

    /// Override API key (or set HEYREACH_API_KEY env var)
    #[arg(long, global = true, value_name = "KEY")]
    pub api_key: Option<String>,

    /// Account to run against (see 'heyreach accounts list')
    #[arg(short = 'a', long, global = true, value_name = "ACCOUNT")]
    pub account: Option<String>,

    /// Print HTTP method, URL, and status code to stderr
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Print the operating manual for an LLM agent driving this CLI
    AgentReadme,
    /// Log in with a HeyReach API key (opens browser to API settings)
    Login(Login),
    /// Manage HeyReach accounts and their API keys
    #[command(subcommand)]
    Accounts(Accounts),
    /// Authentication commands
    #[command(subcommand)]
    Auth(Auth),
    /// Campaign management
    #[command(subcommand)]
    Campaigns(Campaigns),
    /// Lead list management
    #[command(subcommand)]
    Lists(Lists),
    /// Lead operations
    #[command(subcommand)]
    Leads(Leads),
    /// LinkedIn sender account management
    #[command(subcommand)]
    Senders(Senders),
    /// Conversation management
    #[command(subcommand)]
    Conversations(Conversations),
    /// Analytics and statistics
    #[command(subcommand)]
    Stats(Stats),
    /// Webhook management
    #[command(subcommand)]
    Webhooks(Webhooks),
    /// Tag management
    #[command(subcommand)]
    Tags(Tags),
}

// -----------------------------------------------------------------------------
// Login & Accounts
// -----------------------------------------------------------------------------

#[derive(Args, Clone)]
pub struct Login {
    /// Account name to store (default: "default")
    #[arg(value_name = "NAME", default_value = "default")]
    pub name: String,

    /// HeyReach API key (prompted for securely if omitted)
    #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
    pub api_key: Option<String>,

    /// Read the API key from stdin, e.g. `pbpaste | heyreach login --api-key-stdin`
    #[arg(long)]
    pub api_key_stdin: bool,

    /// Do not open the browser to the API settings page automatically
    #[arg(long)]
    pub no_browser: bool,

    /// Replace the key on an account that already exists
    #[arg(long)]
    pub force: bool,

    /// Store the key without calling the API to check it first
    #[arg(long)]
    pub no_verify: bool,
}

#[derive(Subcommand)]
pub enum Accounts {
    /// Add an account and store its API key in the OS keystore
    Add {
        /// Account name
        #[arg(value_name = "NAME")]
        name: String,

        /// HeyReach API key (prompted for securely if omitted)
        #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
        api_key: Option<String>,

        /// Read the API key from stdin
        #[arg(long)]
        api_key_stdin: bool,

        /// Replace an existing account
        #[arg(long)]
        force: bool,

        /// Store without verifying via auth check
        #[arg(long)]
        no_verify: bool,
    },
    /// List configured accounts
    List {
        /// Call the API to verify each stored key is still valid
        #[arg(long)]
        check: bool,
    },
    /// Test authentication for an account
    Test {
        /// Account name
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Remove an account from config and keystore
    Remove {
        /// Account name
        #[arg(value_name = "NAME")]
        name: String,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}

// -----------------------------------------------------------------------------
// Auth
// -----------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum Auth {
    /// Validate the active API key
    Check,
}

// -----------------------------------------------------------------------------
// Campaigns
// -----------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum Campaigns {
    /// List all campaigns
    List {
        /// Number of items to return
        #[arg(long, default_value_t = 10)]
        limit: u32,

        /// Number of items to skip
        #[arg(long, default_value_t = 0)]
        offset: u32,

        /// Filter by status: active, paused, draft, completed
        #[arg(long)]
        status: Option<String>,
    },
    /// Get campaign details
    Get {
        /// Campaign ID
        #[arg(value_name = "ID")]
        id: i64,
    },
    /// Pause a running campaign
    Pause {
        /// Campaign ID
        #[arg(value_name = "ID")]
        id: i64,
    },
    /// Resume a paused campaign
    Resume {
        /// Campaign ID
        #[arg(value_name = "ID")]
        id: i64,
    },
    /// Add leads to a campaign from stdin or --file
    AddLeads {
        /// Campaign ID
        #[arg(value_name = "ID")]
        id: i64,

        /// Path to JSON file containing leads array
        #[arg(long)]
        file: Option<String>,
    },
    /// Stop a specific lead in a campaign
    StopLead {
        /// Campaign ID
        #[arg(value_name = "CAMPAIGN_ID")]
        campaign_id: i64,

        /// Lead ID to stop
        #[arg(long, value_name = "LEAD_ID")]
        lead: i64,
    },
}

// -----------------------------------------------------------------------------
// Lists
// -----------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum Lists {
    /// List all lead lists
    List {
        /// Number of items to return
        #[arg(long, default_value_t = 10)]
        limit: u32,

        /// Number of items to skip
        #[arg(long, default_value_t = 0)]
        offset: u32,
    },
    /// Get list details
    Get {
        /// List ID
        #[arg(value_name = "ID")]
        id: i64,
    },
    /// Create an empty list
    Create {
        /// List name
        #[arg(long, value_name = "NAME")]
        name: String,

        /// List type: user or company
        #[arg(long, default_value = "user")]
        r#type: String,
    },
    /// Get leads from a list
    Leads {
        /// List ID
        #[arg(value_name = "ID")]
        id: i64,

        /// Number of items to return
        #[arg(long, default_value_t = 50)]
        limit: u32,

        /// Number of items to skip
        #[arg(long, default_value_t = 0)]
        offset: u32,

        /// Filter leads by keyword
        #[arg(long)]
        keyword: Option<String>,
    },
    /// Add a lead to a list
    AddLead {
        /// List ID
        #[arg(value_name = "ID")]
        id: i64,

        /// LinkedIn profile URL of the lead
        #[arg(long, value_name = "URL")]
        linkedin_url: String,
    },
    /// Remove a lead from a list
    RemoveLead {
        /// List ID
        #[arg(value_name = "ID")]
        id: i64,

        /// Lead ID to remove
        #[arg(long, value_name = "LEAD_ID")]
        lead: i64,
    },
    /// Get companies from a company list
    Companies {
        /// Company list ID
        #[arg(value_name = "ID")]
        id: i64,

        /// Number of items to return
        #[arg(long, default_value_t = 50)]
        limit: u32,

        /// Number of items to skip
        #[arg(long, default_value_t = 0)]
        offset: u32,
    },
}

// -----------------------------------------------------------------------------
// Leads
// -----------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum Leads {
    /// Get lead details by LinkedIn URL
    Get {
        /// LinkedIn profile URL
        #[arg(long, value_name = "URL")]
        linkedin_url: String,
    },
    /// Get all lists containing a lead
    Lists {
        /// LinkedIn profile URL
        #[arg(long, value_name = "URL")]
        linkedin_url: String,
    },
    /// Update the status of a lead
    UpdateStatus {
        /// Lead ID
        #[arg(long, value_name = "LEAD_ID")]
        lead: i64,

        /// New status: pending, contacted, replied, connected, not_interested, bounced
        #[arg(long, value_name = "STATUS")]
        status: String,
    },
}

// -----------------------------------------------------------------------------
// Senders
// -----------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum Senders {
    /// List all LinkedIn sender accounts
    List {
        /// Number of items to return
        #[arg(long, default_value_t = 10)]
        limit: u32,

        /// Number of items to skip
        #[arg(long, default_value_t = 0)]
        offset: u32,
    },
    /// Get sender account details
    Get {
        /// Sender account ID
        #[arg(value_name = "ID")]
        id: i64,
    },
    /// Get network connections for a sender account
    Network {
        /// Sender account ID
        #[arg(value_name = "ID")]
        id: i64,

        /// Number of items to return
        #[arg(long, default_value_t = 50)]
        limit: u32,

        /// Number of items to skip
        #[arg(long, default_value_t = 0)]
        offset: u32,
    },
}

// -----------------------------------------------------------------------------
// Conversations
// -----------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum Conversations {
    /// List conversations
    List {
        /// Number of items to return
        #[arg(long, default_value_t = 20)]
        limit: u32,

        /// Number of items to skip
        #[arg(long, default_value_t = 0)]
        offset: u32,

        /// Filter by campaign ID
        #[arg(long)]
        campaign_id: Option<i64>,

        /// Filter by sender account ID
        #[arg(long)]
        sender_id: Option<i64>,
    },
    /// Send a message to a lead
    Send {
        /// Lead ID to send message to
        #[arg(long, value_name = "LEAD_ID")]
        lead: i64,

        /// Message text
        #[arg(long, value_name = "MESSAGE")]
        message: String,

        /// Sender account ID
        #[arg(long)]
        sender_id: Option<i64>,
    },
}

// -----------------------------------------------------------------------------
// Stats
// -----------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum Stats {
    /// Get overall stats
    Get {
        /// Start date (yyyy-MM-dd)
        #[arg(long)]
        from: Option<String>,

        /// End date (yyyy-MM-dd)
        #[arg(long)]
        to: Option<String>,

        /// Filter by campaign ID
        #[arg(long)]
        campaign_id: Option<i64>,

        /// Filter by sender account ID
        #[arg(long)]
        sender_id: Option<i64>,
    },
}

// -----------------------------------------------------------------------------
// Webhooks
// -----------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum Webhooks {
    /// List all webhooks
    List {
        /// Number of items to return
        #[arg(long, default_value_t = 10)]
        limit: u32,

        /// Number of items to skip
        #[arg(long, default_value_t = 0)]
        offset: u32,
    },
    /// Get webhook details
    Get {
        /// Webhook ID
        #[arg(value_name = "ID")]
        id: i64,
    },
    /// Create a webhook
    Create {
        /// Webhook name
        #[arg(long, value_name = "NAME")]
        name: String,

        /// Webhook callback URL
        #[arg(long, value_name = "URL")]
        url: String,

        /// Event type (e.g. message_reply, connection_request_accepted)
        #[arg(long, value_name = "EVENT")]
        event: String,

        /// Scope to a specific campaign
        #[arg(long)]
        campaign_id: Option<i64>,
    },
    /// Update a webhook
    Update {
        /// Webhook ID
        #[arg(value_name = "ID")]
        id: i64,

        /// New webhook name
        #[arg(long)]
        name: Option<String>,

        /// New callback URL
        #[arg(long)]
        url: Option<String>,

        /// New event type
        #[arg(long)]
        event: Option<String>,

        /// Activate or deactivate: true or false
        #[arg(long)]
        active: Option<bool>,
    },
    /// Delete a webhook
    Delete {
        /// Webhook ID
        #[arg(value_name = "ID")]
        id: i64,
    },
}

// -----------------------------------------------------------------------------
// Tags
// -----------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum Tags {
    /// Create a workspace tag
    Create {
        /// Tag display name
        #[arg(long, value_name = "NAME")]
        name: String,

        /// Tag color (e.g. #FF0000)
        #[arg(long)]
        color: Option<String>,
    },
}
