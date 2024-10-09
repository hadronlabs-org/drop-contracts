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

export async function calcExchangeRate(context: any): Promise<number> {
  const queryClient = QueryClient.withExtensions(
    await connectComet(context.neutronRPCEndpoint),
    setupAuthExtension,
    setupBankExtension,
    setupStakingExtension,
    setupTxExtension,
  );
  const RPC = createProtobufRpcClient(queryClient);
  const queryService = new QueryClientImpl(RPC);
  const tokenContractConfig = await context.tokenContractClient.queryConfig();
  const { amount } = await queryService.SupplyOf({
    denom: tokenContractConfig.denom,
  });
  let exchangeRateDenominator: Decimal = new Decimal(amount.amount);
  if (exchangeRateDenominator.isZero()) {
    return new Decimal(1).toNumber();
  }
  const delegationsResponse =
    await context.puppeteerContractClient.queryExtension({
      msg: {
        delegations: {},
      },
    });
  const delegationsAmount: Decimal =
    delegationsResponse.delegations.delegations.reduce(
      (acc: Decimal, next: any) => acc.plus(new Decimal(next.amount.amount)),
      new Decimal(0),
    );
  const batchID = await context.coreContractClient.queryCurrentUnbondBatch();
  const batch = await context.coreContractClient.queryUnbondBatch({
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
    const penultimateBatch = await context.coreContractClient.queryUnbondBatch({
      batch_id: String(penultimate),
    });
    if (penultimateBatch.status === 'unbond_requested') {
      unprocessedDassetToUnbond = unprocessedDassetToUnbond.plus(
        batch.total_dasset_amount_to_withdraw,
      );
    }
  }
  const failedBatchID = await context.coreContractClient.queryFailedBatch();
  if (failedBatchID.response !== null) {
    const failedBatch = await context.coreContractClient.queryUnbondBatch({
      batch_id: failedBatchID.response,
    });
    unprocessedDassetToUnbond = unprocessedDassetToUnbond.plus(
      failedBatch.total_dasset_amount_to_withdraw,
    );
  }
  exchangeRateDenominator = exchangeRateDenominator.plus(
    unprocessedDassetToUnbond,
  );

  const totalAsyncTokens: Decimal = new Decimal(
    await context.coreContractClient.queryTotalAsyncTokens(),
  );
  const exchangeRateNumerator: Decimal =
    delegationsAmount.plus(totalAsyncTokens);
  if (exchangeRateNumerator.isZero()) {
    return 1;
  }
  const exchangeRate: Decimal = exchangeRateNumerator.dividedBy(
    exchangeRateDenominator,
  );
  return exchangeRate.toNumber();
}

export async function checkExchangeRate(context: any) {
  if ((await context.coreContractClient.queryContractState()) === 'idle') {
    expect(await compareExchangeRates(context)).toBeTruthy();
  }
}

export async function compareExchangeRates(
  context: any,
  decimals: number = DEFAULT_EXCHANGE_RATE_DECIMALS,
): Promise<boolean> {
  return (
    (await calcExchangeRate(context)).toFixed(decimals) ===
    Number(await context.coreContractClient.queryExchangeRate()).toFixed(
      decimals,
    )
  );
}
