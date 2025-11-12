/*********************
Last names: Iwata, Cruz, Cunanan, Crisologo
Language: Rust
Paradigm(s): Imperative, Functional, Object-Oriented
*********************/

use std::error::Error;
use std::fs::{File};
use std::io::{self, Write};
use std::collections::HashMap;
use chrono::NaiveDate;
use csv::{StringRecord, WriterBuilder};
use serde::Serialize;
use serde_json;

// Structs
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

// Report 1: Infrastructure Trends
// for report 1
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

// Report 2: Financial Efficiencies
// for report 2, contractor stuff
#[derive(Debug, Serialize, Clone)]
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

// Report 3: Performance Metrics
// annd report 3
#[derive(Debug, Serialize)]
struct PerformanceMetrics {
    funding_year: i32,
    type_of_work: String,
    total_projects: i32,
    avg_savings: f64,
    overrun_rate: f64,
    yoy_change: f64,
}

// Summary JSON
#[derive(Debug, Serialize)]
struct SummaryJson {
    total_projects_analyzed: usize,
    total_budget_analyzed: f64,
    global_avg_delay: f64,
    total_contractors: usize,
    total_provinces: usize, // Added per REQ-0009
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut projects: Vec<Project> = Vec::new();
    let mut data_loaded = false;

