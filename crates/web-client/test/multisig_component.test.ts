// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("multisig auth component", () => {
  test("creates a multisig component with default threshold", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      const commitments = [
        new sdk.Word(sdk.u64Array([1, 0, 0, 0])),
        new sdk.Word(sdk.u64Array([2, 0, 0, 0])),
        new sdk.Word(sdk.u64Array([3, 0, 0, 0])),
      ];

      const config = new sdk.AuthFalcon512RpoMultisigConfig(commitments, 2);
      const defaultThreshold = config.defaultThreshold;
      const approvers = config.approvers.length;
      sdk.createAuthFalcon512RpoMultisig(config);

      return { defaultThreshold, approvers };
    });
    expect(result.defaultThreshold).toBe(2);
    expect(result.approvers).toBe(3);
  });

  test("allows per-procedure thresholds", async ({ run }) => {
    const result = await run(async ({ sdk }) => {
      const commitments = [
        new sdk.Word(sdk.u64Array([10, 0, 0, 0])),
        new sdk.Word(sdk.u64Array([11, 0, 0, 0])),
      ];
      const procRoot = new sdk.Word(sdk.u64Array([10, 0, 0, 0]));

      const config = new sdk.AuthFalcon512RpoMultisigConfig(
        commitments,
        2
      ).withProcThresholds([new sdk.ProcedureThreshold(procRoot, 1)]);

      const procThresholds = config.getProcThresholds();

      sdk.createAuthFalcon512RpoMultisig(config);

      const mapped =
        procThresholds?.map((p) => ({
          threshold: p.threshold,
          procRoot: p.procRoot.toHex(),
        })) ?? [];

      return { length: mapped.length, firstThreshold: mapped[0]?.threshold };
    });
    expect(result.length).toBe(1);
    expect(result.firstThreshold).toBe(1);
  });

  test("rejects invalid threshold", async ({ run }) => {
    const result = await run(async ({ sdk }) => {
      try {
        const commitments = [new sdk.Word(sdk.u64Array([7, 0, 0, 0]))];
        new sdk.AuthFalcon512RpoMultisigConfig(commitments, 2);
        return { threw: false };
      } catch (e) {
        return { threw: true, message: e.message };
      }
    });
    expect(result.threw).toBe(true);
  });
});
