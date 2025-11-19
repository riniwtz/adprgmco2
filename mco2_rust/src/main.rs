use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use chrono::NaiveDate;
use csv::{StringRecord, WriterBuilder};
use serde::Serialize;

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

#[derive(Debug, Serialize)]
struct InfrastructureTrends {
    region: String,
    main_island: String,
    total_approved_budget: f64,
    median_savings: f64,
    avg_delay: f64,
    delayed_projects_pct: f64,
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
    yoy_change_savings: f64,
}

#[derive(Debug, Serialize)]
struct SummaryJson {
    total_projects_analyzed: usize,
    total_contractors: usize,
    total_provinces: usize,
    global_avg_delay: f64,
    total_savings: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut projects: Vec<Project> = Vec::new();
    let mut data_loaded = false;

    loop {
        println!("\n=== DPWH Flood Control Data Analysis Pipeline ===");
        println!("[1] Load Dataset");
        println!("[2] Generate Reports");
        println!("[3] Exit");
        print!("Enter choice: ");
        io::stdout().flush().unwrap_or_default();

        let mut choice_str = String::new();
        io::stdin().read_line(&mut choice_str)?;
        let choice: i32 = choice_str.trim().parse().unwrap_or(0);

        match choice {
            1 => {
                let file_path = "../dpwh_flood_control_projects.csv";
                match load_data(file_path) {
                    Ok((total_rows, data)) => {
                        projects = data;
                        data_loaded = true;
                        println!("SUCCESS: {} rows scanned, {} rows filtered (2021-2023).", total_rows, projects.len());
                    }
                    Err(e) => println!("Error loading data: {}", e),
                }
            }
            2 => {
                if !data_loaded {
                    println!("No data loaded.");
                } else {
                    if let Err(e) = generate_reports(&projects) {
                        println!("Error generating reports: {}", e);
                    }
                }
            }
            3 => break,
            _ => println!("Invalid choice."),
        }
    }
    Ok(())
}

fn load_data(file_path: &str) -> Result<(usize, Vec<Project>), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let mut projects = Vec::new();
    let mut total_records = 0;

    for result in rdr.records() {
        let record = result?;
        total_records += 1;
        if let Some(project) = parse_record(&record) {
            projects.push(project);
        }
    }
    Ok((total_records, projects))
}

fn parse_record(record: &StringRecord) -> Option<Project> {
    let funding_year: i32 = record.get(9)?.trim().parse().ok()?;
    if !(2021..=2023).contains(&funding_year) {
        return None;
    }

    let region = record.get(1)?.trim().to_string();
    let main_island = record.get(0)?.trim().to_string();
    let province = record.get(2)?.trim().to_string();
    let contractor = record.get(14)?.trim().to_string();
    let type_of_work = record.get(8)?.trim().to_string();

    if region.is_empty() || main_island.is_empty() || contractor.is_empty() {
        return None;
    }

    let approved_budget: f64 = record.get(11)?.replace(",", "").parse().unwrap_or(0.0);
    let contract_cost: f64 = record.get(12)?.replace(",", "").parse().unwrap_or(0.0);
    let cost_savings = approved_budget - contract_cost;

    let start_date_str = record.get(16)?.trim();
    let actual_date_str = record.get(13)?.trim();
    let completion_delay_days =
        match (NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d"),
               NaiveDate::parse_from_str(actual_date_str, "%Y-%m-%d")) {
            (Ok(start), Ok(actual)) => Some(actual.signed_duration_since(start).num_days()),
            _ => None,
        };

    Some(Project {
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
    })
}

fn generate_reports(projects: &[Project]) -> Result<(), Box<dyn Error>> {
    println!("Generating reports...");

    let report1 = generate_infrastructure_trends(projects);
    write_csv("report1_regional_summary.csv", &report1)?;
    display_report_1(&report1);

    let report2 = generate_financial_efficiencies(projects);
    write_csv("report2_contractor_ranking.csv", &report2)?;
    display_report_2(&report2);

    let report3 = generate_performance_metrics(projects);
    write_csv("report3_annual_trends.csv", &report3)?;
    display_report_3(&report3);

    generate_summary_json(projects)?;

    println!("Reports generated successfully.");
    Ok(())
}

