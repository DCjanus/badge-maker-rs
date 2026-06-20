const { appendFileSync, existsSync, readFileSync } = require("node:fs");
const { spawnSync } = require("node:child_process");

function input(name) {
  return process.env[`INPUT_${name.toUpperCase().replaceAll("-", "_")}`] || "";
}

function warn(message) {
  console.error(`::warning::${message}`);
}

function runGh(args) {
  return spawnSync("gh", args, {
    encoding: "utf8",
    env: {
      ...process.env,
      GH_TOKEN: process.env.GH_TOKEN || input("github-token"),
    },
    stdio: ["ignore", "pipe", "pipe"],
  });
}

function commandMessage(result) {
  return result.stderr.trim() || result.stdout.trim();
}

function ensureLabel(repo, label) {
  const result = runGh([
    "label",
    "create",
    label,
    "--repo",
    repo,
    "--description",
    "Managed Shields.io compatibility report issue",
    "--color",
    "5319e7",
  ]);
  if (result.status === 0) {
    return true;
  }

  const message = commandMessage(result);
  if (message.toLowerCase().includes("already exists")) {
    return true;
  }

  warn(`could not create compatibility report label: ${message}`);
  return false;
}

function findIssue(repo, label) {
  const result = runGh([
    "issue",
    "list",
    "--repo",
    repo,
    "--state",
    "open",
    "--author",
    "github-actions[bot]",
    "--label",
    label,
    "--json",
    "number,updatedAt",
  ]);
  if (result.status !== 0) {
    warn(`could not search compatibility report issues: ${commandMessage(result)}`);
    return undefined;
  }

  const issues = JSON.parse(result.stdout);
  if (issues.length === 0) {
    return undefined;
  }

  issues.sort((left, right) => right.updatedAt.localeCompare(left.updatedAt));
  return issues[0].number;
}

function issueUrl(repo, number) {
  const result = runGh([
    "issue",
    "view",
    String(number),
    "--repo",
    repo,
    "--json",
    "url",
  ]);
  if (result.status !== 0) {
    return `https://github.com/${repo}/issues/${number}`;
  }
  return JSON.parse(result.stdout).url;
}

function createIssue(repo, label, title, bodyFile) {
  const result = runGh([
    "issue",
    "create",
    "--repo",
    repo,
    "--title",
    title,
    "--label",
    label,
    "--body-file",
    bodyFile,
  ]);
  if (result.status !== 0) {
    warn(`could not create compatibility report issue: ${commandMessage(result)}`);
    return undefined;
  }
  return result.stdout.trim();
}

function updateIssue(repo, number, bodyFile) {
  const result = runGh([
    "issue",
    "edit",
    String(number),
    "--repo",
    repo,
    "--body-file",
    bodyFile,
  ]);
  if (result.status !== 0) {
    warn(`could not update compatibility report issue #${number}: ${commandMessage(result)}`);
    return undefined;
  }
  return issueUrl(repo, number);
}

function setOutput(name, value) {
  if (!process.env.GITHUB_OUTPUT) {
    return;
  }
  appendFileSync(process.env.GITHUB_OUTPUT, `${name}=${value}\n`, "utf8");
}

function run() {
  const repo = input("repo");
  const label = input("label");
  const title = input("title");
  const bodyFile = input("body-file");

  if (!existsSync(bodyFile)) {
    warn(`compatibility report body does not exist: ${bodyFile}`);
    return;
  }

  // Touch the body file early so bad paths or permissions fail before gh calls.
  readFileSync(bodyFile, "utf8");

  if (!ensureLabel(repo, label)) {
    return;
  }

  const number = findIssue(repo, label);
  const url =
    number === undefined
      ? createIssue(repo, label, title, bodyFile)
      : updateIssue(repo, number, bodyFile);

  if (!url) {
    return;
  }

  console.log(`compatibility report issue: ${url}`);
  setOutput("issue-url", url);
}

try {
  run();
} catch (error) {
  warn(error instanceof Error ? error.message : String(error));
}
