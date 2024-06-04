import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { AccountData, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice, Coin } from "@cosmjs/stargate";
import { Client as DropCoreClient } from "../../../integration_tests/src/generated/contractLib/dropCore";
import { Client as DropWithdrawalManager } from "../../../integration_tests/src/generated/contractLib/dropWithdrawalManager";
import { Client as DropWithdrawalVoucher } from "../../../integration_tests/src/generated/contractLib/dropWithdrawalVoucher";

const FACTORY_DENOM: string = process.env.FACTORY_DENOM;
const IBC_DENOM: string = process.env.IBC_DENOM;
const TARGET: string = process.env.TARGET;
const MNEMONIC: string = process.env.MNEMONIC;
const NODE_ADDRESS: string = process.env.NODE_ADDRESS;

const BOND_PROB: number = Number(process.env.BOND_PROB);
const UNBOND_PROB: number = Number(process.env.UNBOND_PROB);
const WITHDRAW_PROB: number = Number(process.env.WITHDRAW_PROB);
if (BOND_PROB + UNBOND_PROB + WITHDRAW_PROB !== 1) {
  console.error(
    `BOND_PROB(${BOND_PROB}) + UNBOND_PROB(${UNBOND_PROB}) + WITHDRAW_PROB(${WITHDRAW_PROB}) != 1`
  );
  process.exit(1);
}

enum MODE {
  BOND = "BOND",
  UNBOND = "UNBOND",
  WITHDRAW = "SEND_NFT",
}

async function calculate_mode(
  bond_p: number,
  unbond_p: number,
  withdraw_p: number
): Promise<MODE> {
  let r: number = Math.random();
  if (r < bond_p) {
    return MODE.BOND;
  } else if (r < bond_p + unbond_p) {
    return MODE.UNBOND;
  } else if (r < bond_p + unbond_p + withdraw_p) {
    return MODE.WITHDRAW;
  } else {
    console.error(
      `bond_p + unbond_p + withdraw_p = ${bond_p + unbond_p + withdraw_p}, it should be eq to 1`
    );
    process.exit(1);
  }
}

type ActionLog = {
  mode: MODE;
  funds:
    | Coin
    | {
        nft_id: string;
      };
  txHash: string;
  details: null | {
    batch_id: number;
  };
};

async function bond(
  drop_instance: DropCoreClient,
  address: string,
  fund: Coin
): Promise<ActionLog> {
  return {
    details: null,
    mode: MODE.BOND,
    funds: fund,
    txHash: (
      await drop_instance.bond(address, {}, "auto", "", [
        // contract to ts generates it's own Coin type
        {
          amount: fund.amount,
          denom: fund.denom,
        },
      ])
    ).transactionHash,
  };
}

async function bond_random_amount(
  clientCW: SigningCosmWasmClient,
  drop_instance: DropCoreClient,
  address: string
): Promise<ActionLog | null> {
  let ibc_denom_balance: Coin = await clientCW.getBalance(address, IBC_DENOM);
  if (Number(ibc_denom_balance.amount) === 0) {
    return null;
  }
  const config = await drop_instance.queryConfig();
  const exchange_rate: number = Math.floor(
    Number(await drop_instance.queryExchangeRate())
  );
  const min_exchange_rate: number = exchange_rate + 1;

  const min: number =
    Number(config.lsm_min_bond_amount) < min_exchange_rate
      ? min_exchange_rate
      : Number(config.lsm_min_bond_amount);

  let random_amount: number = Math.floor(
    Math.random() * (Number(ibc_denom_balance.amount) - min) + min
  );
  if (random_amount > Number(ibc_denom_balance.amount)) {
    return null;
  }
  try {
    const res = await bond(drop_instance, address, {
      amount: String(random_amount),
      denom: IBC_DENOM,
    });
    if ((await clientCW.getTx(res.txHash)).code !== 0) {
      return null;
    }
    return res;
  } catch (e) {
    return null;
  }
}

async function unbond(
  drop_instance: DropCoreClient,
  address: string,
  fund: Coin
): Promise<ActionLog> {
  return {
    details: null,
    mode: MODE.UNBOND,
    funds: fund,
    txHash: (
      await drop_instance.unbond(address, "auto", "", [
        // contract to ts generates it's own Coin type
        {
          amount: fund.amount,
          denom: fund.denom,
        },
      ])
    ).transactionHash,
  };
}

async function unbond_random_amount(
  clientCW: SigningCosmWasmClient,
  drop_instance: DropCoreClient,
  address: string
): Promise<ActionLog | null> {
  let factory_balance: Coin = await clientCW.getBalance(address, FACTORY_DENOM);
  if (Number(factory_balance.amount) === 0) {
    return null;
  }

  let random_amount: number = Math.floor(
    Math.random() * Number(factory_balance.amount) + 1
  );

  try {
    const res = await unbond(drop_instance, address, {
      amount: String(random_amount),
      denom: FACTORY_DENOM,
    });
    if ((await clientCW.getTx(res.txHash)).code !== 0) {
      return null;
    }
    return res;
  } catch (e) {
    return null;
  }
}

