use reqwest::{cookie::Jar, Client};
use scraper::{Html, Selector};
use std::sync::Arc;
use chrono::{Datelike, Utc, FixedOffset};
use prettytable::{Table, Row, Cell};
use std::collections::HashMap;
use prettytable::{Attr, color};
use figlet_rs::FIGfont;
use clap::Parser;
use colored::Colorize;

#[derive(Parser)]
struct Cli {
    username: String, 
    password: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args            = Cli::parse();

    let viet_nam_offset = FixedOffset::east_opt(7 * 3600).unwrap();
    let now             = Utc::now().with_timezone(&viet_nam_offset);

    let cookie_store    = Arc::new(Jar::default());
    let client          = Client::builder().cookie_provider(cookie_store.clone()).build()?;

    let login_url       = "https://my.uda.edu.vn/sv/svlogin";
    let mut form        = HashMap::new();

    form.insert("User",                 args.username.as_str());
    form.insert("Password",             args.password.as_str());
    form.insert("__EVENTTARGET",        "Lnew1");
    form.insert("__EVENTARGUMENT",      "");
    form.insert("__VIEWSTATEGENERATOR", "C9E6EC0D");

    println!("Fetching login page...");

    let login_page = client.get(login_url).send().await?.text().await?;

    let document                  = Html::parse_document(&login_page);
    let viewstate_selector        = Selector::parse(r#"input[name="__VIEWSTATE"]"#).unwrap();
    let event_validation_selector = Selector::parse(r#"input[name="__EVENTVALIDATION"]"#).unwrap();

    let viewstate = document
        .select(&viewstate_selector)
        .next()
        .and_then(|input| input.value().attr("value"))
        .unwrap_or_default();

    let event_validation = document
        .select(&event_validation_selector)
        .next()
        .and_then(|input| input.value().attr("value"))
        .unwrap_or_default();

    form.insert("__VIEWSTATE",       viewstate);
    form.insert("__EVENTVALIDATION", event_validation);

    let resp = client
        .post(login_url)
        .header("Content-Type",    "application/x-www-form-urlencoded")
        .header("Host",            "my.uda.edu.vn")
        .header("Origin",          "https://my.uda.edu.vn")
        .header("Referer",         "https://my.uda.edu.vn/sv/svlogin")
        .form(&form)
        .send()
        .await?;

    if resp.status().is_success() {
        let timetable_url = "https://my.uda.edu.vn/sv/tkb";

        println!("Fetching timetable page...");

        let resp_timetable = client.get(timetable_url).send().await?;

        if resp_timetable.status().is_success() {
            let resp_timetable_text = resp_timetable.text_with_charset("utf-8").await?;
            let html                = Html::parse_document(&resp_timetable_text);
            let table_selector      = Selector::parse("#MainContent_GV2").unwrap();
            let tr                  = Selector::parse("tr").unwrap();
            let td                  = Selector::parse("td").unwrap();

            let mut table_pretty    = Table::new();
            let header              = ["THỨ", "BUỔI", "TIẾT", "PHÒNG", "HỌC PHẦN", "GIẢNG VIÊN", "LỚP HỌC TẬP"];

            table_pretty.add_row(Row::new(vec![
                Cell::new(header[0]).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::GREEN)),
                Cell::new(header[1]).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::GREEN)),
                Cell::new(header[2]).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::GREEN)),
                Cell::new(header[3]).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::GREEN)),
                Cell::new(header[4]).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::GREEN)),
                Cell::new(header[5]).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::GREEN)),
                Cell::new(header[6]).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::GREEN)),
            ]));

            let current_weekday = now.weekday();
            let mut map = HashMap::new();

            map.insert("2", chrono::Weekday::Mon);
            map.insert("3", chrono::Weekday::Tue);
            map.insert("4", chrono::Weekday::Wed);
            map.insert("5", chrono::Weekday::Thu);
            map.insert("6", chrono::Weekday::Fri);
            map.insert("7", chrono::Weekday::Sat);
            map.insert("8", chrono::Weekday::Sun);

            if let Some(table) = html.select(&table_selector).next() {
                for row in table.select(&tr) {
                    let row_data: Vec<_> = row
                        .select(&td)
                        .map(|cell| cell.text().collect::<String>().trim().to_string())
                        .collect();

                    let mut row_data_formatted: Vec<&str> = Vec::new();
                    let mut is_current_day                = false;

                    for (i, r) in row_data.iter().enumerate() {
                        match i {
                            0 => {
                                if let Some(&value) = map.get(r.as_str()) {
                                        is_current_day = value == current_weekday;
                                }
                            },
                            3 => {
                                let t = r.split('\n').next().unwrap().trim();
                                row_data_formatted.push(t);
                            }
                            _ => {
                                row_data_formatted.push(r);
                            }

                        }
                    }
                    if row_data_formatted.len() < 2 {
                        continue
                    }

                    if is_current_day {
                        table_pretty.add_row(
                            Row::new(vec![
                                Cell::new(&row_data_formatted[0]).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::YELLOW)), 
                                Cell::new(&row_data_formatted[1]).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::YELLOW)), 
                                Cell::new(&row_data_formatted[2]).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::YELLOW)), 
                                Cell::new(&row_data_formatted[3]).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::YELLOW)), 
                                Cell::new(&row_data_formatted[4]).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::YELLOW)), 
                                Cell::new(&row_data_formatted[5]).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::YELLOW)), 
                                Cell::new(&row_data_formatted[6]).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::YELLOW)), 
                            ])
                        );
                    } else {
                        table_pretty.add_row(Row::from(row_data_formatted));
                    }
                }
            }

            let standard_font  = FIGfont::standard().unwrap();
            let formatted_date = standard_font.convert(&format!("{}/{}/{}", &now.day(), &now.month(), &now.year()).to_string()).unwrap();

            println!("{}", formatted_date);

            if table_pretty.len() < 2 {
                println!("Recheck your username and password");
                panic!("{}", "There is nothing in the timetable to display, if it was a mistake, please recheck your username and password.".red());
            } else {
                table_pretty.printstd();
            }
            return Ok(());
        };
    } else {
        println!("Failed: {}", resp.status());
    }

    Ok(())
}