fn generate_infrastructure_trends(projects: &[Project]) -> Vec<InfrastructureTrends> {
    let mut map: HashMap<(String, String), Vec<&Project>> = HashMap::new();
    for p in projects {
        map.entry((p.region.clone(), p.main_island.clone())).or_default().push(p);
    }

    struct RawData {
        region: String,
        main_island: String,
        total_budget: f64,
        median_savings: f64,
        avg_delay: f64,
        delayed_pct: f64,
        raw_score: f64,
    }

    let mut raw_list = Vec::new();

    for ((region, main_island), group) in map {
        let total_budget: f64 = group.iter().map(|p| p.approved_budget).sum();
        let mut savings: Vec<f64> = group.iter().map(|p| p.cost_savings).collect();
        savings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        let median_savings = if savings.is_empty() { 0.0 } else {
            let mid = savings.len() / 2;
            if savings.len() % 2 == 0 { (savings[mid - 1] + savings[mid]) / 2.0 } else { savings[mid] }
        };

        let delays: Vec<i64> = group.iter().filter_map(|p| p.completion_delay_days).collect();
        let avg_delay = if !delays.is_empty() {
            delays.iter().sum::<i64>() as f64 / delays.len() as f64
        } else { 0.0 };

        let delayed_count = delays.iter().filter(|&&d| d > 30).count();
        let delayed_pct = if !delays.is_empty() {
            (delayed_count as f64 / delays.len() as f64) * 100.0
        } else { 0.0 };

        let divisor = if avg_delay.abs() < 0.001 { 1.0 } else { avg_delay };
        let raw_score = (median_savings / divisor) * 100.0;

        raw_list.push(RawData {
            region, main_island, total_budget, median_savings, avg_delay, delayed_pct, raw_score
        });
    }

    let min_score = raw_list.iter().map(|r| r.raw_score).fold(f64::INFINITY, f64::min);
    let max_score = raw_list.iter().map(|r| r.raw_score).fold(f64::NEG_INFINITY, f64::max);
    let range = max_score - min_score;

    let mut report: Vec<InfrastructureTrends> = raw_list.into_iter().map(|item| {
        let efficiency_score = if range == 0.0 { 100.0 } else { ((item.raw_score - min_score) / range) * 100.0 };
        InfrastructureTrends {
            region: item.region,
            main_island: item.main_island,
            total_approved_budget: item.total_budget,
            median_savings: item.median_savings,
            avg_delay: item.avg_delay,
            delayed_projects_pct: item.delayed_pct,
            efficiency_score,
        }
    }).collect();

    report.sort_by(|a, b| b.efficiency_score.partial_cmp(&a.efficiency_score).unwrap_or(Ordering::Equal));
    report
}

fn generate_financial_efficiencies(projects: &[Project]) -> Vec<FinancialEfficiencies> {
    let mut map: HashMap<String, Vec<&Project>> = HashMap::new();
    for p in projects {
        map.entry(p.contractor.clone()).or_default().push(p);
    }

    let mut report = Vec::new();

    for (contractor, group) in map {
        let num_projects = group.len() as i32;
        if num_projects < 5 { continue; }

        let total_cost: f64 = group.iter().map(|p| p.contract_cost).sum();
        let total_savings: f64 = group.iter().map(|p| p.cost_savings).sum();
        
        let delays: Vec<i64> = group.iter().filter_map(|p| p.completion_delay_days).collect();
        let avg_delay = if !delays.is_empty() {
            delays.iter().sum::<i64>() as f64 / delays.len() as f64
        } else { 0.0 };

        let term1 = 1.0 - (avg_delay / 90.0);
        let term2 = if total_cost != 0.0 { total_savings / total_cost } else { 0.0 };
        let mut reliability_index = term1 * term2 * 100.0;
        if reliability_index > 100.0 { reliability_index = 100.0; }

        let risk_flag = if reliability_index < 50.0 { "High Risk".to_string() } else { "-".to_string() };

        report.push(FinancialEfficiencies {
            rank: 0, contractor, total_cost, num_projects, avg_delay, total_savings, reliability_index, risk_flag,
        });
    }

    report.sort_by(|a, b| b.total_cost.partial_cmp(&a.total_cost).unwrap_or(Ordering::Equal));
    if report.len() > 15 { report.truncate(15); }
    for (i, item) in report.iter_mut().enumerate() { item.rank = (i + 1) as i32; }
    report
}

fn generate_performance_metrics(projects: &[Project]) -> Vec<PerformanceMetrics> {
    let mut map: HashMap<(i32, String), Vec<&Project>> = HashMap::new();
    for p in projects {
        map.entry((p.funding_year, p.type_of_work.clone())).or_default().push(p);
    }

    let mut baseline_savings: HashMap<String, f64> = HashMap::new();
    for ((year, work_type), group) in &map {
        if *year == 2021 {
            let total: f64 = group.iter().map(|p| p.cost_savings).sum();
            baseline_savings.insert(work_type.clone(), total / group.len() as f64);
        }
    }

    let mut report = Vec::new();
    for ((year, work_type), group) in &map {
        let total_projects = group.len() as i32;
        let total_savings: f64 = group.iter().map(|p| p.cost_savings).sum();
        let avg_savings = total_savings / total_projects as f64;
        let overrun_count = group.iter().filter(|p| p.cost_savings < 0.0).count();
        let overrun_rate = (overrun_count as f64 / total_projects as f64) * 100.0;
        
        let yoy_change_savings = match baseline_savings.get(work_type) {
            Some(&base) if base != 0.0 => ((avg_savings - base) / base.abs()) * 100.0,
            _ => 0.0,
        };

        report.push(PerformanceMetrics {
            funding_year: *year, type_of_work: work_type.clone(), total_projects, avg_savings, overrun_rate, yoy_change_savings,
        });
    }

    report.sort_by(|a, b| a.funding_year.cmp(&b.funding_year)
        .then(b.avg_savings.partial_cmp(&a.avg_savings).unwrap_or(Ordering::Equal)));
    report
}

