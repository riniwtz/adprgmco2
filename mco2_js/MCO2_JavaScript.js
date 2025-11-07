/********************
Last names: Cruz, Cunanan, Iwanta, Ngandu
Language: JavaScript
Paradigm(s):
********************/

const fs = require('fs');
const { parse } = require('csv-parse/sync');
const { stringify } = require('csv-stringify/sync');
const { receiveMessageOnPort } = require('worker_threads');

const prompt = require('prompt-sync')();

let running = true;
let FileLoaded = false;
let records = [];

const filePath = 'dpwh_flood_control_projects.csv';

function loadFileSync() {
  const fileContent = fs.readFileSync(filePath, 'utf8');

  const parsed = parse(fileContent, {
    columns: true,
    skip_empty_lines: false
  });

  const result = [];

  for (let data of parsed) {
    if (data.FundingYear && Number(data.FundingYear) === 2024) continue;

    ["ApprovedBudgetForContract", "FundingYear", "ContractCost",
     "ProjectLatitude", "ProjectLongitude",
     "ProvincialCapitalLatitude", "ProvincialCapitalLongitude"]
    .forEach(field => {
      if (data[field] && data[field].trim() !== "") {
        data[field] = Number(data[field]);
      }
    });

    if (data.ActualCompletionDate) data.ActualCompletionDate = new Date(data.ActualCompletionDate);
    if (data.StartDate) data.StartDate = new Date(data.StartDate);

    result.push(data);
  }

  return result;
}

function saveToCSV(records, outputFileName) {
  const csvOutput = stringify(records, { header: true });
  fs.writeFileSync(outputFileName, csvOutput, 'utf8');
  console.log(`CSV written immediately to ${outputFileName}`);
}

function generateCostSavings(ApprovedBudgetForContract, ContractCost){
  return ApprovedBudgetForContract - ContractCost;
}

function generateCompletionDelayDays(sDate, aDate){
  return aDate - sDate;
}



//MAIN
while (running) {
  console.log("Select Language Implementation:");
  console.log("[1] Load File");
  console.log("[2] Print First Record (Test)");
  console.log("[3] Exit");

  let Choice = prompt("Enter Choice: ");

  switch (Choice) {

    case "1":
      if (FileLoaded) {
        console.log("File already loaded.");
        break;
      }
      console.log("Loading file...");
      records = loadFileSync();
      FileLoaded = true;
      console.log("File successfully loaded.");
      break;

    case "2":
      if (!FileLoaded || records.length === 0) {
        console.log("No file loaded yet.");
      } else {
        console.log("Outputs are saved into individual CSV Report Files.");
        console.log("\nReport 1: Regional Flood Mitigation Efficiency Summary");
        console.log(records[0]);
      
        let template = records.map(record => {
          return {
            Region: record.Region,
            ApprovedBudgetForContract: record.ApprovedBudgetForContract,
            ContractCost: record.ContractCost,
            CostSavings: generateCostSavings(record.ApprovedBudgetForContract, record.ContractCost),
            CompletionDelayDays: generateCompletionDelayDays(record.StartDate.getUTCDate(), record.ActualCompletionDate.getUTCDate())
          };
        });
        templaye.sort()
        //get each region and maps it to a new array of objects with unique regions
        const uniqueRegions = Array.from(new Set(records.map(r => r.Region)));
        const report1 = uniqueRegions.map(region => ({
          Region: region,
          TotalApprovedBudget: 0,
          MedianCostSavings: 0,
          AverageCompletionDelayDays: 0,
          PercentageDelay: 0
        }));

        //total approved budget for each region
        template.forEach(data =>{
          let regionData = report1.find(r => r.Region === data.Region);
          if(regionData && !isNaN(data.ApprovedBudgetForContract)){
            regionData.TotalApprovedBudget += data.ApprovedBudgetForContract;
          }
          
        })


        //median cost savings for each region
        saveToCSV(template,"template.csv");
        
        saveToCSV(report1, "report1s.csv");
        
      }
      break;

    case "3":
      console.log("Exiting program...");
      running = false;
      break;

    default:
      console.log("Invalid choice.");
      break;
  }
}