    // the main menu loop
    loop {
        println!("\n=== DPWH Flood Control Data Analysis Pipeline ===");
        println!("[1] Load Dataset (Filter 2021-2023)");
        println!("[2] Generate Reports");
        println!("[3] Exit");
        print!("Enter choice: ");
        // flush to toilet
        io::stdout().flush()?;

        let mut choice_str = String::new();
        io::stdin().read_line(&mut choice_str)?;
        // turn their input into a number, or just 0 if they type garbage
        let choice: i32 = match choice_str.trim().parse() {
            Ok(num) => num,
            Err(_) => 0,
        };

        // handle choice
        match choice {
            1 => {
                println!("Processing dataset...");
                let file_path = "../dpwh_flood_control_projects.csv"; 
                match load_data(file_path) {
                    Ok((record_count, loaded_projects)) => {
                        projects = loaded_projects; 
                        data_loaded = true;
                        println!("SUCCESS: {} rows loaded, {} rows filtered for 2021-2023", record_count, projects.len());
                    }
                    Err(e) => println!("ERROR: Failed to load data: {}", e),
                }
            }
            2 => {
                if !data_loaded {
                    println!("WARNING: Please load the dataset first [Option 1].");
                    continue;
                }
                println!("Generating reports...");
                // this does all the heavy lifting
                let (report1, report2, report3) = generate_reports(&projects)?;

                // ok now just print everything out all nice

                // Display reports
                // Report 1 : Regional Flood Mitigation Efficiency Summary
                println!("\n{:-<130}", "");
                println!("Report 1: Regional Flood Mitigation Efficiency Summary");
                println!("(Filtered: 2021-2023 Projects)");
                println!("{:-<130}", "");
                println!(
                    "{:<20} | {:<15} | {:>18} | {:>18} | {:>12} | {:>12} | {:>12}",
                    "Region", "Main Island", "Total Budget", "Median Savings", "Avg Delay", "High Delay %", "Efficiency"
                );
                println!("{:-<130}", "");
                // loop thru report 1 data
                for r in &report1 {
                    // truncate strings so the table doesn't break
                    let region = if r.region.len() > 18 { format!("{}..", &r.region[..18]) } else { r.region.clone() };
                    let island = if r.main_island.len() > 13 { format!("{}..", &r.main_island[..13]) } else { r.main_island.clone() };
                    println!(
                        "{:<20} | {:<15} | {:>18.2} | {:>18.2} | {:>12.1} | {:>12.1}% | {:>12.1}",
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
                println!("Table also exported to report1_regional_summary.csv");


                // Report 2 : Top Contractors Performance Ranking
                println!("\n{:-<140}", "");
                println!("Report 2: Top Contractors Performance Ranking");
                println!("(Top 15 by Total Contract Cost, >=5 Projects)"); // Updated description
                println!("{:-<140}", "");
                println!(
                    "{:<5} | {:<40} | {:>18} | {:>10} | {:>12} | {:>18} | {:>12} | {:<10}",
                    "Rank", "Contractor", "Total Cost", "Projects", "Avg Delay", "Total Savings", "Reliability", "Risk Flag"
                );
                println!("{:-<140}", "");
                // loop thru report 2, but just the top 15
                for r in report2.iter().take(15) {
                    let contractor_name = if r.contractor.len() > 38 {
                        format!("{}..", &r.contractor[..38])
                    } else {
                        r.contractor.clone()
                    };
                    println!(
                        "{:<5} | {:<40} | {:>18.2} | {:>10} | {:>12.1} | {:>18.2} | {:>12.1} | {:<10}",
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
                println!("Table also exported to report2_contractor_ranking.csv (Top 15)");

                // Report 3 : Annual Project Type Cost Overrun Trends
                println!("\n{:-<120}", "");
                println!("Report 3: Annual Project Type Cost Overrun Trends");
                println!("(Grouped by FundingYear and TypeOfWork)");
                println!("{:-<120}", "");
                println!(
                    "{:<6} | {:<45} | {:>10} | {:>18} | {:>12} | {:>12}",
                    "Year", "Type of Work", "Projects", "Avg Savings", "Overrun %", "YoY Change %"
                );
                 println!("{:-<120}", "");
                // loop thru report 3
                for r in &report3 {
                    let type_of_work = if r.type_of_work.len() > 43 {
                        format!("{}..", &r.type_of_work[..43])
                    } else {
                        r.type_of_work.clone()
                    };
                    println!(
                        "{:<6} | {:<45} | {:>10} | {:>18.2} | {:>12.1}% | {:>12.1}%",
                        r.funding_year,
                        type_of_work,
                        r.total_projects,
                        r.avg_savings,
                        r.overrun_rate,
                        r.yoy_change
                    );
                }
                println!("{:-<120}", "");
                println!("Table also exported to report3_annual_trends.csv");

                println!("\nSUCCESS: Reports saved to CSV files and summary.json created.");
            }
            3 => {
                println!("Exiting application.");
                break; // bye
            }
            _ => println!("Invalid choice, please try again."),
        }
    }

    Ok(())
}

// function where all the row-level parsing and filtering happens
fn parse_data(record: &StringRecord) -> Result<Option<Project>, Box<dyn Error>> {
    let date_format = "%Y-%m-%d"; // this has to match the csv date style

    // REQ-0003: Filter for "Blank Values" first
    // just check if any field is empty after trimming. if so, chuck the whole row
    let has_blank = record.iter().any(|f| f.trim().is_empty());
    if has_blank {
        return Ok(None);
    }

    // Parse funding_year (col 9)
    // grab col 9, trim it, parse
    let funding_year: i32 = record.get(9)
        .ok_or("Missing funding_year at col 9")?
        .trim()
        .parse()?; // '?' returns Err if this fails

    // REQ-0003: Filter for 2021-2023
    // simple range check
    if funding_year < 2021 || funding_year > 2023 {
        return Ok(None); // filtered again
    }

    // Parse Financials (remove potential commas)
    // get col 11, trim, remove commas just in case, then parse
    let approved_budget: f64 = record.get(11)
        .ok_or("Missing approved_budget at col 11")?
        .trim()
        .replace(',', "")
        .parse()?;

    // same for col 12
    let contract_cost: f64 = record.get(12)
        .ok_or("Missing contract_cost at col 12")?
        .trim()
        .replace(',', "")
        .parse()?;

    // REQ-0004: Compute Derived Fields
    // easy math
    let cost_savings = approved_budget - contract_cost;

    // Dates
    let start_str = record.get(16).unwrap_or("").trim();
    let end_str = record.get(13).unwrap_or("").trim();
    
    // try to parse the dates. .ok() turns a Result into an Option, so it's None if parsing fails
    let start_date = NaiveDate::parse_from_str(start_str, date_format).ok();
    let end_date = NaiveDate::parse_from_str(end_str, date_format).ok();
        
    // only if we have BOTH dates can we calculate the delay
    let completion_delay_days = match (start_date, end_date) {
        (Some(s), Some(e)) => Some((e - s).num_days()), // chrono makes this easy
        _ => None, // otherwise, just None
    };
    
    // build the Project struct
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

    Ok(Some(project)) // wrap it in Ok(Some(...)) to signal success
}
    
// opens the file and loops thru records, calling parse_data on each one
fn load_data(file_path: &str) -> Result<(i32, Vec<Project>), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = csv::ReaderBuilder::new().has_headers(true).from_reader(file);
    let mut projects: Vec<Project> = Vec::new();
    let mut record_count = 0;
    let mut skipped_count = 0;

    // loop over each row in the csv
    for result in reader.records() {
        let record = result?; // this is one row
        record_count += 1;

        // call our parser function
        match parse_data(&record) {
            Ok(Some(project)) => {
                // add to vec
                projects.push(project);
            }
            Ok(None) => {
                // this means it was filtered (blank, wrong year, etc)
                skipped_count += 1;
                // println!("Skipping row #{} due to filtering...", record_count);
                continue;
            }
            Err(e) => {
                // this means parsing failed (like text in a number field)
                println!("Skipping row #{} due to parsing error: {}", record_count, e);
                skipped_count += 1;
                continue;
            }
        }
    }
    println!("Skipped {} rows due to row filter/incomplete/errors...", skipped_count);

    // Return the total count and the vector of valid projects
    Ok((record_count, projects))
}

// does all the grouping and math.
fn generate_reports(projects: &[Project]) -> Result<(Vec<InfrastructureTrends>, Vec<FinancialEfficiencies>, Vec<PerformanceMetrics>), Box<dyn Error>> {
    // Report 1: Infrastructure Trends
    // key is (region, island), value is a vec of all projects that match
    let mut region_map: HashMap<(String, String), Vec<&Project>> = HashMap::new();

    // Initialize group by region maps
    for p in projects { // p is a &Project here
        region_map.entry((p.region.clone(), p.main_island.clone())).or_default().push(p);
    }

    let mut report1 = Vec::new();
    // loop over the map. key is ((region, main_island)), group is the Vec<&Project>
    for ((region, main_island), group) in region_map {
        // total_budget
        // iter over the group, map to the budget, and sum it up
        let total_budget: f64 = group.iter().map(|p| p.approved_budget).sum();
        
        // median_savings
        // first collect all savings into a new vec
        let mut savings: Vec<f64> = group.iter().map(|p| p.cost_savings).collect();
        // sort it. need partial_cmp for floats, and unwrap_or to handle NaNs (just treat them as equal)
        savings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = savings.len() / 2;
        // classic median logic
        let median_savings = if savings.len() > 0 {
            if savings.len() % 2 == 0 { // even number
                (savings[mid - 1] + savings[mid]) / 2.0
            } else { // odd number
                savings[mid]
            }
        } else {
            0.0 // no data, median is 0
        };

        // avg_delay
        // unwraps the Some(days) and drops the nones
        let delays: Vec<i64> = group.iter().filter_map(|p| p.completion_delay_days).collect();
        let avg_delay = if !delays.is_empty() {
            delays.iter().sum::<i64>() as f64 / delays.len() as f64
        } else { 0.0 };

        // high_delay_pct
        // count how many delays are over 30 days
        let high_delay_count = delays.iter().filter(|&&d| d > 30).count();
        let high_delay_pct = if !delays.is_empty() {
             (high_delay_count as f64 / delays.len() as f64) * 100.0
        } else { 0.0 };

        // efficiency_score
        // Avoid division by zero for efficiency score
        let raw_score = if avg_delay.abs() > 0.001 { // check for not zero
            (median_savings / avg_delay) * 100.0 
        } else { 
            0.0 
        };
        // Normalize 0-100 per REQ-0006
        let efficiency_score = raw_score.max(0.0).min(100.0); // clamp it

        report1.push(InfrastructureTrends {
            region, main_island, total_budget, median_savings, avg_delay, high_delay_pct, efficiency_score
        });
    }

    // then sort by efficiency score in desc
    report1.sort_by(|a, b| {
        b.efficiency_score.partial_cmp(&a.efficiency_score).unwrap_or(std::cmp::Ordering::Equal)
    });

    // write report 1 to csv
    let mut wtr1 = WriterBuilder::new().from_path("report1_regional_summary.csv")?;
    for row in &report1 { wtr1.serialize(row)?; }
    wtr1.flush()?;

    // Report 2: Financial Efficiencies
    // same pattern, but group by contractor string
    let mut contractor_map: HashMap<String, Vec<&Project>> = HashMap::new();
    for p in projects {
        contractor_map.entry(p.contractor.clone()).or_default().push(p);
    }

    let mut report2 = Vec::new();
    for (contractor, group) in contractor_map {
        let num_projects = group.len() as i32;
        
        // Filter per REQ-0007. skip contractors with < 5 projects
        if num_projects < 5 {
            continue;
        }

        // same calcs as before, just for this contractor's group
        let total_cost: f64 = group.iter().map(|p| p.contract_cost).sum();
        let total_savings: f64 = group.iter().map(|p| p.cost_savings).sum();
        
        let delays: Vec<i64> = group.iter().filter_map(|p| p.completion_delay_days).collect();
        let avg_delay = if !delays.is_empty() {
            delays.iter().sum::<i64>() as f64 / delays.len() as f64
        } else { 0.0 };

        // Reliability Index per REQ-0007
        let total_cost_safe = if total_cost == 0.0 { 1.0 } else { total_cost }; // Avoid div by zero
        let delay_factor = 1.0 - (avg_delay / 90.0);
        let savings_factor = total_savings / total_cost_safe;
        let raw_index = delay_factor * savings_factor * 100.0;
        let reliability_index = raw_index.min(100.0); // Cap at 100

        // Risk Flag per REQ-0007
        let risk_flag = if reliability_index < 50.0 {
            "High Risk".to_string()
        } else {
            "Low Risk".to_string()
        };

        report2.push(FinancialEfficiencies {
            rank: 0, // To be assigned after sorting
            contractor, total_cost, num_projects, avg_delay, total_savings, reliability_index, risk_flag
        });
    }

    // Rank by total ContractCost (descending) per REQ-0007
    // sort by total cost
    report2.sort_by(|a, b| b.total_cost.partial_cmp(&a.total_cost).unwrap_or(std::cmp::Ordering::Equal));
    // now that it's sorted, loop again to assign the rank number
    for (i, row) in report2.iter_mut().enumerate() {
        row.rank = (i + 1) as i32;
    }

    // Write Report 2 (Top 15 per REQ-0007)
    let mut wtr2 = WriterBuilder::new().from_path("report2_contractor_ranking.csv")?;
    for row in report2.iter().take(15) { wtr2.serialize(row)?; } // .take(15) is all we need
    wtr2.flush()?;
    
    // Report 3: Performance Metrics
    // Group by (Year, Type)
    let mut year_type_map: HashMap<(i32, String), Vec<&Project>> = HashMap::new();
    for p in projects {
        year_type_map.entry((p.funding_year, p.type_of_work.clone())).or_default().push(p);
    }
    
    // Create a map of avg_savings by (year, work_type) for YoY calculation
    let mut savings_map: HashMap<(i32, String), f64> = HashMap::new();
    
    let mut report3 = Vec::new();
    // first pass: calculate all the basic stats
    for ((year, work_type), group) in &year_type_map {
        let total_projects = group.len() as i32;
        let avg_savings = group.iter().map(|p| p.cost_savings).sum::<f64>() / total_projects as f64;
        // how many projects had negative savings (overrun)
        let overrun_count = group.iter().filter(|p| p.contract_cost > p.approved_budget).count();
        let overrun_rate = (overrun_count as f64 / total_projects as f64) * 100.0;

        // store the avg_savings in our lookup map
        savings_map.insert((*year, work_type.clone()), avg_savings);

        report3.push(PerformanceMetrics {
             funding_year: *year, type_of_work: work_type.clone(), total_projects, avg_savings, overrun_rate, yoy_change: 0.0 // Placeholder
        });
    }

    // Calculate YoY change (REQ-0008)
    // second pass: now we can calculate the yoy change
    for row in report3.iter_mut() {
        if row.funding_year == 2021 {
            row.yoy_change = 0.0; // Baseline year, no change
        } else {
            // try to find last year's data for the same work type
            let prev_year_savings = savings_map.get(&(row.funding_year - 1, row.type_of_work.clone()));
            if let Some(prev_savings) = prev_year_savings {
                if *prev_savings != 0.0 {
                    // the actual yoy formula
                    row.yoy_change = ((row.avg_savings - prev_savings) / prev_savings.abs()) * 100.0;
                } else {
                    // if last year was 0, just show 100% or 0%
                    row.yoy_change = if row.avg_savings > 0.0 { 100.0 } else { 0.0 }; // Handle zero baseline
                }
            } else {
                row.yoy_change = 0.0; // No data for previous year
            }
        }
    }

    // Sort for better readability (per REQ-0008)
    // sort by year asc, then avg_savings desc
    report3.sort_by(|a, b| {
        a.funding_year.cmp(&b.funding_year)
            .then_with(|| { // .then_with is for when the cmp returns an Option
                b.avg_savings
                    .partial_cmp(&a.avg_savings)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    
    // Write Report 3
    let mut wtr3 = WriterBuilder::new().from_path("report3_annual_trends.csv")?;
    for row in &report3 { wtr3.serialize(row)?; }
    wtr3.flush()?;

    // Summary JSON
    // some global stats
    let delays: Vec<i64> = projects.iter().filter_map(|p| p.completion_delay_days).collect();
    let global_avg_delay = if !delays.is_empty() {
         delays.iter().sum::<i64>() as f64 / delays.len() as f64
    } else { 0.0 };

    // Get total unique provinces (using region as a proxy since province isn't in Project struct)
    // Using region as a proxy for 'provinces' as it's the closest available field
    // stick all regions in a hashset to get unique count
    let total_provinces = projects.iter().map(|p| &p.region).collect::<std::collections::HashSet<_>>().len();

    let summary = SummaryJson {
        total_projects_analyzed: projects.len(),
        total_budget_analyzed: projects.iter().map(|p| p.approved_budget).sum(),
        global_avg_delay,
        total_contractors: report2.len(), // total contractors who met the > 5 proj filter
        total_provinces, // Added per REQ-0009
    };

    let summary_file = File::create("summary.json")?;
    // REQUIRES: cargo add serde_json
    // make it pretty
    serde_json::to_writer_pretty(summary_file, &summary)?;

    // finally, return all the reports
    Ok((report1, report2, report3))
}