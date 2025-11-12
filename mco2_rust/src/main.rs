/********************
Last names: Iwata, Cunanan, Cruz, Ngandu
Language: Rust
Paradigm(s): Imperative, Functional, Object-Oriented
********************/

use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::collections::{HashMap, HashSet};
use chrono::NaiveDate;
use csv::{StringRecord, WriterBuilder};
use serde::Serialize;
use serde_json;
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize)]
struct Project {
    region: String,
    main_island: String,
    contractor: String,
    funding_year: i32,
    type_of_work: String,
    approved_budget: f64,
    contract_cost: f64,
    cost_savings: f64,
    completion_delay_days: Option<i64>,
}

#[derive(Debug, Serialize)]
struct InfrastructureTrends {
    region: String,
    main_island: String,
    total_budget: f64,
    median_savings: f64,
    avg_delay: f64,
    high_delay_pct: f64,
    efficiency_score: f64,
}

#[derive(Debug, Serialize)]
struct FinancialEfficiencies {
    rank: i32,
    contractor: String,
    total_cost: f64,
    num_projects: i32,
    avg_delay: f64,
    total_savings: f64,
    reliability_index: f64,
    risk_flag: String,
}

#[derive(Debug, Serialize)]
struct PerformanceMetrics {
    funding_year: i32,
    type_of_work: String,
    total_projects: i32,
    avg_savings: f64,
    overrun_rate: f64,
    yoy_change: f64,
}

#[derive(Debug, Serialize)]
struct SummaryJson {
    total_projects_analyzed: usize,
    total_budget_analyzed: f64,
    global_avg_delay: f64,
    total_contractors: usize,
    total_provinces: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut projects: Vec<Project> = Vec::new();
    let mut data_loaded = false;

    loop {
        print_menu();
        let choice = get_menu_choice()?;

        let keep_running = match choice {
            1 => handle_load_data(&mut projects, &mut data_loaded),
            2 => handle_generate_reports(&projects, data_loaded),
            3 => handle_exit(),
            _ => handle_invalid_choice(),
        };

        if !keep_running {
            break;
        }
    }
    Ok(())
}

fn print_menu() {
    println!("\n=== DPWH Flood Control Data Analysis Pipeline ===");
    println!("===               By Rintaro Iwata            ===\n");
    println!("[1] Load Dataset (Filter 2021-2023)");
    println!("[2] Generate Reports");
    println!("[3] Exit");
    print!("Enter choice: ");
    io::stdout().flush().unwrap_or_default();
}

fn get_menu_choice() -> Result<i32, Box<dyn Error>> {
    let mut choice_str = String::new();
    io::stdin().read_line(&mut choice_str)?;
    let choice: i32 = match choice_str.trim().parse() {
        Ok(num) => num,
        Err(_) => 0,
    };
    Ok(choice)
}

fn handle_load_data(projects: &mut Vec<Project>, data_loaded: &mut bool) -> bool {
    println!("Processing dataset...");
    let file_path = "../dpwh_flood_control_projects.csv";
    match load_data(file_path) {
        Ok((record_count, loaded_projects)) => {
            *projects = loaded_projects;
            *data_loaded = true;
            println!(
                "SUCCESS: {} rows loaded, {} rows filtered for 2021-2023",
                record_count,
                projects.len()
            );
        }
        Err(e) => println!("ERROR: Failed to load data: {}", e),
    }
    true
}

fn handle_generate_reports(projects: &[Project], data_loaded: bool) -> bool {
    if !data_loaded {
        println!("WARNING: Please load the dataset first [Option 1].");
        return true;
    }
    println!("Generating reports...");

    match generate_reports(projects) {
        Ok((report1, report2, report3)) => {
            display_report_1(&report1);
            display_report_2(&report2);
            display_report_3(&report3);

            println!("\nSUCCESS: Reports saved to CSV files and summary.json created.");
        }
        Err(e) => {
            println!("ERROR: Failed to generate reports: {}", e);
        }
    }
    true
}

fn handle_exit() -> bool {
    println!("Exiting application.");
    false
}

fn handle_invalid_choice() -> bool {
    println!("Invalid choice, please try again.");
    true
}

