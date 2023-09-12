use rand::Rng;
use serenity::{
    async_trait,
    builder::CreateApplicationCommandOption,
    client::{Context, EventHandler},
    model::{
        application::{
            command::{Command, CommandOptionType},
            interaction::{application_command::ApplicationCommandInteraction, Interaction},
        },
        channel::Message,
        gateway::Ready,
        prelude::application_command::CommandDataOptionValue
    },
    utils::Color, prelude::TypeMapKey,
};
use std::sync::{atomic::{Ordering, AtomicUsize}, Arc};

pub struct ConfessionCount;
impl TypeMapKey for ConfessionCount {
    type Value = Arc<AtomicUsize>;
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

        command.defer_ephemeral(&context.http).await.expect("Failed to defer");

        let options = &command.data.options;
        let text_option = &options[0].resolved;

        let Some(CommandDataOptionValue::String(text)) = text_option else {
            return;
        };

        let confession_count = {
            let data = context.data.read().await;
            let counter = data.get::<ConfessionCount>()
                .expect("Failed to get ConfessionCount")
                .clone();
            counter.fetch_add(1, Ordering::Relaxed)
        };

        command.channel_id.send_message(&context.http, |msg| {
            let mut rng = rand::thread_rng();
            let r = rng.gen_range(0..256) as u8;
            let g = rng.gen_range(0..256) as u8;
            let b = rng.gen_range(0..256) as u8;

            msg.embed(|embed| {
                embed
                    .title(format!("Anonymous Confession (#{})", confession_count))
                    .description(text)
                    .footer(|footer| {
                        footer.text("❗ If this confession is ToS-breaking or overtly hateful, you can report it using \"/report\"")
                    })
                    .color(Color::from_rgb(r, g, b));

                if command.data.options.len() > 1 {
                    let attach_option = &options[1].resolved;
                    if let Some(CommandDataOptionValue::Attachment(attachment)) = attach_option {
                        println!("url: {}", attachment.url);
                        embed.image(&attachment.url);
                    }
                }
                embed
            })
        }).await.expect("Failed to send");

        // command.create_interaction_response(&context.http, |interaction| {
        //     interaction.interaction_response_data(|data| {
        //         data.ephemeral(true).content(":white_check_mark: Your confession has been added!")
        //     });
        //     interaction
        // }).await.expect("Unable to interact.");

        command.delete_original_interaction_response(&context.http).await.expect("Failed to delete response");
    }

    pub async fn delete(_context: Context, command: ApplicationCommandInteraction) {
        println!("Delete command from: {}", command.user.name);
    }

    pub async fn report(context: Context, command: ApplicationCommandInteraction) {
        let name = command.user.name;
        let content = format!("{name} tried to report a confession");

        command
            .channel_id
            .send_message(&context.http, |msg| msg.content(content))
            .await
            .expect("Message wasd not sent.");
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
