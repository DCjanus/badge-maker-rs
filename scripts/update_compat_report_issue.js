#!/usr/bin/env bun

import { appendFileSync, existsSync, readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";

function usage() {
  return "usage: update_compat_report_issue.js --repo <owner/repo> --label <label> --title <title> --body-file <path>";
}

function parseArgs(argv) {
  if (argv.includes("--help") || argv.includes("-h")) {
    console.log(usage());
    process.exit(0);
  }

  const args = {};
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    const value = argv[index + 1];
    if (!key?.startsWith("--") || value === undefined) {
      throw new Error(usage());
    }
    args[key.slice(2)] = value;
  }

  for (const required of ["repo", "label", "title", "body-file"]) {
    if (!args[required]) {
      throw new Error(usage());
    }
  }
  return args;
}

function runGh(args) {
  return spawnSync("gh", args, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
}

function commandMessage(result) {
  return result.stderr.trim() || result.stdout.trim();
}

function warn(message) {
  console.error(`::warning::${message}`);
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

function writeOutput(name, value) {
  const githubOutput = process.env.GITHUB_OUTPUT;
  if (!githubOutput) {
    return;
  }
  appendFileSync(githubOutput, `${name}=${value}\n`, "utf8");
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const bodyFile = args["body-file"];
  if (!existsSync(bodyFile)) {
    throw new Error(`report body does not exist: ${bodyFile}`);
  }

  // Touch the body file early so missing permissions or bad paths fail before gh calls.
  readFileSync(bodyFile, "utf8");

  if (!ensureLabel(args.repo, args.label)) {
    return;
  }

  const number = findIssue(args.repo, args.label);
  const url =
    number === undefined
      ? createIssue(args.repo, args.label, args.title, bodyFile)
      : updateIssue(args.repo, number, bodyFile);

  if (url) {
    console.log(`compatibility report issue: ${url}`);
    writeOutput("issue_url", url);
  }
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
