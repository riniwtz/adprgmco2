# ********************
# Last names: Cruz
# Language: R
# Paradigm(s):
# ********************

# --- Libraries ---
library(readr)
library(dplyr)
library(stringr)
library(lubridate)
library(jsonlite)
library(scales)

# Global Variable 'flood_clean' used for report making
flood_clean <<- NULL  


# FUNCTION: Load Data and Clean 
load_and_clean_data <- function() {
  cat("Loading data from 'dpwh_flood_control_projects.csv'...\n")
  df <- read_csv("dpwh_flood_control_projects.csv", show_col_types = FALSE)
  cat("Data loaded successfully. Rows:", nrow(df), "\n")
  
  # Clean and transform
  df <- df %>%
    mutate(
      StartDate = suppressWarnings(ymd(StartDate)),
      ActualCompletionDate = suppressWarnings(ymd(ActualCompletionDate)),
      ApprovedBudgetForContract = as.numeric(ApprovedBudgetForContract),
      ContractCost = as.numeric(ContractCost),
      CostSavings = ApprovedBudgetForContract - ContractCost,
      CompletionDelayDays = as.numeric(ActualCompletionDate - StartDate)
    ) %>%
    filter(FundingYear >= 2021, FundingYear <= 2023) %>%
    group_by(Province) %>%
    mutate(
      ProjectLatitude = ifelse(is.na(ProjectLatitude), mean(ProjectLatitude, na.rm = TRUE), ProjectLatitude),
      ProjectLongitude = ifelse(is.na(ProjectLongitude), mean(ProjectLongitude, na.rm = TRUE), ProjectLongitude)
    ) %>%
    ungroup() %>%
    filter(!is.na(StartDate), !is.na(ActualCompletionDate),
           !is.na(ApprovedBudgetForContract), !is.na(ContractCost))
  
  cat("Data cleaning completed. Rows remaining:", nrow(df), "\n")
  
  # assign 'flood_control' globally
  assign("flood_clean", df, envir = .GlobalEnv)
  cat("'flood_clean' stored in global environment.\n")
}


# FUNCTION: Generate the 3 Reports 
generate_reports <- function() {
  if (is.null(flood_clean)) {
    cat("No dataset loaded. Please load data first.\n")
    return()
  }
  
  cat("Generating analytical reports...\n")
  
  # Report 1: Regional Efficiency
  report1 <- flood_clean %>%
    group_by(Region, MainIsland) %>%
    summarise(
      TotalBudget = sum(ApprovedBudgetForContract, na.rm = TRUE),
      MedianSavings = median(CostSavings, na.rm = TRUE),
      AvgDelay = mean(CompletionDelayDays, na.rm = TRUE),
      PctDelayedOver30 = mean(CompletionDelayDays > 30, na.rm = TRUE) * 100,
      .groups = "drop"
    ) %>%
    mutate(
      RawEfficiency = (MedianSavings / ifelse(AvgDelay == 0, 1, AvgDelay)) * 100
    )
  
  # normalize across the *entire dataset*, not per row
  report1 <- report1 %>%
    mutate(
      EfficiencyScore = scales::rescale(RawEfficiency, to = c(0, 100))
    ) %>%
    arrange(desc(EfficiencyScore)) %>%
    select(-RawEfficiency)
  
  write_csv(report1, "report1_efficiency.csv")
  cat("Report 1 fixed and generated: report1_efficiency.csv\n")
  
  # Report 2: Contractor Ranking
  report2 <- flood_clean %>%
    group_by(Contractor) %>%
    summarise(
      ProjectCount = n(),
      TotalContractCost = sum(ContractCost, na.rm = TRUE),
      AvgDelay = mean(CompletionDelayDays, na.rm = TRUE),
      TotalSavings = sum(CostSavings, na.rm = TRUE)
    ) %>%
    filter(ProjectCount >= 5) %>%
    mutate(
      ReliabilityIndex = pmin(100, (1 - (AvgDelay / 90)) * (TotalSavings / TotalContractCost) * 100),
      RiskFlag = ifelse(ReliabilityIndex < 50, "High Risk", "OK")
    ) %>%
    arrange(desc(TotalContractCost)) %>%
    slice_head(n = 15)
  
  write_csv(report2, "report2_contractors.csv")
  cat("Report 2: report2_contractors.csv\n")
  
  # Report 3: Annual Overrun Trends
  report3_base <- flood_clean %>%
    group_by(FundingYear, TypeOfWork) %>%
    summarise(
      TotalProjects = n(),
      AvgSavings = mean(CostSavings, na.rm = TRUE),
      OverrunRate = mean(CostSavings < 0, na.rm = TRUE) * 100
    ) %>%
    arrange(FundingYear, desc(AvgSavings))
  
  baseline <- report3_base %>%
    filter(FundingYear == 2021) %>%
    select(TypeOfWork, BaselineSavings = AvgSavings)
  
  report3 <- report3_base %>%
    left_join(baseline, by = "TypeOfWork") %>%
    mutate(YoYChangePct = ((AvgSavings - BaselineSavings) / abs(BaselineSavings)) * 100)
  
  write_csv(report3, "report3_trends.csv")
  cat("Report 3: report3_trends.csv\n")
  
  # Summary JSON
  summary_stats <- list(
    total_projects = nrow(flood_clean),
    total_contractors = n_distinct(flood_clean$Contractor),
    total_provinces = n_distinct(flood_clean$Province),
    global_avg_delay = mean(flood_clean$CompletionDelayDays, na.rm = TRUE),
    total_savings = sum(flood_clean$CostSavings, na.rm = TRUE)
  )
  
  write_json(summary_stats, "summary.json", pretty = TRUE)
  cat("Summary JSON generated: summary.json\n")
  cat("All reports successfully generated!\n")
  
}



border <- function() {
  cat("--------------------------------------------\n")
}

# FUNCTION: Display Main Menu 
MainMenu <- function() {
  repeat {
    border()
    cat(" DPWH FLOOD CONTROL ANALYTICS MAIN MENU\n")
    border()
    cat("[1] Load File\n")
    cat("[2] Generate Reports\n")
    cat("[0] Exit\n")
    border()
    
    choice <- readline(prompt = "Enter choice: ")
    
    if (choice == "1") {
      load_and_clean_data()
    } else if (choice == "2") {
      generate_reports()
    } else if (choice == "0") {
      cat("Exiting program.\n")
      break
    } else {
      cat("Invalid choice. Try again.\n")
    }
  }
}

# Shortcut to activate the program again (after 1st run compile)
MainMenu()
