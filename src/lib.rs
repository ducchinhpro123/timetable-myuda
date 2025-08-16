use chrono::{Duration as ChronoDuration, FixedOffset, NaiveDate, Utc};
use prettytable::{color, Attr, Cell, Row, Table};
use scraper::{Html, Selector};

mod config;
pub use config::UserConfig;

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

/// Parses an HTML document to extract timetable.
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

    let table_selector = Selector::parse("#MainContent_GV2").unwrap();
    let mut table_pretty = Table::new();

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

    // Find timetable table and add data to table_pretty
    if let Some(table) = html.select(&table_selector).next() {
        for row in table.select(&tr) {
            let row_data: Vec<_> = row
                .select(&td)
                .map(|cell| cell.text().collect::<String>().trim().to_string())
                .collect();
            let mut cells = Vec::new();
            for (i, data) in row_data.iter().enumerate() {
                // Column number 3 contains additional information which is not helpful
                if i == 3 {
                    cells.push(Cell::new(data.split('\n').next().unwrap().trim()));
                } else {
                    cells.push(Cell::new(data));
                }
            }

            if cells.len() > 0 {
                table_pretty.add_row(Row::new(cells));
            }
        }
    }

    // let utc_now = chrono::offset::Utc::now();
    // println!("Hôm nay là ngày {}", utc_now.format("%d/%m/%Y "));

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
    let mut table = Table::new();
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
