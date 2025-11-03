/*********************
Last names: Iwata, Cruz, Cunanan, Ngandu
Language: Rust
Paradigm(s): Imperative, Object-Oriented
*********************/
use std::fs::File;
use csv::Reader;
use std::io;

fn main() {
    // Escape backslashes or use a raw string literal
    let file_path = r"F:\Coding\adprgmco2\dpwh_flood_control_projects.csv";
    let file = File::open(file_path);

    let mut reader = Reader::from_reader(file);
    let mut total_rows = 0;

    for result in reader.records() {
        let record = result?;
        println!("{:?}", record);
        total_rows += 1;
    }


    // Generate three tabular reports and facilitate analysis of
    // 1. Infrastructure trends
    // 2. Financial efficiencies
    // 3. Performance metrics

    println!("Select Language Implementation");
    println!("[1] Load the file");
    println!("[2] Generate Reports");
    print!("Enter choice: ");

    let mut menu_choice = String::new();
    io::stdin().read_line(&mut menu_choice).expect("Failed to read choice");
    let choice = match menu_choice.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("❌ Invalid input. Please enter a number.");
            return;
        }
    };

    match choice {
        1 => {
            println!("Processing dataset...");
            // Load dataset
            // Count rows
            // Count filtered for 2021 - 2023

            
        }
        2 => {
            // Check first if .csv file is loaded before generating the reports
            // Generate reports if file is loaded

            println!("Generating reports...");
            println!("Outputs saved to individual files...");


            // Report 1: Regional Flood Mitigation Efficiency Summary.

            // Export report 1 to report1_regional_summary.csv


            // Report 2: Top Contractors Performance Ranking.

            // Export report 2 to report2_contractor_ranking.csv


            // Report 3: Annual Project Type Cost Overrun Trends

            // Export report 3 to report3_annual_trends.csv



            // After generating reports:
            /*
                Provision to produce a summary.json aggregating key stats across reports (e.g., total number of projects, total number of contractors,
                total provinces with projects, global average delay, total savings). 
            */
            // Show the summary stats from summary.json

            


        }
        _ => {

        }
    }

}
