use dotenv::dotenv;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use serenity::{
    async_trait,
    client::{/* bridge::gateway::GatewayIntents, */ Client, Context, EventHandler},
    framework::standard::{
        macros::{command, group},
        CommandResult, StandardFramework,
    },
    model::{
        channel::Message,
        gateway::Ready,
        /* interactions::{ApplicationCommand, Interaction, InteractionResponseType}, */
    },
    utils::MessageBuilder,
};
use std::collections::HashMap;
use std::env;

#[group]
#[commands(
    ping,
    getip,
    gethnpost,
    gethealth,
    getblogs,
    getghorginfo,
    getghorgmembers
)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if msg.content == "!ping" {
            let channel = match msg.channel_id.to_channel(&context).await {
                Ok(channel) => channel,
                Err(why) => {
                    println!("Error getting channel: {:?}", why);

                    return;
                }
            };

            // The message builder allows for creating a message by
            // mentioning users dynamically, pushing "safe" versions of
            // content (such as bolding normalized content), displaying
            // emojis, and more.
            let response = MessageBuilder::new()
                .push("User ")
                .push_bold_safe(&msg.author.name)
                .push(" used the 'ping' command in the ")
                .mention(&channel)
                .push(" channel")
                .build();

            if let Err(why) = msg.channel_id.say(&context.http, &response).await {
                println!("Error sending message: {:?}", why);
            }
        }
        if msg.content == "!messageme" {
            // If the `utils`-feature is enabled, then model structs will
            // have a lot of useful methods implemented, to avoid using an
            // often otherwise bulky Context, or even much lower-level `rest`
            // method.
            //
            // In this case, you can direct message a User directly by simply
            // calling a method on its instance, with the content of the
            // message.
            let dm = msg
                .author
                .dm(&context, |m| {
                    m.content("Hello! thanks for using this bot.");

                    m
                })
                .await;

            if let Err(why) = dm {
                println!("Error when direct messaging user: {:?}", why);
            }
        }
        if msg.content == "!hello" {
            // The create message builder allows you to easily create embeds and messages
            // using a builder syntax.
            // This example will create a message that says "Hello, World!", with an embed that has
            // a title, description, three fields, and a footer.
            let msg = msg
                .channel_id
                .send_message(&context.http, |m| {
                    m.content("Hello, World!");
                    m.embed(|e| {
                        e.title("This is a title");
                        e.description("This is a description");
                        e.fields(vec![
                            ("This is the first field", "This is a field body", true),
                            (
                                "This is the second field",
                                "Both of these fields are inline",
                                true,
                            ),
                        ]);
                        e.field(
                            "This is the third field",
                            "This is not an inline field",
                            false,
                        );
                        e.footer(|f| {
                            f.text("This is a footer");

                            f
                        });

                        e
                    });
                    m
                })
                .await;

            if let Err(why) = msg {
                println!("Error sending message: {:?}", why);
            }
        }
    }
    /*  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content("Received event!"))
            })
            .await;
    } */

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        /*   let interactions = ApplicationCommand::get_global_application_commands(&ctx.http).await; */
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    let token = env::var("TOKEN").expect("token");
    // The Application Id is usually the Bot User Id.
    /*  let application_id: u64 = env::var("APPLICATION_ID")
    .expect("Expected an application id in the environment")
    .parse()
    .expect("application id is not a valid id"); */

    let mut client = Client::builder(token)
        .event_handler(Handler)
        /*  .application_id(application_id) */
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}