fn display_report_1(report: &[InfrastructureTrends]) {
    println!("\n{:-<130}", "");
    println!("Report 1: Regional Flood Mitigation Efficiency Summary");
    println!("(Filtered: 2021-2023 Projects)");
    println!("{:-<130}", "");
    println!(
        "{:<20} | {:<15} | {:>18} | {:>18} | {:>12} | {:>12} | {:>12}",
        "Region",
        "Main Island",
        "Total Budget",
        "Median Savings",
        "Avg Delay",
        "High Delay %",
        "Efficiency"
    );
    println!("{:-<130}", "");

    for r in report {
        let region = if r.region.len() > 18 {
            format!("{}..", &r.region[..18])
        } else {
            r.region.clone()
        };
        let island = if r.main_island.len() > 13 {
            format!("{}..", &r.main_island[..13])
        } else {
            r.main_island.clone()
        };

        println!(
            "{:<20} | {:<15} | {:>18.2} | {:>18.2} | {:>12.1} | {:>12.2}% | {:>12.2}",
            region,
            island,
            r.total_budget,
            r.median_savings,
            r.avg_delay,
            r.high_delay_pct,
            r.efficiency_score
        );
    }
    println!("{:-<130}", "");
    println!("Table exported to report1_regional_summary.csv");
}

fn display_report_2(report: &[FinancialEfficiencies]) {
    println!("\n{:-<140}", "");
    println!("Report 2: Top Contractors Performance Ranking");
    println!("(Top 15 by Total Contract Cost, >=5 Projects)");
    println!("{:-<140}", "");
    println!(
        "{:<5} | {:<40} | {:>18} | {:>10} | {:>12} | {:>18} | {:>12} | {:<10}",
        "Rank",
        "Contractor",
        "Total Cost",
        "Projects",
        "Avg Delay",
        "Total Savings",
        "Reliability",
        "Risk Flag"
    );
    println!("{:-<140}", "");
    
    // Only print the top 15, matching the CSV output
    for r in report.iter().take(15) {
        let contractor_name = if r.contractor.len() > 38 {
            format!("{}..", &r.contractor[..38])
        } else {
            r.contractor.clone()
        };

        println!(
            "{:<5} | {:<40} | {:>18.2} | {:>10} | {:>12.1} | {:>18.2} | {:>12.2} | {:<10}",
            r.rank,
            contractor_name,
            r.total_cost,
            r.num_projects,
            r.avg_delay,
            r.total_savings,
            r.reliability_index,
            r.risk_flag
        );
    }
    println!("{:-<140}", "");
    println!("Table exported to report2_contractor_ranking.csv");
}

fn display_report_3(report: &[PerformanceMetrics]) {
    println!("\n{:-<120}", "");
    println!("Report 3: Annual Project Type Cost Overrun Trends");
    println!("(Grouped by FundingYear and TypeOfWork)");
    println!("{:-<120}", "");
    println!(
        "{:<6} | {:<45} | {:>10} | {:>18} | {:>12} | {:>12}",
        "Year",
        "Type of Work",
        "Projects",
        "Avg Savings",
        "Overrun %",
        "YoY Change %"
    );
    println!("{:-<120}", "");
    
    for r in report {
        let type_of_work = if r.type_of_work.len() > 43 {
            format!("{}..", &r.type_of_work[..43])
        } else {
            r.type_of_work.clone()
        };

        println!(
            "{:<6} | {:<45} | {:>10} | {:>18.2} | {:>12.2}% | {:>12.2}%",
            r.funding_year,
            type_of_work,
            r.total_projects,
            r.avg_savings,
            r.overrun_rate,
            r.yoy_change
        );
    }
    println!("{:-<120}", "");
    println!("Table exported to report3_annual_trends.csv");
}

fn parse_data(record: &StringRecord) -> Result<Option<Project>, Box<dyn Error>> {
    let date_format = "%Y-%m-%d";

    // REQ-0003: Filter for "Blank Values"
    if record.iter().any(|f| f.trim().is_empty()) {
        return Ok(None); // Skip row if any field is blank
    }

    // Parse funding_year (col 9)
    let funding_year: i32 = record
        .get(9)
        .ok_or("Missing funding_year at col 9")?
        .trim()
        .parse()?;

    // REQ-0003: Filter for 2021-2023
    if !(2021..=2023).contains(&funding_year) {
        return Ok(None); // Skip row if not in year range
    }

    // Parse Financials (removing commas)
    let approved_budget: f64 = record
        .get(11)
        .ok_or("Missing approved_budget at col 11")?
        .trim()
        .replace(',', "")
        .parse()?;

    let contract_cost: f64 = record
        .get(12)
        .ok_or("Missing contract_cost at col 12")?
        .trim()
        .replace(',', "")
        .parse()?;

    // REQ-0004: Compute Derived Fields
    let cost_savings = approved_budget - contract_cost;

    // Dates
    let start_str = record.get(16).unwrap_or("").trim();
    let end_str = record.get(13).unwrap_or("").trim();
    let start_date = NaiveDate::parse_from_str(start_str, date_format).ok();
    let end_date = NaiveDate::parse_from_str(end_str, date_format).ok();
    let completion_delay_days = match (start_date, end_date) {
        (Some(s), Some(e)) => Some((e - s).num_days()),
        _ => None,
    };

    let project = Project {
        main_island: record.get(0).unwrap_or("").trim().to_string(),
        region: record.get(1).unwrap_or("").trim().to_string(),
        type_of_work: record.get(8).unwrap_or("").trim().to_string(),
        contractor: record.get(14).unwrap_or("").trim().to_string(),
        funding_year,
        approved_budget,
        contract_cost,
        cost_savings,
        completion_delay_days,
    };

    Ok(Some(project))
}

