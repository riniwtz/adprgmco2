/*********************
Last names: Crisologo, Cruz, Cunanan, Iwata
Language: Kotlin
Paradigm(s): Imperative, Functional, Object-Oriented
 *********************/

import java.time.LocalDate
import java.io.File
import java.time.format.DateTimeFormatter
import java.time.format.DateTimeParseException



 // --- helper for safe date parsing ---
 
 // safely converts a string to a localdate, returning null if parsing fails or the string is empty
 private fun String.toDateOrNull(): LocalDate? {
     if (this.isBlank()) return null
     return try {
         // assuming all date format is yyyy-mm-dd in the csv file (iso_local_date)
         LocalDate.parse(this, DateTimeFormatter.ISO_LOCAL_DATE)
     } catch (e: DateTimeParseException) {
         null // returns null on invalid date format
     }
 }
 
 // --- helper for statistical calculations ---
 
 // extension function to calculate the median of a list of doubles
 private fun List<Double>.median(): Double {
    if (isEmpty()) return 0.0
    val sorted = this.sorted()
    val size = sorted.size
    return if (size % 2 == 1) {
        // odd number of elements
        sorted[size / 2]
    } else {
        // even number of elements: average of the two middle elements
        (sorted[size / 2 - 1] + sorted[size / 2]) / 2.0
    }
}