async function send_nft(
  withdrawal_voucher: DropWithdrawalVoucher,
  withdrawal_manager: DropWithdrawalManager,
  address: string,
  nft_id: string
): Promise<ActionLog> {
  const nft_info = await withdrawal_voucher.queryNftInfo({ token_id: nft_id });
  const batch_id: number = Number(nft_info.extension.batch_id);

  return {
    details: {
      batch_id: batch_id,
    },
    mode: MODE.WITHDRAW,
    funds: {
      nft_id: nft_id,
    },
    txHash: (
      await withdrawal_voucher.sendNft(
        address,
        {
          contract: withdrawal_manager.contractAddress,
          token_id: nft_id,
          msg: "eyJ3aXRoZHJhdyI6e319",
        },
        "auto",
        "",
        []
      )
    ).transactionHash,
  };
}

async function withdraw_random_nft(
  clientCW: SigningCosmWasmClient,
  drop_instance: DropCoreClient,
  address: string
): Promise<ActionLog | null> {
  const config = await drop_instance.queryConfig();
  const withdrawal_manager: DropWithdrawalManager = new DropWithdrawalManager(
    clientCW,
    config.withdrawal_manager_contract
  );
  const withdrawal_voucher: DropWithdrawalVoucher = new DropWithdrawalVoucher(
    clientCW,
    config.withdrawal_voucher_contract
  );

  let owned_nfts = (await withdrawal_voucher.queryTokens({ owner: address }))
    .tokens;
  let current_nft_list = owned_nfts;
  while (current_nft_list.length !== 0) {
    current_nft_list = (
      await withdrawal_voucher.queryTokens({
        owner: address,
        start_after: current_nft_list[current_nft_list.length - 1],
      })
    ).tokens;
    owned_nfts = owned_nfts.concat(current_nft_list);
  }

  const owned_nfts_detailed = [];
  for (const nft of owned_nfts) {
    owned_nfts_detailed.push({
      nft_id: nft,
      details: {
        ...(await withdrawal_voucher.queryNftInfo({
          token_id: nft,
        })),
      },
    });
  }

  const withdrawn_nfts = [];
  for (const nft_detailed of owned_nfts_detailed) {
    if (
      (
        await drop_instance.queryUnbondBatch({
          batch_id: nft_detailed.details.extension.batch_id,
        })
      ).status === "withdrawn"
    ) {
      withdrawn_nfts.push(nft_detailed.nft_id);
    }
  }

  if (withdrawn_nfts.length === 0) {
    return null;
  }

  const nft_id: string =
    withdrawn_nfts[Math.floor(Math.random() * withdrawn_nfts.length)];

  try {
    return await send_nft(
      withdrawal_voucher,
      withdrawal_manager,
      address,
      nft_id
    );
  } catch (e) {
    return null;
  }
}

async function main() {
  let mainWallet: DirectSecp256k1HdWallet =
    await DirectSecp256k1HdWallet.fromMnemonic(MNEMONIC, {
      prefix: "neutron",
    });
  let clientCW: SigningCosmWasmClient =
    await SigningCosmWasmClient.connectWithSigner(NODE_ADDRESS, mainWallet, {
      gasPrice: GasPrice.fromString("0.75untrn"),
    });
  let mainAccounts: readonly AccountData[] = await mainWallet.getAccounts();
  let target = new DropCoreClient(clientCW, TARGET);

  let mode: MODE = await calculate_mode(BOND_PROB, UNBOND_PROB, WITHDRAW_PROB);
  let unused_modes: MODE[] = [MODE.BOND, MODE.UNBOND, MODE.WITHDRAW];
  let finished: boolean = false;
  while (!finished && unused_modes.length > 0) {
    switch (mode) {
      case MODE.WITHDRAW: {
        const res = await withdraw_random_nft(
          clientCW,
          target,
          mainAccounts[0].address
        );
        if (res !== null) {
          console.log(res);
          finished = true;
        }
        break;
      }
      case MODE.UNBOND: {
        const res = await unbond_random_amount(
          clientCW,
          target,
          mainAccounts[0].address
        );
        if (res !== null) {
          console.log(res);
          finished = true;
        }
        break;
      }
      case MODE.BOND: {
        const res = await bond_random_amount(
          clientCW,
          target,
          mainAccounts[0].address
        );
        if (res !== null) {
          console.log(res);
          finished = true;
        }
        break;
      }
    }
    unused_modes = unused_modes.filter((element) => element !== mode);
    mode = unused_modes[Math.floor(Math.random() * unused_modes.length)];
  }
  if (unused_modes.length === 0) {
    console.error("Nothing to bond, unbond or withdraw");
    process.exit(1);
  }
}

main();
