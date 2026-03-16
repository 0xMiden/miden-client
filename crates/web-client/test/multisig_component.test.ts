// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("multisig auth component", () => {
  test("creates a multisig component with default threshold", async ({
    client,
    sdk,
  }) => {
    const commitments = [
      new sdk.Word(sdk.u64Array([1, 0, 0, 0])),
      new sdk.Word(sdk.u64Array([2, 0, 0, 0])),
      new sdk.Word(sdk.u64Array([3, 0, 0, 0])),
    ];

    const config = new sdk.AuthFalcon512RpoMultisigConfig(commitments, 2);
    const defaultThreshold = config.defaultThreshold;
    const approvers = config.approvers.length;
    sdk.createAuthFalcon512RpoMultisig(config);

    expect(defaultThreshold).toBe(2);
    expect(approvers).toBe(3);
  });

  test("allows per-procedure thresholds", async ({ client, sdk }) => {
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
      procThresholds?.map((p: any) => ({
        threshold: p.threshold,
        procRoot: p.procRoot.toHex(),
      })) ?? [];

    expect(mapped.length).toBe(1);
    expect(mapped[0].threshold).toBe(1);
  });

  test("rejects invalid threshold", async ({ client, sdk }) => {
    expect(() => {
      const commitments = [new sdk.Word(sdk.u64Array([7, 0, 0, 0]))];
      new sdk.AuthFalcon512RpoMultisigConfig(commitments, 2);
    }).toThrow();
  });
});
