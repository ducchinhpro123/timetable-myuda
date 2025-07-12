use indicatif::ProgressBar;
use chrono::NaiveDate;
use chrono::{Datelike, Duration as ChronoDuration, FixedOffset, Utc};
use prettytable::{color, Attr};
use prettytable::{Cell, Row, Table};
use reqwest::{cookie::Jar, Client};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::Arc;
use rayon::prelude::*;

use std::time::Duration;
// use clap::Parser;
use chrono;

use std::time::Instant;

// #[derive(Parser)]
// struct Cli {
//     username: String,
//     password: String
// }

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
        // Collect all rows first, then process them sequentially
        // (parallel iteration on DOM elements has ownership issues)
        let mut valid_rows = Vec::new();
        
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
                    if (date_time + ChronoDuration::days(1)) >= now {
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
                        valid_rows.push(styled_row);
                    }
                }
            }
        }
        
        // Add all valid rows to the table
        for row in valid_rows {
            table.add_row(row);
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

// Optimized function to extract table content in parallel
fn extract_table_content_parallel(table: &Table) -> Vec<Vec<String>> {
    (1..table.len())
        .into_par_iter()
        .filter_map(|i| {
            table.get_row(i).map(|data| {
                (0..data.len())
                    .filter_map(|j| {
                        data.get_cell(j)
                            .map(|cell| cell.get_content().to_string())
                    })
                    .collect()
            })
        })
        .collect()
}

// Optimized function to find matching courses in parallel
fn find_matching_courses_parallel(
    timetable_content: &[Vec<String>],
    announcement_content: &[Vec<String>],
) -> Vec<usize> {
    timetable_content
        .par_iter()
        .enumerate()
        .filter_map(|(i, timetable_row)| {
            if let Some(course) = timetable_row.get(4) {
                let matches_course = announcement_content
                    .par_iter()
                    .any(|announcement_row| {
                        announcement_row.iter().any(|cell| cell.contains(course))
                    });
                
                if matches_course {
                    Some(i + 1)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
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
    // let args            = Cli::parse();

    // Store cookie so that we can navigate to another page that requires login in the
    // first place

    let total_start = Instant::now();
    let bar = ProgressBar::new_spinner();

    bar.enable_steady_tick(Duration::from_millis(100));

    let cookie_store = Arc::new(Jar::default());
    let client = Client::builder()
        .cookie_provider(cookie_store.clone())
        .build()?;

    let login_url = "https://my.uda.edu.vn/sv/svlogin";
    let timetable_url = "https://my.uda.edu.vn/sv/tkb";
    let exam_schedule_url = "https://my.uda.edu.vn/sv/lichthi";

    bar.set_message("Waiting for my.uda.edu.vn/sv/svlogin response...");
    let login_page = client.get(login_url).send().await?.text().await?;

    let mut form = HashMap::new();

    form.insert("User", "99562");
    form.insert("Password", "NoiComDien@123");
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

    bar.set_message("Login...");
    let resp = client
        .post(login_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Host", "my.uda.edu.vn")
        .header("Origin", "https://my.uda.edu.vn")
        .header("Referer", "https://my.uda.edu.vn/sv/svlogin")
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
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

        bar.set_message("Parsing HTML documents and building tables...");
        
        // Parse HTML documents and build tables sequentially 
        // (HTML objects are not thread-safe due to internal Cell usage)
        let html_timetable = Html::parse_document(&resp_timetable_text);
        let html_exam = Html::parse_document(&resp_exam_text);
        
        // Create selectors once
        let tr = Selector::parse("tr").unwrap();
        let td = Selector::parse("td").unwrap();
        
        // Build tables sequentially but optimize with async where possible
        let announcement_table = annoucement_table(&html_timetable, &tr, &td);
        let mut timetable_table = timetable_table(html_timetable, tr.clone(), td.clone());
        let exam_schedule_table = exam_schedule(&html_exam, &tr, &td);
        
        bar.set_message("Processing table data in parallel...");
        
        // Extract table content in parallel
        let (timetable_content, announcement_content) = rayon::join(
            || extract_table_content_parallel(&timetable_table),
            || extract_table_content_parallel(&announcement_table),
        );

        // Find matching courses in parallel
        let matches_course_indices = find_matching_courses_parallel(&timetable_content, &announcement_content);

        bar.set_message("Styling matched courses...");
        
        // Apply styling to matched courses
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

        bar.set_message("Displaying results...");

        timetable_table.printstd();
        announcement_table.printstd();
        exam_schedule_table.printstd();
        
        let total_elapsed = total_start.elapsed();
        println!("\n🚀 Total execution time: {:.2?}", total_elapsed);
        // println!();
        bar.finish_with_message("Done");
        return Ok(());

    } else {
        eprintln!("Failed: {}", resp.status());
    }

    bar.finish();

    Ok(())
}

