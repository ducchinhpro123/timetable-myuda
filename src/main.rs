use colored::Colorize;
use indicatif::ProgressBar;
use request::{
    cancellation_notice, exam_schedule, extract_upcoming_schedule, timetable_table, UserConfig,
};
use reqwest::{cookie::Jar, Client};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration};

use dotenv::dotenv;
use std::env;

mod quote;
use crate::quote::get_quote;

/*=======================================================================================================+
 |  ███╗   ███╗ █████╗ ██╗███╗   ██╗    ██████╗ ██████╗  ██████╗  ██████╗ ██████╗  █████╗ ███╗   ███╗    |
 |  ████╗ ████║██╔══██╗██║████╗  ██║    ██╔══██╗██╔══██╗██╔═══██╗██╔════╝ ██╔══██╗██╔══██╗████╗ ████║    |
 |  ██╔████╔██║███████║██║██╔██╗ ██║    ██████╔╝██████╔╝██║   ██║██║  ███╗██████╔╝███████║██╔████╔██║    |
 |  ██║╚██╔╝██║██╔══██║██║██║╚██╗██║    ██╔═══╝ ██╔══██╗██║   ██║██║   ██║██╔══██╗██╔══██║██║╚██╔╝██║    |
 |  ██║ ╚═╝ ██║██║  ██║██║██║ ╚████║    ██║     ██║  ██║╚██████╔╝╚██████╔╝██║  ██║██║  ██║██║ ╚═╝ ██║    |
 |  ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝    ╚═╝     ╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝    |
 *=======================================================================================================+
*/
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    let user_config = UserConfig::from_env();

    if let Err(e) = user_config.validate() {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    }

    let username = user_config.get_username().unwrap();
    let password = user_config.get_password().unwrap();

    let bar = ProgressBar::new_spinner();

    bar.enable_steady_tick(Duration::from_millis(5));

    let cookie_store = Arc::new(Jar::default());
    let client = Client::builder()
        .cookie_provider(cookie_store.clone())
        .build()?;

    let login_url = "https://my.uda.edu.vn/sv/svlogin";
    let timetable_url = "https://my.uda.edu.vn/sv/tkb";
    let exam_schedule_url = "https://my.uda.edu.vn/sv/lichthi";

    bar.set_message("Waiting for my.uda.edu.vn/sv/svlogin response...");
    // let login_page = client.get(login_url).send().await?.text().await?;

    let mut form = HashMap::new();

    form.insert("User", username.as_str());
    form.insert("Password", password.as_str());
    form.insert("__EVENTTARGET", "Lnew1");
    // form.insert("__EVENTARGUMENT", "");
    // form.insert("__VIEWSTATEGENERATOR", "C9E6EC0D");

    bar.set_message("Login");
    // Post login request with assigned form of data
    let resp = client
        .post(login_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Host", "my.uda.edu.vn")
        .header("Origin", "https://my.uda.edu.vn")
        .header("Referer", "https://my.uda.edu.vn/sv/svlogin")
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
        )
        .header("Connection", "keep-alive")
        .form(&form)
        .send()
        .await?;

    // If login successfully
    if resp.status().is_success() {
        bar.set_message("Login successfully, getting information");
        let (resp_timetable, resp_exam_schedule) = tokio::join!(
            client.get(timetable_url).send(),
            client.get(exam_schedule_url).send()
        );

        let timetable = resp_timetable?;
        let exam = resp_exam_schedule?;

        if !timetable.status().is_success() {
            eprintln!("Failed to fetch timetable: {}", timetable.status());
            return Ok(());
        }
        if !exam.status().is_success() {
            eprintln!("Failed to fetch exam schedule: {}", exam.status());
            return Ok(());
        }

        let resp_timetable_text = timetable.text_with_charset("utf-8").await?;
        let resp_exam_text = exam.text_with_charset("utf-8").await?;

        if resp_timetable_text.is_empty() {
            eprintln!("Timetable response is empty");
            return Ok(());
        }

        // (HTML objects are not thread-safe due to internal Cell usage)
        let html_timetable = Html::parse_document(&resp_timetable_text);
        let html_exam = Html::parse_document(&resp_exam_text);

        // Create selectors once
        let tr = Selector::parse("tr").unwrap();
        let td = Selector::parse("td").unwrap();

        let announcement_table = cancellation_notice(&html_timetable, &tr, &td);
        let upcoming_schedule = extract_upcoming_schedule(&html_timetable, &tr, &td);
        let timetable_table = timetable_table(html_timetable, tr.clone(), td.clone());
        let exam_schedule_table = exam_schedule(&html_exam, &tr, &td);

        bar.set_message("Displaying results...");
        bar.finish_and_clear();

        if timetable_table.len() > 1 {
            println!("Thời khóa biểu chính thức (Official schedule)");
            timetable_table.printstd();
        } else {
            println!("Thời khóa biểu trống");
        }

        if upcoming_schedule.len() > 1 {
            println!("Thời khóa biểu sắp tới (Upcoming schedule)");
            upcoming_schedule.printstd();
        } else {
            println!("Thời khóa biểu sắp tới trống");
        }

        if announcement_table.len() > 1 {
            println!("Thông báo nghỉ (Cancellation schedule notice)");
            announcement_table.printstd();
        } else {
            println!("Không có thông báo nghỉ");
        }

        if exam_schedule_table.len() > 1 {
            println!("Thông báo thi (Exam schedule notice)");
            exam_schedule_table.printstd();
        } else {
            println!("Không có thông báo thi");
        }

        dotenv().ok();
        let daily_quote_api = env::var("DAILY_QUOTE_API").ok().or_else(|| {
                eprintln!("Warning: DAILY_QUOTE_API not set, using None");
                None
        }).unwrap();

        let quote = get_quote(&daily_quote_api).await;

        match quote {
            Ok(q) => println!(
                "{} - {}",
                q.quote.bright_green().bold(),
                q.author.magenta().italic()
            ),
            Err(_) => eprintln!("Talk is cheap, so me the code - Linus Torvalds"),
        }

        println!("{}", "◕‿◕) GOODBYE!!!".black().on_white());

        return Ok(());
    } else {
        eprintln!("Failed: {}", resp.status());
    }

    Ok(())
}
