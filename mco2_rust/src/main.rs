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

/*
dwph_flood_control_projects.csv columns
    0 - MainIsland: String
    1 - Region: String
    2 - Province: String
    3 - LegislativeDistrict: String
    4 - Municipality: String
    5 - DistrictEngineeringOffice: String
    6 - ProjectId: String
    7 - ProjectName: String
    8 - TypeOfWork: String
    9 - FundingYear: int
    10 - ContractId: String
    11 - ApprovedBudgetForContract: double (contains string)
    12 - ContractCost: double
    13 - ActualCompletionDate: Date (YYYY-mm-dd)
    14 - Contractor: String
    15 - ContractorCount: int
    16 - StartDate: Date (YYYY-mm-dd)
    17 - ProjectLatitude: double
    18 - ProjectLongitude: double
    19 - ProvincialCapital: String
    20 - ProvincialCapitalLatitude: double
    21 - ProvincialCapitalLongitude: double
*/

#[derive(Debug, Clone, Serialize)]
struct Project {
    region: String,
    main_island: String,
    province: String,
    contractor: String,
    funding_year: i32,
    type_of_work: String,
    approved_budget: f64,
    contract_cost: f64,
    cost_savings: f64,
    completion_delay_days: Option<i64>,
}

/// Regional Efficiency Report (Report 1)
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

/// Contractor Efficiency Report (Report 2)
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

/// Annual/Type Performance Report (Report 3)
#[derive(Debug, Serialize)]
struct PerformanceMetrics {
    funding_year: i32,
    type_of_work: String,
    total_projects: i32,
    avg_savings: f64,
    overrun_rate: f64,
    yoy_change: f64,
}

/// Summary JSON output
#[derive(Debug, Serialize)]
struct SummaryJson {
    total_projects_analyzed: usize,
    total_budget_analyzed: f64,
    global_avg_delay: f64,
    total_contractors: usize,
    total_provinces: usize,
}

fn print_menu() {
    println!("\n=== DPWH Flood Control Data Analysis Pipeline ===");
    println!("===               By Group Name               ===\n");
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

/// Main
fn main() -> Result<(), Box<dyn Error>> {
    let mut projects: Vec<Project> = Vec::new();
    let mut data_loaded = false;

    loop {
        print_menu();
        let choice = get_menu_choice()?;

        let keep_running = match choice {
            1 => handle_load_data(&mut projects, &mut data_loaded),
            2 => handle_generate_reports(&projects, data_loaded),
            3 => {
                println!("Exiting application. Goodbye!");
                false
            },
            _ => {
                println!("Invalid choice. Please try again.");
                true
            },
        };

        if !keep_running {
            break;
        }
    }
    Ok(())
}

fn handle_load_data(projects: &mut Vec<Project>, data_loaded: &mut bool) -> bool {
    let file_path = "../dpwh_flood_control_projects.csv"; 
    match load_data(file_path) {
        Ok((record_capacity_count, data)) => {
            *projects = data;
            *data_loaded = true;
            println!(
                "SUCCESS: {} rows loaded, {} rows filtered for 2021-2023",
                record_capacity_count,
                projects.len()
            );
        }
        Err(e) => println!("Error loading data: {}", e),
    }
    true
}

fn load_data(file_path: &str) -> Result<(i32, Vec<Project>), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let mut projects = Vec::new();
    let mut record_capacity_count = 0;
    let mut error_count = 0;

    for result in rdr.records() {
        let record = result?;
        record_capacity_count += 1;

        match parse_data(&record) {
            Ok(Some(project)) => projects.push(project),
            Ok(None) => {
                error_count += 1;
            }
            Err(_) => error_count += 1,
        }
    }

    println!("Skipped/Filtered {} rows.", error_count);
    Ok((record_capacity_count, projects))
}

