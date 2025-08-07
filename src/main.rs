use indicatif::ProgressBar;
use prettytable::{color, Attr, Cell, Table};
use rayon::prelude::*;
use reqwest::{cookie::Jar, Client};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::Arc;
use request::{User, find_matching_courses_parallel, cancellation_notice, timetable_table, exam_schedule, extract_upcoming_schedule};
use std::time::{Duration, Instant};


// Optimized function to extract table content in parallel
fn extract_table_content_parallel(table: &Table) -> Vec<Vec<String>> {
    (1..table.len())
        .into_par_iter()
        .filter_map(|i| {
            table.get_row(i).map(|data| {
                (0..data.len())
                    .filter_map(|j| data.get_cell(j).map(|cell| cell.get_content().to_string()))
                    .collect()
            })
        })
        .collect()
}

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
    let user = User::new();
    let username = if let Some(username) = user.get_username() {
        username
    } else {
        panic!("Username not found");
    };

    let password = if let Some(password) = user.get_password() {
        password
    } else {
        panic!("Password not found");
    };

    let total_start = Instant::now();
    let bar = ProgressBar::new_spinner();

    bar.enable_steady_tick(Duration::from_millis(1));

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
    form.insert("__EVENTARGUMENT", "");
    form.insert("__VIEWSTATEGENERATOR", "C9E6EC0D");


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

        // Parse HTML documents and build tables sequentially
        // (HTML objects are not thread-safe due to internal Cell usage)
        let html_timetable = Html::parse_document(&resp_timetable_text);
        let html_exam = Html::parse_document(&resp_exam_text);

        // Create selectors once
        let tr = Selector::parse("tr").unwrap();
        let td = Selector::parse("td").unwrap();

        // Build tables sequentially but optimize with async where possible
        let announcement_table = cancellation_notice(&html_timetable, &tr, &td);
        let upcoming_schedule = extract_upcoming_schedule(&html_timetable, &tr, &td);
        let mut timetable_table = timetable_table(html_timetable, tr.clone(), td.clone());
        let exam_schedule_table = exam_schedule(&html_exam, &tr, &td);

        // Extract table content in parallel
        let (timetable_content, announcement_content) = rayon::join(
            || extract_table_content_parallel(&timetable_table),
            || extract_table_content_parallel(&announcement_table),
        );

        // Find matching courses in parallel
        let matches_course_indices = find_matching_courses_parallel(&timetable_content, &announcement_content);
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
        bar.finish_with_message("Done");
        // bar.finish_and_clear();
        // bar.finish();

        println!("
███████╗ ██████╗██╗  ██╗███████╗██████╗ ██╗   ██╗██╗     ███████╗
██╔════╝██╔════╝██║  ██║██╔════╝██╔══██╗██║   ██║██║     ██╔════╝
███████╗██║     ███████║█████╗  ██║  ██║██║   ██║██║     █████╗  
╚════██║██║     ██╔══██║██╔══╝  ██║  ██║██║   ██║██║     ██╔══╝  
███████║╚██████╗██║  ██║███████╗██████╔╝╚██████╔╝███████╗███████╗
╚══════╝ ╚═════╝╚═╝  ╚═╝╚══════╝╚═════╝  ╚═════╝ ╚══════╝╚══════╝
            ");
        timetable_table.printstd();
        println!("Upcoming schedule");
        upcoming_schedule.printstd();
        announcement_table.printstd();
        exam_schedule_table.printstd();

        let total_elapsed = total_start.elapsed();
        println!("\nTotal execution time: {total_elapsed:.2?}");
        // println!();
        return Ok(());
    } else {
        eprintln!("Failed: {}", resp.status());
    }

    bar.finish();

    Ok(())
}
