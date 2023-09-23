use rand::Rng;
use std::{
    collections::HashMap,
    default::Default,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
};

use serenity::{
    async_trait,
    builder::{CreateApplicationCommandOption, CreateEmbed},
    client::{Context, EventHandler},
    model::{
        application::{
            command::{Command, CommandOptionType},
            interaction::{application_command::ApplicationCommandInteraction, Interaction},
        },
        channel::Message,
        gateway::Ready,
        id::MessageId,
        prelude::application_command::CommandDataOptionValue,
    },
    prelude::TypeMapKey,
    utils::Color,
};

pub struct ConfessionCount;

impl TypeMapKey for ConfessionCount {
    type Value = AtomicUsize;
}

pub struct Handler;

impl Handler {
    pub fn new() -> Self {
        Self
    }

    fn display_message(msg: Message) {
        if !msg.content.is_empty() {
            println!("[{}] {}", msg.author.name, msg.content)
        } else if !msg.attachments.is_empty() {
            println!("[{}] {}", msg.author.name, msg.attachments[0].url);
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, msg: Message) {
        Handler::display_message(msg);
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        println!("Connected!");
        ConfessionCommands::register_commands(ctx).await;
        println!("Registered Commands!");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        ConfessionCommands::resolve_interaction(ctx, interaction).await
    }
}

//
// TODO: Delete by message number (ex. input: /delete #123)
//
type AuthorID = u64;
type GuildID  = u64;
type ConfessionNumber = u64;

// Confession numbers are ascending anyways, so might store them as flat array?
// type GuildData = HashMap<ConfessionNumber, AuthorID>;

// Option to indicate whether already deleted (or maybe just set to 0)?
//                   VVVVVV
type GuildData = Vec<Option<AuthorID>>;
type ConfessionNumberMap = Mutex<HashMap<GuildID, GuildData>>;
static USER_CONFESSIONS_BY_NUMBER: OnceLock<ConfessionNumberMap> = OnceLock::new();

fn create_confession_number_data() -> ConfessionNumberMap {
    let data = HashMap::new();
    Mutex::new(data)
}

//
// TODO: Once every week clear messages that are older that 2 weeks since the bot cannot 
//       delete them anyway (due to discord bot API limitation)
//
//Mutex choice? std::sync, tokio::sync(https://github.com/tokio-rs/tokio/issues/2599)
type MessageID = u64;
type ConcurrentMap = Mutex<HashMap<MessageID, AuthorID>>;
static CONFESSIONS_TO_USERS: OnceLock<ConcurrentMap> = OnceLock::new();

fn create_confession_data() -> ConcurrentMap {
    let data = HashMap::new();
    Mutex::new(data)
}

pub struct ConfessionCommands;

impl ConfessionCommands {
    pub async fn register_commands(context: Context) {
        let text_option = CreateApplicationCommandOption::default()
            .name("text")
            .description("confession")
            .kind(CommandOptionType::String)
            .required(true)
            .to_owned();

        let attachment_option = CreateApplicationCommandOption::default()
            .name("attachment")
            .description("A file")
            .kind(CommandOptionType::Attachment)
            .required(false)
            .to_owned();

        Command::create_global_application_command(&context.http, |command| {
            command
                .name("confess")
                .description("Confess your sins anonymously")
                .add_option(text_option)
                .add_option(attachment_option)
        }).await.expect("Unable to register the confess command");

        let message_id_option = CreateApplicationCommandOption::default()
            .name("message_id")
            .description("Message id of the confession")
            .kind(CommandOptionType::String)
            .required(true)
            .to_owned();

        Command::create_global_application_command(&context.http, |command| {
            command
                .name("delete")
                .description("Delete your thought silently")
                .add_option(message_id_option)
        }).await.expect("Unable to register the delete command");

        let user_option = CreateApplicationCommandOption::default()
            .name("user")
            .description("User to be reported")
            .kind(CommandOptionType::User)
            .required(true)
            .to_owned();

        let reason_option = CreateApplicationCommandOption::default()
            .name("reason")
            .description("What did they do?")
            .kind(CommandOptionType::String)
            .required(false)
            .to_owned();

        Command::create_global_application_command(&context.http, |command| {
            command
                .name("report")
                .description("Report user")
                .add_option(user_option)
                .add_option(reason_option)
        }).await.expect("Unable to register the report command");
    }