fn parse_data(record: &StringRecord) -> Result<Option<Project>, Box<dyn Error>> {
    // Column Mapping:
    // 0: MainIsland, 1: Region, 2: Province, 8: TypeOfWork, 9: FundingYear
    // 11: ApprovedBudget, 12: ContractCost, 13: ActualCompletionDate, 14: Contractor, 16: StartDate

    // GLOBAL FILTER (REQ-0003)
    let funding_year_str = record.get(9).unwrap_or("").trim();
    if funding_year_str.is_empty() { return Ok(None); }
    
    let funding_year: i32 = match funding_year_str.parse() {
        Ok(y) => y,
        Err(_) => return Ok(None),
    };

    if !(2021..=2023).contains(&funding_year) {
        return Ok(None);
    }

    // REPORT 1 Metrics
    let region = record.get(1).unwrap_or("").trim().to_string();
    let main_island = record.get(0).unwrap_or("").trim().to_string();
    let province = record.get(2).unwrap_or("").trim().to_string();

    // Financials
    let budget_str = record.get(11).unwrap_or("0").replace(",", "");
    let approved_budget: f64 = budget_str.parse().unwrap_or(0.0);
    let cost_str = record.get(12).unwrap_or("0").replace(",", "");
    let contract_cost: f64 = cost_str.parse().unwrap_or(0.0);
    let cost_savings = approved_budget - contract_cost;

    // REPORT 2 Metrics
    let contractor = record.get(14).unwrap_or("").trim().to_string();

    // REPORT 3 Metrics
    let type_of_work = record.get(8).unwrap_or("").trim().to_string();

    // Critical Validations
    if region.is_empty() || main_island.is_empty() || contractor.is_empty() {
        return Ok(None); 
    }

    // Dates & Delay
    // Formula: Delay = Actual Completion - Start Date
    let date_format = "%Y-%m-%d"; 
    let start_date_str = record.get(16).unwrap_or("").trim();
    let actual_date_str = record.get(13).unwrap_or("").trim();

    let completion_delay_days = if let (Ok(start), Ok(actual)) = (
        NaiveDate::parse_from_str(start_date_str, date_format),
        NaiveDate::parse_from_str(actual_date_str, date_format),
    ) {
        let duration = actual.signed_duration_since(start).num_days();
        Some(duration)
    } else {
        None 
    };

    Ok(Some(Project {
        region,
        main_island,
        province,
        contractor,
        funding_year,
        type_of_work,
        approved_budget,
        contract_cost,
        cost_savings,
        completion_delay_days,
    }))
}

fn handle_generate_reports(projects: &[Project], data_loaded: bool) -> bool {
    if !data_loaded {
        println!("No data loaded. Please load dataset first.");
        return true;
    }

    println!("Generating reports...");

    // Report 1: Infrastructure Trends (Group by Region)
    let report1 = generate_infrastructure_trends(projects);
    if let Err(e) = write_csv("report1_regional_summary.csv", &report1) {
        println!("Error writing Report 1: {}", e);
    }
    display_report_1(&report1);

    // Report 2: Financial Efficiencies (Group by Contractor)
    let report2 = generate_financial_efficiencies(projects);
    if let Err(e) = write_csv("report2_contractor_ranking.csv", &report2) {
        println!("Error writing Report 2: {}", e);
    }
    display_report_2(&report2);

    // Report 3: Performance Metrics (Group by Year and Type)
    let report3 = generate_performance_metrics(projects);
    if let Err(e) = write_csv("report3_annual_trends.csv", &report3) {
        println!("Error writing Report 3: {}", e);
    }
    display_report_3(&report3);

    // Summary JSON
    if let Err(e) = generate_summary_json(projects, &report2) {
        println!("Error writing summary JSON: {}", e);
    }

    println!("Reports generated successfully.");
    true
}

/// Generates Report 1
fn generate_infrastructure_trends(projects: &[Project]) -> Vec<InfrastructureTrends> {
    let mut map: HashMap<(String, String), Vec<&Project>> = HashMap::new();
    for p in projects {
        map.entry((p.region.clone(), p.main_island.clone())).or_default().push(p);
    }

    let mut report = Vec::new();

    for ((region, main_island), group) in map {
        let total_budget: f64 = group.iter().map(|p| p.approved_budget).sum();
        let median_savings = calculate_median_savings(&group);
        
        let delays: Vec<i64> = group.iter().filter_map(|p| p.completion_delay_days).collect();
        let avg_delay = if !delays.is_empty() {
            delays.iter().sum::<i64>() as f64 / delays.len() as f64
        } else { 0.0 };

        let delayed_count = delays.iter().filter(|&&d| d > 7).count();
        let high_delay_pct = if !delays.is_empty() {
            (delayed_count as f64 / delays.len() as f64) * 100.0
        } else { 0.0 };

        let savings_ratio = if total_budget > 0.0 {
             group.iter().map(|p| p.cost_savings).sum::<f64>() / total_budget
        } else { 0.0 };
        let efficiency_score = (savings_ratio * 100.0) - (avg_delay * 0.1);

        report.push(InfrastructureTrends {
            region, main_island, total_budget, median_savings, avg_delay, high_delay_pct, efficiency_score,
        });
    }
    report.sort_by(|a, b| b.efficiency_score.partial_cmp(&a.efficiency_score).unwrap_or(Ordering::Equal));
    report
}

