use chrono::{Datelike, Duration as ChronoDuration, FixedOffset, NaiveDate, Utc};
use colored::Colorize;
use prettytable::{
    color,
    format::{self, FormatBuilder},
    Attr, Cell, Row, Table,
};
use scraper::{Html, Selector};
mod config;
pub use config::UserConfig;


#[macro_export]
macro_rules! table_header {

    ( $headers:expr ) => {
        {
            use prettytable::color;

            let mut table = Table::new();
            let cells: Vec<Cell> = $headers.iter().map(|x| {
                Cell::new(x)
                    .with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::GREEN))
            }).collect();
            table.add_row(Row::new(cells));
            table
        }
    }
}

/// Parses an HTML document to extract the class cancellation notice table.
///
/// This function iterates through table rows (`<tr>`) matching the `tr` selector,
/// and for each row, it extracts the text from each table cell (`<td>`)
/// matching the `td` selector.
///
/// # Arguments
///
/// * `html` - A reference to a parsed HTML document from the `scraper` crate.
/// * `tr` - A `Selector` for the table rows (`<tr>`) that contain the cancellation data.
/// * `td` - A `Selector` for the table data cells (`<td>`) within each selected row.
///
/// # Returns
///
/// A `Table` (Vec<Vec<String>>) where each inner vector represents a row
/// and contains the text content of its cells.
pub fn cancellation_notice(html: &Html, tr: &Selector, td: &Selector) -> Table {
    let announcement = Selector::parse("#MainContent_Gtb").unwrap();
    let mut announcement_table = Table::new();

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
        for row in table.select(tr) {
            let row_data: Vec<_> = row
                .select(td)
                .map(|cell| cell.text().collect::<String>().trim().to_string())
                .collect();
            if row_data.len() > 1 {
                announcement_table.add_row(row_data.into());
            }
        }
    }

    announcement_table
}

