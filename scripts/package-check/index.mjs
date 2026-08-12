import { assertPackageConsumer } from "./consumer.mjs";
import { assertPackageContract } from "./contract.mjs";

export async function checkPackage(packageRoot, toolingRoot) {
  const contract = assertPackageContract(packageRoot);
  await assertPackageConsumer({
    ...contract,
    packageRoot,
    toolingRoot,
  });
}
