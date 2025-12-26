import test from "./playwright.global.setup";
import { expect } from "@playwright/test";

test.describe("multisig auth component", () => {
  test("creates a multisig component with default threshold", async ({ page }) => {
    const result = await page.evaluate(() => {
      const commitments = [
        new window.Word(new BigUint64Array([1n, 0n, 0n, 0n])),
        new window.Word(new BigUint64Array([2n, 0n, 0n, 0n])),
        new window.Word(new BigUint64Array([3n, 0n, 0n, 0n])),
      ];

      const config = new window.AuthRpoFalcon512MultisigConfig(
        commitments,
        2
      );
      const defaultThreshold = config.defaultThreshold;
      const approvers = config.approvers.length;
      window.createAuthRpoFalcon512Multisig(config);

      return {
        ok: true,
        defaultThreshold,
        approvers,
      };
    });
    expect(result.defaultThreshold).toBe(2);
    expect(result.approvers).toBe(3);
  });

  test("allows per-procedure thresholds", async ({ page }) => {
    const ok = await page.evaluate(() => {
      const commitments = [
        new window.Word(new BigUint64Array([10n, 0n, 0n, 0n])),
        new window.Word(new BigUint64Array([11n, 0n, 0n, 0n])),
      ];
      const procRoot = new window.Word(
        new BigUint64Array([10n, 0n, 0n, 0n])
      );

      const config = new window.AuthRpoFalcon512MultisigConfig(
        commitments,
        2
      ).withProcThresholds([new window.ProcedureThreshold(procRoot, 1)]);

      const procThresholds = config.getProcThresholds();

      window.createAuthRpoFalcon512Multisig(config);
      return {
        ok: true,
        procThresholds:
          procThresholds?.map((p: any) => ({
            threshold: p.threshold,
            procRoot: p.procRoot.toHex(),
          })) ?? [],
      };
    });

    expect(ok.ok).toBe(true);
    expect(ok.procThresholds.length).toBe(1);
    expect(ok.procThresholds[0].threshold).toBe(1);
  });

  test("rejects invalid threshold", async ({ page }) => {
    const throws = await page.evaluate(() => {
      try {
        const commitments = [
          new window.Word(new BigUint64Array([7n, 0n, 0n, 0n])),
        ];
        new window.AuthRpoFalcon512MultisigConfig(commitments, 2);
        return false;
      } catch (err) {
        return true;
      }
    });

    expect(throws).toBe(true);
  });
});

