/********************
Last names: Cruz, Cunanan, Iwanta, Ngandu
Language: JavaScript
Paradigm(s):
********************/
//download required modules
const fs = require('fs');
const { parse } = require('csv-parse/sync');
const { stringify } = require('csv-stringify/sync');

//prompt-sync for synchronous user input  
const prompt = require('prompt-sync')();

//global variables
let running = true;
let FileLoaded = false;
let records = [];

//file path
const filePath = 'dpwh_flood_control_projects.csv';

//mapping regions to island groups
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

//function to load file synchronously
function loadFileSync() {
  //read file content
  const fileContent = fs.readFileSync(filePath, 'utf8');
  //parse CSV content
  const parsed = parse(fileContent, {
    columns: true,
    skip_empty_lines: false
  });
  console.log(`Total Records Loaded: ${parsed.length}`);
  const result = [];
  //data cleaning and type conversion
  for (let data of parsed) {
    //filter funding year between 2021 and 2023
    if(Number(data.FundingYear) < 2021 || Number(data.FundingYear) > 2023){
        continue; 
      }
    if(isNaN(Number(data.ApprovedBudgetForContract))){
      data.ApprovedBudgetForContract = 0;
    }
    //convert string to number
    data.FundingYear = Number(data.FundingYear);
    data.ApprovedBudgetForContract = Number(data.ApprovedBudgetForContract);
    data.ContractCost = Number(data.ContractCost);
    data.ProjectLatitude = Number(data.ProjectLatitude);
    data.ProjectLongitude = Number(data.ProjectLongitude);
    data.ProvincialCapitalLatitude = Number(data.ProvincialCapitalLatitude);
    data.ProvincialCapitalLongitude = Number(data.ProvincialCapitalLongitude);
    //convert string to date
    data.ActualCompletionDate = new Date(data.ActualCompletionDate);
    data.StartDate = new Date(data.StartDate);
    //append cleaned data to result array
    result.push(data);
  }
  console.log(`Total Records after Cleaning: ${result.length}`);
  //return cleaned data
  return result;
}

//function to save records to CSV
function saveToCSV(records, outputFileName) {
  //convert records to CSV format
  const csvOutput = stringify(records, { header: true });

  //write CSV content to file
  fs.writeFileSync(outputFileName, csvOutput, 'utf8');
  //log success message
  console.log(`Full table exported to ${outputFileName}`);
}

//function to generate cost savings
function generateCostSavings(ApprovedBudgetForContract, ContractCost){
  return ApprovedBudgetForContract - ContractCost;
}
//function to generate completion delay days
function generateCompletionDelayDays(sDate, aDate){
  const msPerDay = 1000 * 60 * 60 * 24;
  return  Math.round((aDate - sDate) / msPerDay);
}

//function to calculate average delay days
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

function printSampleToConsole(records){
  console.table(records.slice(0,2));
}


function toNum(v){
  const n = Number(v);
  return isNaN(n) ? 0 : n;
}

function safeAvg(arr){
  if(!Array.isArray(arr) || arr.length === 0) return 0;
  return arr.reduce((s, x) => s + toNum(x), 0) / arr.length;
}

function median(arr){
  if(!Array.isArray(arr) || arr.length === 0) return 0;
  const sorted = [...arr].map(toNum).sort((a,b)=>a-b);
  const n = sorted.length;
  if(n % 2 === 0) return (sorted[n/2 - 1] + sorted[n/2]) / 2;
  return sorted[Math.floor(n/2)];
}

function percent(count, total){
  if(total === 0) return 0;
  return (count / total) * 100;
}

