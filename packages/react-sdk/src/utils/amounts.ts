export const formatAssetAmount = (
  amount: bigint,
  decimals?: number
): string => {
  if (!decimals || decimals <= 0) {
    return amount.toString();
  }

  const factor = 10n ** BigInt(decimals);
  const whole = amount / factor;
  const fraction = amount % factor;

  if (fraction === 0n) {
    return whole.toString();
  }

  const fractionText = fraction
    .toString()
    .padStart(decimals, "0")
    .replace(/0+$/, "");

  return `${whole.toString()}.${fractionText}`;
};

export const parseAssetAmount = (input: string, decimals?: number): bigint => {
  const value = input.trim();
  if (!value) {
    throw new Error("Amount is required");
  }

  if (!decimals || decimals <= 0) {
    if (value.includes(".")) {
      throw new Error("Amount must be a whole number");
    }
    return BigInt(value);
  }

  const [wholeText, fractionText = ""] = value.split(".");
  if (value.split(".").length > 2) {
    throw new Error("Amount has too many decimal points");
  }

  const normalizedWhole = wholeText.length ? wholeText : "0";
  if (fractionText.length > decimals) {
    throw new Error("Amount has too many decimal places");
  }

  const paddedFraction = fractionText.padEnd(decimals, "0");
  const factor = 10n ** BigInt(decimals);

  return BigInt(normalizedWhole) * factor + BigInt(paddedFraction || "0");
};