fn generate_summary_json(projects: &[Project]) -> Result<(), Box<dyn Error>> {
    let total_savings: f64 = projects.iter().map(|p| p.cost_savings).sum();
    let delays: Vec<i64> = projects.iter().filter_map(|p| p.completion_delay_days).collect();
    let global_avg_delay = if !delays.is_empty() {
        delays.iter().sum::<i64>() as f64 / delays.len() as f64
    } else { 0.0 };

    let total_contractors = projects.iter().map(|p| &p.contractor).collect::<HashSet<_>>().len();
    let total_provinces = projects.iter().map(|p| &p.province).collect::<HashSet<_>>().len();

    let summary = SummaryJson {
        total_projects_analyzed: projects.len(),
        total_contractors,
        total_provinces,
        global_avg_delay,
        total_savings,
    };

    let file = File::create("summary.json")?;
    serde_json::to_writer_pretty(file, &summary)?;
    println!("Exported: summary.json");
    Ok(())
}

fn write_csv<T: Serialize>(filename: &str, data: &[T]) -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new().has_headers(true).from_path(filename)?;
    for item in data { wtr.serialize(item)?; }
    wtr.flush()?;
    println!("Exported: {}", filename);
    Ok(())
}

fn display_report_1(report: &[InfrastructureTrends]) {
    println!("\n{:-<130}", "");
    println!("Report 1: Regional Flood Mitigation Efficiency Summary");
    println!("{:-<130}", "");
    println!("{:<25} | {:<15} | {:>18} | {:>18} | {:>10} | {:>10} | {:>10}",
             "Region", "Main Island", "Total Budget", "Median Savings", "Avg Delay", "Delay>30%", "Score");
    println!("{:-<130}", "");
    for row in report {
        println!("{:<25} | {:<15} | {:>18.2} | {:>18.2} | {:>10.2} | {:>9.2}% | {:>10.2}",
                 row.region, row.main_island, row.total_approved_budget, row.median_savings, row.avg_delay, row.delayed_projects_pct, row.efficiency_score);
    }
    println!("{:-<130}\n", "");
}

fn display_report_2(report: &[FinancialEfficiencies]) {
    println!("\n{:-<150}", "");
    println!("Report 2: Top Contractors Performance Ranking");
    println!("{:-<150}", "");
    println!("{:<5} | {:<40} | {:>18} | {:>10} | {:>10} | {:>18} | {:>10} | {:<10}",
             "Rank", "Contractor", "Total Cost", "Projs", "Avg Delay", "Total Savings", "Rel. Idx", "Risk");
    println!("{:-<150}", "");
    for row in report {
        let name = if row.contractor.len() > 37 { format!("{}...", &row.contractor[..37]) } else { row.contractor.clone() };
        println!("{:<5} | {:<40} | {:>18.2} | {:>10} | {:>10.2} | {:>18.2} | {:>10.2} | {:<10}",
                 row.rank, name, row.total_cost, row.num_projects, row.avg_delay, row.total_savings, row.reliability_index, row.risk_flag);
    }
    println!("{:-<150}\n", "");
}

fn display_report_3(report: &[PerformanceMetrics]) {
    println!("\n{:-<140}", "");
    println!("Report 3: Annual Project Type Cost Overrun Trends");
    println!("{:-<140}", "");
    println!("{:<6} | {:<50} | {:>10} | {:>18} | {:>12} | {:>12}",
             "Year", "Type of Work", "Projects", "Avg Savings", "Overrun %", "YoY (Svngs)");
    println!("{:-<140}", "");
    for row in report {
        let name = if row.type_of_work.len() > 47 { format!("{}...", &row.type_of_work[..47]) } else { row.type_of_work.clone() };
        println!("{:<6} | {:<50} | {:>10} | {:>18.2} | {:>11.2}% | {:>11.2}%",
                 row.funding_year, name, row.total_projects, row.avg_savings, row.overrun_rate, row.yoy_change_savings);
    }
    println!("{:-<140}\n", "");
}