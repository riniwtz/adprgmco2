/********************
Last names: Cruz, Cunanan, Iwanta, Ngandu
Language: JavaScript
Paradigm(s):
********************/

const fs = require('fs');
const { parse } = require('csv-parse/sync');
const { stringify } = require('csv-stringify/sync');
//const { receiveMessageOnPort } = require('worker_threads');

const prompt = require('prompt-sync')();

let running = true;
let FileLoaded = false;
let records = [];

const filePath = 'dpwh_flood_control_projects.csv';

const regionIslandGroup = {
  "National Capital Region": "Luzon",
  "Cordillera Administrative Region": "Luzon",

  "Region I": "Luzon",
  "Region II": "Luzon",
  "Region III": "Luzon",
  "Region IV-A": "Luzon",
  "Region IV-B": "Luzon",
  "Region V": "Luzon",

  "Region VI": "Visayas",
  "Region VII": "Visayas",
  "Region VIII": "Visayas",

  "Region IX": "Mindanao",
  "Region X": "Mindanao",
  "Region XI": "Mindanao",
  "Region XII": "Mindanao",
  "Region XIII": "Mindanao",

  "BARMM": "Mindanao"
};
function loadFileSync() {
  const fileContent = fs.readFileSync(filePath, 'utf8');

  const parsed = parse(fileContent, {
    columns: true,
    skip_empty_lines: false
  });

  const result = [];

  for (let data of parsed) {
  
    if (Number(data.FundingYear) === 2024) continue;
    if(isNaN(Number(data.ApprovedBudgetForContract)) || 
       isNaN(Number(data.ContractCost)) ||
       isNaN(Number(data.ProjectLatitude)) ||
       isNaN(Number(data.ProjectLongitude)) ||
       isNaN(Number(data.ProvincialCapitalLatitude)) ||
       isNaN(Number(data.ProvincialCapitalLongitude))
      ){
        continue;
      }else{
        data.FundingYear = Number(data.FundingYear);
        data.ApprovedBudgetForContract = Number(data.ApprovedBudgetForContract);
        data.ContractCost = Number(data.ContractCost);
        data.ProjectLatitude = Number(data.ProjectLatitude);
        data.ProjectLongitude = Number(data.ProjectLongitude);
        data.ProvincialCapitalLatitude = Number(data.ProvincialCapitalLatitude);
        data.ProvincialCapitalLongitude = Number(data.ProvincialCapitalLongitude);
      }

    data.ActualCompletionDate = new Date(data.ActualCompletionDate);
    data.StartDate = new Date(data.StartDate);
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
  const msPerDay = 1000 * 60 * 60 * 24;
  return  Math.round((aDate - sDate) / msPerDay);
  }

function report1(){
  console.log("Outputs are saved into individual CSV Report Files.");
  console.log("\nReport 1: Regional Flood Mitigation Efficiency Summary");
  
  let template = records.map(record => {
    return {
      Region: record.Region,
      ApprovedBudgetForContract: record.ApprovedBudgetForContract,
      ContractCost: record.ContractCost,
      CostSavings: generateCostSavings(record.ApprovedBudgetForContract, record.ContractCost),
      CompletionDelayDays: generateCompletionDelayDays(record.StartDate, record.ActualCompletionDate)
    };
  });
  template.sort((a, b) => {
    const regionCompare = a.Region.localeCompare(b.Region);
    if (regionCompare !== 0) return regionCompare; // Region A→Z

    return b.CostSavings - a.CostSavings; // CostSavings high→low
  });

  //get each region and maps it to a new array of objects with unique regions
  const uniqueRegions = Array.from(new Set(records.map(r => r.Region)));
  const report1 = uniqueRegions.map(region => ({
    Region: region,
    MainIsland: regionIslandGroup[region],
    TotalApprovedBudget: 0,
    MedianCostSavings: 0,
    AverageCompletionDelayDays: 0,
    PercentageDelay: 0,
    EfficiencyScore: 0
  }));

  //total approved budget for each region
  template.forEach(data =>{
    let regionData = report1.find(r => r.Region === data.Region);
    if(regionData && !isNaN(data.ApprovedBudgetForContract)){
      regionData.TotalApprovedBudget += data.ApprovedBudgetForContract;
    }
  })
  //median cost savings for each region
  //group cost savings by region
  const grouped = {};
  template.forEach(data =>{
    if(!grouped[data.Region]){
      grouped[data.Region] = [];
    }
    grouped[data.Region].push(data.CostSavings);
  });

  //calculate median for each region
  report1.forEach(data =>{
  data.MedianCostSavings = costSavingsMedian(grouped[data.Region]);
  })

  //group completion delay days by region
  const delayDays = {};
  template.forEach(data =>{
    if(!delayDays[data.Region]){
      delayDays[data.Region] = [];
    }
    delayDays[data.Region].push(data.CompletionDelayDays);
  });

  //calculate average completion delay days for each region
  report1.forEach(data =>{
    data.AverageCompletionDelayDays = AverageDelayDays(delayDays[data.Region]);
  });
  
  //calculate percentage delay for each region
  report1.forEach(data =>{
    data.PercentageDelay = PercentageDelay(delayDays[data.Region]);
  })

  //calculate efficiency score for each region
  report1.forEach(data =>{
    data.EfficiencyScore = EfficiencyScore(data.MedianCostSavings, data.AverageCompletionDelayDays);
  });

  report1.sort((a, b) => {
    return b.EfficiencyScore - a.EfficiencyScore;
  });
  
  saveToCSV(report1, "report1s.csv");

  

  

  function costSavingsMedian(dataSet){
    let median = 0;
    let dataSize = dataSet.length;
    if(dataSize % 2 === 0){
      median = (dataSet[dataSize/2] + dataSet[dataSize/2+1])/2
    }else{
      median = dataSet[Math.floor(dataSize/2)];
    }
    return median;
  }

  function AverageDelayDays(dataSet){
    let Average = 0;
    let size = dataSet.length;
    let sum = 0;
    for(let i = 0; i < size; i++){
      sum += dataSet[i];
    }
    Average = sum/size;
    return Average;
  }

  function PercentageDelay(dataSet){
    let percentage = 0;
    let counter = 0;
    let RegionSize = dataSet.length;
    for(let i = 0; i < RegionSize; i++){
      if(dataSet[i] > 30){
        counter++;
      }
    }
    return (counter/RegionSize) * 100;
  }

  function EfficiencyScore(medianCostSavings, AverageDelay){
    return (medianCostSavings/AverageDelay) * 100;
  }
}


function report2(){

  const uniqueContractors = Array.from(new Set(records.map(r => r.Contractor)));
  const report2 = uniqueContractors.map(contractor => ({
    Contractor: contractor,
    TotalProjects: 0,
    AverageCompletionDelayDays: 0,
    TotalCostSavings:0,
    ReliabilityIndex:0
  }));

  let template2 = records.map(record => {
    return {
      Contractor: record.Contractor,
      apbudg: record.ApprovedBudgetForContract,
      cost: record.ContractCost,
      sdate: record.StartDate,
      adate: record.ActualCompletionDate,
      CostSavings: generateCostSavings(record.ApprovedBudgetForContract, record.ContractCost),
      CompletionDelayDays: generateCompletionDelayDays(record.StartDate, record.ActualCompletionDate)
    };
  });


  saveToCSV(template2, "template2.csv");
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
        report1();
        report2();
        
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

