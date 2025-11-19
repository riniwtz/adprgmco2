const fs = require("fs");
const { parse } = require("csv-parse/sync");
const { stringify } = require("csv-stringify/sync");
const prompt = require("prompt-sync")();

let running = true;
let FileLoaded = false;
let records = [];

const filePath = "dpwh_flood_control_projects.csv";

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
  BARMM: "Mindanao",
};

function generateCostSavings(ApprovedBudgetForContract, ContractCost) {
  return ApprovedBudgetForContract - ContractCost;
}

function generateCompletionDelayDays(sDate, aDate) {
  const msPerDay = 1000 * 60 * 60 * 24;
  return Math.round((aDate - sDate) / msPerDay);
}

function AverageDelayDays(dataSet) {
  if (dataSet.length === 0) return 0;
  let sum = 0;
  for (let i = 0; i < dataSet.length; i++) {
    sum += dataSet[i];
  }
  return sum / dataSet.length;
}

function costSavingsMedian(dataSet) {
  if (dataSet.length === 0) return 0;
  dataSet.sort((a, b) => a - b);

  let median = 0;
  let dataSize = dataSet.length;
  let mid = Math.floor(dataSize / 2);

  if (dataSize % 2 === 0) {
    median = (dataSet[mid - 1] + dataSet[mid]) / 2;
  } else {
    median = dataSet[mid];
  }
  return median;
}

function loadFileSync() {
  const fileContent = fs.readFileSync(filePath, "utf8");
  const parsed = parse(fileContent, {
    columns: true,
    skip_empty_lines: true,
  });

  const result = [];
  let skipped = 0;

  for (let data of parsed) {
    let fYear = Number(data.FundingYear);
    if (fYear < 2021 || fYear > 2023) {
      skipped++;
      continue;
    }

    let approvedBudget = Number(
      String(data.ApprovedBudgetForContract).replace(/,/g, "")
    );
    let contractCost = Number(String(data.ContractCost).replace(/,/g, ""));

    if (isNaN(approvedBudget)) approvedBudget = 0.0;

    if (
      isNaN(contractCost) ||
      !data.Region ||
      !data.Contractor ||
      !data.MainIsland
    ) {
      skipped++;
      continue;
    }

    let actualDate = new Date(data.ActualCompletionDate);
    let startDate = new Date(data.StartDate);

    let savings = generateCostSavings(approvedBudget, contractCost);
    let delay = 0;
    if (!isNaN(actualDate) && !isNaN(startDate)) {
      delay = generateCompletionDelayDays(startDate, actualDate);
    }

    data.FundingYear = fYear;
    data.ApprovedBudgetForContract = approvedBudget;
    data.ContractCost = contractCost;
    data.CostSavings = savings;
    data.CompletionDelayDays = delay;
    data.StartDate = startDate;
    data.ActualCompletionDate = actualDate;

    result.push(data);
  }

  console.log(`Total Records Loaded: ${parsed.length}`);
  console.log(`Skipped/Filtered: ${skipped}`);
  console.log(`Total Projects (2021-2023): ${result.length}`);

  return result;
}

function saveToCSV(records, outputFileName) {
  const csvOutput = stringify(records, {
    header: true,
  });
  fs.writeFileSync(outputFileName, csvOutput, "utf8");
  console.log(`Exported: ${outputFileName}`);
}

function printSampleToConsole(records) {
  console.table(records.slice(0, 5));
}

function report1() {
  console.log("\nReport 1: Regional Flood Mitigation Efficiency Summary");
  console.log("(Norm Score 0-100 | Delay > 30 days)");

  const uniqueRegions = Array.from(new Set(records.map((r) => r.Region)));

  let report1 = uniqueRegions.map((region) => {
    let regionRecords = records.filter((r) => r.Region === region);
    let totalBudget = regionRecords.reduce(
      (sum, r) => sum + r.ApprovedBudgetForContract,
      0
    );
    let savingsList = regionRecords.map((r) => r.CostSavings);
    let medianSavings = costSavingsMedian(savingsList);
    let delaysList = regionRecords.map((r) => r.CompletionDelayDays);
    let avgDelay = AverageDelayDays(delaysList);

    let delayedCount = delaysList.filter((d) => d > 30).length;
    let pctDelay =
      delaysList.length > 0 ? (delayedCount / delaysList.length) * 100 : 0;

    let divisor = Math.abs(avgDelay) < 0.001 ? 1.0 : avgDelay;
    let rawScore = (medianSavings / divisor) * 100.0;

    return {
      Region: region,
      MainIsland: regionIslandGroup[region],
      TotalApprovedBudget: Number(totalBudget.toFixed(2)),
      MedianCostSavings: Number(medianSavings.toFixed(2)),
      AverageCompletionDelayDays: Number(avgDelay.toFixed(2)),
      PercentageDelay: Number(pctDelay.toFixed(2)),
      RawScore: rawScore,
      EfficiencyScore: 0,
    };
  });

  let minScore = Math.min(...report1.map((r) => r.RawScore));
  let maxScore = Math.max(...report1.map((r) => r.RawScore));
  let range = maxScore - minScore;

  report1.forEach((r) => {
    if (range === 0) {
      r.EfficiencyScore = 100.0;
    } else {
      r.EfficiencyScore = Number(
        (((r.RawScore - minScore) / range) * 100).toFixed(2)
      );
    }
    delete r.RawScore;
  });

  report1.sort((a, b) => b.EfficiencyScore - a.EfficiencyScore);

  printSampleToConsole(report1);
  saveToCSV(report1, "report1_regional_summary.csv");
}