/// Generates Report 2
fn generate_financial_efficiencies(projects: &[Project]) -> Vec<FinancialEfficiencies> {
    let mut map: HashMap<String, Vec<&Project>> = HashMap::new();
    for p in projects {
        map.entry(p.contractor.clone()).or_default().push(p);
    }

    let mut report = Vec::new();

    for (contractor, group) in map {
        let total_cost: f64 = group.iter().map(|p| p.contract_cost).sum();
        let num_projects = group.len() as i32;
        let total_savings: f64 = group.iter().map(|p| p.cost_savings).sum();

        let delays: Vec<i64> = group.iter().filter_map(|p| p.completion_delay_days).collect();
        let avg_delay = if !delays.is_empty() {
            delays.iter().sum::<i64>() as f64 / delays.len() as f64
        } else { 0.0 };

        let overrun_count = group.iter().filter(|p| p.cost_savings < 0.0).count();
        let overrun_rate = (overrun_count as f64 / num_projects as f64) * 100.0;
        
        let reliability_index = 100.0 - (overrun_rate + (avg_delay / 365.0 * 100.0));
        let risk_flag = if reliability_index < 85.0 { "Critical".to_string() } else { "Stable".to_string() };

        report.push(FinancialEfficiencies {
            rank: 0, contractor, total_cost, num_projects, avg_delay, total_savings, reliability_index, risk_flag,
        });
    }

    report.sort_by(|a, b| b.reliability_index.partial_cmp(&a.reliability_index).unwrap_or(Ordering::Equal));
    for (i, item) in report.iter_mut().enumerate() { item.rank = (i + 1) as i32; }

    report
}

/// Generates Report 3
fn generate_performance_metrics(projects: &[Project]) -> Vec<PerformanceMetrics> {
    let mut map: HashMap<(i32, String), Vec<&Project>> = HashMap::new();
    for p in projects {
        map.entry((p.funding_year, p.type_of_work.clone())).or_default().push(p);
    }

    let mut count_lookup: HashMap<(i32, String), i32> = HashMap::new();
    for ((year, work_type), group) in &map {
        count_lookup.insert((*year, work_type.clone()), group.len() as i32);
    }

    let mut report = Vec::new();

    for ((year, work_type), group) in &map {
        let total_projects = group.len() as i32;
        let total_savings: f64 = group.iter().map(|p| p.cost_savings).sum();
        let avg_savings = total_savings / total_projects as f64;
        let overrun_count = group.iter().filter(|p| p.cost_savings < 0.0).count();
        let overrun_rate = (overrun_count as f64 / total_projects as f64) * 100.0;

        let prev_year = year - 1;
        let yoy_change = if let Some(&prev_count) = count_lookup.get(&(prev_year, work_type.clone())) {
            if prev_count > 0 {
                ((total_projects as f64 - prev_count as f64) / prev_count as f64) * 100.0
            } else { 0.0 }
        } else { 0.0 };

        report.push(PerformanceMetrics {
            funding_year: *year, type_of_work: work_type.clone(), total_projects, avg_savings, overrun_rate, yoy_change,
        });
    }
    
    report.sort_by(|a, b| {
        a.funding_year.cmp(&b.funding_year).then(a.type_of_work.cmp(&b.type_of_work))
    });
    report
}