//REPORT 1 (Regional Flood Mitigation Efficiency Summary)
function report1(){
  console.log('\nReport 1: Regional Flood Mitigation Efficiency Summary');
  //build template with normalized values
  const template = records.map(r => ({
    Region: r.Region || 'Unknown',
    ApprovedBudgetForContract: toNum(r.ApprovedBudgetForContract),
    ContractCost: toNum(r.ContractCost),
    CostSavings: generateCostSavings(toNum(r.ApprovedBudgetForContract), toNum(r.ContractCost)),
    CompletionDelayDays: generateCompletionDelayDays(new Date(r.StartDate), new Date(r.ActualCompletionDate))
  }));

  //set of regions present
  const uniqueRegions = Array.from(new Set(template.map(t => t.Region))).sort((a,b)=>a.localeCompare(b));

  //prepare grouped maps
  const costSavingsByRegion = {};
  const delaysByRegion = {};
  const totalApprovedByRegion = {};

  uniqueRegions.forEach(region => {
    costSavingsByRegion[region] = [];
    delaysByRegion[region] = [];
    totalApprovedByRegion[region] = 0;
  });

  template.forEach(t => {
    costSavingsByRegion[t.Region].push(t.CostSavings);
    delaysByRegion[t.Region].push(t.CompletionDelayDays);
    totalApprovedByRegion[t.Region] += t.ApprovedBudgetForContract;
  });

  const report = uniqueRegions.map(region => {
    const medianCost = median(costSavingsByRegion[region]);
    const avgDelay = safeAvg(delaysByRegion[region]);
    const delayedCount = delaysByRegion[region].filter(d => d > 30).length;
    const pctDelay = percent(delayedCount, delaysByRegion[region].length);
    const effScore = avgDelay === 0 ? 0 : (medianCost / avgDelay) * 100;

    return {
      Region: region,
      MainIsland: regionIslandGroup[region] || 'Unknown',
      TotalApprovedBudget: totalApprovedByRegion[region],
      MedianCostSavings: medianCost,
      AverageCompletionDelayDays: avgDelay,
      PercentageDelay: pctDelay,
      EfficiencyScore: effScore
    };
  });

  //sort by efficiency desc
  report.sort((a,b)=> b.EfficiencyScore - a.EfficiencyScore);

  //format numbers to 2 decimals
  report.forEach(r => {
    r.TotalApprovedBudget = Number(r.TotalApprovedBudget.toFixed(2));
    r.MedianCostSavings = Number(r.MedianCostSavings.toFixed(2));
    r.AverageCompletionDelayDays = Number(r.AverageCompletionDelayDays.toFixed(2));
    r.PercentageDelay = Number(r.PercentageDelay.toFixed(2));
    r.EfficiencyScore = Number(r.EfficiencyScore.toFixed(2));
  });

  printSampleToConsole(report);
  saveToCSV(report, 'report1_regional_summary.csv');
  saveToCSV(template, 'template1.csv');
}

//REPORT 2 (Contractor Reliability Analysis)
function report2(){
  console.log('\nReport 2: Contractor Reliability Analysis');
  console.log('(Top 15 by total ContractCost and With > 5 projects)');

  //normalize
  const template = records.map(r => ({
    Contractor: (r.Contractor || 'Unknown').trim(),
    ApprovedBudgetForContract: toNum(r.ApprovedBudgetForContract),
    ContractCost: toNum(r.ContractCost),
    CostSavings: generateCostSavings(toNum(r.ApprovedBudgetForContract), toNum(r.ContractCost)),
    CompletionDelayDays: generateCompletionDelayDays(new Date(r.StartDate), new Date(r.ActualCompletionDate))
  }));

  //group by contractor
  const byContractor = {};
  template.forEach(t => {
    const key = t.Contractor || 'Unknown';
    if(!byContractor[key]) byContractor[key] = { rows: [], totalContractCost: 0, totalApproved: 0 };
    byContractor[key].rows.push(t);
    byContractor[key].totalContractCost += t.ContractCost;
    byContractor[key].totalApproved += t.ApprovedBudgetForContract;
  });

  //build report array
  let report = Object.keys(byContractor).map(name => {
    const info = byContractor[name];
    const totalProjects = info.rows.length;
    const avgDelay = safeAvg(info.rows.map(r=>r.CompletionDelayDays));
    const totalCostSavings = info.rows.reduce((s,x)=>s + x.CostSavings, 0);
    const contractCost = info.totalContractCost;

    //ReliabilityIndex formula: (1 - (avgDelay/90)) * (totalCostSavings/contractCost) * 100
    let reliability = 0;
    if(contractCost > 0){
      reliability = (1 - (avgDelay / 90)) * (totalCostSavings / contractCost) * 100;
    }
    //reliability = Number(Math.max(0, Math.min(100, reliability)).toFixed(2));

    const flag = reliability < 50 ? 'High Risk' : 'Low Risk';

    return {
      Contractor: name,
      TotalProjects: totalProjects,
      ContractCost: Number(contractCost.toFixed(2)),
      AverageCompletionDelayDays: Number(avgDelay.toFixed(2)),
      TotalCostSavings: Number(totalCostSavings.toFixed(2)),
      ReliabilityIndex: reliability,
      Flag: flag
    };
  });

  //filter contractors with > 5 projects
  report = report.filter(r => r.TotalProjects > 5);

  //sort by ContractCost desc
  report.sort((a,b)=> b.ContractCost - a.ContractCost);

  //top 15
  if(report.length > 15) report = report.slice(0,15);

  printSampleToConsole(report);
  saveToCSV(report, 'report2_contractor_ranking.csv');
}

