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

// Safely converts a String to a LocalDate, returning null if parsing fails or the string is empty.
private fun String.toDateOrNull(): LocalDate? {
    if (this.isBlank()) return null
    return try {
        // Assuming all date format is YYYY-MM-DD in the CSV file
        LocalDate.parse(this, DateTimeFormatter.ISO_LOCAL_DATE)
    } catch (e: DateTimeParseException) {
        null
    }
}

// data class to represent a Flood Control Project to hold strcutrued data like from the CSV

data class FloodProject(
    val projectId: String,
    val contractCost: Double,
    val approvedBudget: Double,
    val startDate: LocalDate?,
    val completionDate: LocalDate?
) {
    val costSavings: Double
        get() = approvedBudget - contractCost
}

// --- File Reading Function ---

fun readProjectsFromFile(fileName: String): List<FloodProject> {
    val projects = mutableListOf<FloodProject>()
    
    // Attempt to read all lines from the file
    val lines = try {
        File(fileName).readLines()
    } catch (e: Exception) {
        println("Error reading file: $fileName. Please ensure the file exists.")
        return emptyList()
    }
    
    if (lines.size <= 1) return emptyList()

    // the regex splits CSV lines while correctly handling commas that appear in quoted fields
    // MIGHT REPLACE AND IMPLEMENT A LIBRARY INSTEAD
    val csvPattern = ",(?=(?:[^\"]*\"[^\"]*\")*[^\"]*$)".toRegex()
    
    // target these columns based on the file header:
    // 6: ProjectId, 11: ApprovedBudgetForContract, 12: ContractCost, 13: ActualCompletionDate, 16: StartDate

    for (line in lines.drop(1)) { // Skip the header line (lines.drop(1))
        if (line.isBlank()) continue
        
        // Split the line and remove surrounding quotes from the fields
        val fields = csvPattern.split(line).map { it.trim().removeSurrounding("\"") }
        
        // Basic checking if we have enough columns
        if (fields.size < 17) continue 

        try {
            // Extract fields and safely convert their types
            val projectId = fields[6]
            val approvedBudget = fields[11].toDoubleOrNull() ?: 0.0
            val contractCost = fields[12].toDoubleOrNull() ?: 0.0
            val completionDate = fields[13].toDateOrNull()
            val startDate = fields[16].toDateOrNull()

            projects.add(
                FloodProject(
                    projectId = projectId,
                    approvedBudget = approvedBudget,
                    contractCost = contractCost,
                    startDate = startDate,
                    completionDate = completionDate
                )
            )
        } catch (e: Exception) {
            // Catch any unexpected parsing errors (e.g., index out of bounds)
            println("Skipping malformed line: $line")
        }
    }
    return projects
}

// --- MAIN ---

fun main() {
    
    val projects = readProjectsFromFile("dpwh_flood_control_projects.csv")
    
    println("--- Data Ingestion Complete ---")
    println("Total projects read: ${projects.size}")
    
    // Print a summary of the first project
    projects.firstOrNull()?.let { project ->
        println("\nFirst Project Read (${project.projectId}):")
        println("  Contract Cost: PHP ${"%.2f".format(project.contractCost)}")
        println("  Cost Savings: PHP ${"%.2f".format(project.costSavings)}")
    }
}
