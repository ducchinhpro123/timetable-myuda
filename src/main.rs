use chrono::NaiveDate;
use chrono::{Datelike, Duration, FixedOffset, Utc};
use prettytable::{color, Attr};
use prettytable::{Cell, Row, Table};
use reqwest::{cookie::Jar, Client};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::Arc;
use clap::Parser;
use colored::Colorize;
use chrono;

use std::time::Instant;

#[derive(Parser)]
struct Cli {
    username: String,
    password: String
}

fn exam_schedule(html: &Html, tr: &Selector, td: &Selector) -> Table {
    let header = [
        "Học kỳ",
        "Tên học phần",
        "Số TC",
        "Ngày thi",
        "Xuất",
        "Thời gian thi",
        "Phòng",
        "Hình thức",
    ];
    let mut table = Table::new();
    let viet_nam_offset = FixedOffset::east_opt(7 * 3600).unwrap();
    let now = Utc::now().with_timezone(&viet_nam_offset);
    let sc = Selector::parse("#MainContent_GV2").unwrap();

    table.add_row(Row::new(vec![
        Cell::new(header[0])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
        Cell::new(header[1])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
        Cell::new(header[2])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
        Cell::new(header[3])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
        Cell::new(header[4])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
        Cell::new(header[5])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
        Cell::new(header[6])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
        Cell::new(header[7])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
    ]));

    if let Some(r) = html.select(&sc).next() {
        for row in r.select(&tr) {
            let row_data: Vec<_> = row
                .select(&td)
                .map(|cell| cell.text().collect::<String>().trim().to_string())
                .collect();
            if row_data.len() > 3 {
                if let Ok(date) = NaiveDate::parse_from_str(&row_data[3], "%d/%m/%Y") {
                    let date_time = date
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_local_timezone(viet_nam_offset)
                        .unwrap();
                    if (date_time + Duration::days(1)) >= now {
                        let styled_row = Row::new(
                            row_data
                                .iter()
                                .enumerate()
                                .map(|(i, v)| {
                                    if i == 3 {
                                        Cell::new(v)
                                            .with_style(Attr::Bold)
                                            .with_style(Attr::ForegroundColor(color::RED))
                                    } else {
                                        Cell::new(v)
                                    }
                                })
                                .collect(),
                        );
                        table.add_row(styled_row);
                    }
                }
            }
        }
    }
    return table;
}

fn timetable_table(html: Html, tr: Selector, td: Selector) -> Table {
    let viet_nam_offset = FixedOffset::east_opt(7 * 3600).unwrap();
    let now = Utc::now().with_timezone(&viet_nam_offset);

    let header = [
        "THỨ",
        "BUỔI",
        "TIẾT",
        "PHÒNG",
        "HỌC PHẦN",
        "GIẢNG VIÊN",
        "LỚP HỌC TẬP",
    ];

    let table_selector = Selector::parse("#MainContent_GV2").unwrap();
    let mut table_pretty = Table::new();

    let current_weekday = now.weekday();
    let next_day = current_weekday.succ();
    let mut map = HashMap::new();

    map.insert("2", chrono::Weekday::Mon);
    map.insert("3", chrono::Weekday::Tue);
    map.insert("4", chrono::Weekday::Wed);
    map.insert("5", chrono::Weekday::Thu);
    map.insert("6", chrono::Weekday::Fri);
    map.insert("7", chrono::Weekday::Sat);
    map.insert("8", chrono::Weekday::Sun);

    table_pretty.add_row(Row::new(vec![
        Cell::new(header[0])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
        Cell::new(header[1])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
        Cell::new(header[2])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
        Cell::new(header[3])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
        Cell::new(header[4])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
        Cell::new(header[5])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
        Cell::new(header[6])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN)),
    ]));

    // Find timetabl table and add data to table_pretty
    if let Some(table) = html.select(&table_selector).next() {
        let mut is_current_day = false;
        let mut is_next_day = false;

        for row in table.select(&tr) {
            let row_data: Vec<_> = row
                .select(&td)
                .map(|cell| cell.text().collect::<String>().trim().to_string())
                .collect();

            let mut row_data_formatted: Vec<&str> = Vec::new();

            for (i, r) in row_data.iter().enumerate() {
                match i {
                    // The column zero contains the day like 2 for Monday, 3 for Thirday and so on...
                    0 => {
                        if let Some(&value) = map.get(r.as_str()) {
                            is_current_day = value == current_weekday;
                            is_next_day = value == next_day;
                        }
                        row_data_formatted.push(r);
                    }
                    // Some data after '\n' that we don't need to include
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
                continue;
            }

            // Highlight the row, that is the current day
            if is_current_day {
                is_next_day = true;
                let mut cells = Vec::new();
                for data in &row_data_formatted {
                    cells.push(
                        Cell::new(data)
                            .with_style(Attr::Bold)
                            .with_style(Attr::ForegroundColor(color::YELLOW)),
                    );
                }
                table_pretty.add_row(Row::new(cells));
                is_current_day = false;
            } else if is_next_day {
                let mut cells = Vec::new();
                for data in &row_data_formatted {
                    cells.push(
                        Cell::new(data)
                            .with_style(Attr::Bold)
                            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
                    );
                }
                table_pretty.add_row(Row::new(cells));
                is_next_day = false;
            } else {
                // The others days
                table_pretty.add_row(Row::from(row_data_formatted));
            }
        }
    }

    let utc_now = chrono::offset::Utc::now();
    println!("Hôm nay là ngày {}", utc_now.format("%d/%m/%Y "));

    return table_pretty;
}