function report2() {
  console.log("\nReport 2: Contractor Reliability Analysis");
  console.log("Top 15 by Total Cost (Projects >= 5)");

  const uniqueContractors = Array.from(
    new Set(records.map((r) => r.Contractor))
  );

  let report2 = uniqueContractors
    .map((contractor) => {
      let group = records.filter((r) => r.Contractor === contractor);
      let totalProjects = group.length;

      if (totalProjects < 5) return null;

      let totalCost = group.reduce((sum, r) => sum + r.ContractCost, 0);
      let totalSavings = group.reduce((sum, r) => sum + r.CostSavings, 0);
      let delays = group.map((r) => r.CompletionDelayDays);
      let avgDelay = AverageDelayDays(delays);

      let term1 = 1.0 - avgDelay / 90.0;
      let term2 = totalCost !== 0 ? totalSavings / totalCost : 0.0;
      let reliabilityIndex = term1 * term2 * 100.0;

      if (reliabilityIndex > 100) reliabilityIndex = 100;

      let flag = reliabilityIndex < 50 ? "High Risk" : "-";

      return {
        Rank: 0,
        Contractor: contractor,
        TotalCost: Number(totalCost.toFixed(2)),
        TotalProjects: totalProjects,
        AverageCompletionDelayDays: Number(avgDelay.toFixed(2)),
        TotalCostSavings: Number(totalSavings.toFixed(2)),
        ReliabilityIndex: Number(reliabilityIndex.toFixed(2)),
        Flag: flag,
      };
    })
    .filter((item) => item !== null);

  report2.sort((a, b) => b.TotalCost - a.TotalCost);
  report2 = report2.slice(0, 15);

  report2.forEach((r, index) => {
    r.Rank = index + 1;
  });

  printSampleToConsole(report2);
  saveToCSV(report2, "report2_contractor_ranking.csv");
}

function report3() {
  console.log("\nReport 3: Annual Project Type Cost Overrun Trends");
  console.log("Sorted: Year Asc, Avg Savings Desc | Baseline: 2021");

  const baselineMap = {};
  records
    .filter((r) => r.FundingYear === 2021)
    .forEach((r) => {
      if (!baselineMap[r.TypeOfWork]) {
        baselineMap[r.TypeOfWork] = {
          sum: 0,
          count: 0,
        };
      }
      baselineMap[r.TypeOfWork].sum += r.CostSavings;
      baselineMap[r.TypeOfWork].count += 1;
    });

  const baselineAverages = {};
  for (let type in baselineMap) {
    baselineAverages[type] = baselineMap[type].sum / baselineMap[type].count;
  }

  const groups = {};
  records.forEach((r) => {
    const key = `${r.FundingYear}|${r.TypeOfWork}`;
    if (!groups[key]) {
      groups[key] = {
        FundingYear: r.FundingYear,
        TypeOfWork: r.TypeOfWork,
        CostSavingsList: [],
      };
    }
    groups[key].CostSavingsList.push(r.CostSavings);
  });

  let report3 = Object.values(groups).map((group) => {
    const totalProjects = group.CostSavingsList.length;
    const sumSavings = group.CostSavingsList.reduce((sum, n) => sum + n, 0);
    const averageCostSavings = sumSavings / totalProjects;

    const overrunCount = group.CostSavingsList.filter((n) => n < 0).length;
    const overrunRate = (overrunCount / totalProjects) * 100;

    let yoyChange = 0;
    const baseAvg = baselineAverages[group.TypeOfWork];

    if (baseAvg !== undefined && baseAvg !== 0) {
      yoyChange = ((averageCostSavings - baseAvg) / Math.abs(baseAvg)) * 100;
    }

    return {
      FundingYear: group.FundingYear,
      TypeOfWork: group.TypeOfWork,
      TotalProjects: totalProjects,
      AverageCostSavings: Number(averageCostSavings.toFixed(2)),
      OverrunRate: Number(overrunRate.toFixed(2)) + "%",
      YoYChange: Number(yoyChange.toFixed(2)) + "%",
    };
  });

  report3.sort((a, b) => {
    if (a.FundingYear !== b.FundingYear) {
      return a.FundingYear - b.FundingYear;
    }
    return parseFloat(b.AverageCostSavings) - parseFloat(a.AverageCostSavings);
  });

  printSampleToConsole(report3);
  saveToCSV(report3, "report3_annual_trends.csv");
}

function generateSummary(records) {
  const totalProjects = records.length;
  const totalContractors = new Set(records.map((r) => r.Contractor)).size;
  const totalProvinces = new Set(records.map((r) => r.Province)).size;
  const sumDelay = records.reduce((sum, r) => sum + r.CompletionDelayDays, 0);
  const avgDelay = totalProjects > 0 ? sumDelay / totalProjects : 0;
  const totalSavings = records.reduce((sum, r) => sum + r.CostSavings, 0);

  const summary = {
    totalProjectsAnalyzed: totalProjects,
    totalContractors: totalContractors,
    totalProvinces: totalProvinces,
    globalAvgDelay: Number(avgDelay.toFixed(2)),
    totalSavings: Number(totalSavings.toFixed(2)),
  };

  fs.writeFileSync("summary.json", JSON.stringify(summary, null, 2), "utf-8");
  console.log("Exported: summary.json");
}

while (running) {
  console.log("\nSelect Language Implementation:");
  console.log("[1] Load File");
  console.log("[2] Generate Reports");
  console.log("[3] Exit");

  let Choice = prompt("Enter Choice: ");

  switch (Choice) {
    case "1":
      console.log("Loading file...");
      records = loadFileSync();
      FileLoaded = true;
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