fn load_data(file_path: &str) -> Result<(i32, Vec<Project>), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);
    let mut projects: Vec<Project> = Vec::new();
    let mut record_count = 0;
    let mut skipped_count = 0;

    for result in reader.records() {
        let record = result?;
        record_count += 1;

        match parse_data(&record) {
            Ok(Some(project)) => {
                projects.push(project);
            }
            Ok(None) => {
                skipped_count += 1;
                println!("Skipping row #{} due to filtering...", record_count);
            }
            Err(e) => {
                println!(
                    "Skipping row #{} due to parsing error: {}",
                    record_count, e
                );
                skipped_count += 1;
            }
        }
    }
    println!("Skipped {} rows due to filtering or parsing errors...", skipped_count);

    Ok((record_count, projects))
}

/// Orchestrates the calculation of all reports, writes them to files, and returns the data.
fn generate_reports(projects: &[Project]) -> Result<(Vec<InfrastructureTrends>, Vec<FinancialEfficiencies>, Vec<PerformanceMetrics>), Box<dyn Error>> {
    let report1 = calculate_infrastructure_trends(projects);
    let report2 = calculate_financial_efficiencies(projects);
    let report3 = calculate_performance_metrics(projects);
    let summary = calculate_summary_json(projects, &report2);

    write_csv(&report1, "report1_regional_summary.csv")?;
    
    // Write only the Top 15 for report 2
    let report2_top15: Vec<_> = report2.iter().take(15).collect();
    write_csv(&report2_top15, "report2_contractor_ranking.csv")?;
    
    write_csv(&report3, "report3_annual_trends.csv")?;
    write_json(&summary, "summary.json")?;

    Ok((report1, report2, report3))
}

/// Report 1: Calculates Infrastructure Trends
fn calculate_infrastructure_trends(projects: &[Project]) -> Vec<InfrastructureTrends> {
    let mut region_map: HashMap<(String, String), Vec<&Project>> = HashMap::new();
    for p in projects {
        region_map
            .entry((p.region.clone(), p.main_island.clone()))
            .or_default()
            .push(p);
    }

    let mut report1 = Vec::new();
    for ((region, main_island), group) in region_map {
        let total_budget: f64 = group.iter().map(|p| p.approved_budget).sum();
        let median_savings = calculate_median_savings(&group);

        let delays: Vec<i64> = group.iter().filter_map(|p| p.completion_delay_days).collect();
        let (avg_delay, high_delay_pct) = if !delays.is_empty() {
            let avg = delays.iter().sum::<i64>() as f64 / delays.len() as f64;
            let high_count = delays.iter().filter(|&&d| d > 30).count();
            let pct = (high_count as f64 / delays.len() as f64) * 100.0;
            (avg, pct)
        } else {
            (0.0, 0.0)
        };
        
        let raw_score = if avg_delay.abs() > 0.001 {
            (median_savings / avg_delay) * 100.0
        } else {
            0.0
        };
        let efficiency_score = raw_score.max(0.0).min(100.0); // per REQ-0006

        report1.push(InfrastructureTrends {
            region,
            main_island,
            total_budget,
            median_savings,
            avg_delay,
            high_delay_pct,
            efficiency_score,
        });
    }

    report1.sort_by(|a, b| {
        b.efficiency_score
            .partial_cmp(&a.efficiency_score)
            .unwrap_or(Ordering::Equal)
    });

    report1
}

fn calculate_financial_efficiencies(projects: &[Project]) -> Vec<FinancialEfficiencies> {
    let mut contractor_map: HashMap<String, Vec<&Project>> = HashMap::new();
    for p in projects {
        contractor_map
            .entry(p.contractor.clone())
            .or_default()
            .push(p);
    }

    let mut report2 = Vec::new();
    for (contractor, group) in contractor_map {
        let num_projects = group.len() as i32;

        if num_projects < 5 { // per REQ-0007
            continue;
        }

        let total_cost: f64 = group.iter().map(|p| p.contract_cost).sum();
        let total_savings: f64 = group.iter().map(|p| p.cost_savings).sum();
        let avg_delay = calculate_avg_delay(&group);

        let total_cost_safe = if total_cost == 0.0 { 1.0 } else { total_cost };
        let delay_factor = 1.0 - (avg_delay / 90.0);
        let savings_factor = total_savings / total_cost_safe;
        let raw_index = delay_factor * savings_factor * 100.0;
        let reliability_index = raw_index.min(100.0); // per REQ-0007

        let risk_flag = if reliability_index < 50.0 {
            "High Risk".to_string()
        } else {
            "Low Risk".to_string()
        }; // per REQ-0007

        report2.push(FinancialEfficiencies {
            rank: 0,
            contractor,
            total_cost,
            num_projects,
            avg_delay,
            total_savings,
            reliability_index,
            risk_flag,
        });
    }

    // Rank by total ContractCost (descending) per REQ-0007
    report2.sort_by(|a, b| b.total_cost.partial_cmp(&a.total_cost).unwrap_or(Ordering::Equal));
    
    for (i, row) in report2.iter_mut().enumerate() {
        row.rank = (i + 1) as i32;
    }

    report2
}

