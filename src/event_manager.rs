use std::any::Any;
use std::cell::UnsafeCell;
use std::io::ErrorKind::Interrupted;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU32, Ordering};
use serenity::async_trait;
use serenity::builder::{CreateApplicationCommand, CreateApplicationCommandOption, CreateEmbed, CreateInteractionResponse};
use serenity::client::{Context, EventHandler};
use serenity::model::application::command::{Command, CommandOptionType};
use serenity::model::channel::{Message};
use serenity::model::gateway::Ready;
use serenity::model::application::interaction::Interaction;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use rand::Rng;
use serenity::utils::Color;

pub struct Handler;

impl Handler{
    pub fn new() -> Self{
        Self{}
    }
    fn display_message(msg: Message){
        if len(&msg.content) == 0 {
            if msg.attachments.is_empty() {
                return
            }
            println!("[{}] {}", msg.author.name, msg.attachments[0].url);
            return
        }
        println!("[{}] {}", msg.author.name, msg.content)
    }
}

pub fn len(str: &String) -> usize{
    str.chars().count()
}


#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        Handler::display_message(msg);
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        println!("Connected!");
        ConfessionCommands::register_commands( ctx).await;
        println!("Registered Commands!");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        ConfessionCommands::resolve_interaction(ctx, interaction).await
    }
}


// delete(link/msg_id), prefix/help(), reveal
pub struct ConfessionCommands;

static mut COUNTER: AtomicU32 = AtomicU32::new(1);

impl ConfessionCommands {
    fn new() -> Self {
        Self{}
    }
}

impl ConfessionCommands {
    pub async fn register_commands(context: Context) {
        let mut text_option = CreateApplicationCommandOption::default();
        text_option
            .name("text")
            .description("confession")
            .kind(CommandOptionType::String)
            .required(true);
        let mut attachment_option = CreateApplicationCommandOption::default();
        attachment_option
                .name("attachment")
                .description("A file")
                .kind(CommandOptionType::Attachment)
                .required(false);
        let confess_command = Command::create_global_application_command(&context.http, |command| {
            command.name("confess").description("Confess your sins anonymously")
                .add_option(text_option)
                .add_option(attachment_option)
        }).await;
        confess_command.expect("Unable to register the confess command");


        let mut message_id_option = CreateApplicationCommandOption::default();
        message_id_option
                .name("message_id")
                .description("Message id of the confession")
                .kind(CommandOptionType::String)
                .required(true);

        let delete_command = Command::create_global_application_command(&context.http, |command| {
            command.name("delete").description("Delete your thought silently")
                .add_option(message_id_option)
        }).await;
        delete_command.expect("Unable to register the delete command");


        let mut user_option = CreateApplicationCommandOption::default();
        user_option
            .name("user")
            .description("User to be reported")
            .kind(CommandOptionType::User)
            .required(true);

        let mut reason_option = CreateApplicationCommandOption::default();
        reason_option
            .name("reason")
            .description("What did they do?")
            .kind(CommandOptionType::String)
            .required(false);
        let report_command = Command::create_global_application_command(&context.http, |command| {
            command.name("report").description("Report user")
                .add_option(user_option)
                .add_option(reason_option)
        }).await;
        report_command.expect("Unable to register the report command");
    }


    pub async fn confess(context: Context, command: ApplicationCommandInteraction){
        println!("Confess command from: {}", command.user.name);
        println!("Options len: {}", command.data.options.len());
        let text_option = &command.data.options[0].as_val();

        command.create_interaction_response(&context.http, |interaction|{
            interaction.interaction_response_data(|data| {
                data.ephemeral(true).content(":white_check_mark: Your confession has been added!")
            });
            interaction
        }).await.expect("unable to interact");
        let CommandDataOptionValue::String(text) = text_option else {
            return;
        };
        command.channel_id.send_message(&context.http, |msg| {
            let mut rng = rand::thread_rng();
            let r = rng.gen_range(0..256) as u8;
            let g = rng.gen_range(0..256) as u8;
            let b = rng.gen_range(0..256) as u8;

            msg.embed(|embed| unsafe {
                embed.title(format!("Anonymous Confession (#{})", COUNTER.fetch_add(1, Ordering::Relaxed)))
                    .description(text)
                    .footer(|footer| {
                        footer.text("❗ If this confession is ToS-breaking or overtly hateful, you can report it using \"/report\"")
                    })
                    .color(Color::from_rgb(r, g, b));
                if command.data.options.len() > 1 {
                    let attach_option = command.data.options[1].as_val();
                    if let CommandDataOptionValue::Attachment(attachment) = attach_option {
                        println!("url:{}", attachment.url);
                        embed.image(&attachment.url);
                    }
                }
                embed
            })
        }).await.expect("Failed to send");
    }
    pub async fn delete(context: Context, command: ApplicationCommandInteraction){
        println!("Delete command from: {}", command.user.name);
    }

    pub async fn report(context: Context, command: ApplicationCommandInteraction) {
        let mut content = String::from(command.user.name);
        content.push_str(" tried to report a confession");

        command.channel_id.send_message(&context.http, |msg|{
            msg.content(content)
        }).await.expect("not sent");
    }

    pub async fn resolve_interaction(context: Context, interaction: Interaction){
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

trait ToOptionValue{
    fn as_val(&self) -> &CommandDataOptionValue;
}

impl ToOptionValue for CommandDataOption{
    fn as_val(&self) -> &CommandDataOptionValue{
        self.resolved.as_ref().unwrap()
    }
}

trait Read{
    fn read(&self) -> u32;
}

impl Read for AtomicU32{
    fn read(&self) -> u32 {
        unsafe{
            self.load(Ordering::Relaxed)
        }
    }
}