/// Summary JSON
fn generate_summary_json(projects: &[Project], contractors_report: &[FinancialEfficiencies]) -> Result<(), Box<dyn Error>> {
    let total_projects_analyzed = projects.len();
    let total_budget_analyzed: f64 = projects.iter().map(|p| p.approved_budget).sum();
    
    let delays: Vec<i64> = projects.iter().filter_map(|p| p.completion_delay_days).collect();
    let global_avg_delay = if !delays.is_empty() {
        delays.iter().sum::<i64>() as f64 / delays.len() as f64
    } else { 0.0 };

    let total_contractors = contractors_report.len();
    let total_provinces = projects.iter().map(|p| &p.province).collect::<HashSet<_>>().len();

    let summary = SummaryJson {
        total_projects_analyzed, total_budget_analyzed, global_avg_delay, total_contractors, total_provinces,
    };

    let file = File::create("summary.json")?;
    serde_json::to_writer_pretty(file, &summary)?;
    Ok(())
}

/// Utility Functions
fn calculate_median_savings(group: &[&Project]) -> f64 {
    let mut savings: Vec<f64> = group.iter().map(|p| p.cost_savings).collect();
    if savings.is_empty() { return 0.0; }
    savings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let mid = savings.len() / 2;
    if savings.len() % 2 == 0 { (savings[mid - 1] + savings[mid]) / 2.0 } else { savings[mid] }
}

fn write_csv<T: Serialize>(filename: &str, data: &[T]) -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new().has_headers(true).from_path(filename)?;
    for item in data { wtr.serialize(item)?; }
    wtr.flush()?;
    println!("Exported: {}", filename);
    Ok(())
}

/// Displays
fn display_report_1(report: &[InfrastructureTrends]) {
    println!("\n{:-<130}", "");
    println!("Report 1: Regional Flood Mitigation Efficiency Summary");
    println!("(Filtered: 2021-2023 Projects)");
    println!("{:-<130}", "");
    println!(
        "{:<25} | {:<15} | {:>18} | {:>18} | {:>12} | {:>12} | {:>12}",
        "Region", "Main Island", "Total Budget", "Median Savings", "Avg Delay", "High Delay %", "Eff. Score"
    );
    println!("{:-<130}", "");

    for row in report {
        println!(
            "{:<25} | {:<15} | {:>18.2} | {:>18.2} | {:>12.2} | {:>12.2}% | {:>12.2}",
            row.region,
            row.main_island,
            row.total_budget,
            row.median_savings,
            row.avg_delay,
            row.high_delay_pct,
            row.efficiency_score
        );
    }
    println!("{:-<130}\n", "");
}

fn display_report_2(report: &[FinancialEfficiencies]) {
    println!("\n{:-<150}", "");
    println!("Report 2: Contractor Efficiency Ranking");
    println!("{:-<150}", "");
    println!(
        "{:<5} | {:<40} | {:>18} | {:>12} | {:>12} | {:>18} | {:>12} | {:<10}",
        "Rank", "Contractor", "Total Cost", "Projects", "Avg Delay", "Total Savings", "Rel. Index", "Risk"
    );
    println!("{:-<150}", "");

    for row in report.iter().take(20) {
        let contractor_display = if row.contractor.len() > 37 {
            format!("{}...", &row.contractor[..37])
        } else {
            row.contractor.clone()
        };

        println!(
            "{:<5} | {:<40} | {:>18.2} | {:>12} | {:>12.2} | {:>18.2} | {:>12.2} | {:<10}",
            row.rank,
            contractor_display,
            row.total_cost,
            row.num_projects,
            row.avg_delay,
            row.total_savings,
            row.reliability_index,
            row.risk_flag
        );
    }
    println!("{:-<150}\n", "");
}

fn display_report_3(report: &[PerformanceMetrics]) {
    println!("\n{:-<130}", "");
    println!("Report 3: Annual Performance Metrics");
    println!("{:-<130}", "");
    println!(
        "{:<6} | {:<50} | {:>10} | {:>18} | {:>12} | {:>12}",
        "Year", "Type of Work", "Projects", "Avg Savings", "Overrun %", "YoY Change"
    );
    println!("{:-<130}", "");

    for row in report {
        let type_display = if row.type_of_work.len() > 47 {
            format!("{}...", &row.type_of_work[..47])
        } else {
            row.type_of_work.clone()
        };

        println!(
            "{:<6} | {:<50} | {:>10} | {:>18.2} | {:>11.2}% | {:>11.2}%",
            row.funding_year,
            type_display,
            row.total_projects,
            row.avg_savings,
            row.overrun_rate,
            row.yoy_change
        );
    }
    println!("{:-<130}\n", "");
}