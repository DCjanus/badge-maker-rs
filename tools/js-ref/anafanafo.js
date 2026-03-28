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

export function runAnafanafoCases(cases) {
  return cases.map(measureCase);
}
