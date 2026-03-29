import { makeBadge as publicMakeBadge } from "badge-maker";

function describeError(error) {
  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
}

function runCase(testCase) {
  try {
    const format = {};
    for (const [key, value] of Object.entries({
      label: testCase.label,
      message: testCase.message,
      color: testCase.color,
      labelColor: testCase.labelColor,
      style: testCase.style,
      logoBase64: testCase.logoBase64,
      logoWidth: testCase.logoWidth,
      links: testCase.links,
      idSuffix: testCase.idSuffix,
    })) {
      if (value !== undefined && value !== null) {
        format[key] = value;
      }
    }

    const output = publicMakeBadge(format);
    return { id: testCase.id, ok: true, output, error: null };
  } catch (error) {
    return { id: testCase.id, ok: false, output: null, error: describeError(error) };
  }
}

export function runBadgeMakerCases(cases) {
  return cases.map(runCase);
}
