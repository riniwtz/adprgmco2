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
  
  const result = [];

  //data cleaning and type conversion
  for (let data of parsed) {
    //filter funding year between 2021 and 2023
    if (Number(data.FundingYear) < 2021 || Number(data.FundingYear) > 2023) continue;
    //filter missing values
    if(isNaN(Number(data.ApprovedBudgetForContract)) || 
       isNaN(Number(data.ContractCost)) ||
       isNaN(Number(data.ProjectLatitude)) ||
       isNaN(Number(data.ProjectLongitude)) ||
       isNaN(Number(data.ProvincialCapitalLatitude)) ||
       isNaN(Number(data.ProvincialCapitalLongitude))
      ){
        continue;
      }else{
        //convert string to number
        data.FundingYear = Number(data.FundingYear);
        data.ApprovedBudgetForContract = Number(data.ApprovedBudgetForContract);
        data.ContractCost = Number(data.ContractCost);
        data.ProjectLatitude = Number(data.ProjectLatitude);
        data.ProjectLongitude = Number(data.ProjectLongitude);
        data.ProvincialCapitalLatitude = Number(data.ProvincialCapitalLatitude);
        data.ProvincialCapitalLongitude = Number(data.ProvincialCapitalLongitude);
      }
    //convert string to date
    data.ActualCompletionDate = new Date(data.ActualCompletionDate);
    data.StartDate = new Date(data.StartDate);
    //append cleaned data to result array
    result.push(data);
  }
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
//REPORT FUNCTIONS

function report1(){
  console.log("Outputs are saved into individual CSV Report Files.");
  console.log("\nReport 1: Regional Flood Mitigation Efficiency Summary");
  console.log("\nFiltered Projects from 2021-2023 and Region");
  
  //map records to new array of objects with required fields
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
    if (regionCompare !== 0) return regionCompare;//sort by region A to Z

    return b.CostSavings - a.CostSavings;//sort by costsavings high to low
  });


  //get each region and maps it to a new array of objects with unique regions
  const uniqueRegions = Array.from(new Set(records.map(r => r.Region)));

  //initialize report1 structure
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
    let sum = 0;
    if(regionData && !isNaN(data.ApprovedBudgetForContract)){
      sum += data.ApprovedBudgetForContract;
    }
    regionData.TotalApprovedBudget = sum.toFixed(2);
  })
  
  
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
  data.MedianCostSavings = costSavingsMedian(grouped[data.Region]).toFixed(2);
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
    data.AverageCompletionDelayDays = AverageDelayDays(delayDays[data.Region]).toFixed(2);
  });
  
  //calculate percentage delay for each region
  report1.forEach(data =>{
    data.PercentageDelay = PercentageDelay(delayDays[data.Region]).toFixed(2);
  })

  //calculate efficiency score for each region
  report1.forEach(data =>{
    data.EfficiencyScore = EfficiencyScore(data.MedianCostSavings, data.AverageCompletionDelayDays).toFixed(2);
  });

  //sort report1 by efficiency score high to low
  report1.sort((a, b) => {
    return b.EfficiencyScore - a.EfficiencyScore;
  });

 
  printSampleToConsole(report1);
  //save report1 to CSV
  saveToCSV(report1, "report1_regional_summary.csv");

  
  //Cost Savings Median Function
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

  //Percentage Delay Function
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
  //Efficiency Score Function
  function EfficiencyScore(medianCostSavings, AverageDelay){
    return (medianCostSavings/AverageDelay) * 100;
  }
}