#[command]
async fn getip(ctx: &Context, msg: &Message) -> CommandResult {
    let resp = reqwest::get("https://httpbin.org/ip")
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    msg.reply(ctx, resp["origin"].clone()).await?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct HNPost {
    by: String,
    descendants: i32,
    id: f64,
    score: i32,
    time: f64,
    title: String,
    r#type: String,
    url: Option<String>,
}

type HNTopStories = Vec<u32>;

#[command]
async fn gethnpost(ctx: &Context, msg: &Message) -> CommandResult {
    let uri_topstories = "https://hacker-news.firebaseio.com/v0/topstories.json";
    let uri_post = "https://hacker-news.firebaseio.com/v0/item/";

    let top_stories = reqwest::get(uri_topstories)
        .await?
        .json::<HNTopStories>()
        .await?;

    for (key, item) in top_stories.iter().enumerate() {
        if key < 30 {
            let uri_item = format!("{}{}.json", uri_post, &item);
            let post = reqwest::get(uri_item.as_str())
                .await?
                .json::<HNPost>()
                .await?;
            let post_url: String = match &post.url {
                Some(url) => url.to_string(),
                None => "none".to_string(),
            };
            msg.reply(
                ctx,
                format!(
                    "{{
title: {}
by: {}
url: {}
}}",
                    &post.title, &post.by, &post_url
                ),
            )
            .await?;
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Welcome {
    welcome: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ResWelcome {
    data: Welcome,
}

#[command]
async fn gethealth(ctx: &Context, msg: &Message) -> CommandResult {
    let client = reqwest::Client::new();

    let mut map = HashMap::new();
    map.insert("query", "{welcome}");

    let res = client
        .post("https://alfianguide-be.herokuapp.com/graphql")
        .json(&map)
        .send()
        .await?;

    let greet: ResWelcome = res.json().await?;

    msg.reply(ctx, &greet.data.welcome).await?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Blog {
    id: String,
    slug: String,
    title: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Blogs {
    blogs: Vec<Blog>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ResBlogs {
    data: Blogs,
}

#[command]
async fn getblogs(ctx: &Context, msg: &Message) -> CommandResult {
    let client = reqwest::Client::new();

    let mut map = HashMap::new();
    map.insert("query", "{  blogs {   id    slug    title    content  } }");

    let res = client
        .post("https://alfianguide-be.herokuapp.com/graphql")
        .json(&map)
        .send()
        .await?;

    let blogs: ResBlogs = res.json().await?;

    for (_key, blog) in blogs.data.blogs.iter().enumerate() {
        let msg = msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(&blog.title);
                    e.description(format!(
                        "```html
                    {}
                    ```",
                        &blog.content
                    ));

                    e
                });
                m
            })
            .await;

        if let Err(why) = msg {
            println!("Error sending message: {:?}", why);
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct GHOrganization {
    login: String,
    id: u32,
    node_id: String,
    url: String,
    repos_url: String,
    events_url: String,
    hooks_url: String,
    issues_url: String,
    members_url: String,
    public_members_url: String,
    avatar_url: String,
    description: String,
    name: String,
    company: Option<String>,
    blog: Option<String>,
    location: Option<String>,
    email: Option<String>,
    twitter_username: Option<String>,
    is_verified: bool,
    has_organization_projects: bool,
    has_repository_projects: bool,
    public_repos: u32,
    public_gists: u32,
    followers: u32,
    following: u32,
    html_url: String,
    created_at: String,
    updated_at: String,
    r#type: String,
}

#[command]
async fn getghorginfo(ctx: &Context, msg: &Message) -> CommandResult {
    let splitmsg = msg.content.split(" ");
    let args: Vec<&str> = splitmsg.collect();
    let (org_name) = (args[1]);
    let client = reqwest::Client::new();
    let res = client
        .get(format!("https://api.github.com/orgs/{}", org_name))
        .header(USER_AGENT, "My Rust Program 1.0")
        .send()
        .await?;

    let org: GHOrganization = res.json().await?;

    let org_location: &str = &org.location.clone().unwrap_or("-".to_string());

    let org_blog: &str = &org.blog.clone().unwrap_or("-".to_string());

    let org_email: &str = &org.email.clone().unwrap_or("-".to_string());

    let org_company: &str = &org.company.clone().unwrap_or("-".to_string());

    let org_twitter = match &org.company {
        Some(x) => ("https://twitter.com/".to_owned() + x),
        None => "-".to_string(),
    };

    let msg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(&org.name);
                e.description(&org.description);
                e.image(&org.avatar_url);
                e.fields(vec![
                    ("Location", org_location, true),
                    ("Email", org_email, true),
                    ("Company", org_company, true),
                    ("Link", &org.html_url, true),
                    ("Blog", org_blog, true),
                    ("Twitter", &org_twitter, true),
                    ("Public Repos", &org.public_repos.to_string(), true),
                ]);

                e
            });
            m
        })
        .await;

    if let Err(why) = msg {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct GHOrgMember {
    login: String,
    id: u32,
    node_id: String,
    avatar_url: String,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
    r#type: String,
    site_admin: bool,
}

type GHOrgMembers = Vec<GHOrgMember>;

#[command]
async fn getghorgmembers(ctx: &Context, msg: &Message) -> CommandResult {
    let splitmsg = msg.content.split(" ");
    let args: Vec<&str> = splitmsg.collect();
    let (org_name) = (args[1]);
    let client = reqwest::Client::new();
    let res = client
        .get(format!("https://api.github.com/orgs/{}/members", org_name))
        .header(USER_AGENT, "My Rust Program 1.0")
        .send()
        .await?;

    let members: GHOrgMembers = res.json().await?;

    for (_key, member) in members.iter().enumerate() {
        let msg = msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(&member.login);
                    e.description(&member.html_url);
                    e.image(&member.avatar_url);

                    e
                });
                m
            })
            .await;

        if let Err(why) = msg {
            println!("Error sending message: {:?}", why);
        }
    }

    Ok(())
}