// data class to represent a flood control project
 
 data class FloodProject(
    val projectId: String,
    //properties are guaranteed non-null by the  parsing logic
    val contractCost: Double,
    val approvedBudget: Double,
    val startDate: LocalDate?,
    val completionDate: LocalDate?,
    val fundingYear: Int?, 
    // fields required for grouping and reporting
    val region: String, 
    val mainIsland: String,
    val contractorName: String, 
    val typeOfWork: String 
) {
    // derived field: cost savings = approvedbudgetforcontract - contractcost. computed once, on first access.
    val costSavings: Double by lazy {
        approvedBudget - contractCost
    }

    // derived field: completion delay days. computed once, on first access.
    val completionDelayDays: Long? by lazy {
        if (startDate != null && completionDate != null) {
            // calculates the duration between start and completion (positive if completion > start)
            ChronoUnit.DAYS.between(startDate, completionDate)
        } else {
            // returns null if either date is missing
            null
        }
    }
}
 
 // --- file reading function ---
 
 // helper class to return both the list of projects and the error count
 data class IngestionResult(
    val projects: List<FloodProject>,
    val totalRowsRead: Int,
    val errorRowCount: Int
 )

 fun readProjectsFromFile(fileName: String): IngestionResult {
     val projects = mutableListOf<FloodProject>()
     var linesReadCount = 0
     var errorRowCount = 0
     
     // attempt to read all lines from the file
     val lines = try {
         File(fileName).readLines()
     } catch (e: Exception) {
         println("Error reading file: $fileName. Please ensure the file exists.")
         return IngestionResult(emptyList(), 0, 0)
     }
     
     if (lines.size <= 1) return IngestionResult(emptyList(), 0, 0)
     linesReadCount = lines.size - 1 // exclude header row

     // regex to split csv lines, correctly handling commas inside quoted fields
     val csvPattern = ",(?=(?:[^\"]*\"[^\"]*\")*[^\"]*$)".toRegex()
     
     // column indices from the csv header (verified)
     val mainIslandIdx = 0
     val regionIdx = 1
     val typeOfWorkIdx = 8
     val fundingYearIdx = 9 
     val projectIdIdx = 6
     val approvedBudgetIdx = 11 // approvedbudgetforcontract
     val contractCostIdx = 12 
     val completionDateIdx = 13 // actualcompletiondate
     val contractorIdx = 14 
     val startDateIdx = 16 
     
     val minColumns = 17 // minimum required columns for valid data
 
     for (line in lines.drop(1)) { // skip the header line
         if (line.isBlank()) continue
         
         // split the line and remove surrounding quotes from the fields
         val fields = csvPattern.split(line).map { it.trim().removeSurrounding("\"") }
         
         // basic validation: check if we have enough columns
         if (fields.size < minColumns) {
             errorRowCount++
             continue
         }
 
         try {
             val projectId = fields[projectIdIdx]
             
             // clean financial fields by removing commas
             val approvedBudgetStr = fields[approvedBudgetIdx].replace(",", "")
             val contractCostStr = fields[contractCostIdx].replace(",", "")
             
             // strict parsing for financial fields
             // if they are null (invalid format/non-numeric like "clustered with..."), treat the entire row as an error and skip it
             val approvedBudget = approvedBudgetStr.toDoubleOrNull()
             val contractCost = contractCostStr.toDoubleOrNull()
             
             // basic validation: check if required financial fields are valid/present
             if (approvedBudget == null || contractCost == null) {
                 errorRowCount++
                 continue // skip this row
             }
             
             // safely parse date and year
             val completionDate = fields[completionDateIdx].toDateOrNull()
             val startDate = fields[startDateIdx].toDateOrNull()
             
             // read funding year directly and safely
             val fundingYear = fields[fundingYearIdx].toIntOrNull()
             
             // extract required grouping fields
             val region = fields.getOrElse(regionIdx) { "N/A Region" } 
             val mainIsland = fields.getOrElse(mainIslandIdx) { "N/A Island" } 
             val contractorName = fields.getOrElse(contractorIdx) { "N/A Contractor" } 
             val typeOfWork = fields.getOrElse(typeOfWorkIdx) { "N/A Work Type" } 
 
             projects.add(
                 FloodProject(
                     projectId = projectId,
                     approvedBudget = approvedBudget, // guaranteed non-null
                     contractCost = contractCost,     // guaranteed non-null
                     startDate = startDate,
                     completionDate = completionDate,
                     fundingYear = fundingYear, // populate the explicit field
                     region = region,
                     mainIsland = mainIsland,
                     contractorName = contractorName,
                     typeOfWork = typeOfWork
                 )
             )
         } catch (e: Exception) {
             // catch any unexpected parsing errors (like index out of bounds on a valid row)
             errorRowCount++
         }
     }
     
     return IngestionResult(projects, linesReadCount, errorRowCount)
 }
 
 // --- filtering function ---
 
 /**
 * filters projects to include only those funded within the specified year range (2021-2023)
 */
 fun filterProjectsByYear(projects: List<FloodProject>, startYear: Int, endYear: Int): List<FloodProject> {
     return projects.filter { project ->
         val year = project.fundingYear
         // filter: year must not be null and year must be within the range [2021, 2023]
         year != null && year in startYear..endYear
     }
 }
 
 // --- more helpers but for outpyt---
 
 /**
 * writes data map to a standardized csv file and displays the first row in table format
 */
 fun writeToCsv(fileName: String, header: List<String>, data: List<List<Any>>) {
     val file = File(fileName)
     val headerString = header.joinToString(",")
     
     // write header
     file.writeText(headerString + "\n")
     
     if (data.isNotEmpty()) {
         // format rows and append to file
         data.forEach { row ->
             // format numbers to two decimals and ensure correct csv structure
             val rowString = row.joinToString(",") {
                 when (it) {
                     is Double -> "%.2f".format(Locale.US, it) // format doubles to 2 decimal places
                     is Float -> "%.2f".format(Locale.US, it)
                     is String -> "\"$it\"" // quote strings that might contain commas
                     else -> it.toString()
                 }
             }
             file.appendText(rowString + "\n")
         }
         
         // no longer print the first row , because it is done in the report function
         println("Success: Report saved to $fileName.")
         
     } else {
         println("Warning: Report $fileName generated but is empty.")
     }
 }
 
 /**
 * writes a summary map to a json file
 */
 fun writeToJson(fileName: String, data: Map<String, Any>) {
     val file = File(fileName)
     // simple manual json serialization for the summary
     val jsonContent = data.entries.joinToString(
         separator = ",\n", 
         prefix = "{\n", 
         postfix = "\n}"
     ) { (key, value) -> 
         val formattedValue = when (value) {
             is String -> "\"$value\""
             is Double -> "%.2f".format(Locale.US, value) // format doubles to 2 decimal places
             is Float -> "%.2f".format(Locale.US, value)
             else -> value.toString()
         }
         "  \"$key\": $formattedValue"
     }
     file.writeText(jsonContent)
     println("Success: Summary saved to $fileName.")
     
     // display the summary json output for verification
     println("--- Summary JSON Check for $fileName ---")
     println(jsonContent)
     println("-------------------------------------------------")
 }
 
 
 // ---------------------------------------------------------------------------------------------------
 // report generation funcs
 // ---------------------------------------------------------------------------------------------------
 
 /**
 * report 1: regional flood mitigation efficiency summary
 */
 fun generateRegionalEfficiencyReport(projects: List<FloodProject>) {
     
     val groupedData = projects
         .filter { it.completionDelayDays != null } // only consider projects with completion data
         .groupBy { Pair(it.region, it.mainIsland) }
         .mapValues { (_, regionalProjects) ->
             
             // calculate required metrics
             val totalBudget = regionalProjects.sumOf { it.approvedBudget } // aggregate total approvedbudgetforcontract
             val costSavingsList = regionalProjects.map { it.costSavings }
             val medianSavings = costSavingsList.median() // median costsavings
             
             val completedProjectsWithDelay = regionalProjects.filter { it.completionDelayDays != null }
             val averageDelay = completedProjectsWithDelay.map { it.completionDelayDays!!.toDouble() }.average() // average completiondelaydays
             
             val highDelayCount = completedProjectsWithDelay.count { it.completionDelayDays!! > 30 }
             val highDelayPct = if (completedProjectsWithDelay.isNotEmpty()) 
                                 (highDelayCount.toDouble() / completedProjectsWithDelay.size) * 100 // percentage of projects with delays >30 days
                                 else 0.0
             
             // calculate efficiency score = (median savings / average delay) * 100, normalized to 0-100
             val efficiencyScore = when {
                 // case 1: perfect/early completion (avgdelay <= 0)
                 averageDelay <= 0.0 -> {
                     // if median savings are positive, reward with max score (100.0); otherwise, 0.0
                     if (medianSavings > 0) 100.0 else 0.0
                 }
                 // case 2: normal calculation (avgdelay > 0)
                 else -> {
                     val rawScore = (medianSavings / averageDelay) * 100 
                     rawScore.coerceIn(0.0, 100.0) // normalize to 0-100
                 }
             }
             
             mapOf(
                 "totalbudget" to totalBudget,
                 "mediansavings" to medianSavings,
                 "avgdelay" to averageDelay,
                 "highdelaypct" to highDelayPct,
                 "efficiencyscore" to efficiencyScore
             )
         }
         
     // sort descending by efficiencyscore
     val sortedData = groupedData.entries
         .sortedByDescending { it.value["efficiencyscore"] as Double }
 
     val header = listOf("region", "mainisland", "totalbudget", "mediansavings", "avgdelay", "highdelaypct", "efficiencyscore")
     val data = sortedData.map { (key, metrics) -> // using destructuring for clean access
         listOf(
             key.first, 
             key.second, 
             metrics["totalbudget"] as Double, 
             metrics["mediansavings"] as Double, 
             metrics["avgdelay"] as Double, 
             metrics["highdelaypct"] as Double, 
             metrics["efficiencyscore"] as Double
         )
     }
 
     writeToCsv("report1_regional_summary.csv", header, data) // output as csv
     
     // --- Console Table Display (Full Report) ---
     val lineLength = 105
     println("\n${"-".repeat(lineLength)}")
     println("Report 1: Regional Flood Mitigation Efficiency Summary")
     println("(Filtered: 2021-2023 Projects)")
     println("${"-".repeat(lineLength)}")
 
     // Print Header
     println(String.format(
         "%-20s | %-15s | %18s | %18s | %12s | %12s | %12s",
         "Region", "Main Island", "Total Budget", "Median Savings", "Avg Delay", "High Delay %", "Efficiency Score"
     ))
     println("${"-".repeat(lineLength)}")
 
     // Print Data Rows (Limit to Top 15 for console display)
     for (row in data.take(15)) {
         val region = row[0] as String
         val mainIsland = row[1] as String
         // Abbreviation/truncation for fit (18 chars for region, 13 for island)
         val displayedRegion = if (region.length > 18) region.substring(0, 17) + ".." else region
         val displayedIsland = if (mainIsland.length > 13) mainIsland.substring(0, 12) + ".." else mainIsland
 
         println(String.format(Locale.US,
             "%-20s | %-15s | %18.2f | %18.2f | %12.1f | %12.2f%% | %12.2f",
             displayedRegion,
             displayedIsland,
             row[2] as Double, // totalbudget
             row[3] as Double, // mediansavings
             row[4] as Double, // avgdelay
             row[5] as Double, // highdelaypct
             row[6] as Double  // efficiencyscore
         ))
     }
     println("${"-".repeat(lineLength)}")
     println("Table exported to report1_regional_summary.csv")
 }
 
 /**
 * report 2: top contractors performance ranking
 */
 fun generateContractorRankingReport(projects: List<FloodProject>, topN: Int = 15, minProjects: Int = 5) {
 
     // only consider projects with completion data
     val completedProjects = projects.filter { it.completionDelayDays != null }
     
     val groupedData = completedProjects
         .groupBy { it.contractorName }
         .mapValues { (_, contractorProjects) ->
             
             val numProjects = contractorProjects.size
             val totalCost = contractorProjects.sumOf { it.contractCost }
             
             // filter contractors that meet the minimum project count requirement (min 5 projects)
             if (numProjects < minProjects) return@mapValues null 
             
             val totalSavings = contractorProjects.sumOf { it.costSavings }
             val averageDelay = contractorProjects.map { it.completionDelayDays!!.toDouble() }.average()
             
             // calculate reliability index
             val delayFactor = 1.0 - (averageDelay / 90.0).coerceIn(0.0, 1.0)
             val savingsFactor = if (totalCost > 0) totalSavings / totalCost else 0.0
             
             val rawIndex = delayFactor * savingsFactor * 100
             val reliabilityIndex = rawIndex.coerceIn(0.0, 100.0) // capped at 100
 
             val riskFlag = if (reliabilityIndex < 50.0) "High Risk" else "Low Risk" // Capitalization added here
             
             mapOf(
                 "totalcost" to totalCost,
                 "numprojects" to numProjects,
                 "avgdelay" to averageDelay,
                 "totalsavings" to totalSavings,
                 "reliabilityindex" to reliabilityIndex,
                 "riskflag" to riskFlag
             )
         }
         .filterValues { it != null } // remove contractors filtered by minprojects
         
     // rank top n by totalcost (descending)
     val topContractors = groupedData.entries
         .sortedByDescending { it.value!!["totalcost"] as Double }
         .take(topN)
 
     val header = listOf("rank", "contractor", "totalcost", "numprojects", "avgdelay", "totalsavings", "reliabilityindex", "riskflag")
     val data = topContractors.mapIndexed { index, entry ->
         val metrics = entry.value!!
         listOf(
             index + 1, // rank
             entry.key, // contractor name
             metrics["totalcost"] as Double, 
             metrics["numprojects"] as Int, 
             metrics["avgdelay"] as Double, 
             metrics["totalsavings"] as Double, 
             metrics["reliabilityindex"] as Double,
             metrics["riskflag"] as String
         )
     }
 
     writeToCsv("report2_contractor_ranking.csv", header, data) // output as csv
     
     // --- Console Table Display (Full Report) ---
     val lineLength = 117
     println("\n${"-".repeat(lineLength)}")
     println("Report 2: Top $topN Contractors Performance Ranking (Min $minProjects Projects)")
     println("(Filtered: 2021-2023 Projects)")
     println("${"-".repeat(lineLength)}")

     // Print Header
     println(String.format(
         "%-6s | %-30s | %18s | %8s | %10s | %18s | %10s | %10s",
         "Rank", "Contractor", "Total Cost", "Projects", "Avg Delay", "Total Savings", "Reliability", "Risk Flag"
     ))
     println("${"-".repeat(lineLength)}")

     // Print Data Rows
     for (row in data) {
         val contractorName = row[1] as String
         // Abbreviation/truncation for fit (30 chars for contractor name)
         val displayedContractor = if (contractorName.length > 30) contractorName.substring(0, 29) + ".." else contractorName

         println(String.format(Locale.US,
             "%-6d | %-30s | %18.2f | %8d | %10.1f | %18.2f | %10.2f | %-10s",
             row[0] as Int, // rank
             displayedContractor,
             row[2] as Double, // totalcost
             row[3] as Int, // numprojects
             row[4] as Double, // avgdelay
             row[5] as Double, // totalsavings
             row[6] as Double, // reliabilityindex
             row[7] as String // riskflag (left-aligned)
         ))
     }
     println("${"-".repeat(lineLength)}")
     println("Table exported to report2_contractor_ranking.csv")
 }
 
 /**
 * report 3: annual project type cost overrun trends
 */
 fun generateAnnualTrendsReport(projects: List<FloodProject>) {
     
     // group by fundingyear and typeofwork
     val groupedData = projects
         .filter { it.fundingYear != null }
         .groupBy { Pair(it.fundingYear!!, it.typeOfWork) }
         .mapValues { (_, workProjects) ->
             
             val totalProjects = workProjects.size
             val avgCostSavings = workProjects.map { it.costSavings }.average()
             
             val overrunCount = workProjects.count { it.costSavings < 0 } // overrun if costsavings < 0
             val overrunRate = (overrunCount.toDouble() / totalProjects) * 100
             
             mapOf(
                 "totalprojects" to totalProjects,
                 "avgsavings" to avgCostSavings,
                 "overrunrate" to overrunRate
             )
         }
     
     // calculate year-over-year (yoy) change (2021 baseline)
     val groupedByYear = groupedData.entries.groupBy { it.key.first } // re-group by year for yoy calc
     
     // get the average savings for each work type in 2021 to use as baseline
     val yearBaselines = groupedByYear[2021]?.associate { it.key.second to it.value["avgsavings"] as Double } ?: emptyMap()
 
     val finalMapWithYoY = groupedData.entries.associate { (key, metrics) ->
         val year = key.first
         val typeOfWork = key.second
         val currentAvg = metrics["avgsavings"] as Double
         var yoyChange = 0.0
 
         if (year > 2021) {
             val baselineAvg = yearBaselines[typeOfWork]
             // only calculate yoy if baseline exists and is not zero (to prevent division by zero)
             if (baselineAvg != null && baselineAvg != 0.0) {
                 // yoy change = ((current avg - 2021 avg) / |2021 avg|) * 100
                 yoyChange = ((currentAvg - baselineAvg) / abs(baselineAvg)) * 100
             } 
         } 
         
         // return a pair of (key, value) to build the map
         key to (metrics + ("yoychange" to yoyChange))
     }
     
     // sort by year (ascending) then by avgsavings (descending)
     val sortedData = finalMapWithYoY.entries 
         .sortedWith(compareBy<Map.Entry<Pair<Int, String>, Map<String, Any>>> { it.key.first } // sort by year
             .thenByDescending { it.value["avgsavings"] as Double } // then by avgsavings
         )
 
     val header = listOf("fundingyear", "typeofwork", "totalprojects", "avgsavings", "overrunrate", "yoychange")
     
     val data = sortedData.map { (key, value) -> 
         listOf(
             key.first,          // fundingyear
             key.second,         // typeofwork
             value["totalprojects"] as Int,
             value["avgsavings"] as Double,
             value["overrunrate"] as Double,
             value["yoychange"] as Double
         )
     }
 
     writeToCsv("report3_annual_trends.csv", header, data) // output as csv
     
     // --- Console Table Display (Full Report) ---
     val lineLength = 104
     println("\n${"-".repeat(lineLength)}")
     println("Report 3: Annual Project Type Cost Overrun Trends")
     println("(Filtered: 2021-2023 Projects)")
     println("${"-".repeat(lineLength)}")
 
     // Print Header
     println(String.format(
         "%-6s | %-35s | %10s | %18s | %15s | %15s",
         "Year", "Type of Work", "Projects", "Avg Savings (PHP)", "Overrun Rate %", "YoY Change %"
     ))
     println("${"-".repeat(lineLength)}")
 
     // Print Data Rows (Limit to Top 15 for console display)
     for (row in data.take(15)) {
         val typeOfWork = row[1] as String
         // Abbreviation/truncation for fit (35 chars for type of work)
         val displayedWorkType = if (typeOfWork.length > 35) typeOfWork.substring(0, 34) + ".." else typeOfWork
 
         println(String.format(Locale.US,
             "%-6d | %-35s | %10d | %18.2f | %15.2f | %15.2f",
             row[0] as Int, // fundingyear
             displayedWorkType,
             row[2] as Int, // totalprojects
             row[3] as Double, // avgsavings
             row[4] as Double, // overrunrate
             row[5] as Double  // yoychange
         ))
     }
     println("${"-".repeat(lineLength)}")
     println("Table exported to report3_annual_trends.csv")
 }
 
 /**
 * summary json generation
 */
 fun generateSummaryJson(allProjects: List<FloodProject>, filteredProjects: List<FloodProject>) {
     println("\n--- Generating Summary JSON ---")
     
     val completedProjects = filteredProjects.filter { it.completionDelayDays != null }
     val avgDelay = completedProjects.map { it.completionDelayDays!!.toDouble() }.average()
     val totalSavings = filteredProjects.sumOf { it.costSavings }
     val uniqueContractors = filteredProjects.map { it.contractorName }.distinct().size
     // using uniqueregions as a measure of geographic coverage
     val uniqueRegions = filteredProjects.map { it.region }.distinct().size
     
     val summaryData = mapOf(
         "total_projects_read" to allProjects.size,
         "total_projects_analyzed" to filteredProjects.size,
         "total_unique_contractors" to uniqueContractors,
         "total_unique_regions" to uniqueRegions, 
         "global_avg_delay" to avgDelay,
         "total_savings_analyzed" to totalSavings
     )
     
     writeToJson("summary.json", summaryData) // output as json
 }
 
 // --- main execution function with interactive menu ---
 
 fun main() {
     val fileName = "dpwh_flood_control_projects.csv" 
     val analysisStartYear = 2021
     val analysisEndYear = 2023
     
     var allProjects: List<FloodProject> = emptyList()
     var totalRowsRead = 0
     var errorRowCount = 0
     var filteredProjects: List<FloodProject> = emptyList()
     
     var isRunning = true
     
     println("--- Data Analysis Pipeline Start ---")
     
     while (isRunning) {
         // print status line
         println("\n-------------------------------------------------------")
         println("Project Status (Analysis Period: $analysisStartYear-$analysisEndYear)")
         println("Total Projects Read: $totalRowsRead | Successful Parses: ${allProjects.size} | Errors: $errorRowCount | Filtered (2021-2023): ${filteredProjects.size}")
         println("-------------------------------------------------------")

         // print menu
         println("Select Action:")
         println("[1] Load Data File (Ingestion)")
         println("[2] Analyze and Generate Reports")
         println("[3] Exit")
         println("-------------------------------------------------------")
 
         print("Enter choice: ")
         val input = readln().toIntOrNull()
 
         when (input) {
             1 -> {
                 // load the file
                 println("\nAttempting to read data from '$fileName'...")
                 val result = readProjectsFromFile(fileName)
                 allProjects = result.projects
                 totalRowsRead = result.totalRowsRead
                 errorRowCount = result.errorRowCount

                 println("--- Data Ingestion Complete ---")
                 // automatically filter after loading to update the status line
                 filteredProjects = filterProjectsByYear(allProjects, analysisStartYear, analysisEndYear)
             }
             2 -> {
                 // generate reports
                 if (allProjects.isEmpty()) {
                     println("Error: No data loaded. Please select [1] Load Data File first.")
                 } else {
                     // ensure filtering happens before reporting
                     filteredProjects = filterProjectsByYear(allProjects, analysisStartYear, analysisEndYear)
                     
                     println("\n--- Starting Report Generation ---")
                     
                     generateRegionalEfficiencyReport(filteredProjects)
                     generateContractorRankingReport(filteredProjects)
                     generateAnnualTrendsReport(filteredProjects)
                     generateSummaryJson(allProjects, filteredProjects)
                     
                     println("\n--- All Reports Generated and Saved ---")
                 }
             }
             3 -> {
                 // exit
                 isRunning = false
                 println("Exiting program. Goodbye.")
             }
             else -> {
                 println("Invalid input. Please enter 1, 2, or 3.")
             }
         }
     }
 }