fn calculate_performance_metrics(projects: &[Project]) -> Vec<PerformanceMetrics> {
    let mut year_type_map: HashMap<(i32, String), Vec<&Project>> = HashMap::new();
    for p in projects {
        year_type_map
            .entry((p.funding_year, p.type_of_work.clone()))
            .or_default()
            .push(p);
    }

    let mut savings_map: HashMap<(i32, String), f64> = HashMap::new();
    let mut report3 = Vec::new();

    for ((year, work_type), group) in &year_type_map {
        let total_projects = group.len() as i32;
        let avg_savings =
            group.iter().map(|p| p.cost_savings).sum::<f64>() / total_projects as f64;
        let overrun_count = group
            .iter()
            .filter(|p| p.contract_cost > p.approved_budget)
            .count();
        let overrun_rate = (overrun_count as f64 / total_projects as f64) * 100.0;

        savings_map.insert((*year, work_type.clone()), avg_savings);

        report3.push(PerformanceMetrics {
            funding_year: *year,
            type_of_work: work_type.clone(),
            total_projects,
            avg_savings,
            overrun_rate,
            yoy_change: 0.0,
        });
    }

    // Calculate YoY
    for row in report3.iter_mut() {
        if row.funding_year == 2021 {
            row.yoy_change = 0.0; // Baseline year
        } else {
            let prev_year_savings =
                savings_map.get(&(row.funding_year - 1, row.type_of_work.clone()));
            
            if let Some(prev_savings) = prev_year_savings {
                if *prev_savings != 0.0 {
                    row.yoy_change =
                        ((row.avg_savings - prev_savings) / prev_savings.abs()) * 100.0;
                } else {
                    row.yoy_change = if row.avg_savings > 0.0 { 100.0 } else { 0.0 };
                }
            } else {
                row.yoy_change = 0.0; // No data for previous year
            }
        }
    }

    // Sort per REQ-0008
    report3.sort_by(|a, b| {
        a.funding_year.cmp(&b.funding_year).then_with(|| {
            b.avg_savings
                .partial_cmp(&a.avg_savings)
                .unwrap_or(Ordering::Equal)
        })
    });

    report3
}

fn calculate_summary_json(projects: &[Project], report2: &[FinancialEfficiencies]) -> SummaryJson {
    let delays: Vec<i64> = projects.iter().filter_map(|p| p.completion_delay_days).collect();
    let global_avg_delay = if !delays.is_empty() {
        delays.iter().sum::<i64>() as f64 / delays.len() as f64
    } else {
        0.0
    };

    // provinces (per REQ-0009)
    let total_provinces = projects
        .iter()
        .map(|p| &p.region)
        .collect::<HashSet<_>>()
        .len();

    SummaryJson {
        total_projects_analyzed: projects.len(),
        total_budget_analyzed: projects.iter().map(|p| p.approved_budget).sum(),
        global_avg_delay,
        total_contractors: report2.len(),
        total_provinces,
    }
}

fn calculate_median_savings(group: &[&Project]) -> f64 {
    let mut savings: Vec<f64> = group.iter().map(|p| p.cost_savings).collect();
    if savings.is_empty() {
        return 0.0;
    }

    savings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let mid = savings.len() / 2;

    if savings.len() % 2 == 0 {
        (savings[mid - 1] + savings[mid]) / 2.0
    } else {
        savings[mid]
    }
}

fn calculate_avg_delay(group: &[&Project]) -> f64 {
    let delays: Vec<i64> = group.iter().filter_map(|p| p.completion_delay_days).collect();
    if delays.is_empty() {
        0.0
    } else {
        delays.iter().sum::<i64>() as f64 / delays.len() as f64
    }
}

// I/O Helpers
fn write_csv<T: Serialize>(data: &[T], filename: &str) -> Result<(), Box<dyn Error>> {
    let mut writer = WriterBuilder::new().from_path(filename)?;
    for row in data {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

fn write_json<T: Serialize>(data: &T, filename: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(filename)?;
    serde_json::to_writer_pretty(file, data)?;
    Ok(())
}