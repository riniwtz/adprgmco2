  # Clean and transform
  df <- df %>%
    mutate(
      StartDate = suppressWarnings(ymd(StartDate)),
      ActualCompletionDate = suppressWarnings(ymd(ActualCompletionDate)),
      
      ApprovedBudgetForContract = as.numeric(ApprovedBudgetForContract),
      ApprovedBudgetForContract = ifelse(is.na(ApprovedBudgetForContract), 0.0, ApprovedBudgetForContract),
      
      ContractCost = as.numeric(ContractCost),
      ContractCost = ifelse(is.na(ContractCost), 0.0, ContractCost),
      
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
    filter(!is.na(StartDate), !is.na(ActualCompletionDate))
  
  cat("Data cleaning completed. Rows remaining:", nrow(df), "\n")
  
  # assign 'flood_clean' globally
  assign("flood_clean", df, envir = .GlobalEnv)
  cat("'flood_clean' stored in global environment.\n")
}