//Report 2 Function
function report2(){

  //map records to new array of objects with required fields
  console.log("\nReport 2: Contractor Reliability Analysis");
  console.log("\nTop Contractors Performance Ranking");
  console.log("(Top 15 by total Cost and With > 5 projects)");
  const uniqueContractors = Array.from(new Set(records.map(r => r.Contractor)));
  const report2 = uniqueContractors.map(contractor => ({
    Contractor: contractor, 
    TotalProjects: 0,
    ContractCost: 0,
    AverageCompletionDelayDays: 0,
    TotalCostSavings:0,
    ReliabilityIndex:0,
    Flag: ""
  }));

  let template2 = records.map(record => {
    return {
      Contractor: record.Contractor,
      apbudg: record.ApprovedBudgetForContract,
      cost: record.ContractCost,
      CostSavings: generateCostSavings(record.ApprovedBudgetForContract, record.ContractCost),
      CompletionDelayDays: generateCompletionDelayDays(record.StartDate, record.ActualCompletionDate)
    };
  });
  

  template2.sort((a, b) => {
    const contractorCompare = a.Contractor.localeCompare(b.Contractor);
    if (contractorCompare !== 0) return contractorCompare; // Contractor A→Z

    return b.ContractCost - a.ContractCost; // CostSavings high→low
  });

  //total approved budget for each region
  template2.forEach(data =>{
    let contractorData = report2.find(r => r.Contractor === data.Contractor);
    let counter = 0;
    if(contractorData){
      contractorData.TotalProjects +=1;
    }
  })

  for (let i = report2.length - 1; i >= 0; i--) {
    if (report2[i].TotalProjects <= 5) {
      report2.splice(i, 1); // removes that row
    }
  }
  template2.forEach(data =>{
    let contractorData = report2.find(r => r.Contractor === data.Contractor);
    
    if(contractorData){
      contractorData.ContractCost += data.cost;
    }
  })

  //group completion delay days by contractor
  const delayDays_Contractor = {};
  template2.forEach(data =>{
    if(!delayDays_Contractor[data.Contractor]){
      delayDays_Contractor[data.Contractor] = [];
    }
    delayDays_Contractor[data.Contractor].push(data.CompletionDelayDays);
  });

  //Generate average completion delay days for each contractor (reused function from report 1)
  report2.forEach(data =>{
    data.AverageCompletionDelayDays = AverageDelayDays(delayDays_Contractor[data.Contractor]).toFixed(2);
  });

  //total cost savings for each contractor
  template2.forEach(data =>{
    let contractorData = report2.find(r => r.Contractor === data.Contractor);
    if(contractorData){
      contractorData.TotalCostSavings += data.CostSavings;
    }
  });

  report2.forEach(r => {
    r.TotalCostSavings = r.TotalCostSavings.toFixed(2);
    r.ContractCost = r.ContractCost.toFixed(2);
    r.ReliabilityIndex = r.ReliabilityIndex.toFixed(2);
  });

  //calculate reliability index and flag for each contractor
  report2.forEach(data =>{
    
    let reliabilityIndex = (1-(data.AverageCompletionDelayDays/90)) * (data.TotalCostSavings/data.ContractCost) * 100;
    data.ReliabilityIndex = reliabilityIndex;
    if(reliabilityIndex > 100){
      data.ReliabilityIndex = 100;
    }
    if(reliabilityIndex < 50){
      data.Flag = "High Risk";
    }else{
      data.Flag = "Low Risk";
    }
  })

  report2.sort((a, b) => {

    return b.ContractCost - a.ContractCost; // CostSavings high→low
  });

  for (let i = report2.length - 1; i >= 15; i--) {
    report2.splice(i, 1); // removes that row
  }
  printSampleToConsole(report2);
  saveToCSV(report2, "report2_contractor_ranking.csv");
  
 

}

function report3() {

  console.log("\nReport 3: Annual Project Type cost overrun trends");
  console.log("\nGrouped by Funding Year and Type of Work");
  
  //map records to new array of objects with required fields
  let template3 = records.map(record => {
    return {
      FundingYear: record.FundingYear,
      TypeOfWork: record.TypeOfWork,
      CostSavings: generateCostSavings(record.ApprovedBudgetForContract, record.ContractCost)
    };
  });

  //group by Funding Year and Type of Work
  const groups = {};
  template3.forEach(item => {
    const key = `${item.FundingYear}|${item.TypeOfWork}`;
    if (!groups[key]) {
      groups[key] = {
        FundingYear: item.FundingYear,
        TypeOfWork: item.TypeOfWork,
        CostSavingsList: []
      };
    }
    groups[key].CostSavingsList.push(item.CostSavings);
  });

  //calculate metrics for each group
  let report3 = Object.values(groups).map(group => {
    const totalProjects = group.CostSavingsList.length;
    const averageCostSavings = group.CostSavingsList.reduce((sum, n) => sum + n, 0) / totalProjects;
    const overrunCount = group.CostSavingsList.filter(n => n < 0).length;
    const overrunRate = (overrunCount / totalProjects) * 100;
    return {
      FundingYear: group.FundingYear,
      TypeOfWork: group.TypeOfWork,
      TotalProjects: totalProjects,
      AverageCostSavings: averageCostSavings,
      OverrunRate: overrunRate,
    };
  });

  //calculate YoY change compared to 2021
  const baseline = report3.filter(r => r.FundingYear == 2021);

  //map through report3 to calculate YoY change
  report3 = report3.map(entry => {
    const base = baseline.find(b => b.TypeOfWork === entry.TypeOfWork);
    if (base && base.AverageCostSavings !== 0) {
      entry.YoYChange = ((entry.AverageCostSavings - base.AverageCostSavings) / Math.abs(base.AverageCostSavings)) * 100;
    }
    return entry;
  });

  //sort by Funding Year then Type of Work
  report3.sort((a, b) => {
  if (a.FundingYear !== b.FundingYear) {
    return a.FundingYear - b.FundingYear;// numeric sort by year
  }
    return b.AverageCostSavings - a.AverageCostSavings; // alphabetic sort by name
  });

  report3.forEach(data =>{
    data.AverageCostSavings = data.AverageCostSavings.toFixed(2);
    data.OverrunRate = data.OverrunRate.toFixed(2)
    if(!isNaN(Number(data.YoYChange))){
      data.YoYChange = data.YoYChange.toFixed(2);
    }
  });


  printSampleToConsole(report3);
  saveToCSV(report3, "report3_annual_trend.csv");
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
    records.reduce((sum, r) => sum + (r.AverageCompletionDelayDays || 0), 0) / totalProjects
  ).toFixed(2);

  //total savings
  const totalSavings = records.reduce((sum, r) => {
    return sum + generateCostSavings(r.ApprovedBudgetForContract, r.ContractCost);
  }, 0).toFixed(2);

  // build summary object
  const summary = {
    totalProjects,
    totalContractors,
    totalProvinces,
    averageDelayDays: Number(avgDelay),
    totalCostSavings: Number(totalSavings)
  };

  // save summary.json
  const fs = require("fs");
  fs.writeFileSync("summary.json", JSON.stringify(summary, null, 2), "utf-8");
  console.log("summary.json generated!");
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
        report3();
        generateSummary(records);


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

