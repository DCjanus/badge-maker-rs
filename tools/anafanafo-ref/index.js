import anafanafo from "anafanafo";

function describeError(error) {
  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
}

function measureCase(testCase) {
  try {
    const width = anafanafo(testCase.text, {
      font: testCase.font,
      guess: testCase.guess ?? true,
    });

    return {
      id: testCase.id,
      ok: true,
      width,
      error: null,
    };
  } catch (error) {
    return {
      id: testCase.id,
      ok: false,
      width: null,
      error: describeError(error),
    };
  }
}

const stdin = await Bun.stdin.text();
const cases = JSON.parse(stdin);
const results = cases.map(measureCase);

process.stdout.write(`${JSON.stringify(results)}\n`);
