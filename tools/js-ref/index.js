import { runAnafanafoCases } from "./anafanafo.js";
import { runBadgeMakerCases } from "./badge-maker.js";

const runners = {
  anafanafo: runAnafanafoCases,
  "badge-maker": runBadgeMakerCases,
};

function usage() {
  const names = Object.keys(runners).join(", ");
  throw new Error(`expected a reference tool name (${names}) as the first argument`);
}

const toolName = process.argv[2];
if (!toolName) {
  usage();
}

const runCases = runners[toolName];
if (!runCases) {
  usage();
}

const stdin = await Bun.stdin.text();
const cases = JSON.parse(stdin);
const results = runCases(cases);

process.stdout.write(`${JSON.stringify(results)}\n`);