fn annoucement_table(html: &Html, tr: &Selector, td: &Selector) -> Table {
    let announcement = Selector::parse("#MainContent_Gtb").unwrap();
    let mut announcement_table = Table::new();

    // Find announcement table and add data to announcement_table
    let header = ["Thời gian nghỉ", "Nội dung nghỉ"];

    announcement_table.add_row(Row::new(vec![
        Cell::new(header[0])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::RED)),
        Cell::new(header[1])
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::RED)),
    ]));

    if let Some(table) = html.select(&announcement).next() {
        for row in table.select(&tr) {
            let row_data: Vec<_> = row
                .select(&td)
                .map(|cell| cell.text().collect::<String>().trim().to_string())
                .collect();
            if row_data.len() > 1 {
                announcement_table.add_row(row_data.into());
            }
        }
    }

    return announcement_table;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args            = Cli::parse();

    // Store cookie so that we can navigate to another page that requires login in the
    // first place

    let total_start = Instant::now();

    let cookie_store = Arc::new(Jar::default());
    let client = Client::builder()
        .cookie_provider(cookie_store.clone())
        .build()?;

    let login_url = "https://my.uda.edu.vn/sv/svlogin";
    let timetable_url = "https://my.uda.edu.vn/sv/tkb";
    let exam_schedule_url = "https://my.uda.edu.vn/sv/lichthi";

    println!("Waiting for my.uda.edu.vn/sv/svlogin response...");
    let login_page = client.get(login_url).send().await?.text().await?;

    let mut form = HashMap::new();

    form.insert("User", args.username.as_str());
    form.insert("Password", args.password.as_str());
    form.insert("__EVENTTARGET", "Lnew1");
    form.insert("__EVENTARGUMENT", "");
    form.insert("__VIEWSTATEGENERATOR", "C9E6EC0D");

    // Minic the browser how it intends to send a request
    let document = Html::parse_document(&login_page);
    let viewstate_selector = Selector::parse(r#"input[name="__VIEWSTATE"]"#).unwrap();
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

    form.insert("__VIEWSTATE", viewstate);
    form.insert("__EVENTVALIDATION", event_validation);

    let resp = client
        .post(login_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Host", "my.uda.edu.vn")
        .header("Origin", "https://my.uda.edu.vn")
        .header("Referer", "https://my.uda.edu.vn/sv/svlogin")
        .form(&form)
        .send()
        .await?;

    // If login successfully
    if resp.status().is_success() {

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

        let html = Html::parse_document(&resp_timetable_text);
        let tr = Selector::parse("tr").unwrap();
        let td = Selector::parse("td").unwrap();
        let anoucement_table = annoucement_table(&html, &tr, &td);
        let mut timetable_table = timetable_table(html, tr, td);
        
        let mut timetable_content = Vec::new();
        let mut announcement_content = Vec::new();

        for i in 1..timetable_table.len() {
            if let Some(data) = timetable_table.get_row(i) {
                let row_content: Vec<String> = (0..data.len()).filter_map(|j| data.get_cell(j).map(|cell| cell.get_content().to_string())).collect();
                timetable_content.push(row_content);
            }
        }

        for i in 1..anoucement_table.len() {
            if let Some(data) = anoucement_table.get_row(i) {
                let row_content: Vec<String> = (0..data.len()).filter_map(|j| data.get_cell(j).map(|cell| cell.get_content().to_string())).collect();
                announcement_content.push(row_content);
            }
        }

        let mut matches_course_indices = Vec::new();
        for (i, timetable_row) in timetable_content.iter().enumerate() {
            for annoucement_row in announcement_content.iter() {
                if let Some(course) = timetable_row.get(4) {
                    let matches_course = annoucement_row.iter().any(|cell| {
                        cell.contains(course)
                    });
                    
                    if matches_course {
                        matches_course_indices.push(i+1);
                        break;
                    }
                }
            }
        }

        for index in matches_course_indices {
            if let Some(row) = timetable_table.get_mut_row(index) {
                for cell_index in 0..row.len() {
                    if let Some(cell) = row.get_mut_cell(cell_index) {
                        let content = cell.get_content().to_string();
                        *cell = Cell::new(&content)
                            .with_style(Attr::Bold)
                            .with_style(Attr::ForegroundColor(color::RED))
                            .with_style(Attr::Dim);
                    }
                }
            }
        }


        timetable_table.printstd();
        anoucement_table.printstd();

        let html = Html::parse_document(&resp_exam_text);
        let tr = Selector::parse("tr").unwrap();
        let td = Selector::parse("td").unwrap();

        let exam_schedule = exam_schedule(&html, &tr, &td);

        exam_schedule.printstd();

        println!("Total time taken: {:?}", total_start.elapsed());
        return Ok(());

    } else {
        println!("Failed: {}", resp.status());
    }

    Ok(())
}

