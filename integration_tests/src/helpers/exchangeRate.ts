import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { connectComet } from '@cosmjs/tendermint-rpc';
import { Decimal } from '@cosmjs/math';
import { QueryClient, createProtobufRpcClient } from '@cosmjs/stargate';
import {
  setupAuthExtension,
  setupBankExtension,
  setupStakingExtension,
  setupTxExtension,
} from '@cosmjs/stargate';
import { QueryClientImpl } from 'cosmjs-types/cosmos/bank/v1beta1/query';

async function calcExchangeRate(
  clientCW: SigningCosmWasmClient,
  coreContract: string,
  endpoint: string,
): Promise<string> {
  const coreConfig = await clientCW.queryContractSmart(coreContract, {
    config: {},
  });

  const FSMState = await clientCW.queryContractSmart(coreContract, {
    contract_state: {},
  });
  if (FSMState !== 'idle') {
    // if state isn't idle then this query'll return cached exchange rate
    return clientCW.queryContractSmart(coreContract, {
      exchange_rate: {},
    });
  }

  const queryClient = QueryClient.withExtensions(
    await connectComet(endpoint),
    setupAuthExtension,
    setupBankExtension,
    setupStakingExtension,
    setupTxExtension,
  );
  const RPC = createProtobufRpcClient(queryClient);
  const queryService = new QueryClientImpl(RPC);
  const { amount } = await queryService.SupplyOf({
    denom: coreConfig.base_denom,
  });

  let exchangeRateDenominator: Decimal = Decimal.fromUserInput(
    amount.amount,
    18,
  );
  if (exchangeRateDenominator.equals(Decimal.zero(18))) {
    return Decimal.one(18).toString();
  }

  const delegations = await clientCW.queryContractSmart(
    coreConfig.puppeteer_contract,
    {
      extension: {
        msg: {
          delegations: {},
        },
      },
    },
  );
  const delegationsAmount: Decimal = delegations[0].delegations.reduce(
    (acc: Decimal, next: any) =>
      acc.plus(Decimal.fromUserInput(next.amount.amount, 18)),
    Decimal.zero(18),
  );

  const batchID = await clientCW.queryContractSmart(coreContract, {
    current_unbond_batch: {},
  });
  const batch = await clientCW.queryContractSmart(coreContract, {
    unbond_batch: {
      batch_id: batchID,
    },
  });
  let unprocessedUnbondedAmount: Decimal = Decimal.zero(18);
  if (batch.status === 'new') {
    unprocessedUnbondedAmount = unprocessedUnbondedAmount.plus(
      Decimal.fromUserInput(batch.expected_native_asset_amount, 18),
    );
  }
  if (Number(batchID) > 0) {
    const penultimate = Number(batchID) - 1;
    const penultimateBatch = await clientCW.queryContractSmart(coreContract, {
      unbond_batch: {
        batch_id: String(penultimate),
      },
    });
    if (penultimateBatch.status === 'unbond_requested') {
      unprocessedUnbondedAmount = unprocessedUnbondedAmount.plus(
        Decimal.fromUserInput(batch.expected_native_asset_amount, 18),
      );
    }
  }
  let failedBatchID = await clientCW.queryContractSmart(coreContract, {
    failed_batch: {},
  });
  if (failedBatchID === null) {
    let failedBatch = await clientCW.queryContractSmart(coreContract, {
      batch_id: failedBatchID,
    });
    unprocessedUnbondedAmount = unprocessedUnbondedAmount.plus(
      Decimal.fromUserInput(failedBatch.total_dasset_amount_to_withdraw, 18),
    );
  }

  exchangeRateDenominator = exchangeRateDenominator.plus(
    unprocessedUnbondedAmount,
  );

  const stakerBalance: Decimal = Decimal.fromUserInput(
    await clientCW.queryContractSmart(coreConfig.staker_contract, {
      all_balance: {},
    }),
    18,
  );
  const totalLSMShares: Decimal = Decimal.fromUserInput(
    await clientCW.queryContractSmart(coreContract, {
      total_lsm_shares: {},
    }),
    18,
  );
  const exchangeRateNumerator: Decimal = delegationsAmount
    .plus(stakerBalance)
    .plus(totalLSMShares);
  if (exchangeRateNumerator.equals(Decimal.zero(18))) {
    return Decimal.one(18).toString();
  }
  // https://github.com/cosmos/cosmjs/issues/1498#issuecomment-1789798440
  // Here is no direct string conversation because it adds 18 additional decimals
  const exchangeRate: Decimal = Decimal.fromUserInput(
    String(
      exchangeRateNumerator.toFloatApproximation() /
        exchangeRateNumerator.toFloatApproximation(),
    ),
    18,
  );
  return exchangeRate.toString();
}
