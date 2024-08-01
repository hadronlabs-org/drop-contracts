import { connectComet } from '@cosmjs/tendermint-rpc';
import { Decimal } from 'decimal.js';
import { QueryClient, createProtobufRpcClient } from '@cosmjs/stargate';
import {
  setupAuthExtension,
  setupBankExtension,
  setupStakingExtension,
  setupTxExtension,
} from '@cosmjs/stargate';
import { QueryClientImpl } from 'cosmjs-types/cosmos/bank/v1beta1/query';
import { expect } from 'vitest';

export const DEFAULT_EXCHANGE_RATE_DECIMALS: number = 8;

export async function calcExchangeRate(
  coreContractClient: any,
  puppeteerContractClient: any,
  stakerContractClient: any,
  tokenContractClient: any,
  endpoint: string,
): Promise<number> {
  const queryClient = QueryClient.withExtensions(
    await connectComet(endpoint),
    setupAuthExtension,
    setupBankExtension,
    setupStakingExtension,
    setupTxExtension,
  );
  const RPC = createProtobufRpcClient(queryClient);
  const queryService = new QueryClientImpl(RPC);
  const tokenContractConfig = await tokenContractClient.queryConfig();
  const { amount } = await queryService.SupplyOf({
    denom: tokenContractConfig.denom,
  });
  let exchangeRateDenominator: Decimal = new Decimal(amount.amount);
  if (exchangeRateDenominator.isZero()) {
    return new Decimal(1).toNumber();
  }
  const delegationsResponse = await puppeteerContractClient.queryExtension({
    msg: {
      delegations: {},
    },
  });
  const delegationsAmount: Decimal =
    delegationsResponse.delegations.delegations.reduce(
      (acc: Decimal, next: any) => acc.plus(new Decimal(next.amount.amount)),
      new Decimal(0),
    );
  const batchID = await coreContractClient.queryCurrentUnbondBatch();
  const batch = await coreContractClient.queryUnbondBatch({
    batch_id: String(batchID),
  });
  let unprocessedDassetToUnbond: Decimal = new Decimal('0');
  if (batch.status === 'new') {
    unprocessedDassetToUnbond = unprocessedDassetToUnbond.plus(
      batch.total_dasset_amount_to_withdraw,
    );
  }
  if (Number(batchID) > 0) {
    const penultimate = Number(batchID) - 1;
    const penultimateBatch = await coreContractClient.queryUnbondBatch({
      batch_id: String(penultimate),
    });
    if (penultimateBatch.status === 'unbond_requested') {
      unprocessedDassetToUnbond = unprocessedDassetToUnbond.plus(
        batch.total_dasset_amount_to_withdraw,
      );
    }
  }
  const failedBatchID = await coreContractClient.queryFailedBatch();
  if (failedBatchID.response !== null) {
    const failedBatch = await coreContractClient.queryUnbondBatch({
      batch_id: failedBatchID.response,
    });
    unprocessedDassetToUnbond = unprocessedDassetToUnbond.plus(
      failedBatch.total_dasset_amount_to_withdraw,
    );
  }
  exchangeRateDenominator = exchangeRateDenominator.plus(
    unprocessedDassetToUnbond,
  );
  const stakerBalance: Decimal = new Decimal(
    await stakerContractClient.queryAllBalance(),
  );
  const totalLSMShares: Decimal = new Decimal(
    await coreContractClient.queryTotalLSMShares(),
  );
  const exchangeRateNumerator: Decimal = delegationsAmount
    .plus(stakerBalance)
    .plus(totalLSMShares);
  if (exchangeRateNumerator.isZero()) {
    return 1;
  }
  const exchangeRate: Decimal = exchangeRateNumerator.dividedBy(
    exchangeRateDenominator,
  );
  return exchangeRate.toNumber();
}

export async function checkExchangeRate(testContext: any) {
  const {
    coreContractClient,
    puppeteerContractClient,
    tokenContractClient,
    neutronRPCEndpoint,
    stakerContractClient,
  } = testContext;
  if ((await coreContractClient.queryContractState()) === 'idle') {
    expect(
      await compareExchangeRates(
        coreContractClient,
        puppeteerContractClient,
        tokenContractClient,
        stakerContractClient,
        neutronRPCEndpoint,
      ),
    ).toBeTruthy();
  }
}

export async function compareExchangeRates(
  coreContractClient: any,
  puppeteerContractClient: any,
  tokenContractClient: any,
  stakerContractClient: any,
  endpoint: string,
  decimals: number = DEFAULT_EXCHANGE_RATE_DECIMALS,
): Promise<boolean> {
  return (
    (
      await calcExchangeRate(
        coreContractClient,
        puppeteerContractClient,
        stakerContractClient,
        tokenContractClient,
        endpoint,
      )
    ).toFixed(decimals) ===
    Number(await coreContractClient.queryExchangeRate()).toFixed(decimals)
  );
}