    pub async fn confess(context: Context, command: ApplicationCommandInteraction) {
        println!("Confess command from: {}", command.user.name);
        println!("Options len: {}", command.data.options.len());

        command
            .defer_ephemeral(&context.http).await
            .expect("Failed to defer");

        let options = &command.data.options;
        let text_option = &options[0].resolved;

        let Some(CommandDataOptionValue::String(text)) = text_option else {
            return;
        };

        let confession_count = {
            let data = context.data.read().await;
            let counter = data
                .get::<ConfessionCount>()
                .expect("Failed to get ConfessionCount");
            counter.fetch_add(1, Ordering::Relaxed)
        };

        let maybe_delivered = command.channel_id.send_message(&context.http, |message| {
            let mut rng = rand::thread_rng();
            let rgb_color = rng.gen_range(0..=0xFFFFFF);
            let footer_text = "❗ If this confession is ToS-breaking or overtly hateful, you can report it using \"/report\"";

            let mut embed = CreateEmbed::default()
                .title(format!("Anonymous Confession (#{})", confession_count))
                .description(text)
                .footer(|footer| footer.text(footer_text))
                .color(Color::new(rgb_color))
                .to_owned();

            if command.data.options.len() > 1 {
                let attach_option = &options[1].resolved;
                if let Some(CommandDataOptionValue::Attachment(attachment)) = attach_option {
                    println!("url: {}", attachment.url);
                    embed.image(&attachment.url);
                }
            }

            message.set_embed(embed)
        }).await;

        let Ok(delivered) = maybe_delivered else {
            let err = maybe_delivered.unwrap_err();
            command.edit_original_interaction_response(&context.http, |edit| {
                edit.content(err.to_string())
            }).await.expect("Unable to edit");
            return;
        };

        let data = CONFESSIONS_TO_USERS.get_or_init(create_confession_data);
        data.lock().unwrap().insert(delivered.id.0, command.user.id.0);

        command
            .delete_original_interaction_response(&context.http).await
            .expect("Failed to delete response");
    }

    pub async fn delete(context: Context, command: ApplicationCommandInteraction) {
        println!("Delete command from: {}", command.user.name);

        let options = &command.data.options;
        let Some(CommandDataOptionValue::String(msg_id_str)) = &options[0].resolved else {
            return;
        };

        let parse_result = msg_id_str.parse();
        if parse_result.is_err() {
            command.create_interaction_response(&context.http, |response| {
                response.interaction_response_data(|data| {
                    data.content("Expected a positive number (usually 18-19 digits).").ephemeral(true)
                })
            }).await.expect("Unable to respond");
            return;
        }

        let msg_id = parse_result.unwrap();
        let mut authors_match = false;
        let mut msg_exists = false;

        {
            let data = CONFESSIONS_TO_USERS.get_or_init(create_confession_data);
            let map_guard = data.lock().unwrap();
            if let Some(actual_author_id) = map_guard.get(&msg_id) {
                msg_exists = true;
                authors_match = command.user.id.0 == *actual_author_id;
            };
        }

        if !msg_exists {
            command.create_interaction_response(&context.http, |response| {
                response.interaction_response_data(|data| {
                    data.content("Message with that id does not exist").ephemeral(true)
                })
            }).await.expect("Unable to respond");
            return;
        }

        if authors_match {
            command.channel_id
                .delete_message(&context.http, MessageId(msg_id)).await
                .expect("Unable to delete");

            command.create_interaction_response(&context.http, |response| {
                response.interaction_response_data(|data| {
                    data.content(":white_check_mark: Confession deleted.").ephemeral(true)
                })
            }).await.expect("Unable to respond");
            return;
        }
        command.create_interaction_response(&context.http, |response| {
            response.interaction_response_data(|data| {
                data.content("You're not the author of that confession or message wasn't recorded").ephemeral(true)
            })
        }).await.expect("Unable to respond");
    }

    pub async fn report(context: Context, command: ApplicationCommandInteraction) {
        let name = &command.user.name;
        let content = format!("{name} tried to report a user");

        command.channel_id .send_message(&context.http, |msg| msg.content(content)).await
            .expect("Message was not sent.");

        command
            .defer_ephemeral(&context.http).await
            .expect("Unable to defer to delete interaction later");

        command
            .delete_original_interaction_response(&context.http).await
            .expect("Unable to close interaction")
    }

    pub async fn resolve_interaction(context: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Command: {:?}", command);
            match command.data.name.as_str() {
                "confess" => ConfessionCommands::confess(context, command).await,
                "delete"  => ConfessionCommands::delete(context, command).await,
                "report"  => ConfessionCommands::report(context, command).await,
                _ => {}
            }
        }
    }
}