//REPORT 3 (Annual Project Type cost overrun trends)
function report3(){
  console.log('\nReport 3: Annual Project Type cost overrun trends');

  //normalize
  const template = records.map(r => ({
    FundingYear: toNum(r.FundingYear),
    TypeOfWork: (r.TypeOfWork || 'Unknown').trim(),
    CostSavings: generateCostSavings(toNum(r.ApprovedBudgetForContract), toNum(r.ContractCost))
  }));

  //group by FundingYear|TypeOfWork
  const groups = {};
  template.forEach(t => {
    const year = t.FundingYear || 0;
    const type = t.TypeOfWork || 'Unknown';
    const key = `${year}|${type}`;
    if(!groups[key]) groups[key] = { FundingYear: year, TypeOfWork: type, CostSavingsList: [] };
    groups[key].CostSavingsList.push(t.CostSavings);
  });

  let report = Object.values(groups).map(g => {
    const totalProjects = g.CostSavingsList.length;
    const avgCostSavings = safeAvg(g.CostSavingsList);
    const overrunCount = g.CostSavingsList.filter(n => toNum(n) < 0).length;
    const overrunRate = percent(overrunCount, totalProjects);

    return {
      FundingYear: g.FundingYear,
      TypeOfWork: g.TypeOfWork,
      TotalProjects: totalProjects,
      AverageCostSavings: avgCostSavings,
      OverrunRate: overrunRate
    };
  });

  //Year-over-Year (YoY) change relative to 2021 per TypeOfWork
  const baselineByType = {};
  report.filter(r => r.FundingYear === 2021).forEach(b => {
    baselineByType[b.TypeOfWork] = b.AverageCostSavings;
  });

  report = report.map(r => {
    const base = baselineByType[r.TypeOfWork];
    if(typeof base !== 'undefined' && Math.abs(base) > 0.000001){
      r.YoYChange = ((r.AverageCostSavings - base) / Math.abs(base)) * 100;
    } else {
      r.YoYChange = 0; //cannot compute meaningful YoY
    }
    return r;
  });

  //sort by FundingYear then by AverageCostSavings desc
  report.sort((a,b)=>{
    if(a.FundingYear !== b.FundingYear) return a.FundingYear - b.FundingYear;
    return b.AverageCostSavings - a.AverageCostSavings;
  });

  //format numbers
  report.forEach(r => {
    r.AverageCostSavings = Number(r.AverageCostSavings.toFixed(2));
    r.OverrunRate = Number(r.OverrunRate.toFixed(2));
    r.YoYChange = Number(r.YoYChange.toFixed(2));
  });

  printSampleToConsole(report);
  saveToCSV(report, 'report3_annual_trend.csv');
}

//Export functions if running in modular env (optional)
if(typeof module !== 'undefined' && module.exports){
  module.exports = { report1, report2, report3 };
}
function getDelayDays(start, end) {
  if (!start || !end) return 0;
  const s = new Date(start);
  const e = new Date(end);
  const diff = (e - s) / (1000 * 60 * 60 * 24);
  return Math.max(diff, 0);
}


function generateSummary(records) {
  //total projects
  const totalProjects = records.length;

  //total unique contractors
  const totalContractors = new Set(records.map(r => r.Contractor)).size;

  //total unique provinces
  const totalProvinces = new Set(records.map(r => r.Province)).size;

  //global average delay (across all projects)
  const avgDelay = (
  records.reduce(
    (sum, r) => sum + getDelayDays(r.StartDate, r.ActualCompletionDate),
    0
  ) / records.length
).toFixed(2);

  //total savings
  const totalSavings = records.reduce((sum, r) => {
    return sum + generateCostSavings(r.ApprovedBudgetForContract, r.ContractCost);
  }, 0).toFixed(2);

  //build summary object
  const summary = {
    totalProjects,
    totalContractors,
    totalProvinces,
    averageDelayDays: Number(avgDelay),
    totalCostSavings: Number(totalSavings)
  };

  //save summary.json
  const fs = require("fs");
  fs.writeFileSync("summary.json", JSON.stringify(summary, null, 2), "utf-8");
  console.log("summary.json generated!");
}

//MAIN
while (running) {
  console.log("\nSelect Language Implementation:");
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
        let continueWhile = true;
        while(continueWhile){
          report1();
          report2();
          report3();
          generateSummary(records);
          
          saveToCSV(records, "full_table_export.csv");
          let menuChoice = prompt("Back to Main Menu? (Y/N): ");
          if (menuChoice.toUpperCase() === "Y") {
            continueWhile = false;
            break;
          }else if(menuChoice.toUpperCase() === "N"){
            
          }else{
            console.log("Invalid choice. Please enter Y or N.");
          }
        }
        
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