pub fn find_matching_courses(
    timetable_content: &[Vec<String>],
    announcement_content: &[Vec<String>],
) -> Vec<usize> {
    timetable_content
        .iter()
        .enumerate()
        .filter_map(|(i, timetable_row)| {
            if let Some(course) = timetable_row.get(4) {
                let matches_course = announcement_content.iter().any(|announcement_row| {
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

fn is_class_today(current_weekday: &str, timetable_weekday_num: &str) -> bool {
    // Convert the weekday number to weekday string
    let timetable_weekday_str = match timetable_weekday_num {
        "2" => "Mon",
        "3" => "Tue",
        "4" => "Wed",
        "5" => "Thu",
        "6" => "Fri",
        "7" => "Sat",
        "8" => "Sun",
        _ => "Invalid day (bad day lol)",
    };

    if timetable_weekday_str == current_weekday {
        return true;
    };
    false
}

/// Parses an HTML document to extract timetable =>
///
/// # Arguments
///
/// * `html` - A reference to a parsed HTML document from the `scraper` crate.
/// * `tr` - A `Selector` for the table rows (`<tr>`) that contain timetable data.
/// * `td` - A `Selector` for the table data cells (`<td>`) within each selected row.
pub fn timetable_table(html: Html, tr: Selector, td: Selector) -> Table {
    let header = [
        "THỨ",
        "BUỔI",
        "TIẾT",
        "PHÒNG",
        "HỌC PHẦN",
        "GIẢNG VIÊN",
        "LỚP HỌC TẬP",
    ];
    let vn_offset = FixedOffset::east_opt(7 * 3600).unwrap();
    let now = Utc::now().with_timezone(&vn_offset);
    let current_weekday = now.weekday().to_string();

    let table_selector = Selector::parse("#MainContent_GV2").unwrap();
    let mut table_pretty = table_header![header];

    let mut classes: Vec<String> = Vec::new();
    let mut have_a_class_today = false;

    // Find timetable table and add data to table_pretty
    if let Some(table) = html.select(&table_selector).next() {
        for row in table.select(&tr) {
            let mut cells = Vec::new();
            for (i, cell) in row.select(&td).enumerate() {
                let cell_text = cell.text().collect::<String>().trim().to_string();

                if i == 3 && cell_text.to_lowercase().contains("online") {
                    let a_selector = Selector::parse("a").unwrap();
                    if let Some(link) = cell.select(&a_selector).next() {
                        cells.push(
                            Cell::new(&format!(
                                "link online ({})",
                                link.value().attr("href").unwrap_or("link unavailable")
                            ))
                            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
                        );
                    }
                }
                if i == 0 {
                    // day of the week column
                    if is_class_today(&current_weekday, &cell_text) {
                        have_a_class_today = true;
                    }
                }
                // if cell_text.to_lowercase().contains("online") { continue; }
                if i == 3 && !cell_text.to_lowercase().contains("online") {
                    cells.push(Cell::new(&cell_text.split('\n').next().unwrap().trim()));
                } else if !cell_text.to_lowercase().contains("online") {
                    cells.push(Cell::new(&cell_text.trim()));
                }
            }

            if cells.len() > 0 {
                if have_a_class_today {
                    let class = cells
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join(" ─ ");
                    classes.push(class);
                    have_a_class_today = false;
                }
                table_pretty.add_row(Row::new(cells));
            }
        }
    }

    if classes.len() > 0 {
        let longest_len = classes.iter().map(|x| x.len()).max().unwrap();
        let dashes = "─".repeat(longest_len / 2);
        println!("{} CLASSES FOR TODAY {}", dashes, dashes);
        for class in classes {
            println!("{}", class.bold());
        }
        println!("─────────────────────{}{}\n", dashes, dashes);
    } else {
        println!("NO CLASSES FOR TODAY\n");
    }
    // let utc_now = chrono::offset::Utc::now();
    // println!("Hôm nay là ngày {}", utc_now.format("%d/%m/%Y "));

    let custom_table_format = FormatBuilder::new()
        .column_separator('|')
        .borders('╿')
        .separators(
            &[
                format::LinePosition::Top,
                format::LinePosition::Intern,
                format::LinePosition::Bottom,
            ],
            // format::LineSeparator::new('─', '+', '+', '+'),
            format::LineSeparator::new('─', '╋', '╋', '╋'),
        )
        .padding(1, 1)
        .build();

    table_pretty.set_format(custom_table_format);

    table_pretty
}

/// The upcoming schedule looks like this:
///  ┌───────┬─────┬──────────────┬──────┬────────┬────────────────────────────────┬───────────────────────┬─────────────────────────┐
///  │ Buổi  │ Thứ │ Ngày bắt đầu │ Tiết │ Phòng  │ Học phần                       │ Giảng viên            │ Lớp học tập             │
///  ├───────┼─────┼──────────────┼──────┼────────┼────────────────────────────────┼───────────────────────┼─────────────────────────┤
///  │ Chiều │ 3   │ 19/08/2025   │ 1-3  │ 703    │ Đồ án công nghệ phần mềm (1tc) │ ThS. Nhiêu Lập Hòa    │                         │
///  │ Sáng  │ 7   │ 23/08/2025   │ 1-3  │ Online │ Đa văn hoá (1tc)               │ ThS. Lê Thị Hồng Thúy │ 7203(ST22A,ST22B,GD22A) │
///  │ Sáng  │ 7   │ 30/08/2025   │ 1-3  │ 906    │ Công nghệ IOT (3tc)            │ TS. Vương Công Đạt    │                         │
///  │ Sáng  │ 7   │ 23/08/2025   │ 4-6  │ 707    │ Lập trình Web 2 (3tc)          │ ĐH. Hồ Xuân Việt      │                         │
///  └───────┴─────┴──────────────┴──────┴────────┴────────────────────────────────┴───────────────────────┴─────────────────────────┘
///
pub fn extract_upcoming_schedule(html: &Html, tr: &Selector, td: &Selector) -> Table {
    let header = [
        "Buổi",
        "Thứ",
        "Ngày bắt đầu",
        "Tiết",
        "Phòng",
        "Học phần",
        "Giảng viên",
        "Lớp học tập",
    ];
    let mut table = table_header!(header);

    let table_selector_id = Selector::parse("#MainContent_GV1").unwrap();

    if let Some(r) = html.select(&table_selector_id).next() {
        for row in r.select(tr) {
            let row_data: Vec<String> = row
                .select(td)
                .map(|cell| cell.text().collect::<String>().trim().to_string())
                .collect();

            let mut cells = Vec::new();
            if row_data.len() < 1 {
                continue;
            }
            for data in &row_data {
                cells.push(Cell::new(data));
            }
            table.add_row(Row::new(cells));
        }
    }
    table
}

pub fn exam_schedule(html: &Html, tr: &Selector, td: &Selector) -> Table {
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

    let mut table = table_header!(header);

    let viet_nam_offset = FixedOffset::east_opt(7 * 3600).unwrap();
    let now = Utc::now().with_timezone(&viet_nam_offset);
    let sc = Selector::parse("#MainContent_GV2").unwrap();

    if let Some(r) = html.select(&sc).next() {
        // Collect all rows first, then process them sequentially
        // (parallel iteration on DOM elements has ownership issues)
        let mut valid_rows = Vec::new();

        for row in r.select(tr) {
            let row_data: Vec<_> = row
                .select(td)
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
    table
}
